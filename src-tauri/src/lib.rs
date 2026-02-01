mod renderer;
mod decoder;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{State, Window, Manager, Emitter};
use renderer::Renderer;
use decoder::Decoder;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;

pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<Renderer>>>,
    pub quality_mode: Arc<Mutex<QualityMode>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub session_id: Arc<Mutex<u64>>,
    pub audio_producer: Arc<Mutex<Option<ringbuf::HeapProducer<f32>>>>,
    pub volume: Arc<Mutex<f32>>,
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
    let audio_producer_clone = state.audio_producer.clone();
    
    let current_session = {
        let mut s = state.session_id.lock().unwrap();
        *s += 1;
        *s
    };

    {
        let mut p = state.is_playing.lock().unwrap();
        *p = true; 
    }
    
    std::thread::spawn(move || {
        let mut decoder = match Decoder::new(&path, quality_mode) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder error: {}", e);
                return;
            }
        };

        let (duration, _, _) = decoder.get_metadata();

        while let Ok(Some(frame_info)) = decoder.decode_next_frame() {
            if *session_id_clone.lock().unwrap() != current_session {
                break;
            }

            // Sync: Audio samples are already in decoder.audio_buffer
            // We need to push them to the producer
            if !decoder.audio_buffer.is_empty() {
                if let Ok(mut guard) = audio_producer_clone.lock() {
                    if let Some(ref mut producer) = *guard {
                        let pushed = producer.push_slice(&decoder.audio_buffer);
                        decoder.audio_buffer.drain(..pushed);
                    }
                }
            }

            while !*playing_clone.lock().unwrap() {
                if *session_id_clone.lock().unwrap() != current_session {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let (frame_data, w, h, stride, current_time) = frame_info;
            
            let _ = window.emit("playback-update", PlaybackPayload {
                current_time,
                duration,
            });

            let mut guard = renderer_clone.lock().unwrap();
            if let Some(r) = guard.as_mut() {
                let _ = r.render_frame(&frame_data, w, h, stride);
            }
            // AV Sync TODO: Use audio clock instead of fixed sleep
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
fn set_volume(state: State<'_, PreviewState>, volume: f32) {
    let mut v = state.volume.lock().unwrap();
    *v = volume.clamp(0.0, 1.0);
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
        // eprintln!("[Backend] Viewport updated: {}x{} at {},{}", width, height, x, y);
        let _ = r.repaint(); 
    }
}

#[tauri::command]
fn set_aspect_ratio(state: State<'_, PreviewState>, mode: renderer::AspectMode) {
    let mut renderer_guard = state.renderer.lock().unwrap();
    if let Some(r) = renderer_guard.as_mut() {
        eprintln!("[Backend] Aspect mode changed to: {:?}", mode);
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

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Audio Setup
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().expect("no default output config");
    
    // Create RingBuffer for 1 second of audio (48000 samples * 2 channels)
    let rb = HeapRb::<f32>::new(96000);
    let (producer, mut consumer) = rb.split();
    
    let volume_state = Arc::new(Mutex::new(1.0f32));
    let volume_clone = volume_state.clone();

    let mut underrun_counter = 0;
    
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let vol = *volume_clone.lock().unwrap();
            let mut underrun_occurred = false;
            for sample in data.iter_mut() {
                match consumer.pop() {
                    Some(s) => *sample = s * vol,
                    None => {
                        *sample = 0.0;
                        underrun_occurred = true;
                    }
                }
            }
            if underrun_occurred {
                underrun_counter += 1;
                if underrun_counter % 100 == 0 {
                    eprintln!("[Audio] Buffer Underrun detected (total: {})", underrun_counter);
                }
            }
        },
        |err| eprintln!("audio stream error: {}", err),
        None
    ).expect("failed to build audio stream");

    stream.play().expect("failed to start audio stream");

    // Prevent stream from being dropped
    Box::leak(Box::new(stream));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(PreviewState {
            renderer: Arc::new(Mutex::new(None)),
            quality_mode: Arc::new(Mutex::new(QualityMode::Native)),
            is_playing: Arc::new(Mutex::new(false)),
            session_id: Arc::new(Mutex::new(0)),
            audio_producer: Arc::new(Mutex::new(Some(producer))),
            volume: volume_state,
        })
        .invoke_handler(tauri::generate_handler![
            open_video,
            set_quality,
            set_volume,
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
