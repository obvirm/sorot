use crate::shape::Shape;
use srgpu::GpuContext;
use srmath::{Color, Mat4};

pub struct Renderer {
    pub background: Color,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl Renderer {
    pub fn new(background: Color) -> Self {
        Self {
            background,
            pipeline: None,
        }
    }

    pub fn init(&mut self, gpu: &GpuContext<'_>) {
        if self.pipeline.is_some() {
            return;
        }

        let source = format!(
            "{}\n{}",
            srshader::builtin::FULLSCREEN_VERTEX,
            srshader::builtin::SDF_TEST_FRAGMENT,
        );

        let shader = gpu
            .device
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("SDF Shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        let layout = gpu
            .device
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SDF Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SDF Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });

        self.pipeline = Some(pipeline);
    }

    pub fn render(
        &mut self,
        gpu: &GpuContext<'_>,
        _shapes: &[Shape],
        _projection: &Mat4,
    ) -> Result<(), &'static str> {
        let output = match gpu.acquire_texture() {
            Some(tex) => tex,
            None => return Err("surface unavailable"),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("AetherRender Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("AetherRender Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.background.r as f64,
                            g: self.background.g as f64,
                            b: self.background.b as f64,
                            a: self.background.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(pipeline) = &self.pipeline {
                pass.set_pipeline(pipeline);
                pass.draw(0..3, 0..1);
            }
        }

        gpu.device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
