use vector::Vec2;
use rect::Rect;
use matrix::Matrix;
use paint::Paint;

use scenegraph::{NodeId, NodeKind, SceneGraph, NODE_NULL};

#[derive(Debug, Clone)]
pub struct DrawRect {
    pub rect: Rect,
    pub paint: Paint,
    pub transform: Matrix,
}

#[derive(Debug, Clone)]
pub struct DrawOval {
    pub center: Vec2,
    pub rx: f32,
    pub ry: f32,
    pub paint: Paint,
    pub transform: Matrix,
}

#[derive(Debug, Clone)]
pub struct DrawPath {
    pub path_id: u32,
    pub paint: Paint,
    pub transform: Matrix,
}

#[derive(Debug, Clone)]
pub struct DrawImage {
    pub img_id: u32,
    pub dst_rect: Rect,
    pub paint: Paint,
    pub transform: Matrix,
}

#[derive(Debug, Clone)]
pub struct DrawText {
    pub text: String,
    pub position: Vec2,
    pub font_size: f32,
    pub paint: Paint,
    pub transform: Matrix,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Rect(DrawRect),
    Oval(DrawOval),
    Path(DrawPath),
    Image(DrawImage),
    Text(DrawText),
}

#[derive(Debug, Clone)]
pub struct DisplayList {
    pub commands: Vec<DrawCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

impl Default for DisplayList {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_display_list(
    graph: &SceneGraph,
    node: NodeId,
    inherited_transform: Matrix,
    paint_override: Option<&Paint>,
    out: &mut DisplayList,
) {
    if node == NODE_NULL {
        return;
    }

    let n = &graph.nodes[node as usize];
    if !n.visible {
        return;
    }

    let t = inherited_transform.then(n.transform);
    let paint = paint_override.unwrap_or(&graph.paints[n.paint_id as usize]);

    match &n.kind {
        NodeKind::Rect { rect } => {
            out.push(DrawCommand::Rect(DrawRect {
                rect: *rect,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Oval { center, rx, ry } => {
            out.push(DrawCommand::Oval(DrawOval {
                center: *center,
                rx: *rx,
                ry: *ry,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Path { path_id } => {
            out.push(DrawCommand::Path(DrawPath {
                path_id: *path_id,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Image { img_id, dst_rect } => {
            out.push(DrawCommand::Image(DrawImage {
                img_id: *img_id,
                dst_rect: *dst_rect,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Text { text, position, font_size } => {
            out.push(DrawCommand::Text(DrawText {
                text: text.clone(),
                position: *position,
                font_size: *font_size,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Group { opacity: _ } | NodeKind::Transform(_) => {}
    }

    let mut child = n.first_child;
    while child != NODE_NULL {
        build_display_list(graph, child, t, None, out);
        child = graph.nodes[child as usize].next_sibling;
    }
}
