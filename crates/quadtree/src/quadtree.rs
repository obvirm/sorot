use vector::Vec2;
use rect::Rect;
use color::Color;
use flatten::FlattenedPath;

/// Edge-distance quadtree: adaptive spatial subdivision.
/// 
/// Only subdivides at edges. Flat regions (pure inside/outside)
/// store a single color value. This compresses the rasterization
/// result better than a full pixel buffer.
#[derive(Debug, Clone)]
pub enum QtNode {
    /// Solidly inside the shape — fill with this color.
    Solid(Color),
    /// Solidly outside — transparent.
    Empty,
    /// Contains edges — needs further subdivision.
    Mixed(Box<[QtNode; 4]>),
}

/// Decision for a quadtree cell based on distance to edges.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CellClass {
    Outside,
    Inside,
    Edge(f32), // distance to nearest edge
}

pub struct EdgeQuadtree {
    root: QtNode,
    max_depth: u32,
    bbox: Rect,
}

impl EdgeQuadtree {
    pub fn build(
        path: &FlattenedPath,
        bbox: Rect,
        fill_color: Color,
        max_depth: u32,
    ) -> Self {
        let root = subdivide(bbox, path, fill_color, 0, max_depth);
        Self { root, max_depth, bbox }
    }

    /// Render quadtree to pixel buffer.
    pub fn render(&self, buffer: &mut [u8], width: u32, height: u32) {
        let stride = (width * 4) as usize;
        render_node(&self.root, self.bbox, buffer, width, height, stride);
    }

    /// Count leaf nodes.
    pub fn leaf_count(&self) -> usize {
        count_leaves(&self.root)
    }
}

fn subdivide(
    rect: Rect,
    path: &FlattenedPath,
    fill_color: Color,
    depth: u32,
    max_depth: u32,
) -> QtNode {
    // Classify the rect by checking 4 corners + center
    let corners = [
        rect.min,
        Vec2::new(rect.max.x, rect.min.y),
        Vec2::new(rect.min.x, rect.max.y),
        rect.max,
        rect.center(),
    ];

    let classes: Vec<CellClass> = corners.iter()
        .map(|&p| classify(p, path))
        .collect();

    let all_outside = classes.iter().all(|c| matches!(c, CellClass::Outside));
    if all_outside {
        return QtNode::Empty;
    }

    let all_inside = classes.iter().all(|c| matches!(c, CellClass::Inside));
    if all_inside {
        return QtNode::Solid(fill_color);
    }

    // Mixed — subdivide if not at max depth
    if depth >= max_depth || rect.width() < 2.0 || rect.height() < 2.0 {
        // Leaf mixed node — approximate with distance-based AA
        let center = rect.center();
        let alpha = match classify(center, path) {
            CellClass::Outside => 0.0,
            CellClass::Inside => 1.0,
            CellClass::Edge(dist) => {
                let edge_alpha = (dist / 2.0).clamp(0.0, 1.0);
                if winding_at(path, center) != 0 { edge_alpha } else { 1.0 - edge_alpha }
            }
        };
        if alpha > 0.0 {
            QtNode::Solid(Color::from_rgba(
                fill_color.unpremultiply().0,
                fill_color.unpremultiply().1,
                fill_color.unpremultiply().2,
                alpha,
            ))
        } else {
            QtNode::Empty
        }
    } else {
        let cx = (rect.min.x + rect.max.x) * 0.5;
        let cy = (rect.min.y + rect.max.y) * 0.5;
        let children = Box::new([
            subdivide(Rect::new(rect.min, Vec2::new(cx, cy)), path, fill_color, depth + 1, max_depth),
            subdivide(Rect::new(Vec2::new(cx, rect.min.y), Vec2::new(rect.max.x, cy)), path, fill_color, depth + 1, max_depth),
            subdivide(Rect::new(Vec2::new(rect.min.x, cy), Vec2::new(cx, rect.max.y)), path, fill_color, depth + 1, max_depth),
            subdivide(Rect::new(Vec2::new(cx, cy), rect.max), path, fill_color, depth + 1, max_depth),
        ]);
        QtNode::Mixed(children)
    }
}

fn classify(p: Vec2, path: &FlattenedPath) -> CellClass {
    let (nearest_dist, _) = nearest_edge(p, path);
    let wn = winding_at(path, p);

    if nearest_dist > 2.0 {
        if wn != 0 { CellClass::Inside } else { CellClass::Outside }
    } else {
        CellClass::Edge(nearest_dist)
    }
}

