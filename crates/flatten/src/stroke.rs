use vector::Vec2;
use pathbuilder::Path;
use crate::{FlattenVerb, FlattenedPath};

/// Convert a FlattenedPath (line segments only) back into a Path.
/// Used by Canvas to convert stroke outlines into stored paths.
pub fn flattened_to_path(flat: &FlattenedPath) -> Path {
    let mut path = Path::new();
    let mut vi = 0;
    let mut pi = 0;

    while vi < flat.verbs.len() {
        let verb = flat.verbs[vi];
        match verb {
            FlattenVerb::MoveTo => {
                path.move_to(flat.points[pi]);
                pi += 1;
            }
            FlattenVerb::LineTo => {
                path.line_to(flat.points[pi]);
                pi += 1;
            }
            FlattenVerb::Close => {
                path.close();
            }
        }
        vi += 1;
    }

    path
}

/// Line cap style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Line join style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Convert a flattened path's stroke into a filled outline path.
///
/// Takes a path that has been flattened to line segments and produces
/// a new closed polygon representing the stroke outline at `width`.
/// The resulting path can be passed to `triangulate` for rendering.
///
/// Algorithm:
/// 1. Walk each subpath's line segments
/// 2. Offset each segment by half width along the normal
/// 3. Join consecutive offset edges using the specified join style
/// 4. Add end caps for open subpaths
/// 5. Produce a single closed polygon
pub fn stroke_path(
    path: &FlattenedPath,
    width: f32,
    join: LineJoin,
    cap: LineCap,
    miter_limit: f32,
) -> FlattenedPath {
    if path.is_empty() || width <= 0.0 {
        return FlattenedPath::new();
    }

    let half_w = width * 0.5;
    let mut result = FlattenedPath::new();

    // Parse the flattened path into subpaths
    let subpaths = extract_subpaths(path);
    for subpath in &subpaths {
        stroke_subpath(subpath, half_w, join, cap, miter_limit, &mut result);
    }

    result
}

/// A subpath extracted from the flattened path.
struct SubPath {
    points: Vec<Vec2>,
    closed: bool,
}

fn extract_subpaths(path: &FlattenedPath) -> Vec<SubPath> {
    let mut subpaths = Vec::new();
    let mut current_points: Vec<Vec2> = Vec::new();
    let mut closed = false;

    for (verb, pts) in path_verbs_iter(path) {
        match verb {
            FlattenVerb::MoveTo => {
                if !current_points.is_empty() {
                    subpaths.push(SubPath {
                        points: std::mem::take(&mut current_points),
                        closed,
                    });
                }
                current_points.push(pts[0]);
                closed = false;
            }
            FlattenVerb::LineTo => {
                current_points.push(pts[0]);
            }
            FlattenVerb::Close => {
                closed = true;
            }
        }
    }

    if !current_points.is_empty() {
        subpaths.push(SubPath { points: current_points, closed });
    }

    subpaths
}

fn path_verbs_iter(path: &FlattenedPath) -> impl Iterator<Item = (FlattenVerb, &[Vec2])> + '_ {
    let mut vi = 0;
    let mut pi = 0;
    let verbs = &path.verbs;
    let points = &path.points;

    std::iter::from_fn(move || {
        if vi >= verbs.len() {
            return None;
        }
        let verb = verbs[vi];
        let n = match verb {
            FlattenVerb::MoveTo | FlattenVerb::LineTo => 1,
            FlattenVerb::Close => 0,
        };
        let pts = &points[pi..pi + n];
        vi += 1;
        pi += n;
        Some((verb, pts))
    })
}

