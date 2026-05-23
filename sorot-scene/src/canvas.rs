use sorot_core::color::Color;
use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;

use crate::display_list::DisplayList;
use crate::graph::{NodeId, NodeKind, SceneGraph};

struct DrawState {
    transform: Matrix3x2,
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
                transform: Matrix3x2::identity(),
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
        self.current.transform = self.current.transform.then(Matrix3x2::translate(x, y));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current.transform = self.current.transform.then(Matrix3x2::scale(sx, sy));
    }

    pub fn rotate(&mut self, radians: f32) {
        self.current.transform = self.current.transform.then(Matrix3x2::rotate(radians));
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

    pub fn draw_path(&mut self, path: &sorot_path::Path) {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        let path_id = self.graph.store_path(path);
        let node = self.graph.add_node(NodeKind::Path { path_id }, paint_id);
        self.graph.set_transform(node, self.current.transform);
    }

    pub fn begin_group(&mut self, opacity: f32) -> NodeId {
        let paint_id = self.graph.add_paint(self.current.paint.clone());
        self.graph.add_node(NodeKind::Group { opacity }, paint_id)
    }

    pub fn end_group(&mut self) {}

    pub fn finalize(&self) -> DisplayList {
        let mut dl = DisplayList::new();
        let identity = Matrix3x2::identity();
        for &root in &self.graph.paint_order {
            if self.graph.nodes[root as usize].parent == u32::MAX {
                crate::display_list::build_display_list(
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
mod tests {
    use super::*;
    use sorot_core::color::Color;

    #[test]
    fn test_draw_rect() {
        let mut c = Canvas::new();
        c.set_paint(Paint::fill(Color::RED));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
        let dl = c.finalize();
        assert_eq!(dl.commands.len(), 1);
    }

    #[test]
    fn test_save_restore_transform() {
        let mut c = Canvas::new();
        c.set_paint(Paint::fill(Color::RED));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));

        c.save();
        c.translate(50.0, 0.0);
        c.set_paint(Paint::fill(Color::BLUE));
        c.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
        c.restore();

        c.draw_rect(Rect::new(Vec2::new(200.0, 0.0), Vec2::new(300.0, 100.0)));
        let dl = c.finalize();
        assert_eq!(dl.commands.len(), 3);
    }
}
