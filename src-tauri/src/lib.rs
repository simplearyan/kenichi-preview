mod renderer;
mod decoder;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{State, Window};
use renderer::Renderer;
use decoder::Decoder;

pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<Renderer>>>,
    pub is_low_quality: Arc<Mutex<bool>>,
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

    // 2. Start Playback Loop in a background thread
    let renderer_clone = state.renderer.clone();
    
    std::thread::spawn(move || {
        // Initialize Decoder INSIDE the thread (fixes !Send issues)
        let mut decoder = match Decoder::new(&path, low_quality) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder error: {}", e);
                return;
            }
        };

        while let Ok(Some(frame_data)) = decoder.decode_next_frame() {
            let (w, h) = decoder.get_dimensions();
            let mut guard = renderer_clone.lock().unwrap();
            if let Some(r) = guard.as_mut() {
                if let Err(e) = r.render_frame(&frame_data, w, h) {
                    eprintln!("Render error: {}", e);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(30)); 
        }
    });

    Ok(())
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
        })
        .invoke_handler(tauri::generate_handler![open_video, toggle_quality])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
