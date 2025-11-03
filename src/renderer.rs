use std::sync::Arc;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgc::device::queue,
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BlendState, BufferUsages, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device,
    DeviceDescriptor, ExperimentalFeatures, Features, FragmentState, Instance, Limits, MemoryHints,
    MultisampleState, Operations, PipelineLayoutDescriptor, PowerPreference, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    camera::{Camera, Camera2Uniform, CameraController, CameraUniform},
    texture::Texture,
    vertex::Vertex,
};

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pub position: PhysicalPosition<f64>,
    render_pipeline: RenderPipeline,
    render_pipeline_colour: RenderPipeline,

    pub render_state: RenderState,

    window: Arc<Window>,
    vertex_buffer: wgpu::Buffer,

    num_verticies: u32,
    index_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: Texture,

    camera: Camera,

    is_surface_configured: bool,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: wgpu::Buffer,
    camera2_uniform_buffer: wgpu::Buffer,
    camera2_uniform: Camera2Uniform,
}

pub enum RenderState {
    Default,
    ColourPass,
    ComplexObject,
}
impl RenderState {
    pub fn next(&self) -> RenderState {
        match self {
            RenderState::Default => RenderState::ColourPass,
            RenderState::ColourPass => RenderState::ComplexObject,
            RenderState::ComplexObject => RenderState::Default,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        color: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        color: [0.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    // Vertex {
    //     position: [-0.0868241, 0.49240386, 0.0],
    //     color: [0.0, 0.0, 0.0],
    //     tex_coords: [0.4131759, 0.99240386],
    // }, // A
    // Vertex {
    //     position: [-0.49513406, 0.06958647, 0.0],
    //     color: [0.5, 0.0, 0.5],
    //     tex_coords: [0.0048659444, 0.56958647],
    // }, // B
    // Vertex {
    //     position: [-0.21918549, -0.44939706, 0.0],
    //     color: [0.5, 0.0, 0.5],
    //     tex_coords: [0.28081453, 0.05060294],
    // }, // C
    // Vertex {
    //     position: [0.35966998, -0.3473291, 0.0],
    //     color: [1.0, 1.0, 0.5],
    //     tex_coords: [0.85967, 0.1526709],
    // }, // D
    // Vertex {
    //     position: [0.44147372, 0.2347359, 0.0],
    //     color: [0.5, 0.0, 0.5],
    //     tex_coords: [0.9414737, 0.7347359],
    // }, // E
];

const INDICES: &[u32] = &[0, 2, 1, 3, 1, 2];

impl Renderer {
    pub async fn new<'a>(window: Arc<Window>) -> Renderer {
        let size = window.inner_size();

        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
                experimental_features: ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_capabilites = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilites
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilites.alpha_modes[0],
            view_formats: vec![],
        };

        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(0.2);

        let diffuse_bytes = include_bytes!("Untitled.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_project(&camera);

        let camera2_uniform = Camera2Uniform {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        };

        let camera_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera2_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera2_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: camera2_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let shader2 = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader2"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader2.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // let storage_buffer = device.create_buffer(&BufferDescriptor {
        //     label: Some("test"),
        //     size: 10,
        //     usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        //     mapped_at_creation: false,
        // });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let render_pipeline_colour_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline_colour = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("wow"),
            layout: Some(&render_pipeline_colour_layout),
            vertex: VertexState {
                module: &shader2,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader2,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        return Renderer {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            render_pipeline_colour,
            render_state: RenderState::Default,
            vertex_buffer,
            index_buffer,
            position: PhysicalPosition::new(0.0, 0.0),
            num_verticies: VERTICES.len() as u32,
            diffuse_bind_group,
            diffuse_texture,
            is_surface_configured: false,
            camera,
            camera_controller,
            camera_uniform,
            camera_bind_group,
            camera_uniform_buffer,

            camera2_uniform,
            camera2_uniform_buffer
        };
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_project(&self.camera);
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.camera2_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera2_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Renderer"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            match self.render_state {
                RenderState::Default => {
                    render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_pipeline(&self.render_pipeline);
                    // render_pass.draw(0..self.num_verticies, 0..1);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }
                RenderState::ColourPass => {
                    render_pass.set_pipeline(&self.render_pipeline_colour);
                    render_pass.draw(0..3, 0..1);
                }
                RenderState::ComplexObject => {}
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.is_surface_configured = true;
    }

    pub fn window(&self) -> Arc<Window> {
        return self.window.clone();
    }

    pub fn input(&mut self, window_event: &WindowEvent) -> bool {
        match window_event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = *position;
                return true;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Space, true) => {
                    self.render_state = self.render_state.next();
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyR, true) => {
                    self.camera2_uniform.scale += 0.1;
                    println!("{}", self.camera2_uniform.scale);
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyT, true) => {
                    self.camera2_uniform.scale -= 0.1;
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyW, true) => {
                    self.camera2_uniform.y -= 0.1 / self.camera2_uniform.scale;
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyA, true) => {
                    self.camera2_uniform.x += 0.1 / self.camera2_uniform.scale;
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyS, true) => {
                    self.camera2_uniform.y += 0.1 / self.camera2_uniform.scale;
                    println!("{}", self.camera2_uniform.scale);
                    self.window.request_redraw();
                    return true;
                }
                (KeyCode::KeyD, true) => {
                    self.camera2_uniform.x -= 0.1 / self.camera2_uniform.scale;
                    self.window.request_redraw();
                    return true;
                }
                (x, y) => self.camera_controller.handle_key(*x, y),
                _ => {
                    return false;
                }
            },
            _ => {
                return false;
            }
        }
    }
}
