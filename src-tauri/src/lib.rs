mod renderer;
mod decoder;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{State, Window, Manager, Emitter};
use renderer::Renderer;
use decoder::Decoder;

pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<Renderer>>>,
    pub quality_mode: Arc<Mutex<QualityMode>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub session_id: Arc<Mutex<u64>>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum QualityMode {
    Native, // 100% resolution
    Fast,   // 50% resolution
    Proxy,  // 25% resolution
}

#[derive(Clone, serde::Serialize)]
struct PlaybackPayload {
    current_time: f64,
    duration: f64,
}

#[tauri::command]
async fn open_video(
    window: Window,
    state: State<'_, PreviewState>,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    let quality_mode = *state.quality_mode.lock().unwrap();

    // 1. Initialize Renderer if not already done
    {
        let needs_init = state.renderer.lock().unwrap().is_none();
        if needs_init {
            let r = Renderer::new(Arc::new(window.clone()))
                .await
                .map_err(|e| e.to_string())?;
            let mut guard = state.renderer.lock().unwrap();
            *guard = Some(r);
        }
    }

    // Repaint to ensure background/viewport is synced
    if let Some(r) = state.renderer.lock().unwrap().as_mut() {
        let _ = r.repaint();
    }

    // 2. Start Playback Loop with session tracking
    let renderer_clone = state.renderer.clone();
    let playing_clone = state.is_playing.clone();
    let session_id_clone = state.session_id.clone();
    
    let current_session = {
        let mut s = state.session_id.lock().unwrap();
        *s += 1;
        *s
    };

    {
        let mut p = state.is_playing.lock().unwrap();
        *p = true; // Set to playing by default when opened
    }
    
    std::thread::spawn(move || {
        // Initialize Decoder
        let mut decoder = match Decoder::new(&path, quality_mode) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder error: {}", e);
                return;
            }
        };

        let (duration, _, _) = decoder.get_metadata();

        while let Ok(Some(frame_info)) = decoder.decode_next_frame() {
            // Check if this session is still valid
            if *session_id_clone.lock().unwrap() != current_session {
                break;
            }

            // Check if we are paused
            while !*playing_clone.lock().unwrap() {
                if *session_id_clone.lock().unwrap() != current_session {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let (frame_data, w, h, stride, current_time) = frame_info;
            
            // Emit progress to frontend
            let _ = window.emit("playback-update", PlaybackPayload {
                current_time,
                duration,
            });

            let mut guard = renderer_clone.lock().unwrap();
            if let Some(r) = guard.as_mut() {
                let _ = r.render_frame(&frame_data, w, h, stride);
            }
            // TODO: Use actual FPS from decoder
            std::thread::sleep(std::time::Duration::from_millis(30)); 
        }
    });

    Ok(())
}

#[tauri::command]
fn toggle_playback(state: State<'_, PreviewState>) -> bool {
    let mut playing_guard = state.is_playing.lock().unwrap();
    *playing_guard = !*playing_guard;
    *playing_guard
}

#[tauri::command]
fn set_quality(state: State<'_, PreviewState>, mode: QualityMode) {
    let mut quality_guard = state.quality_mode.lock().unwrap();
    *quality_guard = mode;
}

#[tauri::command]
fn update_viewport(
    state: State<'_, PreviewState>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let mut renderer_guard = state.renderer.lock().unwrap();
    if let Some(r) = renderer_guard.as_mut() {
        r.set_viewport(x, y, width, height);
        let _ = r.repaint(); // Redraw with current viewport clipping
    }
}

#[tauri::command]
fn set_aspect_ratio(state: State<'_, PreviewState>, mode: renderer::AspectMode) {
    let mut renderer_guard = state.renderer.lock().unwrap();
    if let Some(r) = renderer_guard.as_mut() {
        r.set_aspect_mode(mode);
        let _ = r.repaint();
    }
}

#[tauri::command]
async fn init_renderer(window: Window, state: State<'_, PreviewState>) -> Result<(), String> {
    let needs_init = state.renderer.lock().unwrap().is_none();
    if needs_init {
        let r = Renderer::new(Arc::new(window)).await.map_err(|e| e.to_string())?;
        let mut renderer_guard = state.renderer.lock().unwrap();
        *renderer_guard = Some(r);
        if let Some(r) = renderer_guard.as_mut() {
            let _ = r.repaint();
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(PreviewState {
            renderer: Arc::new(Mutex::new(None)),
            quality_mode: Arc::new(Mutex::new(QualityMode::Native)),
            is_playing: Arc::new(Mutex::new(false)),
            session_id: Arc::new(Mutex::new(0)),
        })
        .invoke_handler(tauri::generate_handler![
            open_video,
            set_quality,
            toggle_playback,
            update_viewport,
            init_renderer,
            set_aspect_ratio
        ])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Resized(size) => {
                    let state = window.state::<PreviewState>();
                    if let Ok(mut guard) = state.renderer.lock() {
                        if let Some(r) = guard.as_mut() {
                            r.resize(*size);
                            let _ = r.repaint();
                        }
                    };
                }
                tauri::WindowEvent::Destroyed => {
                    let state = window.state::<PreviewState>();
                    // Stop Playback
                    if let Ok(mut playing) = state.is_playing.lock() {
                        *playing = false;
                    };
                    // Invalidate Session
                    if let Ok(mut session) = state.session_id.lock() {
                        *session += 1;
                    };
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
