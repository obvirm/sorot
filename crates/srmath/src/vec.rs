use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline(always)]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            self / len
        } else {
            Self::ZERO
        }
    }

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline(always)]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline(always)]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline(always)]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline(always)]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
        )
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self * rhs.x, self * rhs.y)
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(v: [f32; 2]) -> Self {
        Self::new(v[0], v[1])
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            self / len
        } else {
            Self::ZERO
        }
    }

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline(always)]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn from_vec3(v: Vec3, w: f32) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    #[inline(always)]
    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Add for Vec4 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
    }
}

impl Sub for Vec4 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;
    #[inline(always)]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self * rhs.x, self * rhs.y, self * rhs.z, self * rhs.w)
    }
}
