use std::ops::Mul;

use super::vec2::Vec2;

/// Type mask bits for fast-path dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypeMask(u8);

impl TypeMask {
    pub const IDENTITY: u8 = 0;
    pub const TRANSLATE: u8 = 1 << 0;
    pub const SCALE: u8 = 1 << 1;
    pub const AFFINE: u8 = 1 << 2;
    pub const PERSPECTIVE: u8 = 1 << 3;
    pub const RECT_STAYS_RECT: u8 = 1 << 4;

    #[inline] pub fn bits(self) -> u8 { self.0 }
    #[inline] pub fn is_identity(self) -> bool { self.0 == Self::IDENTITY }
    #[inline] pub fn is_translate(self) -> bool { self.0 == Self::TRANSLATE }
    #[inline] pub fn is_affine(self) -> bool { self.0 & Self::PERSPECTIVE == 0 }

    #[inline] fn set(&mut self, bit: u8) { self.0 |= bit; }

    fn compute(a: f32, c: f32, e: f32, b: f32, d: f32, f: f32, p0: f32, p1: f32, p2: f32) -> Self {
        let mut bits = Self::IDENTITY;
        if p0 != 0.0 || p1 != 0.0 || p2 != 1.0 { bits |= Self::PERSPECTIVE; bits |= Self::AFFINE; return Self(bits); }
        if c != 0.0 || b != 0.0 { bits |= Self::AFFINE; }
        if a != 1.0 || d != 1.0 { bits |= Self::SCALE; }
        if e != 0.0 || f != 0.0 { bits |= Self::TRANSLATE; }
        if bits == Self::AFFINE && b == -c && a == d && a * a + b * b == 1.0 { bits |= Self::RECT_STAYS_RECT; }
        Self(bits)
    }
}

/// 3x3 column-major matrix with perspective support.
///
/// Layout (applied as `p' = M * p` for column vector, so row-major storage):
/// ```text
/// | a  c  tx |   sx  kx  tx
/// | b  d  ty | = ky  sy  ty
/// | p0 p1 p2 |   p0  p1  p2
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Matrix {
    /// Scale X, row 0 col 0
    pub a: f32,
    /// Skew X, row 1 col 0
    pub c: f32,
    /// Translate X, row 0 col 2
    pub e: f32,
    /// Skew Y, row 0 col 1
    pub b: f32,
    /// Scale Y, row 1 col 1
    pub d: f32,
    /// Translate Y, row 1 col 2
    pub f: f32,
    /// Perspective X
    pub p0: f32,
    /// Perspective Y
    pub p1: f32,
    /// Perspective Z (almost always 1.0)
    pub p2: f32,

    mask: TypeMask,
}

impl Matrix {
    #[inline]
    pub const fn identity() -> Self {
        Self {
            a: 1.0, c: 0.0, e: 0.0,
            b: 0.0, d: 1.0, f: 0.0,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(0),
        }
    }

