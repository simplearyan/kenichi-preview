mod renderer;
mod decoder;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{State, Window, Manager};
use renderer::Renderer;
use decoder::Decoder;

pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<Renderer>>>,
    pub is_low_quality: Arc<Mutex<bool>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub session_id: Arc<Mutex<u64>>,
}

#[tauri::command]
async fn open_video(
    window: Window,
    state: State<'_, PreviewState>,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    let low_quality = *state.is_low_quality.lock().unwrap();

    // 1. Initialize Renderer if not already done
    let needs_init = state.renderer.lock().unwrap().is_none();
    
    if needs_init {
        let r = Renderer::new(Arc::new(window.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let mut renderer_guard = state.renderer.lock().unwrap();
        *renderer_guard = Some(r);
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
        let mut decoder = match Decoder::new(&path, low_quality) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder error: {}", e);
                return;
            }
        };

        while let Ok(Some(frame_info)) = decoder.decode_next_frame() {
            println!("Frame received in thread. Session: {}", *session_id_clone.lock().unwrap());
            
            // Check if this session is still valid
            if *session_id_clone.lock().unwrap() != current_session {
                println!("[Thread] Session {} ended.", current_session);
                break;
            }

            // Check if we are paused
            while !*playing_clone.lock().unwrap() {
                // Also check session during pause
                if *session_id_clone.lock().unwrap() != current_session {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let (frame_data, w, h, stride) = frame_info;
            let mut guard = renderer_clone.lock().unwrap();
            if let Some(r) = guard.as_mut() {
                if let Err(e) = r.render_frame(&frame_data, w, h, stride) {
                    eprintln!("Render error: {}", e);
                    // If surface is lost/outdated, it might need a resize event or just a reconfigure
                    // The renderer now handles reconfigure internally, so we just log it.
                } else {
                    println!("Render success.");
                }
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
fn toggle_quality(state: State<'_, PreviewState>, low_quality: bool) {
    let mut quality_guard = state.is_low_quality.lock().unwrap();
    *quality_guard = low_quality;
    // Note: In a real app, this would trigger a decoder re-init.
    // For this demo, the next video opened will use the new quality.
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(PreviewState {
            renderer: Arc::new(Mutex::new(None)),
            is_low_quality: Arc::new(Mutex::new(false)),
            is_playing: Arc::new(Mutex::new(false)),
            session_id: Arc::new(Mutex::new(0)),
        })
        .invoke_handler(tauri::generate_handler![open_video, toggle_quality, toggle_playback])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Resized(size) => {
                    let state = window.state::<PreviewState>();
                    if let Ok(mut guard) = state.renderer.lock() {
                        if let Some(r) = guard.as_mut() {
                            r.resize(*size);
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
