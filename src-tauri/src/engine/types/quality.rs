use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum QualityMode {
    Native, // 100% resolution
    Fast,   // 50% resolution
    Proxy,  // 25% resolution
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum AspectMode {
    Fit,     // Letterbox/Pillarbox based on video ratio
    Stretch, // Fill the container (original behavior)
    Cinema,  // 21:9
    Classic, // 4:3
    Wide,    // 16:9
}
