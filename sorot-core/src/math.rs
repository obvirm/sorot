use std::ops::{Add, Div, Mul, Neg, Sub};

pub type Fixed = i32;
pub const FIXED_SHIFT: u32 = 8;
pub const FIXED_ONE: Fixed = 1 << FIXED_SHIFT;
pub const FIXED_HALF: Fixed = FIXED_ONE >> 1;

#[inline]
pub fn float_to_fixed(f: f32) -> Fixed {
    (f * FIXED_ONE as f32) as Fixed
}

#[inline]
pub fn fixed_to_float(f: Fixed) -> f32 {
    f as f32 / FIXED_ONE as f32
}

#[inline]
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    ((a as i64 * b as i64 + FIXED_HALF as i64) >> FIXED_SHIFT) as Fixed
}

#[inline]
pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    (((a as i64) << FIXED_SHIFT) / b as i64) as Fixed
}

#[inline]
pub fn fixed_floor(f: Fixed) -> Fixed {
    f & !(FIXED_ONE - 1)
}

#[inline]
pub fn fixed_ceil(f: Fixed) -> Fixed {
    (f + FIXED_ONE - 1) & !(FIXED_ONE - 1)
}

#[inline]
pub fn fixed_round(f: Fixed) -> Fixed {
    (f + FIXED_HALF) & !(FIXED_ONE - 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub type Point = Vec2;

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_sq().sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::zero()
        }
    }

    #[inline]
    pub fn try_normalize(self) -> Option<Self> {
        let len = self.length();
        if len > 0.0 {
            Some(Self::new(self.x / len, self.y / len))
        } else {
            None
        }
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    #[inline]
    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> f32 {
        (self - other).length()
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self * rhs.x, self * rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        let inv = 1.0 / rhs;
        Self::new(self.x * inv, self.y * inv)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min: min.min(max),
            max: min.max(max),
        }
    }

    #[inline]
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        Self {
            min: origin,
            max: origin + size,
        }
    }

    #[inline]
    pub fn zero() -> Self {
        Self {
            min: Vec2::zero(),
            max: Vec2::zero(),
        }
    }

    #[inline]
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn size(self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    #[inline]
    pub fn area(self) -> f32 {
        self.width() * self.height()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    #[inline]
    pub fn center(self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    #[inline]
    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    #[inline]
    pub fn contains_rect(self, other: Self) -> bool {
        self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y
    }

    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    #[inline]
    pub fn intersect(self, other: Self) -> Self {
        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    #[inline]
    pub fn union_point(self, p: Vec2) -> Self {
        Self {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    #[inline]
    pub fn inset(self, dx: f32, dy: f32) -> Self {
        Self {
            min: Vec2::new(self.min.x + dx, self.min.y + dy),
            max: Vec2::new(self.max.x - dx, self.max.y - dy),
        }
    }

    #[inline]
    pub fn outset(self, dx: f32, dy: f32) -> Self {
        self.inset(-dx, -dy)
    }

    #[inline]
    pub fn translate(self, offset: Vec2) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}

/// 2D affine transformation matrix.
///
/// Layout (row-major):
/// ```text
/// | a  c  e |
/// | b  d  f |
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x2 {
    pub a: f32,
    pub c: f32,
    pub e: f32,
    pub b: f32,
    pub d: f32,
    pub f: f32,
}

impl Matrix3x2 {
    #[inline]
    pub const fn identity() -> Self {
        Self {
            a: 1.0,
            c: 0.0,
            e: 0.0,
            b: 0.0,
            d: 1.0,
            f: 0.0,
        }
    }

    #[inline]
    pub const fn new(a: f32, c: f32, e: f32, b: f32, d: f32, f: f32) -> Self {
        Self { a, c, e, b, d, f }
    }

    #[inline]
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0,
            c: 0.0,
            e: x,
            b: 0.0,
            d: 1.0,
            f: y,
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            c: 0.0,
            e: 0.0,
            b: 0.0,
            d: sy,
            f: 0.0,
        }
    }

    #[inline]
    pub fn rotate(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            a: cos,
            c: -sin,
            e: 0.0,
            b: sin,
            d: cos,
            f: 0.0,
        }
    }

    #[inline]
    pub fn skew_x(radians: f32) -> Self {
        Self {
            a: 1.0,
            c: radians.tan(),
            e: 0.0,
            b: 0.0,
            d: 1.0,
            f: 0.0,
        }
    }

    #[inline]
    pub fn skew_y(radians: f32) -> Self {
        Self {
            a: 1.0,
            c: 0.0,
            e: 0.0,
            b: radians.tan(),
            d: 1.0,
            f: 0.0,
        }
    }

    #[inline]
    pub fn determinant(self) -> f32 {
        self.a * self.d - self.c * self.b
    }

    #[inline]
    pub fn inverse(self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-12 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            c: -self.c * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            b: -self.b * inv_det,
            d: self.a * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
    }

    #[inline]
    pub fn transform_point(self, p: Vec2) -> Vec2 {
        Vec2::new(
            self.a * p.x + self.c * p.y + self.e,
            self.b * p.x + self.d * p.y + self.f,
        )
    }

    #[inline]
    pub fn transform_vector(self, v: Vec2) -> Vec2 {
        Vec2::new(
            self.a * v.x + self.c * v.y,
            self.b * v.x + self.d * v.y,
        )
    }

    #[inline]
    pub fn then(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            c: self.a * other.c + self.c * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            b: self.b * other.a + self.d * other.b,
            d: self.b * other.c + self.d * other.d,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    #[inline]
    pub fn prepend(self, other: Self) -> Self {
        other.then(self)
    }

    #[inline]
    pub fn is_identity(self) -> bool {
        self.a == 1.0
            && self.c == 0.0
            && self.e == 0.0
            && self.b == 0.0
            && self.d == 1.0
            && self.f == 0.0
    }

    #[inline]
    pub fn is_translation_only(self) -> bool {
        self.a == 1.0 && self.c == 0.0 && self.b == 0.0 && self.d == 1.0
    }

    #[inline]
    pub fn get_translation(self) -> Vec2 {
        Vec2::new(self.e, self.f)
    }
}

impl Default for Matrix3x2 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Matrix3x2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.then(rhs)
    }
}

impl Mul<Vec2> for Matrix3x2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        self.transform_point(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_mul() {
        let a = float_to_fixed(2.5);
        let b = float_to_fixed(3.0);
        let result = fixed_to_float(fixed_mul(a, b));
        assert!((result - 7.5).abs() < 0.01);
    }

    #[test]
    fn test_fixed_div() {
        let a = float_to_fixed(7.5);
        let b = float_to_fixed(2.5);
        let result = fixed_to_float(fixed_div(a, b));
        assert!((result - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_vec2_ops() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(-2.0, -2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
        assert_eq!(2.0 * a, Vec2::new(2.0, 4.0));
        assert_eq!(a.dot(b), 11.0);
        assert_eq!(a.cross(b), -2.0);
    }

    #[test]
    fn test_vec2_perp() {
        let v = Vec2::new(1.0, 0.0);
        assert_eq!(v.perp(), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let r2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        assert!(r1.intersects(r2));
        let r3 = Rect::new(Vec2::new(11.0, 11.0), Vec2::new(20.0, 20.0));
        assert!(!r1.intersects(r3));
    }

    #[test]
    fn test_rect_intersect() {
        let r1 = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let r2 = Rect::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let i = r1.intersect(r2);
        assert_eq!(i.min, Vec2::new(5.0, 5.0));
        assert_eq!(i.max, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_matrix_translate() {
        let t = Matrix3x2::translate(10.0, 20.0);
        let p = t.transform_point(Vec2::new(1.0, 2.0));
        assert_eq!(p, Vec2::new(11.0, 22.0));
    }

    #[test]
    fn test_matrix_rotate() {
        let r = Matrix3x2::rotate(std::f32::consts::PI);
        let p = r.transform_point(Vec2::new(1.0, 0.0));
        assert!((p.x + 1.0).abs() < 1e-6);
        assert!(p.y.abs() < 1e-6);
    }

    #[test]
    fn test_matrix_scale() {
        let s = Matrix3x2::scale(2.0, 3.0);
        let p = s.transform_point(Vec2::new(1.0, 1.0));
        assert_eq!(p, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_matrix_then() {
        let t = Matrix3x2::translate(5.0, 0.0);
        let s = Matrix3x2::scale(2.0, 1.0);
        let m = t.then(s);
        let p = m.transform_point(Vec2::new(1.0, 1.0));
        assert_eq!(p, Vec2::new(7.0, 1.0));
    }

    #[test]
    fn test_matrix_inverse() {
        let m = Matrix3x2::translate(10.0, 20.0).then(Matrix3x2::scale(2.0, 3.0));
        let inv = m.inverse().unwrap();
        let p = Vec2::new(5.0, 6.0);
        let transformed = m.transform_point(p);
        let roundtrip = inv.transform_point(transformed);
        assert!((roundtrip.x - p.x).abs() < 1e-5);
        assert!((roundtrip.y - p.y).abs() < 1e-5);
    }
}
