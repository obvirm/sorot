use srcore::color::Color;
use srcore::math::Vec2;
use srpath::{FlattenVerb, FlattenedPath};

use crate::tile::Edge;

pub fn rasterize_path(
    path: &FlattenedPath,
    color: Color,
    buffer: &mut [u8],
    width: u32,
    height: u32,
    even_odd: bool,
) {
    if path.is_empty() {
        return;
    }

    let edges = build_edges(path, width, height);
    if edges.is_empty() {
        return;
    }

    let y_min = edges.iter().map(|e| e.y0).min().unwrap_or(0);
    let y_max = edges.iter().map(|e| e.y1).max().unwrap_or(0);

    let stride = (width * 4) as usize;
    let mut active: Vec<ActiveEdge> = Vec::with_capacity(64);
    let mut edge_idx = 0usize;

    let mut sorted_edges: Vec<usize> = (0..edges.len()).collect();
    sorted_edges.sort_by_key(|&i| edges[i].y0);

    for y in y_min..y_max {
        if y < 0 || y >= height as i32 {
            continue;
        }

        while edge_idx < sorted_edges.len() && edges[sorted_edges[edge_idx]].y0 == y {
            let idx = sorted_edges[edge_idx];
            active.push(ActiveEdge {
                x: edges[idx].x,
                dx: edges[idx].dx,
                y_end: edges[idx].y1,
                winding: edges[idx].winding,
            });
            edge_idx += 1;
        }

        active.retain(|ae| y < ae.y_end);

        active.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        if even_odd {
            fill_scanline_even_odd(y, &active, buffer, stride, &color);
        } else {
            fill_scanline_winding(y, &active, buffer, stride, &color);
        }

        for ae in &mut active {
            ae.x += ae.dx;
        }
    }
}

struct ActiveEdge {
    x: f32,
    dx: f32,
    y_end: i32,
    winding: i8,
}

fn fill_scanline_even_odd(
    y: i32,
    active: &[ActiveEdge],
    buffer: &mut [u8],
    stride: usize,
    color: &Color,
) {
    let mut inside = false;
    let mut span_start: Option<f32> = None;

    let rgba = color.to_premultiplied_u8();
    let row_start = (y as usize) * stride;

    for ae in active {
        if inside {
            if let Some(start) = span_start {
                let x0 = (start.max(0.0).floor() as i32).max(0);
                let x1 = (ae.x.min(buffer.len() as f32 / 4.0).ceil() as i32).max(0);

                let x0u = x0 as usize;
                let x1u = x1 as usize;

                let offset = row_start + x0u * 4;
                let len = (x1u - x0u) * 4;
                if offset + len <= buffer.len() {
                    for px in (offset..offset + len).step_by(4) {
                        blend_pixel(&mut buffer[px..px + 4], &rgba);
                    }
                }
            }
            inside = false;
        } else {
            span_start = Some(ae.x);
            inside = true;
        }
    }
}

fn fill_scanline_winding(
    y: i32,
    active: &[ActiveEdge],
    buffer: &mut [u8],
    stride: usize,
    color: &Color,
) {
    let mut winding: i32 = 0;
    let mut span_start: Option<f32> = None;

    let rgba = color.to_premultiplied_u8();
    let row_start = (y as usize) * stride;

    for ae in active {
        let was_inside = winding != 0;
        winding += ae.winding as i32;
        let is_inside = winding != 0;

        if !was_inside && is_inside {
            span_start = Some(ae.x);
        } else if was_inside && !is_inside {
            if let Some(start) = span_start {
                let x0 = (start.max(0.0).floor() as i32).max(0);
                let x1 = (ae.x.min((buffer.len() / 4) as f32).ceil() as i32).max(0);

                let offset = row_start + x0 as usize * 4;
                let len = (x1 as usize - x0 as usize) * 4;
                if offset + len <= buffer.len() {
                    for px in (offset..offset + len).step_by(4) {
                        blend_pixel(&mut buffer[px..px + 4], &rgba);
                    }
                }
            }
            span_start = None;
        }
    }
}

#[inline]
fn blend_pixel(dst: &mut [u8], src: &[u8; 4]) {
    let sa = src[3] as f32 / 255.0;
    let inv_sa = 1.0 - sa;
    dst[0] = (src[0] as f32 + dst[0] as f32 * inv_sa) as u8;
    dst[1] = (src[1] as f32 + dst[1] as f32 * inv_sa) as u8;
    dst[2] = (src[2] as f32 + dst[2] as f32 * inv_sa) as u8;
    dst[3] = (src[3] as f32 + dst[3] as f32 * inv_sa).min(255.0) as u8;
}

fn build_edges(path: &FlattenedPath, _width: u32, height: u32) -> Vec<Edge> {
    let mut edges = Vec::new();
    let mut first_point: Option<Vec2> = None;
    let mut last_point: Option<Vec2> = None;

    for (verb, points) in path_verbs(path) {
        match verb {
            FlattenVerb::MoveTo => {
                first_point = Some(points[0]);
                last_point = Some(points[0]);
            }
            FlattenVerb::LineTo => {
                let from = last_point.unwrap();
                let to = points[0];

                if from == to {
                    continue;
                }

                let (x0, y0, x1, y1, winding) = if from.y <= to.y {
                    (from.x, from.y, to.x, to.y, 1i8)
                } else {
                    (to.x, to.y, from.x, from.y, -1i8)
                };

                if let Some(e) = Edge::new(x0, y0, x1, y1, winding) {
                    if e.y1 > 0 && e.y0 < height as i32 {
                        edges.push(e);
                    }
                }

                last_point = Some(to);
            }
            FlattenVerb::Close => {
                if let (Some(from), Some(to)) = (last_point, first_point) {
                    if from != to {
                        let (x0, y0, x1, y1, winding) = if from.y <= to.y {
                            (from.x, from.y, to.x, to.y, 1i8)
                        } else {
                            (to.x, to.y, from.x, from.y, -1i8)
                        };
                        if let Some(e) = Edge::new(x0, y0, x1, y1, winding) {
                            if e.y1 > 0 && e.y0 < height as i32 {
                                edges.push(e);
                            }
                        }
                    }
                }
                last_point = first_point;
            }
        }
    }

    edges
}

fn path_verbs(path: &FlattenedPath) -> impl Iterator<Item = (FlattenVerb, &[Vec2])> + '_ {
    let mut vi = 0;
    let mut pi = 0;
    std::iter::from_fn(move || {
        if vi >= path.verbs.len() {
            return None;
        }
        let verb = path.verbs[vi];
        let n = match verb {
            FlattenVerb::MoveTo | FlattenVerb::LineTo => 1,
            FlattenVerb::Close => 0,
        };
        let pts = &path.points[pi..pi + n];
        vi += 1;
        pi += n;
        Some((verb, pts))
    })
}

#[cfg(test)]
#[path = "scanline_test.rs"]
mod tests;
