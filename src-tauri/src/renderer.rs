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
}

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

    pub fn render_frame(&mut self, rgba_data: &[u8], width: u32, height: u32, stride: u32) -> anyhow::Result<()> {
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
                format: TextureFormat::Rgba8Unorm,
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
            // WGPU requires bytes_per_row to be a multiple of 256
            let unaligned_bytes_per_row = stride;
            let align = 256;
            let aligned_bytes_per_row = (unaligned_bytes_per_row + align - 1) & !(align - 1);

            if unaligned_bytes_per_row == aligned_bytes_per_row {
                // Happy path: Data is already aligned
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
                // Unaligned path: Must pad the data
                // Resize buffer if needed (amortized allocation)
                let required_size = (aligned_bytes_per_row * height) as usize;
                if self.padding_buffer.len() < required_size {
                    self.padding_buffer.resize(required_size, 0);
                }

                // Copy row by row
                for y in 0..height {
                    let src_start = (y * unaligned_bytes_per_row) as usize;
                    let src_end = src_start + (width * 4) as usize; // Read only valid pixels, ignore existing padding
                    let dst_start = (y * aligned_bytes_per_row) as usize;
                    
                    // Safety check to avoid index out of bounds
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
                            r: 1.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            if let Some(bind_group) = &self.video_bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);
            }
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        // Critical: Poll to ensure commands are executed and resources potentially cleaned up
        self.device.poll(Maintain::Wait);

        Ok(())
    }
}
