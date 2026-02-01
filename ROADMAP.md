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
- [x] **Transparency Fix**: Implemented fully transparent Webview on top of solid WGPU backend.
- [x] **Solid Landing Background**: WGPU initializes immediately on startup to provide a solid background.
- [x] **Sidebar Layout**: Implemented a three-pane layout (Sidebar | Preview | Controls) with glassmorphism.
- [x] **Playlist Management**: Support for importing multiple files and switching between them.
- [x] **Dynamic Viewports**: Video now clips perfectly to the designated preview area.
- [x] **Color Accuracy**: sRGB texture format used for vibrant, accurate colors.

## Phase 3: Media Polish & Controls (Current Focus)
- [x] **Playhead & Duration Display**:
    - Implement real-time playback time and total duration display in the footer.
    - Synchronize state between Rust backend and React frontend.
- [x] **Aspect Ratio Control**:
    - [x] Implement native letterboxing/pillarboxing in WGPU.
    - [x] Support 16:9, 4:3, 21:9, and Original Fit modes.
    - [x] Added dynamic scaling UI toggle.
- [x] **Quality Selection Dropdown**:
    - [x] Implement three-tier quality modes: Native (100%), Fast (50%), Proxy (25%).
    - [x] Add sleek cycle-button UI in the footer.
    - [x] Automatic video reload on quality change for instant effect.

## Phase 4: Audio & Multi-Media Support
- [ ] **Audio Support**:
    - Implement audio decoding using FFmpeg.
    - Synchronize audio with video playback.
    - Add volume controls to the UI.
- [ ] **Media Card Thumbnails**:
    - Generate background thumbnails for media gallery using sidecar FFmpeg.
    - Implement caching into a local `.kenichi` cache folder.
- [ ] **Image Support**:
    - Add support for opening and displaying static images (JPG, PNG, WEBP).
    - Implement zoom and pan controls for images.

## Phase 5: Advanced Editing & Export
- [ ] **Text & Subtitle Overlays**:
    - Add ability to overlay text and subtitles on the preview.
    - Support for font selection and basic styling.
- [ ] **Production Export**:
    - Use sidecar FFmpeg for high-fidelity rendering and export.
    - Burn text/subtitle overlays into the exported video using FFmpeg filtergraphs.
    - Implement progress tracking for export tasks in the Sidebar.

## Phase 6: Polish & Production
- [ ] **Settings Integration**: Save user preferences (volume, quality, proxy settings).
- [ ] **Distribution**: Sign and notarize the application for Windows and macOS.
