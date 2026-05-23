use sorot_core::math::{Rect, Vec2};
use sorot_path::FlattenedPath;

pub fn compute_sdf(
    path: &FlattenedPath,
    grid_size: u32,
    padding_ratio: f32,
) -> (Vec<u8>, Rect, u32, u32) {
    let bbox = path_bounds(path);
    let pad_x = bbox.width() * padding_ratio;
    let pad_y = bbox.height() * padding_ratio;
    let padded = Rect::new(
        Vec2::new(bbox.min.x - pad_x, bbox.min.y - pad_y),
        Vec2::new(bbox.max.x + pad_x, bbox.max.y + pad_y),
    );

    let width = padded.width();
    let height = padded.height();

    let (w, h) = if width > height {
        let hh = height / width * grid_size as f32;
        (grid_size, (hh as u32).max(1))
    } else {
        let ww = width / height * grid_size as f32;
        ((ww as u32).max(1), grid_size)
    };

    let mut data = vec![128u8; (w * h) as usize];
    let inv_scale_x = width / w as f32;
    let inv_scale_y = height / h as f32;

    for row in 0..h {
        for col in 0..w {
            let px = padded.min.x + (col as f32 + 0.5) * inv_scale_x;
            let py = padded.min.y + (row as f32 + 0.5) * inv_scale_y;
            let p = Vec2::new(px, py);
            let dist = sorot_path::sdf::distance_to_polyline(p, &path.points, true);
            let wn = sorot_path::sdf::winding_number(p, &path.points);
            let signed = if wn != 0 { -dist } else { dist };
            let max_dist = padded.width().max(padded.height()) * padding_ratio;
            let value = ((signed / max_dist) * 0.5 + 0.5).clamp(0.0, 1.0);
            data[(row * w + col) as usize] = (value * 255.0) as u8;
        }
    }

    (data, padded, w, h)
}

fn path_bounds(path: &FlattenedPath) -> Rect {
    if path.points.is_empty() {
        return Rect::zero();
    }
    let mut min = path.points[0];
    let mut max = path.points[0];
    for p in &path.points[1..] {
        min = min.min(*p);
        max = max.max(*p);
    }
    Rect::new(min, max)
}
