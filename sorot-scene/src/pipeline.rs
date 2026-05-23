use rayon::prelude::*;
use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;
use sorot_path::{flatten_path, Path, PathVerb};
use sorot_raster::triangulate;
use sorot_render::render_ir::{GpuVertex, RenderFrame, RenderPacket};

use crate::display_list::{DisplayList, DrawCommand};
use crate::graph::SceneGraph;

const TILE_SIZE: u32 = 32;

pub struct Pipeline;

impl Pipeline {
    pub fn build_frame(
        graph: &SceneGraph,
        dl: &DisplayList,
        screen_width: u32,
        screen_height: u32,
    ) -> RenderFrame {
        let mut frame = RenderFrame::new(screen_width, screen_height, TILE_SIZE);

        let packets: Vec<Option<RenderPacket>> = dl
            .commands
            .par_iter()
            .map(|cmd| match cmd {
                DrawCommand::Rect(dr) => {
                    let path = Path::rect(dr.rect.min, dr.rect.max);
                    Self::shape_to_packet_static(
                        &path, &dr.paint, dr.transform, screen_width, screen_height,
                    )
                }
                DrawCommand::Oval(ov) => {
                    let path = Path::oval(ov.center, ov.rx, ov.ry);
                    Self::shape_to_packet_static(
                        &path, &ov.paint, ov.transform, screen_width, screen_height,
                    )
                }
                DrawCommand::Path(dp) => graph
                    .get_path(dp.path_id)
                    .and_then(|stored| {
                        let mut path = Path::new();
                        let mut pi = 0;
                        for &verb in &stored.verbs {
                            match verb {
                                PathVerb::MoveTo => { path.move_to(stored.points[pi]); pi += 1; }
                                PathVerb::LineTo => { path.line_to(stored.points[pi]); pi += 1; }
                                PathVerb::QuadTo => { path.quad_to(stored.points[pi], stored.points[pi + 1]); pi += 2; }
                                PathVerb::CubicTo => { path.cubic_to(stored.points[pi], stored.points[pi + 1], stored.points[pi + 2]); pi += 3; }
                                PathVerb::Close => { path.close(); }
                            }
                        }
                        Self::shape_to_packet_static(
                            &path, &dp.paint, dp.transform, screen_width, screen_height,
                        )
                    }),
            })
            .collect();

        for packet in packets.into_iter().flatten() {
            frame.bin_packet(packet);
        }

        frame
    }

    fn shape_to_packet_static(
        path: &Path,
        paint: &Paint,
        transform: Matrix3x2,
        screen_w: u32,
        screen_h: u32,
    ) -> Option<RenderPacket> {
        let flat = flatten_path(path, 0.5);
        let mesh = triangulate(&flat);

        if mesh.indices.is_empty() {
            return None;
        }

        let cf = {
            let u8 = paint.color.to_premultiplied_u8();
            [u8[0] as f32 / 255.0, u8[1] as f32 / 255.0, u8[2] as f32 / 255.0, u8[3] as f32 / 255.0]
        };

        let hw = screen_w as f32 * 0.5;
        let hh = screen_h as f32 * 0.5;

        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| {
                let p = transform.transform_point(*v);
                let cx = p.x / hw - 1.0;
                let cy = 1.0 - p.y / hh;
                GpuVertex { clip_x: cx, clip_y: cy, r: cf[0], g: cf[1], b: cf[2], a: cf[3] }
            })
            .collect();

        let (min, max) = {
            let mut min = Vec2::new(f32::MAX, f32::MAX);
            let mut max = Vec2::new(f32::MIN, f32::MIN);
            for v in &mesh.vertices {
                let p = transform.transform_point(*v);
                min = min.min(p);
                max = max.max(p);
            }
            (min, max)
        };

        Some(RenderPacket {
            vertices: vertices.into_boxed_slice(),
            indices: mesh.indices.into_boxed_slice(),
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
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_list::DrawRect;
    use sorot_core::color::Color;

    #[test]
    fn test_shape_to_packet() {
        let path = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let packet = Pipeline::shape_to_packet_static(
            &path, &Paint::fill(Color::RED), Matrix3x2::identity(), 800, 600,
        );
        assert!(packet.is_some());
    }

    #[test]
    fn test_build_frame() {
        let mut graph = SceneGraph::new();
        let mut dl = DisplayList::new();
        dl.commands.push(DrawCommand::Rect(DrawRect {
            rect: Rect::new(Vec2::new(10.0, 10.0), Vec2::new(200.0, 200.0)),
            paint: Paint::fill(Color::BLUE),
            transform: Matrix3x2::identity(),
        }));
        let frame = Pipeline::build_frame(&graph, &dl, 800, 600);
        assert!(frame.non_empty_tiles() > 0);
    }
}
