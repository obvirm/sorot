use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use sorot_scene::render_ir::{RenderFrame, SdfOp};

use crate::backend::GpuBackend;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RawVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl RawVertex {
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
struct SdfRawVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl SdfRawVertex {
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
    shape_vb: Option<wgpu::Buffer>,
    shape_ib: Option<wgpu::Buffer>,
    shape_index_count: u32,
    has_sdf: bool,
}

impl WgpuBackend {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let cshader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("color_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
        });

        let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("color_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color_pipeline"),
            layout: Some(&color_layout),
            vertex: wgpu::VertexState {
                module: &cshader,
                entry_point: Some("vs_main"),
                buffers: &[RawVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &cshader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        let sdf_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let sdf_tex_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sdf_tex_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let sshader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sdf_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SDF_SHADER)),
        });

        let sdf_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sdf_layout"),
            bind_group_layouts: &[&sdf_tex_layout, &sdf_uniform_layout],
            push_constant_ranges: &[],
        });

        let sdf_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sdf_pipeline"),
            layout: Some(&sdf_layout),
            vertex: wgpu::VertexState {
                module: &sshader,
                entry_point: Some("vs_sdf"),
                buffers: &[SdfRawVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sshader,
                entry_point: Some("fs_sdf"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            label: Some("sdf_tex"),
            size: wgpu::Extent3d { width: 128, height: 128, depth_or_array_layers: 1 },
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
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sdf_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_bg"),
            layout: &sdf_tex_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sdf_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sdf_sampler) },
            ],
        });

        let init_u = SdfUniforms { color: [1.0; 4], spread: 0.04, _pad: [0.0; 3] };
        let sdf_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_ub"),
            contents: bytemuck::cast_slice(&[init_u]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sdf_uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_ubg"),
            layout: &sdf_uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: sdf_uniform.as_entire_binding() }],
        });

        let sdf_quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_vb"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let sdf_quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window, surface, device, queue, config,
            color_pipeline, sdf_pipeline,
            sdf_uniform_layout,
            sdf_texture, sdf_bind_group,
            sdf_quad_vb, sdf_quad_ib,
            sdf_uniform, sdf_uniform_bg,
            shape_vb: None, shape_ib: None,
            shape_index_count: 0,
            has_sdf: false,
        }
    }

    /// Upload a full RenderFrame to GPU buffers.
    pub fn upload_frame(&mut self, frame: &RenderFrame) {
        let mut all_verts: Vec<RawVertex> = Vec::new();
        let mut all_idx: Vec<u32> = Vec::new();
        let mut base: u32 = 0;

        for tile in &frame.tiles {
            for packet in &tile.packets {
                for v in &packet.vertices {
                    all_verts.push(RawVertex {
                        position: [v.clip_x, v.clip_y],
                        color: [v.r, v.g, v.b, v.a],
                    });
                }
                for idx in &packet.indices {
                    all_idx.push(base + idx);
                }
                base += packet.vertices.len() as u32;
            }
        }

        if !all_verts.is_empty() && !all_idx.is_empty() {
            self.shape_vb = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shape_vb"),
                contents: bytemuck::cast_slice(&all_verts),
                usage: wgpu::BufferUsages::VERTEX,
            }));
            self.shape_ib = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("shape_ib"),
                contents: bytemuck::cast_slice(&all_idx),
                usage: wgpu::BufferUsages::INDEX,
            }));
            self.shape_index_count = all_idx.len() as u32;
        }

        if let Some(sdf_op) = frame.sdf_ops.first() {
            self.upload_sdf_quad(sdf_op);
            self.has_sdf = true;
        }
    }

    fn upload_sdf_quad(&mut self, op: &SdfOp) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.sdf_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &op.sdf_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(op.sdf_width),
                rows_per_image: Some(op.sdf_height),
            },
            wgpu::Extent3d {
                width: op.sdf_width.min(128),
                height: op.sdf_height.min(128),
                depth_or_array_layers: 1,
            },
        );

        let x0 = op.bounds.min.x / (w * 0.5) - 1.0;
        let y0 = 1.0 - op.bounds.min.y / (h * 0.5);
        let x1 = op.bounds.max.x / (w * 0.5) - 1.0;
        let y1 = 1.0 - op.bounds.max.y / (h * 0.5);

        let quad: [SdfRawVertex; 4] = [
            SdfRawVertex { position: [x0, y0], uv: [0.0, 1.0] },
            SdfRawVertex { position: [x1, y0], uv: [1.0, 1.0] },
            SdfRawVertex { position: [x1, y1], uv: [1.0, 0.0] },
            SdfRawVertex { position: [x0, y1], uv: [0.0, 0.0] },
        ];

        self.sdf_quad_vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_vb"),
            contents: bytemuck::cast_slice(&quad),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cf = {
            let u8 = op.paint.color.to_premultiplied_u8();
            [
                u8[0] as f32 / 255.0,
                u8[1] as f32 / 255.0,
                u8[2] as f32 / 255.0,
                u8[3] as f32 / 255.0,
            ]
        };
        let u = SdfUniforms { color: cf, spread: 0.04, _pad: [0.0; 3] };
        self.queue.write_buffer(&self.sdf_uniform, 0, bytemuck::cast_slice(&[u]));
        self.sdf_uniform_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_ubg"),
            layout: &self.sdf_uniform_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: self.sdf_uniform.as_entire_binding() }],
        });
    }

    fn draw(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost) => { self.surface.configure(&self.device, &self.config); return; }
            Err(_) => return,
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.color_pipeline);
            if let (Some(ref vb), Some(ref ib)) = (&self.shape_vb, &self.shape_ib) {
                if self.shape_index_count > 0 {
                    pass.set_vertex_buffer(0, vb.slice(..));
                    pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..self.shape_index_count, 0, 0..1);
                }
            }

            if self.has_sdf {
                pass.set_pipeline(&self.sdf_pipeline);
                pass.set_bind_group(0, &self.sdf_bind_group, &[]);
                pass.set_bind_group(1, &self.sdf_uniform_bg, &[]);
                pass.set_vertex_buffer(0, self.sdf_quad_vb.slice(..));
                pass.set_index_buffer(self.sdf_quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl GpuBackend for WgpuBackend {
    fn window(&self) -> &Arc<Window> { &self.window }
    fn resize(&mut self, s: winit::dpi::PhysicalSize<u32>) {
        if s.width > 0 && s.height > 0 {
            self.config.width = s.width;
            self.config.height = s.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> { self.draw(); Ok(()) }
}

pub struct WgpuApp {
    backend: Option<WgpuBackend>,
    frame: Option<RenderFrame>,
}

impl WgpuApp {
    pub async fn run(frame: RenderFrame) {
        env_logger::init();
        let el = winit::event_loop::EventLoop::new().unwrap();
        el.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = Self { backend: None, frame: Some(frame) };
        el.run_app(&mut app).unwrap();
    }

    fn setup(&mut self, el: &ActiveEventLoop) {
        let w = el.create_window(
            Window::default_attributes()
                .with_title("sorot — 2D Graphics Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
        ).expect("window");

        let mut be = pollster::block_on(WgpuBackend::new(w));
        if let Some(ref frame) = self.frame {
            be.upload_frame(frame);
        }
        self.backend = Some(be);
    }
}

impl ApplicationHandler for WgpuApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.backend.is_none() { self.setup(el); }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        let Some(be) = self.backend.as_mut() else { return };
        if be.window().id() != wid { return; }
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => be.resize(s),
            WindowEvent::RedrawRequested => {
                let _ = be.render();
                be.window().request_redraw();
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(be) = &self.backend { be.window().request_redraw(); }
    }
}
