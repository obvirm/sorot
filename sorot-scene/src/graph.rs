use sorot_core::math::{Matrix3x2, Rect, Vec2};
use sorot_core::paint::Paint;
use sorot_path::{Path, PathVerb};

pub type NodeId = u32;
pub type PaintId = u32;
pub type PathId = u32;

pub const NODE_NULL: NodeId = u32::MAX;

#[derive(Debug, Clone)]
pub struct StoredPath {
    pub verbs: Vec<PathVerb>,
    pub points: Vec<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeKind {
    Rect { rect: Rect },
    Oval { center: Vec2, rx: f32, ry: f32 },
    Path { path_id: PathId },
    Group { opacity: f32 },
    Transform(Matrix3x2),
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub kind: NodeKind,
    pub paint_id: PaintId,
    pub transform: Matrix3x2,
    pub parent: NodeId,
    pub first_child: NodeId,
    pub next_sibling: NodeId,
    pub visible: bool,
}

pub struct SceneGraph {
    pub nodes: Vec<SceneNode>,
    pub paints: Vec<Paint>,
    pub paths: Vec<StoredPath>,
    pub paint_order: Vec<NodeId>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            paints: Vec::new(),
            paths: Vec::new(),
            paint_order: Vec::new(),
        }
    }

    pub fn add_paint(&mut self, paint: Paint) -> PaintId {
        let id = self.paints.len() as PaintId;
        self.paints.push(paint);
        id
    }

    pub fn store_path(&mut self, path: &Path) -> PathId {
        let id = self.paths.len() as PathId;
        self.paths.push(StoredPath {
            verbs: path.verbs().to_vec(),
            points: path.points().to_vec(),
        });
        id
    }

    pub fn get_path(&self, id: PathId) -> Option<&StoredPath> {
        self.paths.get(id as usize)
    }

    pub fn add_node(&mut self, kind: NodeKind, paint_id: PaintId) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(SceneNode {
            kind,
            paint_id,
            transform: Matrix3x2::identity(),
            parent: NODE_NULL,
            first_child: NODE_NULL,
            next_sibling: NODE_NULL,
            visible: true,
        });
        self.paint_order.push(id);
        id
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        let parent_node = &mut self.nodes[parent as usize];
        let old_first = parent_node.first_child;
        parent_node.first_child = child;

        let child_node = &mut self.nodes[child as usize];
        child_node.parent = parent;
        child_node.next_sibling = old_first;
    }

    pub fn set_transform(&mut self, node: NodeId, transform: Matrix3x2) {
        self.nodes[node as usize].transform = transform;
    }

    pub fn node_bounds(&self, node: NodeId, inherited_transform: Matrix3x2) -> Rect {
        let n = &self.nodes[node as usize];
        let t = inherited_transform.then(n.transform);

        let local = match n.kind {
            NodeKind::Rect { rect } => rect,
            NodeKind::Oval { center, rx, ry } => Rect::new(
                Vec2::new(center.x - rx, center.y - ry),
                Vec2::new(center.x + rx, center.y + ry),
            ),
            NodeKind::Path { .. } => Rect::zero(),
            NodeKind::Group { .. } | NodeKind::Transform(_) => Rect::zero(),
        };

        let mut bounds = transform_rect(local, &t);

        let mut child = n.first_child;
        while child != NODE_NULL {
            let cb = self.node_bounds(child, t);
            if !cb.is_empty() {
                bounds = if bounds.is_empty() {
                    cb
                } else {
                    bounds.union(cb)
                };
            }
            child = self.nodes[child as usize].next_sibling;
        }

        bounds
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn transform_rect(rect: Rect, transform: &Matrix3x2) -> Rect {
    let tl = transform.transform_point(rect.min);
    let tr = transform.transform_point(Vec2::new(rect.max.x, rect.min.y));
    let bl = transform.transform_point(Vec2::new(rect.min.x, rect.max.y));
    let br = transform.transform_point(rect.max);

    let min = tl.min(tr).min(bl).min(br);
    let max = tl.max(tr).max(bl).max(br);
    Rect::new(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sorot_core::color::Color;

    #[test]
    fn test_store_path() {
        let mut g = SceneGraph::new();
        let p = Path::circle(Vec2::new(50.0, 50.0), 30.0);
        let id = g.store_path(&p);
        let stored = g.get_path(id).unwrap();
        assert_eq!(stored.verbs.len(), p.verb_count());
    }

    #[test]
    fn test_add_node() {
        let mut g = SceneGraph::new();
        let paint = g.add_paint(Paint::fill(Color::RED));
        let rect = g.add_node(
            NodeKind::Rect { rect: Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)) },
            paint,
        );
        assert_eq!(rect, 0);
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn test_hierarchy() {
        let mut g = SceneGraph::new();
        let p = g.add_paint(Paint::fill(Color::BLACK));
        let parent = g.add_node(NodeKind::Group { opacity: 1.0 }, p);
        let child = g.add_node(
            NodeKind::Rect { rect: Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)) },
            p,
        );
        g.add_child(parent, child);
        assert_eq!(g.nodes[parent as usize].first_child, child);
        assert_eq!(g.nodes[child as usize].parent, parent);
    }
}
