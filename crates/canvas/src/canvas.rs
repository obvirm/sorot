use color::Color;
use vector::Vec2;
use rect::Rect;
use matrix::Matrix;
use paint::Paint;

use displaylist::DisplayList;
use scenegraph::{NodeId, NodeKind, SceneGraph};

struct DrawState {
    transform: Matrix,
    paint: Paint,
}

pub struct Canvas {
    graph: SceneGraph,
    state_stack: Vec<DrawState>,
    current: DrawState,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            graph: SceneGraph::new(),
            state_stack: Vec::new(),
            current: DrawState {
                transform: Matrix::identity(),
                paint: Paint::fill(Color::BLACK),
            },
        }
    }

    pub fn set_paint(&mut self, paint: Paint) {
        self.current.paint = paint;
    }

    pub fn save(&mut self) {
        self.state_stack.push(DrawState {
            transform: self.current.transform,
            paint: self.current.paint.clone(),
        });
    }

    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current = state;
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.current.transform = self.current.transform.then(Matrix::translate(x, y));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current.transform = self.current.transform.then(Matrix::scale(sx, sy));
    }

    pub fn rotate(&mut self, radians: f32) {
        self.current.transform = self.current.transform.then(Matrix::rotate(radians));
    }

    pub fn draw_rect(&mut self, rect: Rect) {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let node = self.graph.add_node(NodeKind::Rect { rect }, paint_id);
        self.graph.set_transform(node, self.current.transform);
    }

    pub fn draw_oval(&mut self, center: Vec2, rx: f32, ry: f32) {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let node = self.graph.add_node(NodeKind::Oval { center, rx, ry }, paint_id);
        self.graph.set_transform(node, self.current.transform);
    }

    pub fn draw_path(&mut self, path: &pathbuilder::Path) {
        // If stroke paint, convert to stroke outline before storing
        let path_to_store = self.stroke_aware_path(path);
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let path_id = self.graph.store_path(&path_to_store);
        let node = self.graph.add_node(NodeKind::Path { path_id }, paint_id);
        self.graph.set_transform(node, self.current.transform);
    }

    /// If the current paint has a stroke style, flatten + stroke the path
    /// and return the outline as a new Path. Otherwise return the original.
    fn stroke_aware_path(&mut self, path: &pathbuilder::Path) -> pathbuilder::Path {
        match &self.current.paint.style {
            paint::PaintStyle::Fill => {
                path.clone()
            }
            paint::PaintStyle::Stroke { width, cap, join } => {
                // Flatten the path
                let flat = flatten::flatten_path(path, 0.5);
                // Convert stroke to fill outline
                let stroke_cap = match cap {
                    paint::LineCap::Butt => flatten::stroke::LineCap::Butt,
                    paint::LineCap::Round => flatten::stroke::LineCap::Round,
                    paint::LineCap::Square => flatten::stroke::LineCap::Square,
                };
                let stroke_join = match join {
                    paint::LineJoin::Miter => flatten::stroke::LineJoin::Miter,
                    paint::LineJoin::Round => flatten::stroke::LineJoin::Round,
                    paint::LineJoin::Bevel => flatten::stroke::LineJoin::Bevel,
                };
                let outline = flatten::stroke::stroke_path(&flat, *width, stroke_join, stroke_cap, 4.0);
                // Convert back to Path for storage
                flatten::stroke::flattened_to_path(&outline)
            }
        }
    }

    /// Draw a line from p0 to p1 with current paint stroke style.
    pub fn draw_line(&mut self, p0: Vec2, p1: Vec2) {
        let mut path = pathbuilder::Path::new();
        path.move_to(p0);
        path.line_to(p1);
        self.draw_path(&path);
    }

    /// Draw text using SDF rendering at position (x, y).
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32) {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let node = self.graph.add_node(
            NodeKind::Text {
                text: text.to_string(),
                position: Vec2::new(x, y),
                font_size,
            },
            paint_id,
        );
        self.graph.set_transform(node, self.current.transform);
    }

    /// Draw an image from raw RGBA pixel data.
    pub fn draw_image_rgba(
        &mut self,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        dst_rect: Rect,
    ) {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let img_id = self.graph.store_image(pixels, width, height);
        let node = self.graph.add_node(NodeKind::Image { img_id, dst_rect }, paint_id);
        self.graph.set_transform(node, self.current.transform);
    }

    /// Draw an image from a file path.
    pub fn draw_image(&mut self, path: &str, dst_rect: Rect) -> Result<(), String> {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let pixels = rgba.into_raw();
                self.draw_image_rgba(pixels, w, h, dst_rect);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load image '{}': {}", path, e)),
        }
    }

    pub fn begin_group(&mut self, opacity: f32) -> NodeId {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        self.graph.add_node(NodeKind::Group { opacity }, paint_id)
    }

    pub fn end_group(&mut self) {}

    pub fn finalize(&self) -> DisplayList {
        let mut dl = DisplayList::new();
        let identity = Matrix::identity();
        for &root in &self.graph.paint_order {
            if self.graph.nodes[root as usize].parent == u32::MAX {
                displaylist::display_list::build_display_list(
                    &self.graph, root, identity, None, &mut dl,
                );
            }
        }
        dl
    }

    pub fn graph(&self) -> &SceneGraph {
        &self.graph
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "canvas_test.rs"]
mod tests;
