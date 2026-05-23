use srcore::color::Color;

/// Blend two slices of premultiplied RGBA pixels using src-over.
///
/// Processes 4 pixels at a time for auto-vectorization.
/// Layout: [r0,g0,b0,a0, r1,g1,b1,a1, ...]
pub fn blend_src_over(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len());
    let pixel_count = dst.len() / 4;
    let simd_count = (pixel_count / 4) * 4;
    let simd_bytes = simd_count * 4;

    for i in (0..simd_bytes).step_by(16) {
        let sr0 = src[i] as f32;
        let sg0 = src[i + 1] as f32;
        let sb0 = src[i + 2] as f32;
        let sa0 = src[i + 3] as f32 * (1.0 / 255.0);

        let dr0 = dst[i] as f32;
        let dg0 = dst[i + 1] as f32;
        let db0 = dst[i + 2] as f32;
        let da0 = dst[i + 3] as f32;

        let inv0 = 1.0 - sa0;

        dst[i] = (sr0 + dr0 * inv0).min(255.0) as u8;
        dst[i + 1] = (sg0 + dg0 * inv0).min(255.0) as u8;
        dst[i + 2] = (sb0 + db0 * inv0).min(255.0) as u8;
        dst[i + 3] = ((sa0 * 255.0) + da0 * inv0).min(255.0) as u8;

        let sr1 = src[i + 4] as f32;
        let sg1 = src[i + 5] as f32;
        let sb1 = src[i + 6] as f32;
        let sa1 = src[i + 7] as f32 * (1.0 / 255.0);
        let dr1 = dst[i + 4] as f32;
        let dg1 = dst[i + 5] as f32;
        let db1 = dst[i + 6] as f32;
        let da1 = dst[i + 7] as f32;
        let inv1 = 1.0 - sa1;
        dst[i + 4] = (sr1 + dr1 * inv1).min(255.0) as u8;
        dst[i + 5] = (sg1 + dg1 * inv1).min(255.0) as u8;
        dst[i + 6] = (sb1 + db1 * inv1).min(255.0) as u8;
        dst[i + 7] = ((sa1 * 255.0) + da1 * inv1).min(255.0) as u8;

        let sr2 = src[i + 8] as f32;
        let sg2 = src[i + 9] as f32;
        let sb2 = src[i + 10] as f32;
        let sa2 = src[i + 11] as f32 * (1.0 / 255.0);
        let dr2 = dst[i + 8] as f32;
        let dg2 = dst[i + 9] as f32;
        let db2 = dst[i + 10] as f32;
        let da2 = dst[i + 11] as f32;
        let inv2 = 1.0 - sa2;
        dst[i + 8] = (sr2 + dr2 * inv2).min(255.0) as u8;
        dst[i + 9] = (sg2 + dg2 * inv2).min(255.0) as u8;
        dst[i + 10] = (sb2 + db2 * inv2).min(255.0) as u8;
        dst[i + 11] = ((sa2 * 255.0) + da2 * inv2).min(255.0) as u8;

        let sr3 = src[i + 12] as f32;
        let sg3 = src[i + 13] as f32;
        let sb3 = src[i + 14] as f32;
        let sa3 = src[i + 15] as f32 * (1.0 / 255.0);
        let dr3 = dst[i + 12] as f32;
        let dg3 = dst[i + 13] as f32;
        let db3 = dst[i + 14] as f32;
        let da3 = dst[i + 15] as f32;
        let inv3 = 1.0 - sa3;
        dst[i + 12] = (sr3 + dr3 * inv3).min(255.0) as u8;
        dst[i + 13] = (sg3 + dg3 * inv3).min(255.0) as u8;
        dst[i + 14] = (sb3 + db3 * inv3).min(255.0) as u8;
        dst[i + 15] = ((sa3 * 255.0) + da3 * inv3).min(255.0) as u8;
    }

    for i in (simd_bytes..dst.len()).step_by(4) {
        let sa = src[i + 3] as f32 * (1.0 / 255.0);
        let inv = 1.0 - sa;
        dst[i] = (src[i] as f32 + dst[i] as f32 * inv).min(255.0) as u8;
        dst[i + 1] = (src[i + 1] as f32 + dst[i + 1] as f32 * inv).min(255.0) as u8;
        dst[i + 2] = (src[i + 2] as f32 + dst[i + 2] as f32 * inv).min(255.0) as u8;
        dst[i + 3] = ((sa * 255.0) + dst[i + 3] as f32 * inv).min(255.0) as u8;
    }
}

/// Fill a pixel buffer with a premultiplied color. 4-pixel unrolled.
pub fn fill_color(dst: &mut [u8], color: Color) {
    let rgba = color.to_premultiplied_u8();
    let len = dst.len() / 4;
    let simd_count = (len / 4) * 4;
    let simd_bytes = simd_count * 4;

    for i in (0..simd_bytes).step_by(16) {
        dst[i..i + 4].copy_from_slice(&rgba);
        dst[i + 4..i + 8].copy_from_slice(&rgba);
        dst[i + 8..i + 12].copy_from_slice(&rgba);
        dst[i + 12..i + 16].copy_from_slice(&rgba);
    }

    for i in (simd_bytes..dst.len()).step_by(4) {
        dst[i..i + 4].copy_from_slice(&rgba);
    }
}

/// Blend a single premultiplied f32 color into a u8 pixel buffer.
#[inline]
pub fn blend_pixel_src_over(dst: &mut [u8; 4], src_r: f32, src_g: f32, src_b: f32, src_a: f32) {
    let inv = 1.0 - src_a;
    dst[0] = (src_r * 255.0 + dst[0] as f32 * inv).min(255.0) as u8;
    dst[1] = (src_g * 255.0 + dst[1] as f32 * inv).min(255.0) as u8;
    dst[2] = (src_b * 255.0 + dst[2] as f32 * inv).min(255.0) as u8;
    dst[3] = (src_a * 255.0 + dst[3] as f32 * inv).min(255.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_src_over_opaque() {
        let mut dst = [0u8, 0, 0, 0, 0, 0, 0, 0];
        let src = [255u8, 0, 0, 255, 0, 255, 0, 128];
        blend_src_over(&mut dst, &src);
        assert_eq!(dst[0], 255);
        assert_eq!(dst[3], 255);
        assert!(dst[5] > 0);
    }

    #[test]
    fn test_fill_color() {
        let mut buf = vec![0u8; 40];
        fill_color(&mut buf, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 0);
        assert_eq!(buf[3], 255);
        assert_eq!(buf[4], 255);
    }
}
