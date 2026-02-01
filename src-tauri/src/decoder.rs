use crate::QualityMode;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::resampling::context::Context as Resampler;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::audio::Audio;
use ffmpeg_next::util::frame::video::Video;
use std::path::Path;

pub struct Decoder {
    input_ctx: ffmpeg::format::context::Input,
    video_stream_index: usize,
    audio_stream_index: Option<usize>,
    decoder: ffmpeg::decoder::Video,
    audio_decoder: Option<ffmpeg::decoder::Audio>,
    scaler: Context,
    resampler: Option<Resampler>,
    // Metadata
    duration_secs: f64,
    time_base: ffmpeg::util::rational::Rational,
    audio_time_base: ffmpeg::util::rational::Rational,
    // Caching frames to avoid re-allocation
    raw_frame: Video,
    scaled_frame: Video,
    audio_frame: Audio,
    // Audio Buffer (Interleaved samples)
    pub audio_buffer: Vec<f32>,
}

impl Decoder {
    pub fn new(path: &Path, quality: QualityMode) -> anyhow::Result<Self> {
        eprintln!("[Decoder] Initializing FFmpeg...");
        ffmpeg::init()?;

        let input_ctx = input(&path)?;
        let video_stream = input_ctx
            .streams()
            .best(Type::Video)
            .ok_or_else(|| anyhow::anyhow!("No video stream found"))?;

        let video_stream_index = video_stream.index();
        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;

        let decoder = context_decoder.decoder().video()?;

        let width = decoder.width();
        let height = decoder.height();

        let (target_width, target_height) = match quality {
            QualityMode::Native => (width, height),
            QualityMode::Fast => (width / 2, height / 2),
            QualityMode::Proxy => (width / 4, height / 4),
        };

        let scaler = Context::get(
            decoder.format(),
            width,
            height,
            Pixel::RGBA,
            target_width,
            target_height,
            Flags::BILINEAR,
        )?;

        let duration_secs = input_ctx.duration() as f64 / 1_000_000.0;
        let time_base = video_stream.time_base();

        // Audio Setup
        let audio_stream = input_ctx.streams().best(Type::Audio);
        let mut audio_decoder = None;
        let mut audio_stream_index = None;
        let mut resampler = None;
        let mut audio_time_base = ffmpeg::util::rational::Rational(0, 1);

        if let Some(s) = audio_stream {
            let context = ffmpeg::codec::context::Context::from_parameters(s.parameters())?;
            let ad = context.decoder().audio()?;
            audio_stream_index = Some(s.index());
            audio_time_base = s.time_base();

            // Setup Resampler: Convert to Float32, 48kHz, Stereo
            resampler = Some(ffmpeg::software::resampling::context::Context::get(
                ad.format(),
                ad.channel_layout(),
                ad.rate(),
                ffmpeg::util::format::sample::Sample::F32(
                    ffmpeg::util::format::sample::Type::Packed,
                ),
                ffmpeg::channel_layout::ChannelLayout::STEREO,
                48000,
            )?);
            audio_decoder = Some(ad);
            eprintln!("[Decoder] Audio stream found and resampler initialized.");
        }

        Ok(Self {
            input_ctx,
            video_stream_index,
            audio_stream_index,
            decoder,
            audio_decoder,
            scaler,
            resampler,
            duration_secs,
            time_base,
            audio_time_base,
            raw_frame: Video::empty(),
            scaled_frame: Video::empty(),
            audio_frame: Audio::empty(),
            audio_buffer: Vec::with_capacity(4096),
        })
    }

    pub fn get_metadata(&self) -> (f64, u32, u32) {
        (
            self.duration_secs,
            self.decoder.width(),
            self.decoder.height(),
        )
    }

    pub fn decode_next_frame(&mut self) -> anyhow::Result<Option<(Vec<u8>, u32, u32, u32, f64)>> {
        for (stream, packet) in self.input_ctx.packets() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet)?;
                if self.decoder.receive_frame(&mut self.raw_frame).is_ok() {
                    self.scaler.run(&self.raw_frame, &mut self.scaled_frame)?;

                    let stride = self.scaled_frame.stride(0) as i32;
                    let width = self.scaled_frame.width();
                    let height = self.scaled_frame.height();

                    if stride <= 0 {
                        eprintln!("Invalid stride: {}", stride);
                        return Ok(None);
                    }

                    let pts = self.raw_frame.pts().unwrap_or(0);
                    let pts_secs = pts as f64 * (self.time_base.0 as f64 / self.time_base.1 as f64);

                    return Ok(Some((
                        self.scaled_frame.data(0).to_vec(),
                        width,
                        height,
                        stride as u32,
                        pts_secs,
                    )));
                }
            } else if Some(stream.index()) == self.audio_stream_index {
                if let Some(ref mut ad) = self.audio_decoder {
                    ad.send_packet(&packet)?;
                    while ad.receive_frame(&mut self.audio_frame).is_ok() {
                        if let Some(ref mut resampler) = self.resampler {
                            let mut resampled = Audio::empty();
                            resampler.run(&self.audio_frame, &mut resampled)?;

                            // Extract Float32 samples
                            let data = resampled.data(0);
                            let samples: &[f32] = unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const f32,
                                    data.len() / 4,
                                )
                            };
                            self.audio_buffer.extend_from_slice(samples);
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}
