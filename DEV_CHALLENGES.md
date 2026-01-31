# Dev Challenges & Resolutions - Kenichi Preview

## Challenge 1: `STATUS_ACCESS_VIOLATION` (0xc0000005) on Older Hardware
**Issue**: Application crashes immediately upon initialization or when starting video playback on older GPUs like the NVIDIA GeForce GT 730.

**Investigation**:
- Detailed unbuffered logging revealed the crash occurred during WGPU initialization, specifically at `surface.configure` or when requesting a device.
- Log analysis on GT 730 showed it was defaulting to the **DX12** backend, which is often unstable on older Kepler/Fermi architectures.

**Resolution**:
- Downgraded `wgpu` to `0.19.0` to match known stable configurations from legacy projects (`Kenichi-legacy`).
- Explicitly filtered out the DX12 backend using `Backends::all() & !Backends::DX12`, forcing the app to use DX11 or OpenGL (Gl).
- Sanitized surface dimensions using `.max(1)` to prevent zero-size configuration crashes.

## Challenge 2: WGPU Stride Alignment
**Issue**: Video textures appeared skewed or caused internal WGPU errors.

**Resolution**:
- Implemented stride alignment logic in `decoder.rs` to ensure frame data rows are multiples of 256 bytes, as required by `wgpu::Queue::write_texture`.
- Used `copy_from_slice` to pack raw FFmpeg frames into padded buffers.

## Challenge 3: WGPU API Version Mismatches
**Issue**: Downgrading WGPU caused multiple compilation errors due to breaking changes in newer versions (like `MemoryHints`, `entry_point` being an `Option`, and the `cache` field).

**Resolution**:
- Refactored `renderer.rs` to align with the 0.19 API (e.g., changing `entry_point: Some("vs_main")` to `entry_point: "vs_main"`).
- Removed fields that didn't exist in 0.19.

## Current Challenge: Video Not Visible in UI
**Status**: Rendering logs show "Render success", but the UI displays a black screen.
**Plan**: Investigating the render pipeline, shader bindings, and surface presentation logic.
