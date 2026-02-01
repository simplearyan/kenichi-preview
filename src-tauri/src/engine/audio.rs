use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioSession {
    _stream: Box<dyn StreamTrait>,
}

// CPAL Stream is not Send on some platforms (like Windows WASAPI) because of COM/COINIT.
// However, since we only store it to keep the audio alive and don't call methods on it
// after initialization, it is safe to manually implement Send and Sync to allow
// it to be part of the Tauri/Engine state.
unsafe impl Send for AudioSession {}
unsafe impl Sync for AudioSession {}

impl AudioSession {
    pub fn new(
        volume: Arc<Mutex<f32>>,
        mut consumer: ringbuf::HeapConsumer<f32>,
    ) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device
            .default_output_config()
            .expect("no default output config");

        let volume_clone = volume.clone();
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
                    if underrun_counter % 200 == 0 {
                        // Silent underrun unless frequent
                    }
                }
            },
            |err| eprintln!("audio stream error: {}", err),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            _stream: Box::new(stream),
        })
    }
}
