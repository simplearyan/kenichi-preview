# Kenichi Preview Roadmap

This document outlines the planned feature development for the Kenichi Preview application.

## Phase 1: Core Stability (Current Focus)
- [x] Basic Video Playback (WGPU + FFmpeg)
- [x] Hardware Acceleration (DirectX/Vulkan/Metal)
- [x] Play/Pause Controls
- [x] **Fix Crash on Load (Stride Alignment)**: Resolve `STATUS_ACCESS_VIOLATION` by ensuring video frame data is correctly padded to 256 bytes per row as required by WGPU.

## Phase 2: Enhanced Media Support
- [ ] **Audio Support**:
    - Implement audio decoding using FFmpeg.
    - Synchronize audio with video playback.
    - Add volume controls to the UI.
- [ ] **Image Support**:
    - Add support for opening and displaying static images (JPG, PNG, WEBP).
    - Implement zoom and pan controls for images.

## Phase 3: Advanced Features
- [ ] **Proxy Generation**:
    - Add option to generate low-resolution proxy files for smooth playback of 4K/8K content.
    - Implement background transcoding task.
- [ ] **Performance Settings**:
    - "Low Quality" preview mode (already partially implemented with resolution scaling).
    - Configurable cache settings.

## Phase 4: Polish & Production
- [ ] **Settings Integration**: Save user preferences (volume, quality, proxy settings).
- [ ] **Distribution**: Sign and notarize the application for Windows and macOS.
