use sorot_core::color::Color;

use crate::surface::Surface;

pub fn box_blur(src: &Surface, radius: u32) -> Surface {
    let mut tmp = Surface::new(src.width, src.height);
    let r = radius as i32;

    for y in 0..src.height as i32 {
        for x in 0..src.width as i32 {
            let (mut sum_r, mut sum_g, mut sum_b, mut sum_a) = (0.0f32, 0.0, 0.0, 0.0);
            let mut count = 0u32;
            for ky in -r..=r {
                for kx in -r..=r {
                    let c = src.get_pixel(x + kx, y + ky);
                    sum_r += c.r;
                    sum_g += c.g;
                    sum_b += c.b;
                    sum_a += c.a;
                    count += 1;
                }
            }
            let inv = 1.0 / count as f32;
            tmp.set_pixel(
                x,
                y,
                Color::from_premultiplied(sum_r * inv, sum_g * inv, sum_b * inv, sum_a * inv),
            );
        }
    }

    tmp
}

pub fn gaussian_blur_horizontal(src: &Surface, sigma: f32) -> Surface {
    let radius = (sigma * 3.0).ceil() as i32;
    let mut tmp = Surface::new(src.width, src.height);

    let kernel: Vec<f32> = (-radius..=radius)
        .map(|x| (-(x * x) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let kernel_sum: f32 = kernel.iter().sum();

    for y in 0..src.height as i32 {
        for x in 0..src.width as i32 {
            let (mut sum_r, mut sum_g, mut sum_b, mut sum_a) = (0.0f32, 0.0, 0.0, 0.0);
            for (ki, kx) in (-radius..=radius).enumerate() {
                let c = src.get_pixel(x + kx, y);
                let w = kernel[ki];
                sum_r += c.r * w;
                sum_g += c.g * w;
                sum_b += c.b * w;
                sum_a += c.a * w;
            }
            let inv = 1.0 / kernel_sum;
            tmp.set_pixel(
                x,
                y,
                Color::from_premultiplied(sum_r * inv, sum_g * inv, sum_b * inv, sum_a * inv),
            );
        }
    }

    tmp
}

pub fn gaussian_blur_vertical(src: &Surface, sigma: f32) -> Surface {
    let radius = (sigma * 3.0).ceil() as i32;
    let mut tmp = Surface::new(src.width, src.height);

    let kernel: Vec<f32> = (-radius..=radius)
        .map(|y| (-(y * y) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let kernel_sum: f32 = kernel.iter().sum();

    for y in 0..src.height as i32 {
        for x in 0..src.width as i32 {
            let (mut sum_r, mut sum_g, mut sum_b, mut sum_a) = (0.0f32, 0.0, 0.0, 0.0);
            for (ki, ky) in (-radius..=radius).enumerate() {
                let c = src.get_pixel(x, y + ky);
                let w = kernel[ki];
                sum_r += c.r * w;
                sum_g += c.g * w;
                sum_b += c.b * w;
                sum_a += c.a * w;
            }
            let inv = 1.0 / kernel_sum;
            tmp.set_pixel(
                x,
                y,
                Color::from_premultiplied(sum_r * inv, sum_g * inv, sum_b * inv, sum_a * inv),
            );
        }
    }

    tmp
}

pub fn gaussian_blur(src: &Surface, sigma: f32) -> Surface {
    let horiz = gaussian_blur_horizontal(src, sigma);
    gaussian_blur_vertical(&horiz, sigma)
}

pub fn drop_shadow(src: &Surface, dx: i32, dy: i32, blur_sigma: f32, color: Color) -> Surface {
    let mut shadow = Surface::new(src.width, src.height);
    for y in 0..src.height as i32 {
        for x in 0..src.width as i32 {
            let c = src.get_pixel(x, y);
            shadow.set_pixel(x, y, Color::from_premultiplied(0.0, 0.0, 0.0, c.a));
        }
    }

    let blurred = gaussian_blur(&shadow, blur_sigma);

    let mut result = Surface::new(src.width, src.height);
    for y in 0..result.height as i32 {
        for x in 0..result.width as i32 {
            let bx = x - dx;
            let by = y - dy;
            if bx < 0 || by < 0 || bx >= blurred.width as i32 || by >= blurred.height as i32 {
                continue;
            }
            let bc = blurred.get_pixel(bx, by);
            let shaded = Color {
                r: color.r * bc.a,
                g: color.g * bc.a,
                b: color.b * bc.a,
                a: color.a * bc.a,
            };
            result.set_pixel(x, y, shaded);
        }
    }

    result
}

pub fn color_matrix(src: &Surface, matrix: &[f32; 20]) -> Surface {
    let mut result = Surface::new(src.width, src.height);
    for y in 0..src.height as i32 {
        for x in 0..src.width as i32 {
            let c = src.get_pixel(x, y);
            let (r, g, b, a) = c.unpremultiply();

            let nr = matrix[0] * r + matrix[1] * g + matrix[2] * b + matrix[3] * a + matrix[4];
            let ng = matrix[5] * r + matrix[6] * g + matrix[7] * b + matrix[8] * a + matrix[9];
            let nb = matrix[10] * r + matrix[11] * g + matrix[12] * b + matrix[13] * a + matrix[14];
            let na = matrix[15] * r + matrix[16] * g + matrix[17] * b + matrix[18] * a + matrix[19];

            result.set_pixel(
                x,
                y,
                Color::from_rgba(nr.clamp(0.0, 1.0), ng.clamp(0.0, 1.0), nb.clamp(0.0, 1.0), na.clamp(0.0, 1.0)),
            );
        }
    }
    result
}

pub fn grayscale_matrix() -> [f32; 20] {
    [
        0.2126, 0.7152, 0.0722, 0.0, 0.0,
        0.2126, 0.7152, 0.0722, 0.0, 0.0,
        0.2126, 0.7152, 0.0722, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

pub fn sepia_matrix() -> [f32; 20] {
    [
        0.393, 0.769, 0.189, 0.0, 0.0,
        0.349, 0.686, 0.168, 0.0, 0.0,
        0.272, 0.534, 0.131, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_blur() {
        let mut s = Surface::new(10, 10);
        s.set_pixel(5, 5, Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let blurred = box_blur(&s, 1);
        assert!(blurred.get_pixel(4, 5).r > 0.0);
    }

    #[test]
    fn test_gaussian_blur_identity() {
        let mut s = Surface::new(10, 10);
        s.set_pixel(5, 5, Color::from_rgba(0.0, 1.0, 0.0, 1.0));
        let blurred = gaussian_blur(&s, 0.3);
        assert!(blurred.get_pixel(5, 5).g > 0.0);
    }

    #[test]
    fn test_grayscale() {
        let mut s = Surface::new(5, 5);
        s.fill(Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let gray = color_matrix(&s, &grayscale_matrix());
        let pixel = gray.get_pixel(2, 2);
        assert!((pixel.r - 0.2126).abs() < 0.01);
    }
}
