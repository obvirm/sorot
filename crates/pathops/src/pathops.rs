use vector::Vec2;
use flatten::FlattenedPath;

/// SDF-based boolean path operations — no CPU pathops needed.
///
/// Traditional engines (Skia, Cairo) do CPU boolean operations
/// on path geometry (intersect edges, rebuild topology). This is
/// O(n²) and numerically fragile.
///
/// Our approach: evaluate SDF for both paths at each pixel,
/// combine SDF values using boolean rules, rasterize directly.
/// No edge topology reconstruction needed.
#[derive(Debug, Clone, Copy)]
pub enum PathOp {
    Union,
    Intersect,
    Subtract,
    Xor,
}

/// Compute SDF for a flattened path at point p.
fn path_sdf(p: Vec2, path: &FlattenedPath) -> f32 {
    let dist = unsigned_distance(p, path);
    let wn = winding(p, path);
    if wn != 0 { -dist } else { dist }
}

fn unsigned_distance(p: Vec2, path: &FlattenedPath) -> f32 {
    let n = path.points.len();
    if n < 2 { return f32::MAX; }
    let mut min = f32::MAX;
    for i in 0..n {
        let a = path.points[i];
        let b = path.points[(i + 1) % n];
        let ab = b - a; let ap = p - a;
        let t = (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let d = (p - (a + ab * t)).length();
        if d < min { min = d; }
    }
    min
}

fn winding(p: Vec2, path: &FlattenedPath) -> i32 {
    let n = path.points.len();
    if n < 3 { return 0; }
    let mut wn = 0;
    for i in 0..n {
        let a = path.points[i];
        let b = path.points[(i + 1) % n];
        if a.y <= p.y && b.y > p.y && (b - a).cross(p - a) > 0.0 { wn += 1; }
        else if b.y <= p.y && a.y > p.y && (b - a).cross(p - a) < 0.0 { wn -= 1; }
    }
    wn
}

/// Evaluate two-path SDF at point p using a boolean operation.
pub fn composite_sdf(p: Vec2, path_a: &FlattenedPath, path_b: &FlattenedPath, op: PathOp) -> f32 {
    let sdf_a = path_sdf(p, path_a);
    let sdf_b = path_sdf(p, path_b);

    match op {
        // Union: p inside either A or B → min(sdf_a, sdf_b)
        // Closer to inside = smaller (more negative) SDF
        PathOp::Union => sdf_a.min(sdf_b),

        // Intersect: p inside both A and B → max(sdf_a, sdf_b)
        // Further from inside = larger SDF
        PathOp::Intersect => sdf_a.max(sdf_b),

        // Subtract (A - B): p inside A but NOT B → max(sdf_a, -sdf_b)
        // Must be inside A AND outside B
        PathOp::Subtract => sdf_a.max(-sdf_b),

        // Xor: inside exactly one → min(max(a, -b), max(-a, b))
        PathOp::Xor => sdf_a.max(-sdf_b).min(sdf_b.max(-sdf_a)),
    }
}

/// Rasterize a composite SDF boolean operation directly to a pixel buffer.
pub fn rasterize_op(
    path_a: &FlattenedPath,
    path_b: &FlattenedPath,
    op: PathOp,
    buffer: &mut [u8],
    width: u32,
    height: u32,
    color_a: [u8; 4],
    color_b: [u8; 4],
) {
    let stride = (width * 4) as usize;
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let csdf = composite_sdf(p, path_a, path_b, op);

            // SDF → alpha via smoothstep
            let alpha = if csdf <= -1.0 { 1.0 }
                else if csdf >= 1.0 { 0.0 }
                else { (1.0 - (csdf + 1.0) * 0.5).clamp(0.0, 1.0) };

            if alpha > 0.0 {
                let idx = (y as usize * stride) + (x as usize * 4);
                // Pick color based on which path is dominant
                let sdf_a = path_sdf(p, path_a);
                let color = if sdf_a < path_sdf(p, path_b) { color_a } else { color_b };
                let sa = color[3] as f32 / 255.0 * alpha;
                let inv = 1.0 - sa;
                buffer[idx] = (color[0] as f32 * sa + buffer[idx] as f32 * inv) as u8;
                buffer[idx + 1] = (color[1] as f32 * sa + buffer[idx + 1] as f32 * inv) as u8;
                buffer[idx + 2] = (color[2] as f32 * sa + buffer[idx + 2] as f32 * inv) as u8;
                buffer[idx + 3] = ((sa * 255.0) + buffer[idx + 3] as f32 * inv).min(255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatten::{FlattenVerb, FlattenedPath};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> FlattenedPath {
        let mut p = FlattenedPath::new();
        p.verbs.push(FlattenVerb::MoveTo); p.points.push(Vec2::new(x, y));
        p.verbs.push(FlattenVerb::LineTo); p.points.push(Vec2::new(x + w, y));
        p.verbs.push(FlattenVerb::LineTo); p.points.push(Vec2::new(x + w, y + h));
        p.verbs.push(FlattenVerb::LineTo); p.points.push(Vec2::new(x, y + h));
        p.verbs.push(FlattenVerb::Close);
        p
    }

    #[test]
    fn test_union_overlap() {
        let a = rect(10.0, 10.0, 40.0, 40.0);
        let b = rect(30.0, 30.0, 40.0, 40.0);
        // Point inside both
        let sdf = composite_sdf(Vec2::new(35.0, 35.0), &a, &b, PathOp::Union);
        assert!(sdf < 0.0, "union: point inside both should be negative SDF");
        // Point outside both
        let sdf2 = composite_sdf(Vec2::new(5.0, 5.0), &a, &b, PathOp::Union);
        assert!(sdf2 > 0.0);
    }

    #[test]
    fn test_subtract() {
        let a = rect(10.0, 10.0, 40.0, 40.0);
        let b = rect(20.0, 20.0, 20.0, 20.0);
        // Point inside A but outside B
        let sdf = composite_sdf(Vec2::new(15.0, 15.0), &a, &b, PathOp::Subtract);
        assert!(sdf < 0.0, "subtract: inside A-outside B should be negative");
        // Point inside both
        let sdf2 = composite_sdf(Vec2::new(25.0, 25.0), &a, &b, PathOp::Subtract);
        assert!(sdf2 > 0.0, "subtract: inside both should be positive (removed)");
    }

    #[test]
    fn test_intersect() {
        let a = rect(10.0, 10.0, 40.0, 40.0);
        let b = rect(30.0, 30.0, 40.0, 40.0);
        // Point inside both
        let sdf = composite_sdf(Vec2::new(35.0, 35.0), &a, &b, PathOp::Intersect);
        assert!(sdf < 0.0);
        // Point inside A only
        let sdf2 = composite_sdf(Vec2::new(15.0, 15.0), &a, &b, PathOp::Intersect);
        assert!(sdf2 > 0.0);
    }
}
