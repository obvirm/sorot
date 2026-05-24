use srvec2::Vec2;

use srbezier::{Cubic, Quad};
use srpath::{Path, PathVerb};

#[derive(Debug, Clone)]
pub struct FlattenedPath {
    pub points: Vec<Vec2>,
    pub verbs: Vec<FlattenVerb>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlattenVerb {
    MoveTo,
    LineTo,
    Close,
}

impl FlattenedPath {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            verbs: Vec::new(),
        }
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            points: Vec::with_capacity(n),
            verbs: Vec::with_capacity(n),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.verbs.clear();
    }
}

impl Default for FlattenedPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Flatten a path into line segments using adaptive subdivision.
///
/// `tolerance` is the maximum distance between the curve and its linear approximation.
pub fn flatten_path(path: &Path, tolerance: f32) -> FlattenedPath {
    let mut result = FlattenedPath::new();
    let mut current_point = Vec2::zero();
    let mut subpath_start = Vec2::zero();
    let mut pending_move = false;

    for seg in path.iter() {
        match seg.verb {
            PathVerb::MoveTo => {
                if pending_move {
                    result.points.push(subpath_start);
                    result.verbs.push(FlattenVerb::MoveTo);
                }
                subpath_start = seg.points[0];
                current_point = subpath_start;
                pending_move = true;
            }
            PathVerb::LineTo => {
                if pending_move {
                    result.points.push(subpath_start);
                    result.verbs.push(FlattenVerb::MoveTo);
                    pending_move = false;
                }
                result.points.push(seg.points[0]);
                result.verbs.push(FlattenVerb::LineTo);
                current_point = seg.points[0];
            }
            PathVerb::QuadTo => {
                if pending_move {
                    result.points.push(subpath_start);
                    result.verbs.push(FlattenVerb::MoveTo);
                    pending_move = false;
                }
                let q = Quad::new(current_point, seg.points[0], seg.points[1]);
                flatten_cubic(&q.to_cubic(), tolerance, &mut result);
                current_point = seg.points[1];
            }
            PathVerb::CubicTo => {
                if pending_move {
                    result.points.push(subpath_start);
                    result.verbs.push(FlattenVerb::MoveTo);
                    pending_move = false;
                }
                let c = Cubic::new(current_point, seg.points[0], seg.points[1], seg.points[2]);
                flatten_cubic(&c, tolerance, &mut result);
                current_point = seg.points[2];
            }
            PathVerb::Close => {
                if !pending_move {
                    if current_point != subpath_start {
                        result.points.push(subpath_start);
                        result.verbs.push(FlattenVerb::LineTo);
                    }
                    result.verbs.push(FlattenVerb::Close);
                }
                current_point = subpath_start;
                pending_move = false;
            }
        }
    }

    result
}

fn flatten_cubic(cubic: &Cubic, tolerance: f32, out: &mut FlattenedPath) {
    let tol_sq = tolerance * tolerance;

    let mut stack: Vec<Cubic> = Vec::with_capacity(32);
    stack.push(*cubic);

    while let Some(c) = stack.pop() {
        if is_flat_enough(&c, tol_sq) {
            out.points.push(c.p3);
            out.verbs.push(FlattenVerb::LineTo);
        } else {
            let (left, right) = c.split(0.5);
            stack.push(right);
            stack.push(left);
        }
    }
}

/// Test if a cubic is flat enough by checking distance of control points from the chord.
fn is_flat_enough(c: &Cubic, tol_sq: f32) -> bool {
    let chord = c.p3 - c.p0;
    let chord_len_sq = chord.length_sq();

    if chord_len_sq < 1e-12 {
        let d1 = (c.p1 - c.p0).length_sq();
        let d2 = (c.p2 - c.p0).length_sq();
        return d1 < tol_sq && d2 < tol_sq;
    }

    let u = (c.p1 - c.p0).dot(chord) / chord_len_sq;
    let v = (c.p2 - c.p0).dot(chord) / chord_len_sq;

    if u < 0.0 || v > 1.0 {
        let d1 = if u < 0.0 {
            (c.p1 - c.p0).length_sq()
        } else {
            (c.p1 - c.p3).length_sq()
        };
        let d2 = if v > 1.0 {
            (c.p2 - c.p3).length_sq()
        } else {
            (c.p2 - c.p0).length_sq()
        };
        return d1 < tol_sq && d2 < tol_sq;
    }

    let ref_p1 = c.p0 + chord * u;
    let ref_p2 = c.p0 + chord * v;

    let d1_sq = (c.p1 - ref_p1).length_sq();
    let d2_sq = (c.p2 - ref_p2).length_sq();

    d1_sq < tol_sq && d2_sq < tol_sq
}

#[cfg(test)]
#[path = "flatten_test.rs"]
mod tests;
