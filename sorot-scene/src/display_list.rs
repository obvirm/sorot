use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;

use crate::graph::{NodeId, NodeKind, SceneGraph, NODE_NULL};

#[derive(Debug, Clone)]
pub struct DrawRect {
    pub rect: Rect,
    pub paint: Paint,
    pub transform: Matrix3x2,
}

#[derive(Debug, Clone)]
pub struct DrawOval {
    pub center: Vec2,
    pub rx: f32,
    pub ry: f32,
    pub paint: Paint,
    pub transform: Matrix3x2,
}

#[derive(Debug, Clone)]
pub struct DrawPath {
    pub path_verbs: Vec<u8>,
    pub path_points: Vec<Vec2>,
    pub paint: Paint,
    pub transform: Matrix3x2,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Rect(DrawRect),
    Oval(DrawOval),
    Path(DrawPath),
}

#[derive(Debug, Clone)]
pub struct DisplayList {
    pub commands: Vec<DrawCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
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
    inherited_transform: Matrix3x2,
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

    match n.kind {
        NodeKind::Rect { rect } => {
            out.push(DrawCommand::Rect(DrawRect {
                rect,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Oval { center, rx, ry } => {
            out.push(DrawCommand::Oval(DrawOval {
                center,
                rx,
                ry,
                paint: paint.clone(),
                transform: t,
            }));
        }
        NodeKind::Path {
            first_verb: _,
            verb_count: _,
        } => {
            out.push(DrawCommand::Path(DrawPath {
                path_verbs: Vec::new(),
                path_points: Vec::new(),
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
