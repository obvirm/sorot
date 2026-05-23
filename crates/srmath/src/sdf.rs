use crate::Vec2;

pub trait Sdf {
    fn evaluate(&self, p: Vec2) -> f32;
    fn bounding_box(&self) -> Option<(Vec2, Vec2)> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Sdf for Circle {
    fn evaluate(&self, p: Vec2) -> f32 {
        p.length() - self.radius
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub half_size: Vec2,
}

impl Rectangle {
    pub fn new(size: Vec2) -> Self {
        Self {
            half_size: size * 0.5,
        }
    }
}

impl Sdf for Rectangle {
    fn evaluate(&self, p: Vec2) -> f32 {
        let d = p.abs() - self.half_size;
        d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0)
    }

    fn bounding_box(&self) -> Option<(Vec2, Vec2)> {
        Some((-self.half_size, self.half_size))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoundedRectangle {
    pub half_size: Vec2,
    pub radius: f32,
}

impl RoundedRectangle {
    pub fn new(size: Vec2, radius: f32) -> Self {
        Self {
            half_size: size * 0.5,
            radius,
        }
    }
}

impl Sdf for RoundedRectangle {
    fn evaluate(&self, p: Vec2) -> f32 {
        let d = p.abs() - self.half_size + Vec2::splat(self.radius);
        d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0) - self.radius
    }

    fn bounding_box(&self) -> Option<(Vec2, Vec2)> {
        Some((-self.half_size, self.half_size))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }
}

impl Sdf for Line {
    fn evaluate(&self, p: Vec2) -> f32 {
        let pa = p - self.start;
        let ba = self.end - self.start;
        let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
        (pa - ba * h).length()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

impl Triangle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self { a, b, c }
    }
}

impl Sdf for Triangle {
    fn evaluate(&self, p: Vec2) -> f32 {
        let e0 = self.b - self.a;
        let e1 = self.c - self.b;
        let e2 = self.a - self.c;
        let v0 = p - self.a;
        let v1 = p - self.b;
        let v2 = p - self.c;

        let pq0 = v0 - e0 * (v0.dot(e0) / e0.dot(e0)).clamp(0.0, 1.0);
        let pq1 = v1 - e1 * (v1.dot(e1) / e1.dot(e1)).clamp(0.0, 1.0);
        let pq2 = v2 - e2 * (v2.dot(e2) / e2.dot(e2)).clamp(0.0, 1.0);

        let s = e0.x * e2.y - e0.y * e2.x;

        let d = (pq0.length_squared())
            .min(pq1.length_squared())
            .min(pq2.length_squared())
            .sqrt();

        let inside = s.signum() * (v0.x * e0.y - v0.y * e0.x).signum() > 0.0;
        if inside { -d } else { d }
    }
}

#[inline(always)]
pub fn sdf_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

#[inline(always)]
pub fn sdf_intersection(d1: f32, d2: f32) -> f32 {
    d1.max(d2)
}

#[inline(always)]
pub fn sdf_difference(d1: f32, d2: f32) -> f32 {
    d1.max(-d2)
}

#[inline(always)]
pub fn sdf_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d1 * (1.0 - h) + d2 * h - k * h * (1.0 - h)
}

#[inline(always)]
pub fn sdf_smooth_intersection(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 - 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d1 * (1.0 - h) + d2 * h + k * h * (1.0 - h)
}

#[inline(always)]
pub fn sdf_round(d: f32, r: f32) -> f32 {
    d - r
}

#[inline(always)]
pub fn sdf_onion(d: f32, thickness: f32) -> f32 {
    d.abs() - thickness
}

#[inline(always)]
pub fn sdf_annular(d: f32, r: f32) -> f32 {
    d.abs() - r
}
