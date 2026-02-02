use super::state::PreviewState;
use std::path::PathBuf;
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
                    eprintln!("[PlaybackEngine] Seeking to {}s", target);
                    if let Err(e) = decoder.seek(target) {
                        eprintln!("[PlaybackEngine] Seek failed: {}", e);
                    } else {
                        current_time = target;
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
                    eprintln!(
                        "[PlaybackEngine] Iteration {}, current_pts: {}",
                        iteration_count, current_time
                    );
                }

                match result {
                    crate::engine::decoder::DecodeResult::Video {
                        data,
                        width,
                        height,
                        stride,
                        pts,
                    } => {
                        current_time = pts;
                        let mut guard = renderer_clone.lock().unwrap();
                        if let Some(r) = guard.as_mut() {
                            let _ = r.render_frame(&data, width, height, stride);
                        }
                        // Only sleep for video pacing
                        std::thread::sleep(std::time::Duration::from_millis(30));
                    }
                    crate::engine::decoder::DecodeResult::Audio { pts } => {
                        current_time = pts;
                        // Don't sleep here, but yield to prevent CPU pinning if buffer is full
                        std::thread::yield_now();
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
                }

                let _ = window.emit(
                    "playback-update",
                    crate::engine::state::PlaybackPayload {
                        current_time,
                        duration,
                        status: crate::engine::state::PlaybackStatus::Playing,
                    },
                );
            }

            eprintln!(
                "[PlaybackEngine] Thread finished for session {}",
                current_session
            );
        });
    }
}
