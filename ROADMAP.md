# Kenichi Preview Roadmap

This document outlines the planned feature development for the Kenichi Preview application.

## Phase 1: Core Stability
- [x] Basic Video Playback (WGPU + FFmpeg)
- [x] Hardware Acceleration (DirectX/Vulkan/Metal/OpenGL)
- [x] Play/Pause Controls
- [x] **Fix Crash on Load (Stride Alignment)**: Resolved via row padding.
- [x] **Fix Crash on Older GPUs (GT 730)**: Downgraded to WGPU 0.19 and blocked unstable backends (DX12).
- [x] **Fix Video Geometry**: Corrected degenerate triangles and shader math.

## Phase 2: UI & Layout Redesign (Current Focus)
- [ ] **Transparency Fix**: Remove semi-opaque backgrounds to make the WGPU layer fully visible.
- [ ] **Sidebar Layout**: Implement a three-pane layout (Sidebar | Preview | Controls) inspired by KenichiConverter.
- [ ] **Playlist Management**: Support importing and switching between multiple video/image files.

## Phase 3: Enhanced Media Support
- [ ] **Audio Support**:
    - Implement audio decoding using FFmpeg.
    - Synchronize audio with video playback.
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
