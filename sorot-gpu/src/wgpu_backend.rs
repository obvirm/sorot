use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use sorot_render::render_ir::{RenderFrame, SdfOp};

use crate::backend::GpuBackend;
use crate::pass::{PassGraph, PassKind};
use crate::render_target::TexturePool;

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
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
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
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CompositeVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl CompositeVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
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
fn vs(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
@fragment
fn fs(@location(1) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
"#;

const SDF_SHADER: &str = r#"
@group(0) @binding(0) var sdf_tex: texture_2d<f32>;
@group(0) @binding(1) var sdf_sampler: sampler;

struct Uniforms { color: vec4<f32>, spread: f32 }

@group(1) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
@fragment
fn fs(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let d = textureSample(sdf_tex, sdf_sampler, uv).r;
    let a = smoothstep(0.5 - u.spread, 0.5 + u.spread, d);
    return vec4<f32>(u.color.rgb, u.color.a * a);
}
"#;

const COMPOSITE_SHADER: &str = r#"
@group(0) @binding(0) var tex_a: texture_2d<f32>;
@group(0) @binding(1) var tex_b: texture_2d<f32>;
@group(0) @binding(2) var sam: sampler;

@vertex
fn vs(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
@fragment
fn fs(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let a = textureSample(tex_a, sam, uv);
    let b = textureSample(tex_b, sam, uv);
    let inv_a = 1.0 - a.a;
    return vec4<f32>(a.rgb + b.rgb * inv_a, a.a + b.a * inv_a);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SdfUniform {
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
    surface_format: wgpu::TextureFormat,

    color_pipeline: wgpu::RenderPipeline,
    sdf_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,

    sdf_tex_layout: wgpu::BindGroupLayout,
    sdf_uni_layout: wgpu::BindGroupLayout,
    composite_layout: wgpu::BindGroupLayout,

    texture_pool: TexturePool,
    composite_vb: wgpu::Buffer,
    fullscreen_ib: wgpu::Buffer,

    // dynamic state per frame
    frame: Option<RenderFrame>,
}

impl WgpuBackend {
    pub async fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
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
            label: Some("color"), source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
        });
        let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("color_layout"), bind_group_layouts: &[], push_constant_ranges: &[],
        });
        let color_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("color_pipe"),
            layout: Some(&color_layout),
            vertex: wgpu::VertexState {
                module: &cshader, entry_point: Some("vs"), buffers: &[RawVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &cshader, entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None,
        });

        let sdf_tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_tex"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2, multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None,
                },
            ],
        });
        let sdf_uni_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sdf_uni"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None,
                }, count: None,
            }],
        });

        let sshader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sdf"), source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SDF_SHADER)),
        });
        let sdf_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sdf_layout"), bind_group_layouts: &[&sdf_tex_layout, &sdf_uni_layout],
            push_constant_ranges: &[],
        });
        let sdf_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sdf_pipe"),
            layout: Some(&sdf_layout),
            vertex: wgpu::VertexState {
                module: &sshader, entry_point: Some("vs"), buffers: &[SdfVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sshader, entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None,
        });

        let composite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("composite"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2, multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2, multisampled: false,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None,
                },
            ],
        });

        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite"), source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPOSITE_SHADER)),
        });
        let composite_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite_pl"), bind_group_layouts: &[&composite_layout],
            push_constant_ranges: &[],
        });
        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite_pipe"),
            layout: Some(&composite_pl),
            vertex: wgpu::VertexState {
                module: &composite_shader, entry_point: Some("vs"),
                buffers: &[CompositeVertex::LAYOUT], compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_shader, entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format, blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None,
        });

        let mut texture_pool = TexturePool::new(&device);

        let composite_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("composite_vb"),
            contents: bytemuck::cast_slice(&COMPOSITE_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let fullscreen_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fs_ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            window, surface, device, queue, config, surface_format: format,
            color_pipeline, sdf_pipeline, composite_pipeline,
            sdf_tex_layout, sdf_uni_layout, composite_layout,
            texture_pool,
            composite_vb, fullscreen_ib,
            frame: None,
        }
    }

    pub fn set_frame(&mut self, frame: RenderFrame) {
        self.frame = Some(frame);
    }

    fn draw(&mut self) {
        let frame = match self.frame.as_ref() {
            Some(f) => f,
            None => return,
        };

        self.texture_pool.release_all();
        let shape_rt = self.texture_pool.acquire(
            &self.device, self.config.width, self.config.height, self.surface_format,
        );
        let sdf_rt = self.texture_pool.acquire(
            &self.device, self.config.width, self.config.height, self.surface_format,
        );

        let graph = PassGraph::from_frame(frame, shape_rt, sdf_rt);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

        for pass_def in &graph.passes {
            match &pass_def.kind {
                PassKind::Shape { packet_ids, target_id } => {
                    let rt = self.texture_pool.get(*target_id);
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("shape_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &rt.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&self.color_pipeline);
                    for &id in packet_ids {
                        let pkt = frame.get_packet(id);
                        if pkt.indices.is_empty() { continue; }
                        let verts: Vec<RawVertex> = pkt.vertices.iter().map(|v| RawVertex {
                            position: [v.clip_x, v.clip_y],
                            color: [v.r, v.g, v.b, v.a],
                        }).collect();
                        let vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("tmp_vb"), contents: bytemuck::cast_slice(&verts),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                        let ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("tmp_ib"), contents: bytemuck::cast_slice(&pkt.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                        pass.set_vertex_buffer(0, vb.slice(..));
                        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..pkt.indices.len() as u32, 0, 0..1);
                    }
                }

                PassKind::Sdf { op, target_id } => {
                    let rt = self.texture_pool.get(*target_id);
                    let (_tex, sdf_bg, _uni_buf, sdf_uni_bg, sdf_vb) =
                        self.build_sdf_resources(op);

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("sdf_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &rt.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&self.sdf_pipeline);
                    pass.set_bind_group(0, &sdf_bg, &[]);
                    pass.set_bind_group(1, &sdf_uni_bg, &[]);
                    pass.set_vertex_buffer(0, sdf_vb.slice(..));
                    pass.set_index_buffer(self.fullscreen_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..6, 0, 0..1);
                }

                PassKind::Composite { input_ids, clear_color } => {
                    let output = match self.surface.get_current_texture() {
                        Ok(f) => f,
                        Err(wgpu::SurfaceError::Lost) => {
                            self.surface.configure(&self.device, &self.config);
                            return;
                        }
                        Err(_) => return,
                    };
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let rt_a = self.texture_pool.get(input_ids[0]);
                    let rt_b = self.texture_pool.get(input_ids[1]);
                    let sam = self.texture_pool.sampler();

                    let composite_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("composite_bg"),
                        layout: &self.composite_layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&rt_a.view) },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&rt_b.view) },
                            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sam) },
                        ],
                    });

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("composite_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear_color[0], g: clear_color[1],
                                    b: clear_color[2], a: clear_color[3],
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&self.composite_pipeline);
                    pass.set_bind_group(0, &composite_bg, &[]);
                    pass.set_vertex_buffer(0, self.composite_vb.slice(..));
                    pass.set_index_buffer(self.fullscreen_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn build_sdf_resources(
        &self,
        op: &SdfOp,
    ) -> (wgpu::Texture, wgpu::BindGroup, wgpu::Buffer, wgpu::BindGroup, wgpu::Buffer) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;

        let sdf_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sdf_tex"),
            size: wgpu::Extent3d {
                width: op.sdf_width.min(256),
                height: op.sdf_height.min(256),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &sdf_tex, mip_level: 0,
                origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All,
            },
            &*op.sdf_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(op.sdf_width),
                rows_per_image: Some(op.sdf_height),
            },
            wgpu::Extent3d {
                width: op.sdf_width.min(256),
                height: op.sdf_height.min(256),
                depth_or_array_layers: 1,
            },
        );

        let sdf_view = sdf_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sdf_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sdf_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sdf_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_bg"),
            layout: &self.sdf_tex_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sdf_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sdf_sampler) },
            ],
        });

        let cf = {
            let u8 = op.paint.color.to_premultiplied_u8();
            [u8[0] as f32 / 255.0, u8[1] as f32 / 255.0, u8[2] as f32 / 255.0, u8[3] as f32 / 255.0]
        };
        let su = SdfUniform { color: cf, spread: 0.04, _pad: [0.0; 3] };
        let sdf_uni_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_uni"), contents: bytemuck::cast_slice(&[su]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let sdf_uni_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sdf_uni_bg"),
            layout: &self.sdf_uni_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: sdf_uni_buf.as_entire_binding() }],
        });

        let x0 = op.bounds.min.x / (w * 0.5) - 1.0;
        let y0 = 1.0 - op.bounds.min.y / (h * 0.5);
        let x1 = op.bounds.max.x / (w * 0.5) - 1.0;
        let y1 = 1.0 - op.bounds.max.y / (h * 0.5);

        let quad: [SdfVertex; 4] = [
            SdfVertex { position: [x0, y0], uv: [0.0, 1.0] },
            SdfVertex { position: [x1, y0], uv: [1.0, 1.0] },
            SdfVertex { position: [x1, y1], uv: [1.0, 0.0] },
            SdfVertex { position: [x0, y1], uv: [0.0, 0.0] },
        ];
        let sdf_vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sdf_vb"), contents: bytemuck::cast_slice(&quad),
            usage: wgpu::BufferUsages::VERTEX,
        });

        (sdf_tex, sdf_bg, sdf_uni_buf, sdf_uni_bg, sdf_vb)
    }
}

const COMPOSITE_QUAD: [CompositeVertex; 4] = [
    CompositeVertex { position: [-1.0, 1.0], uv: [0.0, 1.0] },
    CompositeVertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
    CompositeVertex { position: [1.0, -1.0], uv: [1.0, 0.0] },
    CompositeVertex { position: [-1.0, -1.0], uv: [0.0, 0.0] },
];

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
        if let Some(ref f) = self.frame { be.set_frame(f.clone()); }
        self.backend = Some(be);
    }
}

impl ApplicationHandler for WgpuApp {
    fn resumed(&mut self, el: &ActiveEventLoop) { if self.backend.is_none() { self.setup(el); } }
    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, ev: WindowEvent) {
        let Some(be) = self.backend.as_mut() else { return };
        if be.window().id() != wid { return; }
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => be.resize(s),
            WindowEvent::RedrawRequested => { let _ = be.render(); be.window().request_redraw(); }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(be) = &self.backend { be.window().request_redraw(); }
    }
}
