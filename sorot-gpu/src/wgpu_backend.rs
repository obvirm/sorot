use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use sorot_core::color::Color;
use sorot_core::math::Rect;
use sorot_raster::TriMesh;

use crate::backend::GpuBackend;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SdfVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl SdfVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };
}

const COLOR_SHADER: &str = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@location(1) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
"#;

const SDF_SHADER: &str = r#"
@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var sdf_sampler: sampler;

struct Uniforms {
    color: vec4<f32>,
    spread: f32,
}

@group(1) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs_sdf(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_sdf(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let dist = textureSample(sdf_tex, sdf_sampler, uv).r;
    let alpha = smoothstep(0.5 - u.spread, 0.5 + u.spread, dist);
    return vec4<f32>(u.color.rgb, u.color.a * alpha);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SdfUniforms {
    color: [f32; 4],
    spread: f32,
    _pad: [f32; 3],
}

pub struct WgpuBackend {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    color_pipeline: wgpu::RenderPipeline,
    sdf_pipeline: wgpu::RenderPipeline,
    sdf_uniform_layout: wgpu::BindGroupLayout,
    sdf_texture: wgpu::Texture,
    sdf_bind_group: wgpu::BindGroup,
    sdf_quad_vb: wgpu::Buffer,
    sdf_quad_ib: wgpu::Buffer,
    sdf_uniform: wgpu::Buffer,
    sdf_uniform_bg: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
}

impl WgpuBackend {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("no suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("color_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
        });

        let color_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("color_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color_pipeline"),
            layout: Some(&color_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sdf_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let sdf_texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sdf_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sdf_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SDF_SHADER)),
        });

        let sdf_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sdf_pipeline_layout"),
            bind_group_layouts: &[&sdf_texture_layout, &sdf_uniform_layout],
            push_constant_ranges: &[],
        });

        let sdf_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sdf_pipeline"),
            layout: Some(&sdf_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sdf_shader,
                entry_point: Some("vs_sdf"),
                buffers: &[SdfVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sdf_shader,
                entry_point: Some("fs_sdf"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sdf_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sdf_texture"),
            size: wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sdf_view = sdf_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sdf_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sdf_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sdf_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_bind_group"),
            layout: &sdf_texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sdf_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sdf_sampler),
                },
            ],
        });

        let sdf_init_uniform = SdfUniforms {
            color: [1.0, 1.0, 1.0, 1.0],
            spread: 0.04,
            _pad: [0.0; 3],
        };

        let sdf_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_uniform"),
            contents: bytemuck::cast_slice(&[sdf_init_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sdf_uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_uniform_bg"),
            layout: &sdf_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sdf_uniform.as_entire_binding(),
            }],
        });

        let sdf_quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_quad_vb"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let sdf_quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_quad_ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            color_pipeline,
            sdf_pipeline,
            sdf_uniform_layout,
            sdf_texture,
            sdf_bind_group,
            sdf_quad_vb,
            sdf_quad_ib,
            sdf_uniform,
            sdf_uniform_bg,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
        }
    }

    pub fn upload_meshes(&mut self, meshes: &[(TriMesh, Color)]) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut base_index: u32 = 0;

        for (mesh, color) in meshes {
            let cf = {
                let u8 = color.to_premultiplied_u8();
                [
                    u8[0] as f32 / 255.0,
                    u8[1] as f32 / 255.0,
                    u8[2] as f32 / 255.0,
                    u8[3] as f32 / 255.0,
                ]
            };

            for v in &mesh.vertices {
                let px = v.x / (w * 0.5) - 1.0;
                let py = 1.0 - v.y / (h * 0.5);
                all_vertices.push(Vertex {
                    position: [px, py],
                    color: cf,
                });
            }

            for idx in &mesh.indices {
                all_indices.push(base_index + idx);
            }

            base_index += mesh.vertices.len() as u32;
        }

        if all_vertices.is_empty() || all_indices.is_empty() {
            return;
        }

        self.vertex_buffer = Some(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vertex_buffer"),
                    contents: bytemuck::cast_slice(&all_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        );

        self.index_buffer = Some(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("index_buffer"),
                    contents: bytemuck::cast_slice(&all_indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );

        self.index_count = all_indices.len() as u32;
    }

    pub fn upload_sdf(
        &mut self,
        sdf_data: &[u8],
        sdf_w: u32,
        sdf_h: u32,
        bounds: Rect,
        color: Color,
    ) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.sdf_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            sdf_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(sdf_w),
                rows_per_image: Some(sdf_h),
            },
            wgpu::Extent3d {
                width: sdf_w.min(128),
                height: sdf_h.min(128),
                depth_or_array_layers: 1,
            },
        );

        let x0 = bounds.min.x / (w * 0.5) - 1.0;
        let y0 = 1.0 - bounds.min.y / (h * 0.5);
        let x1 = bounds.max.x / (w * 0.5) - 1.0;
        let y1 = 1.0 - bounds.max.y / (h * 0.5);

        let quad_vertices: [SdfVertex; 4] = [
            SdfVertex { position: [x0, y0], uv: [0.0, 1.0] },
            SdfVertex { position: [x1, y0], uv: [1.0, 1.0] },
            SdfVertex { position: [x1, y1], uv: [1.0, 0.0] },
            SdfVertex { position: [x0, y1], uv: [0.0, 0.0] },
        ];

        self.sdf_quad_vb = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sdf_quad_vb"),
                contents: bytemuck::cast_slice(&quad_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let cf = {
            let u8 = color.to_premultiplied_u8();
            [
                u8[0] as f32 / 255.0,
                u8[1] as f32 / 255.0,
                u8[2] as f32 / 255.0,
                u8[3] as f32 / 255.0,
            ]
        };

        let sdf_uniform = SdfUniforms {
            color: cf,
            spread: 0.04,
            _pad: [0.0; 3],
        };

        self.queue.write_buffer(
            &self.sdf_uniform,
            0,
            bytemuck::cast_slice(&[sdf_uniform]),
        );

        self.sdf_uniform_bg = self
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sdf_uniform_bg"),
                layout: &self.sdf_uniform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.sdf_uniform.as_entire_binding(),
                }],
            });
    }

    fn draw(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.color_pipeline);
            if let (Some(ref vb), Some(ref ib)) = (&self.vertex_buffer, &self.index_buffer) {
                if self.index_count > 0 {
                    pass.set_vertex_buffer(0, vb.slice(..));
                    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..self.index_count, 0, 0..1);
                }
            }

            pass.set_pipeline(&self.sdf_pipeline);
            pass.set_bind_group(0, &self.sdf_bind_group, &[]);
            pass.set_bind_group(1, &self.sdf_uniform_bg, &[]);
            pass.set_vertex_buffer(0, self.sdf_quad_vb.slice(..));
            pass.set_index_buffer(self.sdf_quad_ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..6, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl GpuBackend for WgpuBackend {
    fn window(&self) -> &Arc<Window> {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.draw();
        Ok(())
    }
}

pub struct WgpuApp {
    backend: Option<WgpuBackend>,
    meshes: Vec<(TriMesh, Color)>,
    sdf_data: Option<(Vec<u8>, u32, u32, Rect, Color)>,
}

impl WgpuApp {
    pub async fn run(
        meshes: Vec<(TriMesh, Color)>,
        sdf_data: Option<(Vec<u8>, u32, u32, Rect, Color)>,
    ) {
        env_logger::init();
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = Self {
            backend: None,
            meshes,
            sdf_data,
        };
        event_loop.run_app(&mut app).unwrap();
    }

    fn setup(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("sorot — 2D Graphics Engine")
                    .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
            )
            .expect("failed to create window");

        let mut backend = pollster::block_on(WgpuBackend::new(window));
        backend.upload_meshes(&self.meshes);

        if let Some((ref data, w, h, bounds, color)) = self.sdf_data {
            backend.upload_sdf(data, w, h, bounds, color);
        }

        self.backend = Some(backend);
    }
}

impl ApplicationHandler for WgpuApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.backend.is_none() {
            self.setup(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(backend) = self.backend.as_mut() else {
            return;
        };

        if backend.window().id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                backend.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let _ = backend.render();
                backend.window().request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(backend) = &self.backend {
            backend.window().request_redraw();
        }
    }
}