fn stroke_subpath(
    subpath: &SubPath,
    half_w: f32,
    _join: LineJoin,
    cap: LineCap,
    miter_limit: f32,
    out: &mut FlattenedPath,
) {
    if subpath.points.len() < 2 {
        return;
    }

    let pts = &subpath.points;
    let n = pts.len();

    // Compute offset points for the left and right sides
    let mut left_offsets: Vec<Vec2> = Vec::with_capacity(n);
    let mut right_offsets: Vec<Vec2> = Vec::with_capacity(n);

    // Compute normals for each segment
    let mut normals: Vec<Vec2> = Vec::with_capacity(n);
    for i in 0..(if subpath.closed { n } else { n - 1 }) {
        let j = (i + 1) % n;
        let dir = (pts[j] - pts[i]).normalize();
        // Perpendicular: (-y, x) = left normal
        normals.push(Vec2::new(-dir.y, dir.x));
    }

    if normals.is_empty() {
        return;
    }

    // For each vertex, compute the miter direction (bisector of the two adjacent normals)
    let vertex_count = n;
    for i in 0..vertex_count {
        let prev_i = if subpath.closed {
            (i + n - 1) % n
        } else if i == 0 {
            0 // first point, use first segment's normal
        } else {
            i - 1
        };
        let next_i = if subpath.closed || (subpath.closed && i == n - 1) {
            (i + 1) % n
        } else if i == 0 {
            0
        } else {
            i % (n - 1)
        };

        let n_prev = normals[prev_i.min(normals.len() - 1)];
        let n_next = normals[next_i.min(normals.len() - 1)];

        if !subpath.closed && (i == 0 || i == n - 1) {
            // Endpoints: just use the single normal
            let n = if i == 0 { normals[0] } else { normals[normals.len() - 1] };
            left_offsets.push(pts[i] + n * half_w);
            right_offsets.push(pts[i] - n * half_w);
        } else {
            // Interior or closed vertex: miter the two normals
            let bisector = (n_prev + n_next).normalize();
            let cos_theta = n_prev.dot(n_next).clamp(-1.0, 1.0);
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt().max(1e-6);
            let miter_len = 1.0 / sin_theta;

            if miter_len > miter_limit {
                // Miter limit exceeded: fall back to bevel
                // Use a single bevel vertex for each side (the bevel midpoint)
                let bevel_n = (n_prev + n_next).normalize();
                let bevel_len = (bevel_n.dot(n_prev)).max(0.001);
                let bevel_offset = bevel_n * (half_w / bevel_len);
                left_offsets.push(pts[i] + bevel_offset);
                right_offsets.push(pts[i] - bevel_offset);
            } else {
                // Use miter
                let miter_offset = bisector * half_w * miter_len;
                left_offsets.push(pts[i] + miter_offset);
                right_offsets.push(pts[i] - miter_offset);
            }
        }
    }

    if left_offsets.is_empty() || right_offsets.is_empty() {
        return;
    }

    // Build the stroke outline polygon
    // Forward: left offsets (in order)
    // Caps (if open)
    // Reverse: right offsets (in reverse order)
    // Caps (if open)

    let mut outline = Vec::new();

    // Left side (forward)
    outline.extend(&left_offsets);

    if subpath.closed {
        // For closed paths, we need to bridge left and right sides
        // Add the right side in reverse order
        outline.extend(right_offsets.iter().rev());
        // Close back to start
        if outline.len() >= 3 {
            add_polygon_to_flattened(&outline, out);
        }
        return;
    }

    // Open path: add end cap
    if left_offsets.len() >= 2 && right_offsets.len() >= 2 {
        let last_left = left_offsets[left_offsets.len() - 1];
        let last_right = right_offsets[right_offsets.len() - 1];
        let first_left = left_offsets[0];
        let first_right = right_offsets[0];

        match cap {
            LineCap::Butt => {
                // Right side reversed
                for p in right_offsets.iter().rev() {
                    outline.push(*p);
                }
            }
            LineCap::Square => {
                let end_dir = (left_offsets[left_offsets.len() - 1] - left_offsets[left_offsets.len() - 2]).normalize();
                let start_dir = (left_offsets[1] - left_offsets[0]).normalize();
                let end_ext = end_dir * half_w;
                let start_ext = start_dir * half_w;

                // Extend end cap
                outline.push(last_left + end_ext);
                outline.push(last_right + end_ext);

                // Right side reversed
                for p in right_offsets.iter().rev() {
                    outline.push(*p);
                }

                // Extend start cap
                outline.push(first_right - start_ext);
                outline.push(first_left - start_ext);
            }
            LineCap::Round => {
                // For round caps, add arcs directly to the outline
                add_cap_arc_to_outline(last_left, last_right, half_w, &mut outline);
                // Right side reversed
                for p in right_offsets.iter().rev() {
                    outline.push(*p);
                }
                add_cap_arc_to_outline(first_right, first_left, half_w, &mut outline);
            }
        }
    }

    if !subpath.closed && outline.len() >= 3 {
        add_polygon_to_flattened(&outline, out);
    }
}

fn add_cap_arc_to_outline(from: Vec2, to: Vec2, half_w: f32, outline: &mut Vec<Vec2>) {
    let center = (from + to) * 0.5;
    let r = half_w;
    let steps = 6;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let angle = std::f32::consts::PI * t;
        let base_angle = (from - center).y.atan2((from - center).x);
        let a = base_angle + angle;
        let p = center + Vec2::new(a.cos(), a.sin()) * r;
        outline.push(p);
    }
}

fn add_polygon_to_flattened(points: &[Vec2], out: &mut FlattenedPath) {
    if points.len() < 3 {
        return;
    }

    out.points.push(points[0]);
    out.verbs.push(FlattenVerb::MoveTo);

    for p in &points[1..] {
        out.points.push(*p);
        out.verbs.push(FlattenVerb::LineTo);
    }

    out.verbs.push(FlattenVerb::Close);
}
