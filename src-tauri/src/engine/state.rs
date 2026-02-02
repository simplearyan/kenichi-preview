use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum QualityMode {
    Native, // 100% resolution
    Fast,   // 50% resolution
    Proxy,  // 25% resolution
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Buffering,
    Finished,
    Error,
}

#[derive(Clone, Serialize)]
pub struct PlaybackPayload {
    pub current_time: f64,
    pub duration: f64,
    pub status: PlaybackStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum AspectMode {
    Fit,     // Letterbox/Pillarbox based on video ratio
    Stretch, // Fill the container (original behavior)
    Cinema,  // 21:9
    Classic, // 4:3
    Wide,    // 16:9
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone)]
pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<crate::engine::renderer::Renderer>>>,
    pub quality_mode: Arc<Mutex<QualityMode>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub session_id: Arc<Mutex<u64>>,
    pub audio_producer: Arc<Mutex<Option<ringbuf::HeapProducer<f32>>>>,
    pub volume: Arc<Mutex<f32>>,
    pub seek_target: Arc<Mutex<Option<f64>>>,
}
