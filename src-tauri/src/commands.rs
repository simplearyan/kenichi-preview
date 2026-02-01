use tauri::{State, Window, Emitter, Manager};
use crate::engine::{Engine, QualityMode, AspectMode};
use std::path::PathBuf;
use std::sync::Arc;

#[tauri::command]
pub async fn open_video(
    window: Window,
    engine: State<'_, Engine>,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    let quality_mode = *engine.state.quality_mode.lock().unwrap();

    // 1. Initialize Renderer if not already done
    {
        let needs_init = engine.state.renderer.lock().unwrap().is_none();
        if needs_init {
            let r = crate::engine::renderer::Renderer::new(Arc::new(window.clone()))
                .await
                .map_err(|e| e.to_string())?;
            let mut guard = engine.state.renderer.lock().unwrap();
            *guard = Some(r);
        }
    }

    if let Some(r) = engine.state.renderer.lock().unwrap().as_mut() {
        let _ = r.repaint();
    }

    let renderer_clone = engine.state.renderer.clone();
    let playing_clone = engine.state.is_playing.clone();
    let session_id_clone = engine.state.session_id.clone();
    let audio_producer_clone = engine.state.audio_producer.clone();
    
    let current_session = {
        let mut s = engine.state.session_id.lock().unwrap();
        *s += 1;
        *s
    };

    {
        let mut p = engine.state.is_playing.lock().unwrap();
        *p = true; 
    }

    std::thread::spawn(move || {
        let mut decoder = match crate::engine::decoder::Decoder::new(&path, quality_mode) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder error: {}", e);
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

        let mut current_time;
        let (duration, _, _) = decoder.get_metadata();

        while let Ok(Some(result)) = decoder.decode_next() {
            if *session_id_clone.lock().unwrap() != current_session {
                break;
            }

            match result {
                crate::engine::decoder::DecodeResult::Video { data, width, height, stride, pts } => {
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
                if let Ok(mut guard) = audio_producer_clone.lock() {
                    if let Some(ref mut producer) = *guard {
                        let pushed = producer.push_slice(&decoder.audio_buffer);
                        decoder.audio_buffer.drain(..pushed);
                        
                        // If buffer is very full, slow down slightly
                        if producer.len() > 24000 { // ~0.5s of audio
                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                    }
                }
            }

            while !*playing_clone.lock().unwrap() {
                if *session_id_clone.lock().unwrap() != current_session {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let _ = window.emit("playback-update", crate::engine::state::PlaybackPayload {
                current_time,
                duration,
            });
        }

        // Keep session alive for static images or just to show the last frame
        while *session_id_clone.lock().unwrap() == current_session {
             std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    Ok(())
}

#[tauri::command]
pub fn toggle_playback(engine: State<'_, Engine>) -> bool {
    let mut playing_guard = engine.state.is_playing.lock().unwrap();
    *playing_guard = !*playing_guard;
    *playing_guard
}

#[tauri::command]
pub fn set_quality(engine: State<'_, Engine>, mode: QualityMode) {
    let mut quality_guard = engine.state.quality_mode.lock().unwrap();
    *quality_guard = mode;
}

#[tauri::command]
pub fn set_volume(engine: State<'_, Engine>, volume: f32) {
    let mut v = engine.state.volume.lock().unwrap();
    *v = volume.clamp(0.0, 1.0);
}

#[tauri::command]
pub fn update_viewport(
    engine: State<'_, Engine>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let mut renderer_guard = engine.state.renderer.lock().unwrap();
    if let Some(r) = renderer_guard.as_mut() {
        r.set_viewport(x, y, width, height);
        let _ = r.repaint(); 
    }
}

#[tauri::command]
pub fn set_aspect_ratio(engine: State<'_, Engine>, mode: AspectMode) {
    let mut renderer_guard = engine.state.renderer.lock().unwrap();
    if let Some(r) = renderer_guard.as_mut() {
        r.set_aspect_mode(mode);
        let _ = r.repaint();
    }
}

#[tauri::command]
pub async fn init_renderer(window: Window, engine: State<'_, Engine>) -> Result<(), String> {
    let needs_init = engine.state.renderer.lock().unwrap().is_none();
    if needs_init {
        let r = crate::engine::renderer::Renderer::new(Arc::new(window)).await.map_err(|e| e.to_string())?;
        let mut renderer_guard = engine.state.renderer.lock().unwrap();
        *renderer_guard = Some(r);
        if let Some(r) = renderer_guard.as_mut() {
            let _ = r.repaint();
        }
    }
    Ok(())
}
#[tauri::command]
pub fn get_app_cache_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
