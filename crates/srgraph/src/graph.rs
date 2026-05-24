use srvec2::Vec2;
use srrect::Rect;
use srmatrix::Matrix;
use srpaint::Paint;
use srpath::{Path, PathVerb};
use srflatten::flatten_path;
use srtri::{triangulate, TriMesh};

pub type NodeId = u32;
pub type PaintId = u32;
pub type PathId = u32;

pub const NODE_NULL: NodeId = u32::MAX;

#[derive(Debug, Clone)]
pub struct StoredPath {
    pub verbs: Vec<PathVerb>,
    pub points: Vec<Vec2>,
    pub mesh: TriMesh,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeKind {
    Rect { rect: Rect },
    Oval { center: Vec2, rx: f32, ry: f32 },
    Path { path_id: PathId },
    Group { opacity: f32 },
    Transform(Matrix),
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub kind: NodeKind,
    pub paint_id: PaintId,
    pub transform: Matrix,
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
        let flat = flatten_path(path, 0.5);
        let mesh = triangulate(&flat);
        let bounds = if mesh.vertices.is_empty() {
            Rect::zero()
        } else {
            let mut min = mesh.vertices[0];
            let mut max = mesh.vertices[0];
            for v in &mesh.vertices[1..] {
                min = min.min(*v);
                max = max.max(*v);
            }
            Rect::new(min, max)
        };
        self.paths.push(StoredPath {
            verbs: path.verbs().to_vec(),
            points: path.points().to_vec(),
            mesh,
            bounds,
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
            transform: Matrix::identity(),
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

    pub fn set_transform(&mut self, node: NodeId, transform: Matrix) {
        self.nodes[node as usize].transform = transform;
    }

    pub fn node_bounds(&self, node: NodeId, inherited_transform: Matrix) -> Rect {
        let n = &self.nodes[node as usize];
        let t = inherited_transform.then(n.transform);

        let local = match n.kind {
            NodeKind::Rect { rect } => rect,
            NodeKind::Oval { center, rx, ry } => Rect::new(
                Vec2::new(center.x - rx, center.y - ry),
                Vec2::new(center.x + rx, center.y + ry),
            ),
            NodeKind::Path { path_id } => {
                self.get_path(path_id)
                    .map(|s| s.bounds)
                    .unwrap_or(Rect::zero())
            }
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

fn transform_rect(rect: Rect, transform: &Matrix) -> Rect {
    let tl = transform.transform_point(rect.min);
    let tr = transform.transform_point(Vec2::new(rect.max.x, rect.min.y));
    let bl = transform.transform_point(Vec2::new(rect.min.x, rect.max.y));
    let br = transform.transform_point(rect.max);

    let min = tl.min(tr).min(bl).min(br);
    let max = tl.max(tr).max(bl).max(br);
    Rect::new(min, max)
}

#[cfg(test)]
#[path = "graph_test.rs"]
mod tests;
