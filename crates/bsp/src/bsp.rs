use vector::Vec2;
use rect::Rect;
use color::Color;
use pathbuilder::Path;
use flatten::{flatten_path, FlattenedPath, FlattenVerb};

#[derive(Debug, Clone)]
struct BspEdge {
    a: Vec2, b: Vec2, winding: i8,
    normal: Vec2, offset: f32,
}

impl BspEdge {
    fn distance(&self, p: Vec2) -> f32 { self.normal.dot(p - self.a) }
}

enum BspNode {
    Leaf { winding: i32, solid: bool },
    Split { edge: BspEdge, front: Box<BspNode>, back: Box<BspNode> },
}

pub struct BspTree { root: Option<BspNode>, bbox: Rect }

pub fn build_bsp(path: &FlattenedPath) -> BspTree {
    let edges = collect_edges(path);
    if edges.is_empty() { return BspTree { root: None, bbox: Rect::zero() }; }
    let mut min = edges[0].a.min(edges[0].b);
    let mut max = edges[0].a.max(edges[0].b);
    for e in &edges[1..] { min = min.min(e.a).min(e.b); max = max.max(e.a).max(e.b); }
    let bbox = Rect::new(min, max);
    BspTree { root: Some(build_node(&edges)), bbox }
}

fn collect_edges(path: &FlattenedPath) -> Vec<BspEdge> {
    let mut edges = Vec::new();
    let mut first = None; let mut last = None;
    let mut vi = 0; let mut pi = 0;
    while vi < path.verbs.len() {
        match path.verbs[vi] {
            FlattenVerb::MoveTo => { first = Some(path.points[pi]); last = first; pi += 1; }
            FlattenVerb::LineTo => {
                let from = last.unwrap(); let to = path.points[pi]; pi += 1;
                if from == to { vi += 1; continue; }
                let dir = (to - from).normalize();
                let normal = dir.perp();
                edges.push(BspEdge {
                    a: from, b: to,
                    winding: if from.y <= to.y { 1 } else { -1 },
                    normal, offset: normal.dot(from),
                });
                last = Some(to);
            }
            FlattenVerb::Close => { last = first; }
        }
        vi += 1;
    }
    edges
}

fn build_node(edges: &[BspEdge]) -> BspNode {
    if edges.is_empty() { return BspNode::Leaf { winding: 0, solid: true }; }
    let center = center_of_edges(edges);
    let winding = compute_winding(edges, center);
    let bbox = edges_bbox(edges);
    let corners = [bbox.min, Vec2::new(bbox.max.x, bbox.min.y), Vec2::new(bbox.min.x, bbox.max.y), bbox.max];
    let all_same = corners.iter().all(|&c| compute_winding(edges, c) == winding);
    if all_same { return BspNode::Leaf { winding, solid: true }; }

    let split_idx = pick_split(edges);
    let split_edge = edges[split_idx].clone();
    let mut front = Vec::new(); let mut back = Vec::new();
    for (i, e) in edges.iter().enumerate() {
        if i == split_idx { continue; }
        if e.normal.dot(e.a - split_edge.a) > 0.0 { front.push(e.clone()); } else { back.push(e.clone()); }
    }

    if front.is_empty() && back.is_empty() { return BspNode::Leaf { winding, solid: false }; }
    BspNode::Split {
        edge: split_edge,
        front: Box::new(build_node(&front)),
        back: Box::new(build_node(&back)),
    }
}

fn center_of_edges(edges: &[BspEdge]) -> Vec2 {
    let mut sum = Vec2::zero(); for e in edges { sum = sum + e.a + e.b; }
    sum / (2.0 * edges.len() as f32)
}

fn edges_bbox(edges: &[BspEdge]) -> Rect {
    let mut min = edges[0].a.min(edges[0].b); let mut max = edges[0].a.max(edges[0].b);
    for e in &edges[1..] { min = min.min(e.a).min(e.b); max = max.max(e.a).max(e.b); }
    Rect::new(min, max)
}

fn compute_winding(edges: &[BspEdge], p: Vec2) -> i32 {
    edges.iter().map(|e| {
        if e.a.y <= p.y && e.b.y > p.y && (e.b - e.a).cross(p - e.a) > 0.0 { e.winding as i32 }
        else if e.b.y <= p.y && e.a.y > p.y && (e.b - e.a).cross(p - e.a) < 0.0 { -(e.winding as i32) }
        else { 0 }
    }).sum()
}

fn pick_split(edges: &[BspEdge]) -> usize {
    let mut best = 0; let mut best_score = i32::MAX;
    for (i, edge) in edges.iter().enumerate() {
        let (mut f, mut b) = (0i32, 0i32);
        for (j, e) in edges.iter().enumerate() {
            if j == i { continue; }
            if e.normal.dot(e.a - edge.a) > 0.0 { f += 1; } else { b += 1; }
        }
        let score = (f - b).abs();
        if score < best_score { best_score = score; best = i; }
    }
    best
}

impl BspTree {
    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32, color: Color) {
        let Some(ref root) = self.root else { return };
        let rgba = color.to_premultiplied_u8(); let stride = (width * 4) as usize;
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                if traverse(root, Vec2::new(x as f32 + 0.5, y as f32 + 0.5)) != 0 {
                    let idx = (y as usize * stride) + (x as usize * 4);
                    let sa = rgba[3] as f32 / 255.0; let inv = 1.0 - sa;
                    buffer[idx] = (rgba[0] as f32 + buffer[idx] as f32 * inv) as u8;
                    buffer[idx+1] = (rgba[1] as f32 + buffer[idx+1] as f32 * inv) as u8;
                    buffer[idx+2] = (rgba[2] as f32 + buffer[idx+2] as f32 * inv) as u8;
                    buffer[idx+3] = ((sa * 255.0) + buffer[idx+3] as f32 * inv).min(255.0) as u8;
                }
            }
        }
    }
}

fn traverse(node: &BspNode, p: Vec2) -> i32 {
    match node {
        BspNode::Leaf { winding, .. } => *winding,
        BspNode::Split { edge, front, back } => {
            if edge.distance(p) > 0.0 { traverse(front, p) } else { traverse(back, p) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bsp_triangle() {
        let mut p = Path::new();
        p.move_to(Vec2::new(50.0, 10.0)); p.line_to(Vec2::new(90.0, 90.0));
        p.line_to(Vec2::new(10.0, 90.0)); p.close();
        let flat = flatten_path(&p, 0.5);
        let tree = build_bsp(&flat);
        let mut buf = vec![0u8; 100 * 100 * 4];
        tree.render(&mut buf, 100, 100, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let c = ((50 * 100 + 50) * 4) as usize;
        assert!(buf[c + 3] > 0);
    }
}
