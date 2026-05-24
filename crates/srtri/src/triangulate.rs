use srvec2::Vec2;
use srflatten::{FlattenVerb, FlattenedPath};

#[derive(Debug, Clone)]
pub struct TriMesh {
    pub vertices: Vec<Vec2>,
    pub indices: Vec<u32>,
}

pub fn triangulate(path: &FlattenedPath) -> TriMesh {
    let mut all_vertices: Vec<Vec2> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut subpath_start_idx: Option<u32> = None;

    let mut vi = 0;
    let mut pi = 0;

    while vi < path.verbs.len() {
        let verb = path.verbs[vi];
        match verb {
            FlattenVerb::MoveTo => {
                let p = path.points[pi];
                if let Some(start) = subpath_start_idx {
                    triangulate_subpath_slice(
                        &all_vertices,
                        start as usize..all_vertices.len(),
                        &mut all_indices,
                    );
                }
                let base = all_vertices.len() as u32;
                all_vertices.push(p);
                subpath_start_idx = Some(base);
                pi += 1;
            }
            FlattenVerb::LineTo => {
                let p = path.points[pi];
                all_vertices.push(p);
                pi += 1;
            }
            FlattenVerb::Close => {
                if let Some(start) = subpath_start_idx {
                    if all_vertices.last() != Some(&all_vertices[start as usize]) {
                        all_vertices.push(all_vertices[start as usize]);
                    }
                    triangulate_subpath_slice(
                        &all_vertices,
                        start as usize..all_vertices.len(),
                        &mut all_indices,
                    );
                    subpath_start_idx = None;
                }
            }
        }
        vi += 1;
    }

    if let Some(start) = subpath_start_idx {
        let range = start as usize..all_vertices.len();
        triangulate_subpath_slice(&all_vertices, range, &mut all_indices);
    }

    TriMesh {
        vertices: all_vertices,
        indices: all_indices,
    }
}

fn triangulate_subpath_slice(
    vertices: &[Vec2],
    range: std::ops::Range<usize>,
    out_indices: &mut Vec<u32>,
) {
    let slice = &vertices[range.clone()];
    let n = slice.len();

    if n < 3 {
        return;
    }

    if n == 3 {
        let base = range.start as u32;
        if area(slice) >= 0.0 {
            out_indices.push(base);
            out_indices.push(base + 1);
            out_indices.push(base + 2);
        }
        return;
    }

    let mut indices: Vec<u32> = (0..n as u32).collect();
    let verts: Vec<Vec2> = slice.to_vec();
    let mut remaining = n;
    let mut safety = n * 3;

    while remaining > 3 && safety > 0 {
        safety -= 1;
        let mut ear_found = false;

        for i in 0..remaining {
            let prev = (i + remaining - 1) % remaining;
            let next = (i + 1) % remaining;

            let a = verts[indices[prev] as usize];
            let b = verts[indices[i] as usize];
            let c = verts[indices[next] as usize];

            let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
            if cross <= 0.0 {
                continue;
            }

            let mut is_ear = true;
            for j in 0..remaining {
                if j == prev || j == i || j == next {
                    continue;
                }
                let p = verts[indices[j] as usize];
                if point_in_triangle(p, a, b, c) {
                    is_ear = false;
                    break;
                }
            }

            if is_ear {
                let base = range.start as u32;
                out_indices.push(base + indices[prev]);
                out_indices.push(base + indices[i]);
                out_indices.push(base + indices[next]);
                indices.remove(i);
                remaining -= 1;
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            break;
        }
    }

    if remaining == 3 {
        let base = range.start as u32;
        let a = verts[indices[0] as usize];
        let b = verts[indices[1] as usize];
        let c = verts[indices[2] as usize];
        if (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) > 0.0 {
            out_indices.push(base + indices[0]);
            out_indices.push(base + indices[1]);
            out_indices.push(base + indices[2]);
        }
    }
}

fn area(poly: &[Vec2]) -> f32 {
    let n = poly.len();
    if n < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    sum * 0.5
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    (d1 > 0.0 && d2 > 0.0 && d3 > 0.0) || (d1 < 0.0 && d2 < 0.0 && d3 < 0.0)
}

fn sign(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

#[cfg(test)]
#[path = "triangulate_test.rs"]
mod tests;
