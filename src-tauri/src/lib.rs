mod commands;
mod engine;

use engine::Engine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = Engine::new();

    // Initialize Audio (Try)
    if let Err(e) = engine.init_audio() {
        eprintln!("[Audio] Failed to initialize: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::open_video,
            commands::set_quality,
            commands::set_volume,
            commands::toggle_playback,
            commands::update_viewport,
            commands::init_renderer,
            commands::set_aspect_ratio,
            commands::get_app_cache_dir
        ])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        let engine = window.state::<Engine>();
                        if let Ok(mut guard) = engine.state.renderer.lock() {
                            if let Some(r) = guard.as_mut() {
                                r.resize(*size);
                                let _ = r.repaint();
                            }
                        };
                    }
                }
                tauri::WindowEvent::Moved(_) => {
                    // Repaint on move to ensure smooth transition
                    let engine = window.state::<Engine>();
                    if let Ok(mut guard) = engine.state.renderer.lock() {
                        if let Some(r) = guard.as_mut() {
                            let _ = r.repaint();
                        }
                    };
                }
                tauri::WindowEvent::ScaleFactorChanged { .. } => {
                    // Repaint on scale factor change
                    let engine = window.state::<Engine>();
                    if let Ok(mut guard) = engine.state.renderer.lock() {
                        if let Some(r) = guard.as_mut() {
                            let _ = r.repaint();
                        }
                    };
                }
                tauri::WindowEvent::Destroyed => {
                    let engine = window.state::<Engine>();
                    // Stop Playback
                    if let Ok(mut playing) = engine.state.is_playing.lock() {
                        *playing = false;
                    };
                    // Invalidate Session
                    if let Ok(mut session) = engine.state.session_id.lock() {
                        *session += 1;
                    };
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
