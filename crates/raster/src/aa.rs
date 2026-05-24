use color::Color;

pub const AA_SAMPLES: u32 = 1;

/// Compute coverage for a pixel at (px, py) from an edge crossing.
/// The edge is defined by x position at this scanline and its slope.
///
/// Uses the top-left fill convention: a pixel center is inside
/// if it's strictly to the right of a downward edge.
#[inline]
pub fn coverage_for_edge(px: f32, _py: f32, edge_x: f32, edge_dx: f32) -> f32 {
    let edge_at_pixel = edge_x + edge_dx * 0.5;
    let dist = px - edge_at_pixel;
    (dist + 0.5).clamp(0.0, 1.0)
}

/// Blend a source color onto a pixel buffer with coverage alpha.
#[inline]
pub fn blend_pixel_coverage(dst: &mut [u8], src: &[u8; 4], coverage: f32) {
    if coverage <= 0.0 {
        return;
    }
    if coverage >= 1.0 {
        let sa = src[3] as f32 / 255.0;
        let inv_sa = 1.0 - sa;
        dst[0] = (src[0] as f32 + dst[0] as f32 * inv_sa) as u8;
        dst[1] = (src[1] as f32 + dst[1] as f32 * inv_sa) as u8;
        dst[2] = (src[2] as f32 + dst[2] as f32 * inv_sa) as u8;
        dst[3] = (src[3] as f32 + dst[3] as f32 * inv_sa).min(255.0) as u8;
        return;
    }

    let sa = src[3] as f32 / 255.0 * coverage;
    let inv_sa = 1.0 - sa;
    dst[0] = (src[0] as f32 * coverage + dst[0] as f32 * inv_sa) as u8;
    dst[1] = (src[1] as f32 * coverage + dst[1] as f32 * inv_sa) as u8;
    dst[2] = (src[2] as f32 * coverage + dst[2] as f32 * inv_sa) as u8;
    dst[3] = (sa * 255.0 + dst[3] as f32 * inv_sa).min(255.0) as u8;
}

/// Clear buffer to transparent black.
pub fn clear_buffer(buffer: &mut [u8]) {
    buffer.fill(0);
}

/// Fill buffer with a solid color.
pub fn fill_buffer(buffer: &mut [u8], color: &Color) {
    let rgba = color.to_premultiplied_u8();
    for chunk in buffer.chunks_exact_mut(4) {
        chunk[0] = rgba[0];
        chunk[1] = rgba[1];
        chunk[2] = rgba[2];
        chunk[3] = rgba[3];
    }
}