    #[inline]
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0, c: 0.0, e: x,
            b: 0.0, d: 1.0, f: y,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(TypeMask::TRANSLATE),
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        let bits = if sx == sy { TypeMask::TRANSLATE | TypeMask::SCALE | TypeMask::RECT_STAYS_RECT } else { TypeMask::SCALE };
        Self {
            a: sx, c: 0.0, e: 0.0,
            b: 0.0, d: sy, f: 0.0,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(bits),
        }
    }

    #[inline]
    pub fn rotate(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            a: cos, c: -sin, e: 0.0,
            b: sin, d: cos,  f: 0.0,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(TypeMask::AFFINE | TypeMask::RECT_STAYS_RECT),
        }
    }

    #[inline]
    pub fn skew_x(radians: f32) -> Self {
        Self {
            a: 1.0, c: radians.tan(), e: 0.0,
            b: 0.0, d: 1.0,          f: 0.0,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(TypeMask::AFFINE),
        }
    }

    #[inline]
    pub fn skew_y(radians: f32) -> Self {
        Self {
            a: 1.0,           c: 0.0, e: 0.0,
            b: radians.tan(), d: 1.0, f: 0.0,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(TypeMask::AFFINE),
        }
    }

    #[inline]
    pub fn determinant(self) -> f32 {
        self.a * self.d - self.c * self.b
    }

    #[inline]
    pub fn inverse(self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-12 { return None; }
        let inv_det = 1.0 / det;
        let m = Self {
            a: self.d * inv_det,
            c: -self.c * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            b: -self.b * inv_det,
            d: self.a * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(0),
        };
        Some(Self { mask: TypeMask::compute(m.a, m.c, m.e, m.b, m.d, m.f, m.p0, m.p1, m.p2), ..m })
    }

    /// Map a point through this matrix. Type-mask dispatched for performance.
    #[inline]
    pub fn map_point(self, p: Vec2) -> Vec2 {
        match self.mask.bits() {
            TypeMask::IDENTITY => p,
            TypeMask::TRANSLATE => Vec2::new(p.x + self.e, p.y + self.f),
            TypeMask::TRANSLATE | TypeMask::SCALE => Vec2::new(self.a * p.x + self.e, self.d * p.y + self.f),
            _ if self.mask.is_affine() => Vec2::new(
                self.a * p.x + self.c * p.y + self.e,
                self.b * p.x + self.d * p.y + self.f,
            ),
            _ => {
                let w = self.p0 * p.x + self.p1 * p.y + self.p2;
                let inv = if w != 0.0 { 1.0 / w } else { 0.0 };
                Vec2::new(
                    (self.a * p.x + self.c * p.y + self.e) * inv,
                    (self.b * p.x + self.d * p.y + self.f) * inv,
                )
            }
        }
    }

    /// Legacy alias for map_point.
    #[inline]
    pub fn transform_point(self, p: Vec2) -> Vec2 { self.map_point(p) }

    /// Map a vector (no translation applied).
    #[inline]
    pub fn map_vector(self, v: Vec2) -> Vec2 {
        Vec2::new(self.a * v.x + self.c * v.y, self.b * v.x + self.d * v.y)
    }

    /// Legacy alias for map_vector.
    #[inline]
    pub fn transform_vector(self, v: Vec2) -> Vec2 { self.map_vector(v) }

    /// Map a rect by mapping 4 corners and taking the bounding box.
    pub fn map_rect(self, r: super::rect::Rect) -> super::rect::Rect {
        let tl = self.map_point(r.min);
        let tr = self.map_point(Vec2::new(r.max.x, r.min.y));
        let bl = self.map_point(Vec2::new(r.min.x, r.max.y));
        let br = self.map_point(r.max);
        super::rect::Rect::new(
            tl.min(tr).min(bl).min(br),
            tl.max(tr).max(bl).max(br),
        )
    }

    /// Batch-map points in-place.
    pub fn map_points(self, pts: &mut [Vec2]) {
        match self.mask.bits() {
            TypeMask::IDENTITY => {}
            TypeMask::TRANSLATE => {
                for p in pts.iter_mut() { p.x += self.e; p.y += self.f; }
            }
            _ => {
                for p in pts.iter_mut() { *p = self.map_point(*p); }
            }
        }
    }

    /// `self = self * other` (self applies first, then other). This matches the old `then` semantics.
    #[inline]
    pub fn pre_concat(self, other: Self) -> Self {
        let m = Self {
            a: self.a * other.a + self.c * other.b,
            c: self.a * other.c + self.c * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            b: self.b * other.a + self.d * other.b,
            d: self.b * other.c + self.d * other.d,
            f: self.b * other.e + self.d * other.f + self.f,
            p0: 0.0, p1: 0.0, p2: 1.0,
            mask: TypeMask(0),
        };
        Self { mask: TypeMask::compute(m.a, m.c, m.e, m.b, m.d, m.f, m.p0, m.p1, m.p2), ..m }
    }

    /// Legacy alias for pre_concat: `self.then(other)` = self applied first, then other.
    #[inline]
    pub fn then(self, other: Self) -> Self { self.pre_concat(other) }

    /// Legacy alias: `self.prepend(other)` = other applied first, then self.
    #[inline]
    pub fn prepend(self, other: Self) -> Self { other.pre_concat(self) }

    /// Post-concat: `self = other * self` (other applied first, then self).
    #[inline]
    pub fn post_concat(self, other: Self) -> Self { other.pre_concat(self) }

    #[inline]
    pub fn is_identity(self) -> bool { self.mask.is_identity() }

    #[inline]
    pub fn is_translation_only(self) -> bool { self.mask.is_translate() }

    #[inline]
    pub fn is_affine(self) -> bool { self.mask.is_affine() }

    #[inline]
    pub fn get_translation(self) -> Vec2 { Vec2::new(self.e, self.f) }

    pub fn decompose_scale(self) -> (f32, f32) {
        let sx = Vec2::new(self.a, self.b).length();
        let sy = Vec2::new(self.c, self.d).length();
        (sx, sy)
    }

    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        (self.a - other.a).abs() < epsilon
            && (self.c - other.c).abs() < epsilon
            && (self.e - other.e).abs() < epsilon
            && (self.b - other.b).abs() < epsilon
            && (self.d - other.d).abs() < epsilon
            && (self.f - other.f).abs() < epsilon
    }

    /// Raw column access for GPU upload (column-major 4x4).
    pub fn to_cols_4x4(self) -> [f32; 16] {
        [
            self.a, self.b, 0.0, self.p0,
            self.c, self.d, 0.0, self.p1,
            0.0,    0.0,    1.0, 0.0,
            self.e, self.f, 0.0, self.p2,
        ]
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool { self.approx_eq(*other, 1e-7) }
}

