use sorot_core::math::{Rect, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct Cubic {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
}

impl Cubic {
    #[inline]
    pub fn new(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        Self { p0, p1, p2, p3 }
    }

    #[inline]
    pub fn from_line(p0: Vec2, p3: Vec2) -> Self {
        let third = (p3 - p0) * (1.0 / 3.0);
        Self {
            p0,
            p1: p0 + third,
            p2: p0 + third * 2.0,
            p3,
        }
    }

    pub fn eval(self, t: f32) -> Vec2 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        self.p0 * mt3
            + self.p1 * (3.0 * mt2 * t)
            + self.p2 * (3.0 * mt * t2)
            + self.p3 * t3
    }

    pub fn derivative(self, t: f32) -> Vec2 {
        let mt = 1.0 - t;
        let a = (self.p1 - self.p0) * 3.0;
        let b = (self.p2 - self.p1) * 3.0;
        let c = (self.p3 - self.p2) * 3.0;
        a * (mt * mt) + b * (2.0 * mt * t) + c * (t * t)
    }

    pub fn split(self, t: f32) -> (Self, Self) {
        let p01 = self.p0.lerp(self.p1, t);
        let p12 = self.p1.lerp(self.p2, t);
        let p23 = self.p2.lerp(self.p3, t);
        let p012 = p01.lerp(p12, t);
        let p123 = p12.lerp(p23, t);
        let p0123 = p012.lerp(p123, t);

        (
            Self::new(self.p0, p01, p012, p0123),
            Self::new(p0123, p123, p23, self.p3),
        )
    }

    pub fn bounding_box(self) -> Rect {
        let mut min = self.p0.min(self.p3);
        let mut max = self.p0.max(self.p3);

        for t in find_cubic_extrema(self.p0, self.p1, self.p2, self.p3) {
            let p = self.eval(t);
            min = min.min(p);
            max = max.max(p);
        }

        Rect::new(min, max)
    }

    pub fn is_degenerate(self) -> bool {
        let d01 = (self.p1 - self.p0).length_sq();
        let d12 = (self.p2 - self.p1).length_sq();
        let d23 = (self.p3 - self.p2).length_sq();
        d01 < 1e-12 && d12 < 1e-12 && d23 < 1e-12
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
}

impl Quad {
    #[inline]
    pub fn new(p0: Vec2, p1: Vec2, p2: Vec2) -> Self {
        Self { p0, p1, p2 }
    }

    pub fn eval(self, t: f32) -> Vec2 {
        let mt = 1.0 - t;
        self.p0 * (mt * mt) + self.p1 * (2.0 * mt * t) + self.p2 * (t * t)
    }

    pub fn derivative(self, t: f32) -> Vec2 {
        let mt = 1.0 - t;
        (self.p1 - self.p0) * (2.0 * mt) + (self.p2 - self.p1) * (2.0 * t)
    }

    pub fn to_cubic(self) -> Cubic {
        Cubic::new(
            self.p0,
            self.p0 + (self.p1 - self.p0) * (2.0 / 3.0),
            self.p2 + (self.p1 - self.p2) * (2.0 / 3.0),
            self.p2,
        )
    }

    pub fn split(self, t: f32) -> (Self, Self) {
        let p01 = self.p0.lerp(self.p1, t);
        let p12 = self.p1.lerp(self.p2, t);
        let p012 = p01.lerp(p12, t);

        (Self::new(self.p0, p01, p012), Self::new(p012, p12, self.p2))
    }

    pub fn bounding_box(self) -> Rect {
        let cubic = self.to_cubic();
        cubic.bounding_box()
    }
}

fn find_cubic_extrema(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Vec<f32> {
    let mut result = Vec::with_capacity(4);

    let a = -p0 + p1 * 3.0 - p2 * 3.0 + p3;
    let b = p0 * 2.0 - p1 * 4.0 + p2 * 2.0;
    let c = -p0 + p1;

    for t in solve_quadratic(a.x, b.x, c.x) {
        if t > 0.0 && t < 1.0 {
            result.push(t);
        }
    }
    for t in solve_quadratic(a.y, b.y, c.y) {
        if t > 0.0 && t < 1.0 {
            result.push(t);
        }
    }

    result
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a.abs() < 1e-12 {
        if b.abs() < 1e-12 {
            return Vec::new();
        }
        return vec![-c / b];
    }

    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return Vec::new();
    }

    let sqrt_disc = disc.sqrt();
    let inv_2a = 0.5 / a;
    vec![(-b - sqrt_disc) * inv_2a, (-b + sqrt_disc) * inv_2a]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_eval() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, 2.0),
            Vec2::new(4.0, 0.0),
        );
        let start = c.eval(0.0);
        let end = c.eval(1.0);
        let mid = c.eval(0.5);
        assert!((start.x - 0.0).abs() < 1e-6);
        assert!((end.x - 4.0).abs() < 1e-6);
        assert!((mid.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_split() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 0.0),
        );
        let (left, right) = c.split(0.5);
        let mid = c.eval(0.5);
        assert!((left.p3 - mid).length() < 1e-6);
        assert!((right.p0 - mid).length() < 1e-6);
    }

    #[test]
    fn test_cubic_bbox() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, -1.0),
            Vec2::new(4.0, 0.0),
        );
        let bbox = c.bounding_box();
        assert!(bbox.min.x <= 0.0);
        assert!(bbox.max.x >= 4.0);
    }

    #[test]
    fn test_quad_to_cubic() {
        let q = Quad::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(2.0, 0.0),
        );
        let c = q.to_cubic();
        let diff_start = (q.eval(0.0) - c.eval(0.0)).length();
        let diff_mid = (q.eval(0.5) - c.eval(0.5)).length();
        let diff_end = (q.eval(1.0) - c.eval(1.0)).length();
        assert!(diff_start < 1e-6);
        assert!(diff_mid < 1e-6);
        assert!(diff_end < 1e-6);
    }
}
