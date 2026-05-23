use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub type Point = Vec2;

impl Vec2 {
    #[inline] pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
    #[inline] pub const fn zero() -> Self { Self { x: 0.0, y: 0.0 } }
    #[inline] pub const fn splat(v: f32) -> Self { Self { x: v, y: v } }
    #[inline] pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y }
    #[inline] pub fn cross(self, other: Self) -> f32 { self.x * other.y - self.y * other.x }
    #[inline] pub fn length_sq(self) -> f32 { self.dot(self) }
    #[inline] pub fn length(self) -> f32 { self.length_sq().sqrt() }
    #[inline] pub fn normalize(self) -> Self { let len = self.length(); if len > 0.0 { Self::new(self.x / len, self.y / len) } else { Self::zero() } }
    #[inline] pub fn try_normalize(self) -> Option<Self> { let len = self.length(); if len > 0.0 { Some(Self::new(self.x / len, self.y / len)) } else { None } }
    #[inline] pub fn lerp(self, other: Self, t: f32) -> Self { Self::new(self.x + (other.x - self.x) * t, self.y + (other.y - self.y) * t) }
    #[inline] pub fn min(self, other: Self) -> Self { Self::new(self.x.min(other.x), self.y.min(other.y)) }
    #[inline] pub fn max(self, other: Self) -> Self { Self::new(self.x.max(other.x), self.y.max(other.y)) }
    #[inline] pub fn abs(self) -> Self { Self::new(self.x.abs(), self.y.abs()) }
    #[inline] pub fn floor(self) -> Self { Self::new(self.x.floor(), self.y.floor()) }
    #[inline] pub fn ceil(self) -> Self { Self::new(self.x.ceil(), self.y.ceil()) }
    #[inline] pub fn perp(self) -> Self { Self::new(-self.y, self.x) }
    #[inline] pub fn distance_to(self, other: Self) -> f32 { (self - other).length() }
}

impl Add for Vec2 { type Output = Self; #[inline] fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) } }
impl Sub for Vec2 { type Output = Self; #[inline] fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) } }
impl Mul<f32> for Vec2 { type Output = Self; #[inline] fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs) } }
impl Mul<Vec2> for f32 { type Output = Vec2; #[inline] fn mul(self, rhs: Vec2) -> Vec2 { Vec2::new(self * rhs.x, self * rhs.y) } }
impl Div<f32> for Vec2 { type Output = Self; #[inline] fn div(self, rhs: f32) -> Self { let inv = 1.0 / rhs; Self::new(self.x * inv, self.y * inv) } }
impl Neg for Vec2 { type Output = Self; #[inline] fn neg(self) -> Self { Self::new(-self.x, -self.y) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_ops() { let a = Vec2::new(1.0, 2.0); let b = Vec2::new(3.0, 4.0); assert_eq!(a + b, Vec2::new(4.0, 6.0)); assert_eq!(a.dot(b), 11.0); assert_eq!(a.cross(b), -2.0); }
    #[test] fn test_perp() { assert_eq!(Vec2::new(1.0, 0.0).perp(), Vec2::new(0.0, 1.0)); }
}
