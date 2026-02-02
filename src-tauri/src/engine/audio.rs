use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::Ordering;
use std::sync::Arc;

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
        volume: Arc<std::sync::atomic::AtomicU32>,
        mut consumer: ringbuf::HeapConsumer<f32>,
    ) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
        log::info!("[Audio] Output device selected: {}", device_name);

        let config = device
            .default_output_config()
            .expect("no default output config");

        let volume_clone = volume.clone();
        let mut underrun_counter = 0;
        let mut callback_counter = 0u64;

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Lock-free volume read
                let vol_int = volume_clone.load(Ordering::Relaxed);
                let vol = vol_int as f32 / 1000.0;
                let mut underrun_occurred = false;

                // Diagnostic: Log occasionally to prove audio thread is alive
                callback_counter += 1;
                if callback_counter % 200 == 0 { // Approx every ~2-4 seconds depending on buffer size
                     // log::debug!("[Audio] Callback active. Volume: {}", vol);
                     // Note: println in realtime thread is bad practice, but okay for temporary debug
                }

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
                        eprintln!("[Audio] Buffer Underrun (Starvation) x{}", underrun_counter);
                    }
                }
            },
            |err| eprintln!("[Audio] Stream error: {}", err),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            _stream: Box::new(stream),
        })
    }
}
