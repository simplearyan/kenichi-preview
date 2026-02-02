use super::state::QualityMode;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
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
    // Metadata
    duration_secs: f64,
    time_base: ffmpeg::util::rational::Rational,
    _audio_time_base: ffmpeg::util::rational::Rational,
    raw_frame: Video,
    scaled_frame: Video,
    audio_frame: Audio,
    // Audio Buffer (Interleaved samples)
    pub audio_buffer: Vec<f32>,
    audio_pts_counter: u64,
}

impl Decoder {
    pub fn new(path: &Path, quality: QualityMode) -> anyhow::Result<Self> {
        log::info!(
            "[Decoder] Opening media: {:?}, Quality: {:?}",
            path,
            quality
        );
        ffmpeg::init()?;
        let input_ctx = input(&path).map_err(|e| {
            log::error!("[Decoder] Failed to open input for {:?}: {}", path, e);
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

            audio_decoder = Some(ad);
        }

        Ok(Self {
            input_ctx,
            video_stream_index,
            audio_stream_index,
            decoder,
            audio_decoder,
            scaler,
            duration_secs,
            time_base,
            _audio_time_base: audio_time_base,
            raw_frame: Video::empty(),
            scaled_frame: Video::empty(),
            audio_frame: Audio::empty(),
            audio_buffer: Vec::with_capacity(4096),
            audio_pts_counter: 0,
        })
    }

    pub fn seek(&mut self, time_secs: f64) -> anyhow::Result<()> {
        let timestamp = (time_secs * 1_000_000.0) as i64;
        // Seek to timestamp in microseconds (AV_TIME_BASE is 1,000,000)
        self.input_ctx.seek(timestamp, ..timestamp)?;

        // Flush internal buffers
        if let Some(ref mut d) = self.decoder {
            d.flush();
        }
        if let Some(ref mut ad) = self.audio_decoder {
            ad.flush();
        }

        // Reset buffers
        self.audio_buffer.clear();
        self.audio_pts_counter = (time_secs * 48000.0) as u64; // Approx reset

        Ok(())
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
                    if let Err(e) = d.send_packet(&packet) {
                        log::warn!("[Decoder] Video send_packet error: {:?} - continuing", e);
                        // Continue to try receive_frame anyway, or just next packet
                    }
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
                    match ad.send_packet(&packet) {
                        Ok(_) => {}
                        Err(e) => {
                            log::warn!(
                                "[Decoder] Audio send_packet error: {:?} - ignoring and continuing",
                                e
                            );
                        }
                    }
                    let mut frames_decoded = 0;
                    while ad.receive_frame(&mut self.audio_frame).is_ok() {
                        frames_decoded += 1;
                        // Manual conversion now handles all audio (see below)

                        // Manual conversion for I16 formats to avoid swresample issues
                        let frame_format = self.audio_frame.format();
                        let frame_rate = self.audio_frame.rate();
                        let frame_channels = self.audio_frame.channels();

                        // Check if we can use manual conversion (I16 Packed or F32 Planar, 1-2 channels)
                        let use_manual =
                            (matches!(frame_format, ffmpeg::util::format::sample::Sample::I16(_))
                                || matches!(
                                    frame_format,
                                    ffmpeg::util::format::sample::Sample::F32(_)
                                ))
                                && frame_channels <= 2;

                        if use_manual {
                            let input_samples = self.audio_frame.samples();
                            let output_samples = if frame_rate == 48000 {
                                input_samples
                            } else {
                                ((input_samples as u64 * 48000) / frame_rate as u64) as usize
                            };

                            // Handle F32 Planar (most video files)
                            if matches!(
                                frame_format,
                                ffmpeg::util::format::sample::Sample::F32(
                                    ffmpeg::util::format::sample::Type::Planar
                                )
                            ) {
                                let left_data = self.audio_frame.data(0);
                                let left_f32: &[f32] = unsafe {
                                    std::slice::from_raw_parts(
                                        left_data.as_ptr() as *const f32,
                                        input_samples,
                                    )
                                };

                                if frame_channels == 1 {
                                    for i in 0..output_samples {
                                        let src_idx = if frame_rate == 48000 {
                                            i
                                        } else {
                                            ((i as u64 * frame_rate as u64) / 48000) as usize
                                        };
                                        if src_idx >= input_samples {
                                            break;
                                        }
                                        let sample = left_f32[src_idx];
                                        self.audio_buffer.push(sample);
                                        self.audio_buffer.push(sample);
                                    }
                                } else {
                                    let right_data = self.audio_frame.data(1);
                                    let right_f32: &[f32] = unsafe {
                                        std::slice::from_raw_parts(
                                            right_data.as_ptr() as *const f32,
                                            input_samples,
                                        )
                                    };
                                    for i in 0..output_samples {
                                        let src_idx = if frame_rate == 48000 {
                                            i
                                        } else {
                                            ((i as u64 * frame_rate as u64) / 48000) as usize
                                        };
                                        if src_idx >= input_samples {
                                            break;
                                        }
                                        self.audio_buffer.push(left_f32[src_idx]);
                                        self.audio_buffer.push(right_f32[src_idx]);
                                    }
                                }
                            } else {
                                // Handle I16 Packed (audio-only files)
                                let input_data = self.audio_frame.data(0);
                                let input_i16: &[i16] = unsafe {
                                    std::slice::from_raw_parts(
                                        input_data.as_ptr() as *const i16,
                                        input_samples * frame_channels as usize,
                                    )
                                };

                                for i in 0..output_samples {
                                    let src_idx = if frame_rate == 48000 {
                                        i
                                    } else {
                                        ((i as u64 * frame_rate as u64) / 48000) as usize
                                    };
                                    if src_idx >= input_samples {
                                        break;
                                    }

                                    if frame_channels == 1 {
                                        let mono = input_i16[src_idx] as f32 / 32768.0;
                                        self.audio_buffer.push(mono);
                                        self.audio_buffer.push(mono);
                                    } else {
                                        let left = input_i16[src_idx * 2] as f32 / 32768.0;
                                        let right = input_i16[src_idx * 2 + 1] as f32 / 32768.0;
                                        self.audio_buffer.push(left);
                                        self.audio_buffer.push(right);
                                    }
                                }
                            }

                            self.audio_pts_counter += output_samples as u64;
                        } else {
                            // Skip exotic formats for now
                            log::warn!(
                                "[Decoder] Skipping unsupported format: {:?} {}Hz {}ch",
                                frame_format,
                                frame_rate,
                                frame_channels
                            );
                            continue;
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
