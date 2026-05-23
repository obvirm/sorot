use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;
use sorot_path::{Path, PathCache};
use sorot_raster::triangulate;

use crate::display_list::{DisplayList, DrawCommand};
use crate::render_ir::{GpuVertex, RenderFrame, RenderPacket, SdfOp};

const TILE_SIZE: u32 = 32;

pub struct Pipeline {
    pub path_cache: PathCache,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            path_cache: PathCache::new(),
        }
    }

    /// Consume a DisplayList and produce a RenderFrame ready for GPU submission.
    pub fn build_frame(
        &mut self,
        dl: &DisplayList,
        screen_width: u32,
        screen_height: u32,
    ) -> RenderFrame {
        let mut frame = RenderFrame::new(screen_width, screen_height, TILE_SIZE);

        for cmd in &dl.commands {
            match cmd {
                DrawCommand::Rect(dr) => {
                    let path = Path::rect(dr.rect.min, dr.rect.max);
                    if let Some(packet) = self.shape_to_packet(
                        &path,
                        &dr.paint,
                        dr.transform,
                        screen_width,
                        screen_height,
                    ) {
                        frame.bin_packet(packet);
                    }
                }
                DrawCommand::Oval(ov) => {
                    let path = Path::oval(ov.center, ov.rx, ov.ry);
                    if let Some(packet) = self.shape_to_packet(
                        &path,
                        &ov.paint,
                        ov.transform,
                        screen_width,
                        screen_height,
                    ) {
                        frame.bin_packet(packet);
                    }
                }
                DrawCommand::Path(_dp) => {
                    // Path with stored verbs/points — tessellate from stored data
                }
            }
        }

        frame
    }

    fn shape_to_packet(
        &mut self,
        path: &Path,
        paint: &Paint,
        transform: Matrix3x2,
        screen_w: u32,
        screen_h: u32,
    ) -> Option<RenderPacket> {
        let flat = self.path_cache.get_or_flatten(path, 0.5).clone();
        let mesh = triangulate(&flat);

        if mesh.indices.is_empty() {
            return None;
        }

        let cf = {
            let u8 = paint.color.to_premultiplied_u8();
            [
                u8[0] as f32 / 255.0,
                u8[1] as f32 / 255.0,
                u8[2] as f32 / 255.0,
                u8[3] as f32 / 255.0,
            ]
        };

        let hw = screen_w as f32 * 0.5;
        let hh = screen_h as f32 * 0.5;

        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);

        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| {
                let p = transform.transform_point(*v);
                let cx = p.x / hw - 1.0;
                let cy = 1.0 - p.y / hh;
                min = min.min(p);
                max = max.max(p);
                GpuVertex {
                    clip_x: cx,
                    clip_y: cy,
                    r: cf[0],
                    g: cf[1],
                    b: cf[2],
                    a: cf[3],
                }
            })
            .collect();

        let clip_rect = Rect::new(min, max).intersect(Rect::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(screen_w as f32, screen_h as f32),
        ));

        Some(RenderPacket {
            vertices,
            indices: mesh.indices,
            paint: paint.clone(),
            transform,
            clip_rect,
        })
    }

    pub fn sdf_op(
        &self,
        sdf_data: Vec<u8>,
        sdf_w: u32,
        sdf_h: u32,
        bounds: Rect,
        paint: Paint,
    ) -> SdfOp {
        SdfOp {
            sdf_data,
            sdf_width: sdf_w,
            sdf_height: sdf_h,
            bounds,
            paint,
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_list::DrawRect;
    use sorot_core::color::Color;

    #[test]
    fn test_shape_to_packet() {
        let mut pipeline = Pipeline::new();
        let path = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let packet = pipeline.shape_to_packet(
            &path,
            &Paint::fill(Color::RED),
            Matrix3x2::identity(),
            800,
            600,
        );
        assert!(packet.is_some());
    }

    #[test]
    fn test_build_frame() {
        let mut pipeline = Pipeline::new();
        let mut dl = DisplayList::new();
        dl.commands.push(DrawCommand::Rect(DrawRect {
            rect: Rect::new(Vec2::new(10.0, 10.0), Vec2::new(200.0, 200.0)),
            paint: Paint::fill(Color::BLUE),
            transform: Matrix3x2::identity(),
        }));

        let frame = pipeline.build_frame(&dl, 800, 600);
        assert!(!frame.tiles.is_empty());
        let has_packets: bool = frame.tiles.iter().any(|t| !t.packets.is_empty());
        assert!(has_packets);
    }
}
