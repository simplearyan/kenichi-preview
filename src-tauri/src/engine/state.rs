use super::types::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PreviewState {
    pub renderer: Arc<Mutex<Option<crate::engine::output::Renderer>>>,
    pub quality_mode: Arc<Mutex<QualityMode>>,
    pub is_playing: Arc<Mutex<bool>>,
    pub session_id: Arc<Mutex<u64>>,
    pub audio_producer: Arc<Mutex<Option<ringbuf::HeapProducer<f32>>>>,
    pub volume: Arc<std::sync::atomic::AtomicU32>,
    pub seek_target: Arc<Mutex<Option<f64>>>,
    pub sync_mode: Arc<Mutex<SyncMode>>,
}
