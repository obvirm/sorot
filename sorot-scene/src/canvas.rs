use sorot_core::color::Color;
use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;

use crate::display_list::DisplayList;
use crate::graph::{NodeId, NodeKind, SceneGraph};

pub struct Canvas {
    graph: SceneGraph,
    transform_stack: Vec<Matrix3x2>,
    paint_stack: Vec<Paint>,
    current_paint: Paint,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            graph: SceneGraph::new(),
            transform_stack: Vec::new(),
            paint_stack: Vec::new(),
            current_paint: Paint::fill(Color::BLACK),
        }
    }

    pub fn set_paint(&mut self, paint: Paint) {
        self.current_paint = paint;
    }

    pub fn save(&mut self) {
        self.transform_stack.push(self.current_transform());
        self.paint_stack.push(self.current_paint.clone());
    }

    pub fn restore(&mut self) {
        if let Some(t) = self.transform_stack.pop() {
            self.current_node_mut().transform = t;
        }
        if let Some(p) = self.paint_stack.pop() {
            self.current_paint = p;
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_node_mut().transform = self
            .current_node_mut()
            .transform
            .then(Matrix3x2::translate(x, y));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current_node_mut().transform = self
            .current_node_mut()
            .transform
            .then(Matrix3x2::scale(sx, sy));
    }

    pub fn rotate(&mut self, radians: f32) {
        self.current_node_mut().transform = self
            .current_node_mut()
            .transform
            .then(Matrix3x2::rotate(radians));
    }

    pub fn draw_rect(&mut self, rect: Rect) {
        let paint_id = self.graph.add_paint(self.current_paint.clone());
        self.graph.add_node(NodeKind::Rect { rect }, paint_id);
    }

    pub fn draw_oval(&mut self, center: Vec2, rx: f32, ry: f32) {
        let paint_id = self.graph.add_paint(self.current_paint.clone());
        self.graph.add_node(NodeKind::Oval { center, rx, ry }, paint_id);
    }

    pub fn draw_path(&mut self, path: &sorot_path::Path) {
        let paint_id = self.graph.add_paint(self.current_paint.clone());
        self.graph.add_node(
            NodeKind::Path {
                first_verb: 0,
                verb_count: path.verb_count() as u32,
            },
            paint_id,
        );
    }

    pub fn begin_group(&mut self, opacity: f32) -> NodeId {
        let paint_id = self.graph.add_paint(self.current_paint.clone());
        self.graph.add_node(NodeKind::Group { opacity }, paint_id)
    }

    pub fn end_group(&mut self) {}

    pub fn finalize(&mut self) -> DisplayList {
        let mut dl = DisplayList::new();
        for &root in &self.graph.paint_order.clone() {
            if self.graph.nodes[root as usize].parent == u32::MAX {
                crate::display_list::build_display_list(
                    &self.graph,
                    root,
                    Matrix3x2::identity(),
                    None,
                    &mut dl,
                );
            }
        }
        dl
    }

    fn current_transform(&self) -> Matrix3x2 {
        Matrix3x2::identity()
    }

    fn current_node_mut(&mut self) -> &mut crate::graph::SceneNode {
        &mut self.graph.nodes[0]
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

    #[test]
    fn test_draw_rect() {
        let mut canvas = Canvas::new();
        canvas.set_paint(Paint::fill(Color::RED));
        canvas.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
        let dl = canvas.finalize();
        assert_eq!(dl.commands.len(), 1);
    }

    #[test]
    fn test_draw_oval() {
        let mut canvas = Canvas::new();
        canvas.set_paint(Paint::fill(Color::BLUE));
        canvas.draw_oval(Vec2::new(50.0, 50.0), 30.0, 20.0);
        let dl = canvas.finalize();
        assert_eq!(dl.commands.len(), 1);
    }
}
