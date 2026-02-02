use super::state::PreviewState;
use super::state::SyncMode;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{Emitter, Window};

pub struct PlaybackEngine {
    state: PreviewState,
    window: Window,
}

impl PlaybackEngine {
    pub fn new(state: PreviewState, window: Window) -> Self {
        Self { state, window }
    }

    pub fn playback_thread(&self, path: PathBuf) {
        let renderer_clone = self.state.renderer.clone();
        let playing_clone = self.state.is_playing.clone();
        let session_id_clone = self.state.session_id.clone();
        let audio_producer_clone = self.state.audio_producer.clone();
        let seek_target_clone = self.state.seek_target.clone();
        let sync_mode_clone = self.state.sync_mode.clone();
        let quality_mode = *self.state.quality_mode.lock().unwrap();
        let window = self.window.clone();

        // Capture the current session ID to ensure we don't run old threads
        let current_session = {
            let s = session_id_clone.lock().unwrap();
            *s
        };

        std::thread::spawn(move || {
            let mut decoder = match crate::engine::decoder::Decoder::new(&path, quality_mode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[PlaybackEngine] Decoder error: {}", e);
                    return;
                }
            };

            if decoder.video_stream_index.is_none() {
                let mut guard = renderer_clone.lock().unwrap();
                if let Some(r) = guard.as_mut() {
                    r.clear_video();
                    let _ = r.repaint();
                }
            }

            let mut current_time = 0.0;
            let mut reference_start_time: Option<Instant> = None;
            let (duration, _, _) = decoder.get_metadata();

            // Initial update so UI knows duration immediately
            let _ = window.emit(
                "playback-update",
                crate::engine::state::PlaybackPayload {
                    current_time,
                    duration,
                    status: crate::engine::state::PlaybackStatus::Playing,
                },
            );

            let mut iteration_count = 0;

            loop {
                // Check if session has changed (user opened new file)
                if *session_id_clone.lock().unwrap() != current_session {
                    break;
                }

                // Check for Seek Request
                let mut seek_opt = None;
                {
                    let mut guard = seek_target_clone.lock().unwrap();
                    if let Some(target) = *guard {
                        seek_opt = Some(target);
                        *guard = None; // Reset
                    }
                }

                if let Some(target) = seek_opt {
                    log::info!("[PlaybackEngine] Seeking to {}s", target);
                    if let Err(e) = decoder.seek(target) {
                        log::error!("[PlaybackEngine] Seek failed: {}", e);
                    } else {
                        current_time = target;
                        reference_start_time = None; // Reset clock on seek
                                                     // Send immediate update
                        let _ = window.emit(
                            "playback-update",
                            crate::engine::state::PlaybackPayload {
                                current_time,
                                duration,
                                status: crate::engine::state::PlaybackStatus::Buffering,
                            },
                        );
                    }
                }

                let decode_result = decoder.decode_next();
                let result = match decode_result {
                    Ok(Some(r)) => r,
                    Ok(None) => {
                        let _ = window.emit(
                            "playback-update",
                            crate::engine::state::PlaybackPayload {
                                current_time,
                                duration,
                                status: crate::engine::state::PlaybackStatus::Finished,
                            },
                        );
                        break; // EOF
                    }
                    Err(e) => {
                        eprintln!("[PlaybackEngine] Error during decoding: {}", e);
                        let _ = window.emit(
                            "playback-update",
                            crate::engine::state::PlaybackPayload {
                                current_time,
                                duration,
                                status: crate::engine::state::PlaybackStatus::Error,
                            },
                        );
                        break;
                    }
                };

                iteration_count += 1;
                if iteration_count % 100 == 0 {
                    log::debug!(
                        "[PlaybackEngine] Iteration {}, current_pts: {}",
                        iteration_count,
                        current_time
                    );
                }

                let mut should_emit_update = false;

                match result {
                    crate::engine::decoder::DecodeResult::Video {
                        data,
                        width,
                        height,
                        stride,
                        pts,
                    } => {
                        current_time = pts;
                        should_emit_update = true; // Always emit on video frame

                        let mut guard = renderer_clone.lock().unwrap();
                        if let Some(r) = guard.as_mut() {
                            let _ = r.render_frame(&data, width, height, stride);
                        }

                        // Dynamic Pacing
                        let mode = *sync_mode_clone.lock().unwrap();
                        match mode {
                            SyncMode::Fixed => {
                                std::thread::sleep(Duration::from_millis(30));
                            }
                            SyncMode::Realtime => {
                                if reference_start_time.is_none() {
                                    reference_start_time = Some(
                                        Instant::now()
                                            .checked_sub(Duration::from_secs_f64(current_time))
                                            .unwrap_or_else(Instant::now),
                                    );
                                }

                                let target_time = reference_start_time.unwrap()
                                    + Duration::from_secs_f64(current_time);
                                let now = Instant::now();
                                if target_time > now {
                                    std::thread::sleep(target_time - now);
                                }
                            }
                        }
                    }
                    crate::engine::decoder::DecodeResult::Audio { pts } => {
                        // Only update time from audio if there is NO video stream
                        if decoder.video_stream_index.is_none() {
                            current_time = pts;
                            should_emit_update = true;

                            // Dynamic Pacing for Audio-Only Mode
                            // We must sleep to keep the loop from racing ahead of real-time
                            let mode = *sync_mode_clone.lock().unwrap();
                            match mode {
                                SyncMode::Fixed => {
                                    // Audio doesn't have "frames" per se, but we can sleep a bit
                                    // to simulate ~60fps updates or similar.
                                    std::thread::sleep(Duration::from_millis(16));
                                }
                                SyncMode::Realtime => {
                                    if reference_start_time.is_none() {
                                        reference_start_time = Some(
                                            Instant::now()
                                                .checked_sub(Duration::from_secs_f64(current_time))
                                                .unwrap_or_else(Instant::now),
                                        );
                                    }

                                    let target_time = reference_start_time.unwrap()
                                        + Duration::from_secs_f64(current_time);
                                    let now = Instant::now();
                                    if target_time > now {
                                        std::thread::sleep(target_time - now);
                                    }
                                }
                            }
                        } else {
                            // If there is video, the video branch handles the sleeping.
                            // We just let audio pass through to the buffer.
                            std::thread::yield_now();
                        }
                    }
                }

                if !decoder.audio_buffer.is_empty() {
                    let samples_to_push = decoder.audio_buffer.len();
                    if let Ok(mut guard) = audio_producer_clone.lock() {
                        if let Some(ref mut producer) = *guard {
                            let pushed = producer.push_slice(&decoder.audio_buffer);
                            decoder.audio_buffer.drain(..pushed);

                            if iteration_count % 100 == 0 {
                                eprintln!(
                                    "[PlaybackEngine] Pushed {}/{} samples to audio producer",
                                    pushed, samples_to_push
                                );
                            }

                            // If buffer is very full, slow down slightly
                            if producer.len() > 24000 {
                                // ~0.5s of audio
                                std::thread::sleep(std::time::Duration::from_millis(5));
                            }
                        }
                    }
                }

                if !*playing_clone.lock().unwrap() {
                    let _ = window.emit(
                        "playback-update",
                        crate::engine::state::PlaybackPayload {
                            current_time,
                            duration,
                            status: crate::engine::state::PlaybackStatus::Paused,
                        },
                    );

                    while !*playing_clone.lock().unwrap() {
                        if *session_id_clone.lock().unwrap() != current_session {
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    reference_start_time = None; // Reset clock on resume
                }

                if should_emit_update {
                    let _ = window.emit(
                        "playback-update",
                        crate::engine::state::PlaybackPayload {
                            current_time,
                            duration,
                            status: crate::engine::state::PlaybackStatus::Playing,
                        },
                    );
                }
            }

            eprintln!(
                "[PlaybackEngine] Thread finished for session {}",
                current_session
            );
        });
    }
}
