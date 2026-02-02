pub mod audio;
pub mod decoder;
pub mod playback;
pub mod renderer;
pub mod state;
pub mod sync;

pub use state::*;
use std::sync::{Arc, Mutex};

pub struct Engine {
    pub state: PreviewState,
    pub _audio_session: Arc<Mutex<Option<audio::AudioSession>>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: PreviewState {
                renderer: Arc::new(Mutex::new(None)),
                quality_mode: Arc::new(Mutex::new(QualityMode::Native)),
                is_playing: Arc::new(Mutex::new(false)),
                session_id: Arc::new(Mutex::new(0)),
                audio_producer: Arc::new(Mutex::new(None)),
                volume: Arc::new(Mutex::new(1.0)),
                seek_target: Arc::new(Mutex::new(None)),
                sync_mode: Arc::new(Mutex::new(SyncMode::Realtime)),
            },
            _audio_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_audio(&self) -> anyhow::Result<()> {
        use ringbuf::HeapRb;

        // Create RingBuffer for 1 second of audio (48000 samples * 2 channels)
        let rb = HeapRb::<f32>::new(96000);
        let (producer, consumer) = rb.split();

        let session = audio::AudioSession::new(self.state.volume.clone(), consumer)?;

        // Connect the producer to the engine state for the decoder to use
        let mut producer_guard = self.state.audio_producer.lock().unwrap();
        *producer_guard = Some(producer);

        let mut session_guard = self._audio_session.lock().unwrap();
        *session_guard = Some(session);

        Ok(())
    }
}
