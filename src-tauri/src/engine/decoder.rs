use super::state::QualityMode;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::resampling::context::Context as Resampler;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::audio::Audio;
use ffmpeg_next::util::frame::video::Video;
use std::path::Path;

pub enum DecodeResult {
    Video {
        data: Vec<u8>,
        width: u32,
        height: u32,
        stride: u32,
        pts: f64,
    },
    Audio {
        pts: f64,
    },
}

pub struct Decoder {
    input_ctx: ffmpeg::format::context::Input,
    pub video_stream_index: Option<usize>,
    audio_stream_index: Option<usize>,
    decoder: Option<ffmpeg::decoder::Video>,
    audio_decoder: Option<ffmpeg::decoder::Audio>,
    scaler: Option<Context>,
    resampler: Option<Resampler>,
    resampler_in_format: Option<ffmpeg::util::format::sample::Sample>,
    resampler_in_rate: u32,
    resampler_in_layout: ffmpeg::channel_layout::ChannelLayout,
    // Metadata
    duration_secs: f64,
    time_base: ffmpeg::util::rational::Rational,
    _audio_time_base: ffmpeg::util::rational::Rational,
    raw_frame: Video,
    scaled_frame: Video,
    audio_frame: Audio,
    resampled_frame: Audio,
    // Audio Buffer (Interleaved samples)
    pub audio_buffer: Vec<f32>,
    audio_pts_counter: u64,
}

impl Decoder {
    pub fn new(path: &Path, quality: QualityMode) -> anyhow::Result<Self> {
        eprintln!("[Decoder] Opening media: {:?}", path);
        ffmpeg::init()?;
        let input_ctx = input(&path).map_err(|e| {
            eprintln!("[Decoder] Failed to open input for {:?}: {}", path, e);
            anyhow::anyhow!("FFmpeg input error: {}", e)
        })?;

        // Video Setup
        let video_stream = input_ctx.streams().best(Type::Video);
        let mut decoder = None;
        let mut scaler = None;
        let mut video_stream_index = None;
        let mut time_base = ffmpeg::util::rational::Rational(0, 1);

        if let Some(s) = video_stream {
            eprintln!("[Decoder] Found video stream at index {}", s.index());
            let context_decoder = ffmpeg::codec::context::Context::from_parameters(s.parameters())?;
            let ad = context_decoder.decoder().video().map_err(|e| {
                eprintln!("[Decoder] Failed to open video decoder: {}", e);
                e
            })?;

            let width = ad.width();
            let height = ad.height();
            let (target_width, target_height) = match quality {
                QualityMode::Native => (width, height),
                QualityMode::Fast => (width / 2, height / 2),
                QualityMode::Proxy => (width / 4, height / 4),
            };

            scaler = Some(
                Context::get(
                    ad.format(),
                    width,
                    height,
                    Pixel::RGBA,
                    target_width,
                    target_height,
                    Flags::BILINEAR,
                )
                .map_err(|e| {
                    eprintln!("[Decoder] Failed to initialize scaler: {}", e);
                    e
                })?,
            );

            video_stream_index = Some(s.index());
            time_base = s.time_base();
            decoder = Some(ad);
        }

        let duration_secs = input_ctx.duration() as f64 / 1_000_000.0;
        eprintln!("[Decoder] Media duration identified: {}s", duration_secs);

        // Audio Setup
        let audio_stream = input_ctx.streams().best(Type::Audio);
        let mut audio_decoder = None;
        let mut audio_stream_index = None;
        let mut resampler = None;
        let mut resampler_in_format = None;
        let mut resampler_in_rate = 0;
        let mut resampler_in_layout = ffmpeg::channel_layout::ChannelLayout::empty();
        let mut audio_time_base = ffmpeg::util::rational::Rational(0, 1);

        if let Some(s) = audio_stream {
            eprintln!("[Decoder] Found audio stream at index {}", s.index());
            let context = ffmpeg::codec::context::Context::from_parameters(s.parameters())?;
            let ad = context.decoder().audio().map_err(|e| {
                eprintln!("[Decoder] Failed to open audio decoder: {}", e);
                e
            })?;
            audio_stream_index = Some(s.index());
            audio_time_base = s.time_base();

            let target_rate = 48000;
            let target_layout = ffmpeg::channel_layout::ChannelLayout::STEREO;

            // Defensive Layout Check: WAVs often have unspecified layout in parameters
            let src_layout = if ad.channel_layout().is_empty() {
                let layout = ffmpeg::channel_layout::ChannelLayout::default(ad.channels() as i32);
                eprintln!(
                    "[Decoder] Audio layout unspecified, defaulting to {:?} based on {} channels",
                    layout,
                    ad.channels()
                );
                layout
            } else {
                ad.channel_layout()
            };

            eprintln!(
                "[Decoder] Audio: {:?} format, {}Hz, {:?} ({} channels)",
                ad.format(),
                ad.rate(),
                src_layout,
                ad.channels()
            );

            resampler = Some(
                ffmpeg::software::resampling::context::Context::get(
                    ad.format(),
                    src_layout,
                    ad.rate(),
                    ffmpeg::util::format::sample::Sample::F32(
                        ffmpeg::util::format::sample::Type::Packed,
                    ),
                    target_layout,
                    target_rate,
                )
                .map_err(|e| {
                    eprintln!("[Decoder] Failed to initialize resampler: {}", e);
                    e
                })?,
            );
            resampler_in_format = Some(ad.format());
            resampler_in_rate = ad.rate();
            resampler_in_layout = src_layout;
            audio_decoder = Some(ad);
        }

        Ok(Self {
            input_ctx,
            video_stream_index,
            audio_stream_index,
            decoder,
            audio_decoder,
            scaler,
            resampler,
            resampler_in_format,
            resampler_in_rate,
            resampler_in_layout,
            duration_secs,
            time_base,
            _audio_time_base: audio_time_base,
            raw_frame: Video::empty(),
            scaled_frame: Video::empty(),
            audio_frame: Audio::empty(),
            resampled_frame: Audio::empty(),
            audio_buffer: Vec::with_capacity(4096),
            audio_pts_counter: 0,
        })
    }