fn nearest_edge(p: Vec2, path: &FlattenedPath) -> (f32, usize) {
    let mut min_dist = f32::MAX;
    let mut min_idx = 0;
    let n = path.points.len();
    if n < 2 { return (min_dist, min_idx); }

    for i in 0..n {
        let a = path.points[i];
        let b = if path.verbs.get(i).map_or(false, |v| *v == flatten::FlattenVerb::Close) {
            path.points[0]
        } else if i + 1 < n {
            path.points[i + 1]
        } else {
            continue;
        };
        let ab = b - a;
        let ap = p - a;
        let t = (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let closest = a + ab * t;
        let dist = (p - closest).length();
        if dist < min_dist {
            min_dist = dist;
            min_idx = i;
        }
    }
    (min_dist, min_idx)
}

fn winding_at(path: &FlattenedPath, p: Vec2) -> i32 {
    let n = path.points.len();
    if n < 3 { return 0; }
    let mut wn = 0;
    for i in 0..n {
        let a = path.points[i];
        let b = path.points[(i + 1) % n];
        if a.y <= p.y && b.y > p.y && (b - a).cross(p - a) > 0.0 {
            wn += 1;
        } else if b.y <= p.y && a.y > p.y && (b - a).cross(p - a) < 0.0 {
            wn -= 1;
        }
    }
    wn
}

fn render_node(node: &QtNode, rect: Rect, buffer: &mut [u8], _width: u32, _height: u32, stride: usize) {
    match node {
        QtNode::Solid(color) => {
            let rgba = color.to_premultiplied_u8();
            for py in (rect.min.y.max(0.0) as i32)..(rect.max.y.min(_height as f32).ceil() as i32) {
                for px in (rect.min.x.max(0.0) as i32)..(rect.max.x.min(_width as f32).ceil() as i32) {
                    let idx = (py as usize * stride) + (px as usize * 4);
                    if idx + 4 <= buffer.len() {
                        let sa = rgba[3] as f32 / 255.0;
                        let inv = 1.0 - sa;
                        buffer[idx] = (rgba[0] as f32 + buffer[idx] as f32 * inv) as u8;
                        buffer[idx + 1] = (rgba[1] as f32 + buffer[idx + 1] as f32 * inv) as u8;
                        buffer[idx + 2] = (rgba[2] as f32 + buffer[idx + 2] as f32 * inv) as u8;
                        buffer[idx + 3] = ((sa * 255.0) + buffer[idx + 3] as f32 * inv).min(255.0) as u8;
                    }
                }
            }
        }
        QtNode::Empty => {}
        QtNode::Mixed(children) => {
            let cx = (rect.min.x + rect.max.x) * 0.5;
            let cy = (rect.min.y + rect.max.y) * 0.5;
            render_node(&children[0], Rect::new(rect.min, Vec2::new(cx, cy)), buffer, _width, _height, stride);
            render_node(&children[1], Rect::new(Vec2::new(cx, rect.min.y), Vec2::new(rect.max.x, cy)), buffer, _width, _height, stride);
            render_node(&children[2], Rect::new(Vec2::new(rect.min.x, cy), Vec2::new(cx, rect.max.y)), buffer, _width, _height, stride);
            render_node(&children[3], Rect::new(Vec2::new(cx, cy), rect.max), buffer, _width, _height, stride);
        }
    }
}

fn count_leaves(node: &QtNode) -> usize {
    match node {
        QtNode::Solid(_) | QtNode::Empty => 1,
        QtNode::Mixed(children) => children.iter().map(|c| count_leaves(c)).sum(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatten::{FlattenVerb, FlattenedPath};

    #[test]
    fn test_quadtree_square() {
        let mut p = FlattenedPath::new();
        p.verbs.push(FlattenVerb::MoveTo);
        p.points.push(Vec2::new(10.0, 10.0));
        p.verbs.push(FlattenVerb::LineTo);
        p.points.push(Vec2::new(90.0, 10.0));
        p.verbs.push(FlattenVerb::LineTo);
        p.points.push(Vec2::new(90.0, 90.0));
        p.verbs.push(FlattenVerb::LineTo);
        p.points.push(Vec2::new(10.0, 90.0));
        p.verbs.push(FlattenVerb::Close);

        let bbox = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let tree = EdgeQuadtree::build(&p, bbox, Color::from_rgba(1.0, 0.0, 0.0, 1.0), 4);
        let mut buf = vec![0u8; 100 * 100 * 4];
        tree.render(&mut buf, 100, 100);
        let center = ((50 * 100 + 50) * 4) as usize;
        assert!(buf[center + 3] > 0, "center should be filled");
        assert!(tree.leaf_count() > 0);
    }
}
