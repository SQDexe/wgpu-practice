use {
    anyhow::Result as DynResult,
    bytemuck::cast_slice,
    cgmath::{
        Deg,
        Vector3,
        Quaternion,
        InnerSpace,
        Rotation3,
        Zero
        },
    log::*,
    wgpu::{
        *,
        util::*
        },
    winit::{
        dpi::PhysicalSize,
        event_loop::ActiveEventLoop,
        keyboard::KeyCode,
        platform::windows::WindowExtWindows,
        window::*
        },
    std::{
        iter::once,
        sync::Arc
        },
    crate::{
        camera::*,
        instance::{
            Instance as ModelInstance,
            InstanceRaw
            },
        texture::Texture,
        vertex::Vertex,
        utils::VertexInfo
        }
    };

const VERTICES: &[Vertex] = &[
    Vertex::new([-0.0868241,   0.49240386, 0.0], [0.4131759,    0.00759614]),
    Vertex::new([-0.49513406,  0.06958647, 0.0], [0.0048659444, 0.43041354]),
    Vertex::new([-0.21918549, -0.44939706, 0.0], [0.28081453,   0.949397]),
    Vertex::new([ 0.35966998, -0.3473291,  0.0], [0.85967,      0.84732914]),
    Vertex::new([ 0.44147372,  0.2347359,  0.0], [0.9414737,    0.2652641]),
    ];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
    ];

// Store the game state
pub struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    diffuse_bind_group: BindGroup,
    diffuse_texture: Texture,
    depth_texture: Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_controller: CameraController,
    instances: Vec<ModelInstance>,
    instance_buffer: Buffer,
    is_surface_configured: bool
    }

impl State {
    pub async fn new(window: Arc<Window>) -> DynResult<Self> {
        window.set_title("WGPU Practice");
        window.set_resizable(false);
        match window.request_inner_size(PhysicalSize { width: 320, height: 180}) {
            Some(PhysicalSize { width, height }) => info!("Set initial size of: {width}x{height}"),
            _ => info!("Unable to set size")
            }
        // window.set_window_icon(window_icon);
        // window.set_taskbar_icon(taskbar_icon);

        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            .. Default::default()
            });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
            })
            .await?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: Some("Device Descriptor"),
            required_features: Features::empty(),
            required_limits: Limits::default(),
            memory_hints: Default::default(),
            trace: Trace::Off
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|e| e.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
            };

        let diffuse_texture = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../assets/happy-tree.png"),
            Some("Happy Tree")
            )?;

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false
                        },
                    count: None
                    },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    // Matches the filterable field of the corresponding entry above
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None
                    }
                ]
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(diffuse_texture.get_view())
                    },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(diffuse_texture.get_sampler())
                    }
                ]
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, Some("Depth Texture"));

        let camera = Camera::new(
            [0.0, 1.0, 2.0],
            [0.0; 3],
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0
            );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_projection(&camera);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
            });

        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                        },
                    count: None
                    }
                ]
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                    }
                ]
            });

        let camera_controller = CameraController::new(0.25);

        let shader = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout
                ],
            push_constant_ranges: &[]
            });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    Vertex::DESC,
                    InstanceRaw::DESC
                    ]
                },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(ColorTargetState {
                        format: config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL
                        })
                    ],
                compilation_options: PipelineCompilationOptions::default()
                }),
            primitive: PrimitiveState {
                topology:PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // Other modes require Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Other option requires Features::DEPTH_CLPI_CONTROL
                unclipped_depth: false,
                // Other option requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false
                },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default()
                }),
            multisample: MultisampleState {
                count: 1,
                mask: ! 0,
                alpha_to_coverage_enabled: false
                },
            multiview: None,
            cache: None
            });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(VERTICES),
            usage: BufferUsages::VERTEX
            });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(INDICES),
            usage: BufferUsages::INDEX
            });

        const NUM_INSTANCE_PER_ROW: u32 = 8;
        const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(
            NUM_INSTANCE_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCE_PER_ROW as f32 * 0.5
            );

        let instances: Vec<_> = (0 .. NUM_INSTANCE_PER_ROW * NUM_INSTANCE_PER_ROW)
            .map(|i| {
                let (x, z) = (
                    i % NUM_INSTANCE_PER_ROW,
                    i / NUM_INSTANCE_PER_ROW
                    ); 
                
                let position = Vector3::new(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT;

                // Check at Zero point as Quaternions can affect scale, if not used properly
                let rotation = match position.is_zero() {
                    true => Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0)),
                    false => Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                    };

                ModelInstance::new(position, rotation)
                })
            .collect();

        let instances_data: Vec<_> = instances.iter()
            .map(ModelInstance::to_raw)
            .collect();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: cast_slice(&instances_data),
            usage: BufferUsages::VERTEX
            });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances,
            instance_buffer,
            is_surface_configured: false
            })
        }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_projection(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, cast_slice(&[self.camera_uniform]));
        }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();

        // Can't render untill the surface is ready
        if ! self.is_surface_configured {
            return Ok(());
            }

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Enocder")
            });

        /* A mutable borrow of encoder needs to be dropped */ {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store
                            },
                        depth_slice: None
                        })
                    ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.get_view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store
                        }),
                    stencil_ops: None
                    }),
                occlusion_query_set: None,
                timestamp_writes: None
                });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0 .. self.num_indices, 0, 0 .. self.instances.len() as u32);
            }

        self.queue.submit(once(encoder.finish()));
        output.present();

        Ok(())
        }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if let PhysicalSize { width: width @ 1 .., height: height @ 1 .. } = size {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, Some("Depth Texture"));
            self.is_surface_configured = true;
            }
        }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.camera_controller.handle_key(code, is_pressed);
        
        if ! is_pressed {
            return;
            }

        match code {
            KeyCode::Escape =>
                event_loop.exit(),
            KeyCode::KeyF =>
                self.set_fullscreen(true),
            KeyCode::KeyE =>
                self.set_fullscreen(false),    
            _ => ()
            };
        }

    fn set_fullscreen(&self, turn_on: bool) {
        self.window.set_fullscreen(match turn_on {
            true => Some(Fullscreen::Borderless(None)),
            false => None
            });
        }

    pub fn get_window(&self) -> &Window {
        &self.window
        }
    }