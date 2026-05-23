use srcore::math::{Matrix, Rect, Vec2};
use srcore::paint::Paint;
use srpath::Path;
use srrender::render_ir::{GpuVertex, RenderFrame, RenderPacket};

use crate::display_list::{DisplayList, DrawCommand};
use crate::geometry_cache::GeometryCache;
use crate::graph::SceneGraph;

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
            let packet = match cmd {
                DrawCommand::Rect(dr) => {
                    let path = Path::rect(dr.rect.min, dr.rect.max);
                    let mesh = self.geom_cache.get_mesh(&path, 0.5);
                    Self::mesh_to_packet(mesh, &dr.paint, dr.transform, screen_width, screen_height)
                }
                DrawCommand::Oval(ov) => {
                    let path = Path::oval(ov.center, ov.rx, ov.ry);
                    let mesh = self.geom_cache.get_mesh(&path, 0.5);
                    Self::mesh_to_packet(mesh, &ov.paint, ov.transform, screen_width, screen_height)
                }
                DrawCommand::Path(dp) => graph
                    .get_path(dp.path_id)
                    .map(|stored| {
                        Self::mesh_to_packet(&stored.mesh, &dp.paint, dp.transform, screen_width, screen_height)
                    })
                    .flatten(),
            };

            if let Some(p) = packet {
                frame.bin_packet(p);
            }
        }

        frame
    }

    fn mesh_to_packet(
        mesh: &srraster::TriMesh,
        paint: &Paint,
        transform: Matrix,
        screen_w: u32,
        screen_h: u32,
    ) -> Option<RenderPacket> {
        if mesh.indices.is_empty() {
            return None;
        }

        let cf = {
            let u8 = paint.color.to_premultiplied_u8();
            [u8[0] as f32 / 255.0, u8[1] as f32 / 255.0, u8[2] as f32 / 255.0, u8[3] as f32 / 255.0]
        };

        let hw = screen_w as f32 * 0.5;
        let hh = screen_h as f32 * 0.5;

        let (mut min, mut max) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));

        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| {
                let p = transform.transform_point(*v);
                min = min.min(p);
                max = max.max(p);
                GpuVertex {
                    clip_x: p.x / hw - 1.0,
                    clip_y: 1.0 - p.y / hh,
                    r: cf[0], g: cf[1], b: cf[2], a: cf[3],
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
mod tests {
    use super::*;
    use crate::display_list::DrawRect;
    use srcore::color::Color;

    #[test]
    fn test_build_frame() {
        let mut pipeline = Pipeline::new();
        let graph = SceneGraph::new();
        let mut dl = DisplayList::new();
        dl.commands.push(DrawCommand::Rect(DrawRect {
            rect: Rect::new(Vec2::new(10.0, 10.0), Vec2::new(200.0, 200.0)),
            paint: Paint::fill(Color::BLUE),
            transform: Matrix::identity(),
        }));
        let frame = pipeline.build_frame(&graph, &dl, 800, 600);
        assert!(frame.non_empty_tiles() > 0);
    }
}
