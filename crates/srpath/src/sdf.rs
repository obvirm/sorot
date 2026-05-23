use srcore::math::Vec2;

/// Signed distance from a point to a line segment.
/// Positive = point is to the "right" of the directed edge.
#[inline]
pub fn signed_distance_to_line(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    ab.cross(ap) / ab.length()
}

/// Unsigned distance from a point to a line segment.
#[inline]
pub fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_sq();

    if ab_len_sq < 1e-12 {
        return ap.length();
    }

    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

/// Minimum unsigned distance from point p to a polyline defined by points.
#[inline]
pub fn distance_to_polyline(p: Vec2, points: &[Vec2], closed: bool) -> f32 {
    if points.len() < 2 {
        return f32::MAX;
    }

    let n = if closed {
        points.len()
    } else {
        points.len() - 1
    };

    let mut min_dist = f32::MAX;
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let d = distance_to_segment(p, a, b);
        if d < min_dist {
            min_dist = d;
        }
    }

    min_dist
}

/// Determine winding number of point p relative to polygon defined by points.
/// Uses the standard ray-crossing algorithm.
pub fn winding_number(p: Vec2, points: &[Vec2]) -> i32 {
    let n = points.len();
    if n < 3 {
        return 0;
    }

    let mut wn = 0;

    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];

        if a.y <= p.y {
            if b.y > p.y {
                let cross = (b - a).cross(p - a);
                if cross > 0.0 {
                    wn += 1;
                }
            }
        } else {
            if b.y <= p.y {
                let cross = (b - a).cross(p - a);
                if cross < 0.0 {
                    wn -= 1;
                }
            }
        }
    }

    wn
}

/// Anti-aliased coverage using signed distance field.
/// Returns alpha value [0, 1] based on signed distance to edge.
#[inline]
pub fn sdf_alpha(signed_distance: f32, range: f32) -> f32 {
    if signed_distance >= range {
        1.0
    } else if signed_distance <= -range {
        0.0
    } else {
        (signed_distance / range) * 0.5 + 0.5
    }
}

/// Smoothstep variant for SDF edge AA.
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
#[path = "sdf_test.rs"]
mod tests;