impl Default for Matrix {
    fn default() -> Self { Self::identity() }
}

impl Mul for Matrix {
    type Output = Self;
    #[inline] fn mul(self, rhs: Self) -> Self { self.pre_concat(rhs) }
}

impl Mul<Vec2> for Matrix {
    type Output = Vec2;
    #[inline] fn mul(self, rhs: Vec2) -> Vec2 { self.map_point(rhs) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Matrix::identity();
        assert!(m.is_identity());
        assert_eq!(m.map_point(Vec2::new(5.0, 3.0)), Vec2::new(5.0, 3.0));
    }

    #[test]
    fn test_translate() {
        let m = Matrix::translate(10.0, 20.0);
        assert!(m.is_translation_only());
        assert_eq!(m.map_point(Vec2::new(1.0, 2.0)), Vec2::new(11.0, 22.0));
    }

    #[test]
    fn test_scale() {
        let m = Matrix::scale(2.0, 3.0);
        assert_eq!(m.map_point(Vec2::new(1.0, 1.0)), Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_rotate() {
        let m = Matrix::rotate(std::f32::consts::PI);
        let p = m.map_point(Vec2::new(1.0, 0.0));
        assert!((p.x + 1.0).abs() < 1e-5);
        assert!(p.y.abs() < 1e-5);
    }

    #[test]
    fn test_then() {
        let m = Matrix::translate(5.0, 0.0).then(Matrix::scale(2.0, 1.0));
        assert_eq!(m.map_point(Vec2::new(1.0, 1.0)), Vec2::new(7.0, 1.0));
    }

    #[test]
    fn test_inverse() {
        let m = Matrix::translate(10.0, 20.0).then(Matrix::scale(2.0, 3.0));
        let inv = m.inverse().unwrap();
        let p = Vec2::new(5.0, 6.0);
        let rt = inv.map_point(m.map_point(p));
        assert!((rt.x - p.x).abs() < 1e-4);
    }

    #[test]
    fn test_map_vector() {
        let m = Matrix::translate(10.0, 20.0).then(Matrix::scale(2.0, 1.0));
        assert_eq!(m.map_vector(Vec2::new(3.0, 4.0)), Vec2::new(6.0, 4.0));
    }

    #[test]
    fn test_decompose() {
        let (sx, sy) = Matrix::scale(2.0, 3.0).decompose_scale();
        assert!((sx - 2.0).abs() < 1e-5);
        assert!((sy - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_map_points() {
        let m = Matrix::translate(1.0, 2.0);
        let mut pts = vec![Vec2::new(3.0, 4.0), Vec2::new(5.0, 6.0)];
        m.map_points(&mut pts);
        assert_eq!(pts[0], Vec2::new(4.0, 6.0));
    }
}
