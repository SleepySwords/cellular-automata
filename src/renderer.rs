use std::{sync::Arc};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgt::CommandEncoderDescriptor,
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BlendState, BufferUsages, ColorTargetState, ColorWrites, ComputePassDescriptor,
    ComputePipelineDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Features,
    FragmentState, Instance, Limits, MemoryHints, MultisampleState, Operations, Origin3d,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration, SurfaceError,
    TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    camera::{Camera, CameraController},
    texture::Texture,
    vertex::Vertex,
};

pub struct State {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pub position: PhysicalPosition<f64>,
    render_pipeline: RenderPipeline,

    pub render_state: RenderState,

    window: Arc<Window>,
    vertex_buffer: wgpu::Buffer,

    num_verticies: u32,
    index_buffer: wgpu::Buffer,
    pub camera: Camera,
    camera_bind_group: wgpu::BindGroup,

    texture_bind_group: wgpu::BindGroup,
    presentation_texture: Texture,

    is_surface_configured: bool,
    pub camera_controller: CameraController,
    camera_uniform_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    compute_texture: Texture,
    is_mouse_pressed: bool,
    texture_bind_group_layout: wgpu::BindGroupLayout,
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
];

const INDICES: &[u32] = &[0, 2, 1, 3, 1, 2];

impl State {
    pub async fn new<'a>(window: Arc<Window>) -> State {
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
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        };

        let camera_controller = CameraController::new(0.1);

        let diffuse_bytes = include_bytes!("ok.png");
        let presentation_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "Presentation texture").unwrap();

        let compute_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "Compute texture").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&presentation_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&presentation_texture.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&compute_texture.view),
                },
            ],
        });

        let camera_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("automata/gol.wgsl").into()),
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

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Automota"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        return State {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            render_state: RenderState::Default,
            vertex_buffer,
            index_buffer,
            position: PhysicalPosition::new(0.0, 0.0),
            num_verticies: VERTICES.len() as u32,
            texture_bind_group,
            texture_bind_group_layout,
            presentation_texture,
            compute_texture,
            is_surface_configured: false,
            camera,
            camera_controller,
            camera_bind_group,
            camera_uniform_buffer,

            compute_pipeline,
            is_mouse_pressed: false,
        };
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera]),
        );
    }

    pub fn run_compute(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Compute"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute pass"),
                timestamp_writes: None,
            });
            compute_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            compute_pass.set_pipeline(&self.compute_pipeline);

            compute_pass.dispatch_workgroups(
                self.presentation_texture.size.width,
                self.presentation_texture.size.height,
                1,
            );
        }

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.compute_texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.presentation_texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            Extent3d {
                width: self.presentation_texture.size.width,
                height: self.presentation_texture.size.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
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
                    render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_pipeline(&self.render_pipeline);
                    // render_pass.draw(0..self.num_verticies, 0..1);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }
                RenderState::ColourPass => {}
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
            // TODO: move all this logic to the camera controller
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_mouse_pressed {
                    let delta_x = (position.x - self.position.x) as f32;
                    let delta_y = (position.y - self.position.y) as f32;

                    // Add notes here
                    self.camera.x += (delta_x / self.size.width as f32) * 2.0 / self.camera.scale;
                    self.camera.y -= (delta_y / self.size.height as f32) * 2.0 / self.camera.scale;
                }
                self.position = *position;
                return true;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.is_mouse_pressed = state.is_pressed();
                }
                return true;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, _) => todo!(),
                MouseScrollDelta::PixelDelta(physical_position) => {
                    self.camera.scale += (physical_position.y as f32) / 1000.0;
                    return true;
                }
            },
            WindowEvent::DroppedFile(path) => {
                let bytes = std::fs::read(path).unwrap();

                let new_presentation_texture =
                    Texture::from_bytes(&self.device, &self.queue, &bytes, "Presentation Texture").unwrap();
                let new_compute_texture =
                    Texture::from_bytes(&self.device, &self.queue, &bytes, "Compute Texture").unwrap();

                let texture_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &new_presentation_texture.view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&new_presentation_texture.sampler),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&new_compute_texture.view),
                        },
                    ],
                });

                self.texture_bind_group = texture_bind_group;
                self.compute_texture = new_compute_texture;
                self.presentation_texture = new_presentation_texture;

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
                    self.run_compute();
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
