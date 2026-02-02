use serde::{Deserialize, Serialize};

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
pub enum SyncMode {
    Realtime, // Clock sync (Frame Accurate)
    Fixed,    // Dumb sleep (Fixed Step)
}
