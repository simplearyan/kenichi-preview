# 🎥 Kenichi Preview

**Kenichi Preview** is a high-performance, native-layer video player built with **Tauri v2**, **Rust**, **WGPU**, and **FFmpeg**. It bypasses the standard Webview video tag to achieve frame-accurate rendering and professional-grade performance through a direct-to-GPU pipeline.

![App Screenshot](file:///C:/Users/aryan/.gemini/antigravity/brain/c83ebed3-5d70-4fe3-afea-0869d3014031/uploaded_media_1769849973941.png)

## 🚀 Native Rendering Engine

The core of Kenichi Preview is its custom-built rendering engine:

- **WGPU Pipeline**: Direct rendering of decoded video frames into GPU textures using WGSL shaders. This ensures zero-copy overhead for the webview and perfect color accuracy.
- **FFmpeg Decoding**: High-efficiency frame extraction using `ffmpeg-next`. Supports a wide range of codecs and containers.
- **Hardware Agnostic**: Automatically detects and adjusts to hardware limits (e.g., WGPU device limits) for maximum compatibility across different GPUs.

## ✨ Features

- **Double-Click or Drag & Drop**: Easy file importing through native file dialogs or simple drag-drop gestures.
- **Smart Quality Mode**:
    - **Quality Focus**: Native resolution playback with full visual fidelity.
    - **Performance Mode**: Real-time 1/4 resolution downscaling and frame-discarding for smooth playback on low-end hardware.
- **Advanced Audio Engineering**:
    - **Native Audio Output**: Low-latency playback via `cpal` with ring-buffer architecture.
    - **Dynamic Resampling**: Automatic real-time parameter adjustment (sample rate/layout) for glitch-free playback of any audio format.
    - **Sample-Accurate Sync**: Precision playhead synchronization using sample counting (PTS) for audio-only files.
- **Pro UI / UX**:
    - **Glassmorphism Design**: Sleek "Pro Gray" interface with neon-yellow accents.
    - **Frameless Window**: Custom title bar and window controls for a native OS feel.
    - **Modern Stack**: React 19, Tailwind CSS v4, and Lucide Icons.

## 🛠️ Technical Implementation

- **Language**: Rust (Backend) + TypeScript (Frontend)
- **Framework**: Tauri v2, React 19
- **Graphics API**: WGPU (WGSL Shaders) - Zero-copy texture rendering
- **Audio API**: cpal (Output), swresample (Processing)
- **Multimedia**: FFmpeg (via `ffmpeg-next`)
- **Styling**: Tailwind CSS v4

## 📦 Getting Started

### Prerequisites
- **Rust Toolchain**: `rustc`, `cargo`
- **FFmpeg 6.x Shared Libraries**: Required for the `ffmpeg-next` build.
- **Node.js**: `npm` or `yarn`

### Setup & Run
```bash
# Clone the repository
git clone <repo-url>
cd KenichiPreview

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

## 📄 License
Internal Development - Kenichi Project.
