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

## Challenge 4: Video Not Visible (Black Screen)
**Issue**: Video was rendering but not visible in the UI. Rendering confirm logs were present.

**Investigation**:
- Found that the original vertex shader math for the full-screen quad was degenerate.
- The Tauri/Webview layer had semi-opaque backgrounds that were dimming/blocking the WGPU layer.

**Resolution**:
- Implemented a high-performance **full-screen triangle** optimization in `shader.wgsl` (3 vertices instead of 6).
- Removed semi-opaque backgrounds from the frontend, ensuring the "preview area" is fully transparent.

## Challenge 5: Viewport Bleeding & Layout Alignment
**Issue**: The WGPU video layer was drawing over the entire window background, appearing behind the sidebar and title bar.

**Resolution**:
- Implemented **WGPU Viewport Constraints**.
- Frontend uses `getBoundingClientRect()` and `devicePixelRatio` to calculate the exact physical pixel coordinates of the preview area.
- Rust backend exposes an `update_viewport` command to dynamically clip WGPU rendering to the intended container.

## Challenge 6: Color Unsaturation (sRGB Mismatch)
**Issue**: Video preview appeared washed out/unsaturated compared to dedicated media players.

**Investigation**:
- Screenshots compared against reference players confirmed a significant loss in color depth and saturation.
- Diagnosed as a **Color Space Mismatch**: FFmpeg decodes to sRGB values, but the WGPU texture was initialized as `Rgba8Unorm`. When the shader outputted these to an `Srgb` surface, the hardware attempted to linearize already-gamma-corrected values, leading to a "washed out" look.

**Resolution**:
- Updated the video texture format to `Rgba8UnormSrgb` to enable automatic hardware-level conversion from sRGB to Linear space on texture sampling.
- This ensures the shader works with mathematically correct intensities, and the surface correctly handles the final sRGB conversion for display.
