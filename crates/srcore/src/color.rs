use std::ops::{Add, Mul};

/// Premultiplied alpha color (linear f32 components).
///
/// All channels are stored premultiplied:
/// `r = red * alpha`, `g = green * alpha`, `b = blue * alpha`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Create from premultiplied components.
    #[inline]
    pub const fn from_premultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create from unpremultiplied (straight) components.
    #[inline]
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r * a,
            g: g * a,
            b: b * a,
            a,
        }
    }

    /// Create from unpremultiplied u8 components.
    #[inline]
    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        let a = a as f32 / 255.0;
        Self::from_rgba(r, g, b, a)
    }

    /// Unpremultiply and return (r, g, b, a).
    #[inline]
    pub fn unpremultiply(self) -> (f32, f32, f32, f32) {
        if self.a <= 0.0 {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let inv_a = 1.0 / self.a;
        (self.r * inv_a, self.g * inv_a, self.b * inv_a, self.a)
    }

    /// Convert to unpremultiplied u8 components.
    #[inline]
    pub fn to_rgba_u8(self) -> [u8; 4] {
        let (r, g, b, a) = self.unpremultiply();
        [
            (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Convert to premultiplied u8 components (for GPU upload).
    #[inline]
    pub fn to_premultiplied_u8(self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Alpha over compositing (assumes both premultiplied).
    ///
    /// dst = src + dst * (1 - src.a)
    #[inline]
    pub fn over(self, dst: Self) -> Self {
        let inv_a = 1.0 - self.a;
        Self {
            r: self.r + dst.r * inv_a,
            g: self.g + dst.g * inv_a,
            b: self.b + dst.b * inv_a,
            a: self.a + dst.a * inv_a,
        }
    }

    /// Source-over compositing (self over dst), both premultiplied.
    #[inline]
    pub fn blend_over(self, dst: Self) -> Self {
        self.over(dst)
    }

    pub fn to_linear(self) -> Self {
        Self {
            r: srgb_to_linear(self.r),
            g: srgb_to_linear(self.g),
            b: srgb_to_linear(self.b),
            a: self.a,
        }
    }

    pub fn to_srgb(self) -> Self {
        Self {
            r: linear_to_srgb(self.r),
            g: linear_to_srgb(self.g),
            b: linear_to_srgb(self.b),
            a: self.a,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl Add for Color {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
#[path = "color_test.rs"]
mod tests;
