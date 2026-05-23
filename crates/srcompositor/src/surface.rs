use srcore::color::Color;

use crate::blend::BlendMode;

#[derive(Debug, Clone)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Surface {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u8; (width * height * 4) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn fill(&mut self, color: Color) {
        let rgba = color.to_premultiplied_u8();
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = rgba[0];
            chunk[1] = rgba[1];
            chunk[2] = rgba[2];
            chunk[3] = rgba[3];
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return Color::TRANSPARENT;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        let r = self.pixels[idx] as f32 / 255.0;
        let g = self.pixels[idx + 1] as f32 / 255.0;
        let b = self.pixels[idx + 2] as f32 / 255.0;
        let a = self.pixels[idx + 3] as f32 / 255.0;
        Color::from_premultiplied(r, g, b, a)
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        let rgba = color.to_premultiplied_u8();
        self.pixels[idx] = rgba[0];
        self.pixels[idx + 1] = rgba[1];
        self.pixels[idx + 2] = rgba[2];
        self.pixels[idx + 3] = rgba[3];
    }

    pub fn composite(&mut self, src: &Surface, x: i32, y: i32, mode: BlendMode) {
        for row in 0..src.height as i32 {
            for col in 0..src.width as i32 {
                let dx = x + col;
                let dy = y + row;
                if dx < 0 || dy < 0 || dx >= self.width as i32 || dy >= self.height as i32 {
                    continue;
                }
                let sc = src.get_pixel(col, row);
                let dc = self.get_pixel(dx, dy);
                let result = mode.blend(sc, dc);
                self.set_pixel(dx, dy, result);
            }
        }
    }

    pub fn composite_rect(
        &mut self,
        src: &Surface,
        src_rect: (u32, u32, u32, u32),
        dst_x: i32,
        dst_y: i32,
        mode: BlendMode,
    ) {
        let (sx, sy, sw, sh) = src_rect;
        for row in 0..sh as i32 {
            for col in 0..sw as i32 {
                let dx = dst_x + col;
                let dy = dst_y + row;
                if dx < 0 || dy < 0 || dx >= self.width as i32 || dy >= self.height as i32 {
                    continue;
                }
                let sc = src.get_pixel(sx as i32 + col, sy as i32 + row);
                let dc = self.get_pixel(dx, dy);
                let result = mode.blend(sc, dc);
                self.set_pixel(dx, dy, result);
            }
        }
    }

    pub fn to_rgba_u8(&self) -> Vec<u8> {
        let mut result = vec![0u8; self.pixels.len()];
        for (chunk, out) in self.pixels.chunks_exact(4).zip(result.chunks_exact_mut(4)) {
            let r = chunk[0] as f32 / 255.0;
            let g = chunk[1] as f32 / 255.0;
            let b = chunk[2] as f32 / 255.0;
            let a = chunk[3] as f32 / 255.0;
            if a > 0.0 {
                let inv = 1.0 / a;
                out[0] = (r * inv * 255.0 + 0.5).min(255.0) as u8;
                out[1] = (g * inv * 255.0 + 0.5).min(255.0) as u8;
                out[2] = (b * inv * 255.0 + 0.5).min(255.0) as u8;
            } else {
                out[0] = 0;
                out[1] = 0;
                out[2] = 0;
            }
            out[3] = chunk[3];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_new() {
        let s = Surface::new(100, 50);
        assert_eq!(s.pixels.len(), 20000);
    }

    #[test]
    fn test_fill() {
        let mut s = Surface::new(10, 10);
        s.fill(Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let pixel = s.get_pixel(5, 5);
        assert!((pixel.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_composite_src_over() {
        let mut dst = Surface::new(10, 10);
        dst.fill(Color::from_rgba(0.0, 0.0, 1.0, 1.0));

        let mut src = Surface::new(5, 5);
        src.fill(Color::from_rgba(1.0, 0.0, 0.0, 0.5));

        dst.composite(&src, 0, 0, BlendMode::SrcOver);
        let pixel = dst.get_pixel(2, 2);
        assert!(pixel.r > 0.0);
        assert!(pixel.b > 0.0);
    }
}
