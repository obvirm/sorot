use crate::shape::Shape;
use srgpu::GpuContext;
use srmath::{Color, Mat4};

pub struct Renderer {
    pub background: Color,
}

impl Renderer {
    pub fn new(background: Color) -> Self {
        Self { background }
    }

    pub fn render(
        &self,
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
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        gpu.device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
