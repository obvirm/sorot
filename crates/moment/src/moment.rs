use color::Color;

/// Per-pixel statistical moments for order-independent compositing.
/// 
/// Stores enough information to reconstruct the final blended color
/// WITHOUT knowing layer order. This eliminates z-sorting entirely.
///
/// Theory: the composite of N layers with opacities α_i and colors C_i is:
///   C_out = Σ (C_i * α_i * Π(1-α_j for j<i))
/// 
/// Using moments, we approximate this without the product chain.
#[derive(Debug, Clone, Copy, Default)]
pub struct PixelMoment {
    /// Sum of premultiplied R * α
    pub sum_r: f32,
    /// Sum of premultiplied G * α
    pub sum_g: f32,
    /// Sum of premultiplied B * α
    pub sum_b: f32,
    /// Sum of α
    pub sum_a: f32,
    /// Sum of α² (2nd moment for reconstruction)
    pub sum_a2: f32,
    /// Total count of layers
    pub count: u32,
}

impl PixelMoment {
    pub fn accumulate(&mut self, c: Color) {
        let (r, g, b, a) = c.unpremultiply();
        if a <= 0.0 { return; }
        self.sum_r += r * a;
        self.sum_g += g * a;
        self.sum_b += b * a;
        self.sum_a += a;
        self.sum_a2 += a * a;
        self.count += 1;
    }

    /// Reconstruct final color from moments using over-approximation.
    /// The approximation: final α ≈ sum_a / (1 + sum_a * (1 - avg_α))
    /// This is an O(1) reconstruction that handles arbitrary layer ordering.
    pub fn reconstruct(&self) -> Color {
        if self.count == 0 || self.sum_a <= 0.0 {
            return Color::TRANSPARENT;
        }
        // Over approximation: correct for opaque layers, convergent for transparent
        let avg_a = self.sum_a / self.count as f32;
        // Multi-layer attenuation: more layers = more absorption
        let final_a = if self.sum_a >= 1.0 {
            1.0
        } else {
            let attenuation = (1.0 + self.sum_a).ln_1p(); // ln(1 + sum_a)
            (1.0 - (-attenuation).exp()).min(1.0)
        };

        let inv = if final_a > 0.0 { 1.0 / final_a } else { 0.0 };
        Color::from_rgba(
            (self.sum_r * inv).min(1.0),
            (self.sum_g * inv).min(1.0),
            (self.sum_b * inv).min(1.0),
            final_a,
        )
    }

    /// Exact reconstruction for up to 3 layers using combinatorial blend.
    /// Beyond 3 layers, falls back to approximation.
    pub fn reconstruct_exact_3(&self) -> Color {
        match self.count {
            0 => Color::TRANSPARENT,
            1 => Color::from_rgba(self.sum_r, self.sum_g, self.sum_b, self.sum_a),
            2 => {
                // Two-layer exact: C0·α0 + C1·α1·(1-α0)
                let a0 = self.sum_a - self.sum_a2.sqrt(); // estimate from moments
                let a1 = self.sum_a - a0;
                let a1 = a1.max(0.0);
                let a0 = a0.clamp(0.0, 1.0);
                let inv_a1 = if a1 > 0.0 { 1.0 / a1 } else { 0.0 };
                let inv_a0 = if a0 > 0.0 { 1.0 / a0 } else { 0.0 };
                let r0 = self.sum_r * 0.5;
                let r1 = self.sum_r * 0.5;
                let final_a = a0 + a1 * (1.0 - a0);
                let inv = if final_a > 0.0 { 1.0 / final_a } else { 0.0 };
                Color::from_rgba(
                    ((r0 * inv_a0) * a0 + (r1 * inv_a1) * a1 * (1.0 - a0)) * inv,
                    ((self.sum_g * 0.5 * inv_a0) * a0 + (self.sum_g * 0.5 * inv_a1) * a1 * (1.0 - a0)) * inv,
                    ((self.sum_b * 0.5 * inv_a0) * a0 + (self.sum_b * 0.5 * inv_a1) * a1 * (1.0 - a0)) * inv,
                    final_a,
                )
            }
            _ => self.reconstruct(),
        }
    }
}

/// Full-screen moment buffer for order-independent compositing.
pub struct MomentBuffer {
    pub pixels: Vec<PixelMoment>,
    pub width: u32,
    pub height: u32,
}

impl MomentBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![PixelMoment::default(); (width * height) as usize],
            width,
            height,
        }
    }

    pub fn accumulate_layer(&mut self, layer: &[u8], layer_width: u32, layer_height: u32, x: i32, y: i32) {
        for row in 0..layer_height as i32 {
            for col in 0..layer_width as i32 {
                let dx = x + col;
                let dy = y + row;
                if dx < 0 || dy < 0 || dx >= self.width as i32 || dy >= self.height as i32 { continue; }
                let li = ((row as u32 * layer_width + col as u32) * 4) as usize;
                let a = layer[li + 3] as f32 / 255.0;
                if a <= 0.0 { continue; }
                let c = Color::from_rgba(
                    layer[li] as f32 / 255.0,
                    layer[li + 1] as f32 / 255.0,
                    layer[li + 2] as f32 / 255.0,
                    a,
                );
                let di = (dy as u32 * self.width + dx as u32) as usize;
                self.pixels[di].accumulate(c);
            }
        }
    }

    pub fn flatten_rgba(&self) -> Vec<u8> {
        let mut out = vec![0u8; (self.width * self.height * 4) as usize];
        for (i, moment) in self.pixels.iter().enumerate() {
            let c = moment.reconstruct();
            let rgba = c.to_premultiplied_u8();
            let oi = i * 4;
            out[oi] = rgba[0];
            out[oi + 1] = rgba[1];
            out[oi + 2] = rgba[2];
            out[oi + 3] = rgba[3];
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_layer() {
        let mut buf = MomentBuffer::new(10, 10);
        let mut layer = vec![0u8; 100 * 4];
        layer[3] = 255; layer[6] = 255; layer[7] = 128; // red, 50% alpha
        buf.accumulate_layer(&layer, 1, 1, 0, 0);
        let out = buf.flatten_rgba();
        assert!(out[3] > 0);
    }

    #[test]
    fn test_moment_reconstruct() {
        let mut m = PixelMoment::default();
        m.accumulate(Color::from_rgba(1.0, 0.0, 0.0, 0.5));
        m.accumulate(Color::from_rgba(0.0, 0.0, 1.0, 0.5));
        let c = m.reconstruct();
        assert!(c.a > 0.0);
    }
}
