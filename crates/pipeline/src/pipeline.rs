use vector::Vec2;
use rect::Rect;
use matrix::Matrix;
use paint::Paint;
use pathbuilder::Path;
use triangulate::triangulate;
use renderir::render_ir::{GpuVertex, ImagePacket, RenderFrame, RenderPacket, TextOp};

use displaylist::{DisplayList, DrawCommand};
use geometrycache::GeometryCache;
use scenegraph::SceneGraph;

pub struct Pipeline {
    pub geom_cache: GeometryCache,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { geom_cache: GeometryCache::new() }
    }

    pub fn build_frame(
        &mut self,
        graph: &SceneGraph,
        dl: &DisplayList,
        screen_width: u32,
        screen_height: u32,
    ) -> RenderFrame {
        let mut frame = RenderFrame::new(screen_width, screen_height, 32);

        for cmd in &dl.commands {
            match cmd {
                DrawCommand::Rect(dr) => {
                    let path = Path::rect(dr.rect.min, dr.rect.max);
                    let mesh = self.geom_cache.get_mesh(&path, 0.5);
                    if let Some(p) = Self::mesh_to_packet(mesh, &dr.paint, dr.transform, screen_width, screen_height) {
                        frame.bin_packet(p);
                    }
                }
                DrawCommand::Oval(ov) => {
                    let path = Path::oval(ov.center, ov.rx, ov.ry);
                    let mesh = self.geom_cache.get_mesh(&path, 0.5);
                    if let Some(p) = Self::mesh_to_packet(mesh, &ov.paint, ov.transform, screen_width, screen_height) {
                        frame.bin_packet(p);
                    }
                }
                DrawCommand::Path(dp) => {
                    if let Some(stored) = graph.get_path(dp.path_id) {
                        if let Some(p) = Self::mesh_to_packet(&stored.mesh, &dp.paint, dp.transform, screen_width, screen_height) {
                            frame.bin_packet(p);
                        }
                    }
                }
                DrawCommand::Image(di) => {
                    if let Some(img) = graph.get_image(di.img_id) {
                        frame.image_ops.push(ImagePacket {
                            pixels: img.pixels.clone(),
                            width: img.width,
                            height: img.height,
                            dst_rect: di.dst_rect,
                            transform: di.transform,
                            paint: di.paint.clone(),
                        });
                    }
                }
                DrawCommand::Text(dt) => {
                    frame.text_ops.push(TextOp {
                        text: dt.text.clone(),
                        position: dt.position,
                        font_size: dt.font_size,
                        paint: dt.paint.clone(),
                    });
                }
            }
        }

        frame
    }

    fn mesh_to_packet(
        mesh: &triangulate::TriMesh,
        paint: &Paint,
        transform: Matrix,
        screen_w: u32,
        screen_h: u32,
    ) -> Option<RenderPacket> {
        if mesh.indices.is_empty() {
            return None;
        }

        let color = paint.color();
        let cf = {
            let u8 = color.to_premultiplied_u8();
            [u8[0] as f32 / 255.0, u8[1] as f32 / 255.0, u8[2] as f32 / 255.0, u8[3] as f32 / 255.0]
        };

        let hw = screen_w as f32 * 0.5;
        let hh = screen_h as f32 * 0.5;

        // Check if this is a gradient fill
        let is_gradient = matches!(paint.fill_kind, paint::FillKind::Gradient(_));

        let (mut min, mut max) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));

        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| {
                let p = transform.transform_point(*v);
                min = min.min(p);
                max = max.max(p);
                let (r, g, b, a) = if is_gradient {
                    // Per-vertex gradient sampling for mesh-based gradient approximation
                    // Project vertex onto gradient axis
                    let grad_color = match &paint.fill_kind {
                        paint::FillKind::Gradient(g) => {
                            match &g.kind {
                                paint::GradientType::Linear { x0, y0, x1, y1 } => {
                                    let dx = x1 - x0;
                                    let dy = y1 - y0;
                                    let len_sq = dx * dx + dy * dy;
                                    let t = if len_sq > 0.0 {
                                        ((v.x - x0) * dx + (v.y - y0) * dy) / len_sq
                                    } else { 0.0 };
                                    paint.color_at(t)
                                }
                                paint::GradientType::Radial { cx, cy, radius } => {
                                    let dx = v.x - cx;
                                    let dy = v.y - cy;
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    let t = if *radius > 0.0 { dist / radius } else { 0.0 };
                                    paint.color_at(t)
                                }
                            }
                        }
                        _ => paint.color(),
                    };
                    let u8 = grad_color.to_premultiplied_u8();
                    (u8[0] as f32 / 255.0, u8[1] as f32 / 255.0, u8[2] as f32 / 255.0, u8[3] as f32 / 255.0)
                } else {
                    (cf[0], cf[1], cf[2], cf[3])
                };
                GpuVertex {
                    clip_x: p.x / hw - 1.0,
                    clip_y: 1.0 - p.y / hh,
                    r, g, b, a,
                }
            })
            .collect();

        Some(RenderPacket {
            vertices: vertices.into_boxed_slice(),
            indices: mesh.indices.clone().into_boxed_slice(),
            paint: paint.clone(),
            transform,
            clip_rect: Rect::new(min, max).intersect(Rect::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(screen_w as f32, screen_h as f32),
            )),
        })
    }
}

impl Default for Pipeline {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
#[path = "pipeline_test.rs"]
mod tests;
