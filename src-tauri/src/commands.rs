use tauri::{State, Window, Manager};
use crate::engine::{Engine, QualityMode, AspectMode, SyncMode};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[tauri::command]
pub async fn open_video(
    window: Window,
    engine: State<'_, Engine>,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(path);

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

    let session_id_clone = engine.state.session_id.clone();
    
    // Increment session ID to stop any previous playback
    {
        let mut s = session_id_clone.lock().unwrap();
        *s += 1;
    }

    {
        let mut p = engine.state.is_playing.lock().unwrap();
        *p = true; 
    }

    let playback_engine = crate::engine::playback::PlaybackEngine::new(engine.state.clone(), window);
    playback_engine.playback_thread(path);

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
pub fn set_sync_mode(engine: State<'_, Engine>, mode: SyncMode) {
    eprintln!("[Command] Setting SyncMode to {:?}", mode);
    let mut guard = engine.state.sync_mode.lock().unwrap();
    *guard = mode;
}

#[tauri::command]
pub fn seek_video(engine: State<'_, Engine>, time: f64) {
    eprintln!("[Command] seek_video requested to {}s", time);
    let mut guard = engine.state.seek_target.lock().unwrap();
    *guard = Some(time);
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
pub fn set_volume(engine: State<'_, Engine>, volume: f32) {
    eprintln!("[Command] Setting Volume: {}", volume);
    // Store as scaled u32 (x1000)
    let vol_int = (volume.clamp(0.0, 1.0) * 1000.0) as u32;
    engine.state.volume.store(vol_int, Ordering::Relaxed);
}
#[tauri::command]
pub fn get_app_cache_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
