use wgpu::*;
use std::sync::Arc;

pub struct Renderer {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: tauri::PhysicalSize<u32>,
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub video_texture: Option<Texture>,
    pub video_texture_view: Option<TextureView>,
    pub video_bind_group: Option<BindGroup>,
    // Buffer to handle stride alignment padding
    pub padding_buffer: Vec<u8>,
    pub container_viewport: Option<Rect>,
    pub current_aspect_mode: AspectMode,
    pub last_video_size: Option<(u32, u32)>,
}

use super::state::{AspectMode, Rect};

impl Renderer {
    pub async fn new(window: Arc<tauri::Window>) -> anyhow::Result<Self> {
        eprintln!("[Renderer] initializing...");
        let size = window.inner_size()?;

        // Legacy Fix with DX12 Exclusion
        // GT 730 crashes on DX12. We explicitly filter it out.
        // This allows DX11 (implicitly), Vulkan, or GL.
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all() & !Backends::DX12,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;
        eprintln!("[Renderer] Surface created. Requesting adapter...");

        eprintln!("[Renderer] Requesting adapter (DX11 Preferred)...");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None, 
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;

        eprintln!("[Renderer] Adapter found: {:?}", adapter.get_info());
        let limits = adapter.limits();
        eprintln!("[Renderer] Adapter limits determined: {:?}", limits);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: limits,
                },
                None,
            )
            .await?;
        eprintln!("[Renderer] Device requested, Queue ready.");

        // Ensure non-zero dimensions to prevent surface configuration crashes
        let width = size.width.max(1);
        let height = size.height.max(1);
        eprintln!("[Renderer] Configuring surface with size: {}x{}", width, height);

        let caps = surface.get_capabilities(&adapter);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        eprintln!("[Renderer] Surface configured.");

        // Load Shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Video Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        eprintln!("[Renderer] Shader module created.");

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Video Bind Group Layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Video Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Video Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });
        eprintln!("[Renderer] Render pipeline created.");

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        eprintln!("[Renderer] Initialization complete.");
        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            bind_group_layout,
            sampler,
            video_texture: None,
            video_texture_view: None,
            video_bind_group: None,
            padding_buffer: Vec::new(),
            container_viewport: None,
            current_aspect_mode: AspectMode::Fit,
            last_video_size: None,
        })
    }

    pub fn resize(&mut self, new_size: tauri::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.container_viewport = Some(Rect { x, y, width, height });
    }

    pub fn set_aspect_mode(&mut self, mode: AspectMode) {
        self.current_aspect_mode = mode;
    }

    fn calculate_actual_viewport(&self) -> Option<Rect> {
        let container = self.container_viewport?;
        let (v_w, v_h) = self.last_video_size?;

        if v_w == 0 || v_h == 0 {
            return Some(container);
        }

        match self.current_aspect_mode {
            AspectMode::Stretch => Some(container),
            _ => {
                let target_ratio = match self.current_aspect_mode {
                    AspectMode::Fit => (v_w as f32) / (v_h as f32),
                    AspectMode::Cinema => 21.0 / 9.0,
                    AspectMode::Classic => 4.0 / 3.0,
                    AspectMode::Wide => 16.0 / 9.0,
                    AspectMode::Stretch => unreachable!(),
                };

                let container_ratio = container.width / container.height;

                let (final_w, final_h) = if container_ratio > target_ratio {
                    // Container is wider than video ratio -> Pillarbox
                    let h = container.height;
                    let w = h * target_ratio;
                    (w, h)
                } else {
                    // Container is taller than video ratio -> Letterbox
                    let w = container.width;
                    let h = w / target_ratio;
                    (h, w); // WAIT, I swapped h and w in my head
                    (w, h)
                };

                Some(Rect {
                    x: container.x + (container.width - final_w) / 2.0,
                    y: container.y + (container.height - final_h) / 2.0,
                    width: final_w,
                    height: final_h,
                })
            }
        }
    }

    pub fn render_frame(&mut self, rgba_data: &[u8], width: u32, height: u32, stride: u32) -> anyhow::Result<()> {
        self.last_video_size = Some((width, height));
        if width == 0 || height == 0 || stride == 0 {
            return Ok(());
        }
        
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(SurfaceError::Outdated) | Err(SurfaceError::Lost) => {
                eprintln!("[Renderer] Surface outdated/lost, reconfiguring...");
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[Renderer] Surface error: {:?}", e);
                return Err(e.into());
            }
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let texture_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // 1. Reuse or Create Texture
        let needs_new_texture = self.video_texture.as_ref().map_or(true, |t| {
            t.width() != width || t.height() != height
        });

        if needs_new_texture {
            let texture = self.device.create_texture(&TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label: Some("Video Frame"),
                view_formats: &[],
            });
            
            let texture_view = texture.create_view(&TextureViewDescriptor::default());
            let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("Video Bind Group"),
            });

            self.video_texture = Some(texture);
            self.video_texture_view = Some(texture_view);
            self.video_bind_group = Some(bind_group);
        }

        // 2. Upload Data
        if let Some(texture) = &self.video_texture {
            let unaligned_bytes_per_row = stride;
            let align = 256;
            let aligned_bytes_per_row = (unaligned_bytes_per_row + align - 1) & !(align - 1);

            if unaligned_bytes_per_row == aligned_bytes_per_row {
                self.queue.write_texture(
                    ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    rgba_data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                    texture_size,
                );
            } else {
                let required_size = (aligned_bytes_per_row * height) as usize;
                if self.padding_buffer.len() < required_size {
                    self.padding_buffer.resize(required_size, 0);
                }

                for y in 0..height {
                    let src_start = (y * unaligned_bytes_per_row) as usize;
                    let src_end = src_start + (width * 4) as usize;
                    let dst_start = (y * aligned_bytes_per_row) as usize;
                    
                    if src_end <= rgba_data.len() && (dst_start + (width * 4) as usize) <= self.padding_buffer.len() {
                        self.padding_buffer[dst_start..dst_start + (width * 4) as usize]
                            .copy_from_slice(&rgba_data[src_start..src_end]);
                    }
                }

                self.queue.write_texture(
                    ImageCopyTexture {
                        texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    &self.padding_buffer,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                    texture_size,
                );
            }
        }

        // 3. Render
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.012,
                            g: 0.012,
                            b: 0.014,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            if let Some(rect) = self.calculate_actual_viewport() {
                render_pass.set_viewport(rect.x, rect.y, rect.width, rect.height, 0.0, 1.0);
            }

            if let Some(bind_group) = &self.video_bind_group {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.device.poll(Maintain::Wait);

        Ok(())
    }

    pub fn repaint(&mut self) -> anyhow::Result<()> {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(SurfaceError::Outdated) | Err(SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Repaint Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Repaint Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.012,
                            g: 0.012,
                            b: 0.014,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            if let Some(rect) = self.calculate_actual_viewport() {
                render_pass.set_viewport(rect.x, rect.y, rect.width, rect.height, 0.0, 1.0);
            }

            if let Some(bind_group) = &self.video_bind_group {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.device.poll(Maintain::Wait);
        Ok(())
    }
}