    pub fn get_metadata(&self) -> (f64, u32, u32) {
        let (w, h) = if let Some(ref d) = self.decoder {
            (d.width(), d.height())
        } else {
            (0, 0)
        };
        (self.duration_secs, w, h)
    }

    pub fn decode_next(&mut self) -> anyhow::Result<Option<DecodeResult>> {
        for (stream, packet) in self.input_ctx.packets() {
            let pts = packet.pts().unwrap_or(0);

            if Some(stream.index()) == self.video_stream_index {
                if let (Some(ref mut d), Some(ref mut s)) = (&mut self.decoder, &mut self.scaler) {
                    d.send_packet(&packet)?;
                    if d.receive_frame(&mut self.raw_frame).is_ok() {
                        s.run(&self.raw_frame, &mut self.scaled_frame)?;

                        let stride = self.scaled_frame.stride(0) as i32;
                        let width = self.scaled_frame.width();
                        let height = self.scaled_frame.height();

                        if stride <= 0 {
                            continue;
                        }

                        let pts_secs =
                            pts as f64 * (self.time_base.0 as f64 / self.time_base.1 as f64);

                        return Ok(Some(DecodeResult::Video {
                            data: self.scaled_frame.data(0).to_vec(),
                            width,
                            height,
                            stride: stride as u32,
                            pts: pts_secs,
                        }));
                    }
                }
            } else if Some(stream.index()) == self.audio_stream_index {
                let audio_time_base = stream.time_base();
                let _pts_secs = pts as f64 * (audio_time_base.0 as f64 / audio_time_base.1 as f64);

                if let Some(ref mut ad) = self.audio_decoder {
                    ad.send_packet(&packet)?;
                    let mut frames_decoded = 0;
                    while ad.receive_frame(&mut self.audio_frame).is_ok() {
                        frames_decoded += 1;
                        if let Some(ref mut resampler) = self.resampler {
                            // Dynamic Re-initialization if parameters changed
                            let frame_format = self.audio_frame.format();
                            let frame_rate = self.audio_frame.rate();
                            let frame_channels = self.audio_frame.channels();
                            let frame_layout = if self.audio_frame.channel_layout().is_empty() {
                                ffmpeg::channel_layout::ChannelLayout::default(
                                    frame_channels as i32,
                                )
                            } else {
                                self.audio_frame.channel_layout()
                            };

                            let needs_reinit = self.resampler_in_format != Some(frame_format)
                                || self.resampler_in_rate != frame_rate
                                || self.resampler_in_layout != frame_layout;

                            if needs_reinit {
                                eprintln!(
                                    "[Decoder] Audio parameters changed! Re-initializing resampler: {:?}, {}Hz, {:?} -> {:?}, {}Hz, {:?}",
                                    self.resampler_in_format,
                                    self.resampler_in_rate,
                                    self.resampler_in_layout,
                                    frame_format,
                                    frame_rate,
                                    frame_layout
                                );

                                let target_rate = 48000;
                                let target_layout = ffmpeg::channel_layout::ChannelLayout::STEREO;

                                if let Ok(new_resampler) =
                                    ffmpeg::software::resampling::context::Context::get(
                                        frame_format,
                                        frame_layout,
                                        frame_rate,
                                        ffmpeg::util::format::sample::Sample::F32(
                                            ffmpeg::util::format::sample::Type::Packed,
                                        ),
                                        target_layout,
                                        target_rate,
                                    )
                                {
                                    *resampler = new_resampler;
                                    self.resampler_in_format = Some(frame_format);
                                    self.resampler_in_rate = frame_rate;
                                    self.resampler_in_layout = frame_layout;
                                } else {
                                    eprintln!("[Decoder] Failed to re-initialize resampler!");
                                    continue;
                                }
                            }

                            // Correctly size the resampled frame
                            // We resample to 48000 Hz Stereo
                            let target_rate = 48000;
                            let delay = resampler.delay().map(|d| d.input).unwrap_or(0);
                            let out_samples = ((self.audio_frame.samples() as i64 + delay)
                                * target_rate as i64
                                / self.audio_frame.rate() as i64)
                                as usize;

                            unsafe {
                                self.resampled_frame.alloc(
                                    ffmpeg::util::format::sample::Sample::F32(
                                        ffmpeg::util::format::sample::Type::Packed,
                                    ),
                                    out_samples,
                                    ffmpeg::channel_layout::ChannelLayout::STEREO,
                                );
                            }

                            let _run_result =
                                resampler.run(&self.audio_frame, &mut self.resampled_frame)?;

                            // ffmpeg-next 6.1 run() returns Result<usize, Error> or Option<Delay>
                            // but actually it usually returns usize in Result.
                            // The compiler says Option<Delay> so we handle that if it's the case.
                            // If it's usize, we convert it.
                            let converted = self.resampled_frame.samples();

                            // Extract Float32 samples
                            let data = self.resampled_frame.data(0);
                            // Only take the samples that were actually converted
                            let samples: &[f32] = unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const f32,
                                    converted * 2, // 2 channels
                                )
                            };
                            self.audio_buffer.extend_from_slice(samples);

                            // Diagnostic: Check if we are getting literal silence
                            let max_amplitude = samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
                            if self.audio_pts_counter % 48000 < converted as u64 {
                                eprintln!(
                                    "[Decoder] Audio Amplitude: {:.6}, samples: {}, frames_in_packet: {}",
                                    max_amplitude, converted, frames_decoded
                                );
                            }

                            self.audio_pts_counter += converted as u64;
                        }
                    }
                    if frames_decoded > 0 {
                        // Use sample counter for reliable audio timing (48kHz Stereo)
                        let calculated_pts = self.audio_pts_counter as f64 / 48000.0;
                        return Ok(Some(DecodeResult::Audio {
                            pts: calculated_pts,
                        }));
                    } else {
                        // No frames yet, continue reading packets
                        continue;
                    }
                }
            }
        }

        Ok(None)
    }
}
