use srvec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[inline] pub fn new(min: Vec2, max: Vec2) -> Self { Self { min: min.min(max), max: min.max(max) } }
    #[inline] pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self { Self { min: origin, max: origin + size } }
    #[inline] pub fn zero() -> Self { Self { min: Vec2::zero(), max: Vec2::zero() } }
    #[inline] pub fn width(self) -> f32 { self.max.x - self.min.x }
    #[inline] pub fn height(self) -> f32 { self.max.y - self.min.y }
    #[inline] pub fn size(self) -> Vec2 { Vec2::new(self.width(), self.height()) }
    #[inline] pub fn area(self) -> f32 { self.width() * self.height() }
    #[inline] pub fn is_empty(self) -> bool { self.min.x >= self.max.x || self.min.y >= self.max.y }
    #[inline] pub fn center(self) -> Vec2 { Vec2::new((self.min.x + self.max.x) * 0.5, (self.min.y + self.max.y) * 0.5) }
    #[inline] pub fn contains(self, p: Vec2) -> bool { p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y }
    #[inline] pub fn contains_rect(self, other: Self) -> bool { self.min.x <= other.min.x && self.max.x >= other.max.x && self.min.y <= other.min.y && self.max.y >= other.max.y }
    #[inline] pub fn intersects(self, other: Self) -> bool { self.min.x < other.max.x && self.max.x > other.min.x && self.min.y < other.max.y && self.max.y > other.min.y }
    #[inline] pub fn intersect(self, other: Self) -> Self { Self { min: self.min.max(other.min), max: self.max.min(other.max) } }
    #[inline] pub fn union(self, other: Self) -> Self { Self { min: self.min.min(other.min), max: self.max.max(other.max) } }
    #[inline] pub fn union_point(self, p: Vec2) -> Self { Self { min: self.min.min(p), max: self.max.max(p) } }
    #[inline] pub fn inset(self, dx: f32, dy: f32) -> Self { Self { min: Vec2::new(self.min.x + dx, self.min.y + dy), max: Vec2::new(self.max.x - dx, self.max.y - dy) } }
    #[inline] pub fn outset(self, dx: f32, dy: f32) -> Self { self.inset(-dx, -dy) }
    #[inline] pub fn translate(self, offset: Vec2) -> Self { Self { min: self.min + offset, max: self.max + offset } }
}

impl Default for Rect { fn default() -> Self { Self::zero() } }

#[cfg(test)]
#[path = "rect_test.rs"]
mod tests;
