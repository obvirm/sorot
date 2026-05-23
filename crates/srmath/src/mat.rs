use crate::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ],
    };

    pub fn from_rows(r0: Vec4, r1: Vec4, r2: Vec4, r3: Vec4) -> Self {
        Self {
            cols: [
                Vec4::new(r0.x, r1.x, r2.x, r3.x),
                Vec4::new(r0.y, r1.y, r2.y, r3.y),
                Vec4::new(r0.z, r1.z, r2.z, r3.z),
                Vec4::new(r0.w, r1.w, r2.w, r3.w),
            ],
        }
    }

    pub fn ortho_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let r_width = 1.0 / (right - left);
        let r_height = 1.0 / (top - bottom);
        let r_depth = 1.0 / (far - near);

        Self {
            cols: [
                Vec4::new(2.0 * r_width, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 2.0 * r_height, 0.0, 0.0),
                Vec4::new(0.0, 0.0, -r_depth, 0.0),
                Vec4::new(
                    -(right + left) * r_width,
                    -(top + bottom) * r_height,
                    -near * r_depth,
                    1.0,
                ),
            ],
        }
    }

    #[inline(always)]
    pub fn transform_vec3(&self, v: Vec3) -> Vec3 {
        let v4 = Vec4::new(v.x, v.y, v.z, 1.0);
        let r = Vec4::new(
            self.cols[0].x * v4.x + self.cols[1].x * v4.y + self.cols[2].x * v4.z + self.cols[3].x * v4.w,
            self.cols[0].y * v4.x + self.cols[1].y * v4.y + self.cols[2].y * v4.z + self.cols[3].y * v4.w,
            self.cols[0].z * v4.x + self.cols[1].z * v4.y + self.cols[2].z * v4.z + self.cols[3].z * v4.w,
            self.cols[0].w * v4.x + self.cols[1].w * v4.y + self.cols[2].w * v4.z + self.cols[3].w * v4.w,
        );
        Vec3::new(r.x, r.y, r.z)
    }

    pub fn to_cols_array(&self) -> [[f32; 4]; 4] {
        [
            [self.cols[0].x, self.cols[0].y, self.cols[0].z, self.cols[0].w],
            [self.cols[1].x, self.cols[1].y, self.cols[1].z, self.cols[1].w],
            [self.cols[2].x, self.cols[2].y, self.cols[2].z, self.cols[2].w],
            [self.cols[3].x, self.cols[3].y, self.cols[3].z, self.cols[3].w],
        ]
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    pub fn new(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let sx = self.scale.x;
        let sy = self.scale.y;

        Mat4 {
            cols: [
                Vec4::new(cos * sx, sin * sx, 0.0, 0.0),
                Vec4::new(-sin * sy, cos * sy, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(self.translation.x, self.translation.y, 0.0, 1.0),
            ],
        }
    }

    pub fn inverse_matrix(&self) -> Mat4 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let sx = 1.0 / self.scale.x;
        let sy = 1.0 / self.scale.y;

        Mat4 {
            cols: [
                Vec4::new(cos * sx, -sin * sx, 0.0, 0.0),
                Vec4::new(sin * sy, cos * sy, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(-self.translation.x, -self.translation.y, 0.0, 1.0),
            ],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoundingBox {
    pub const EMPTY: Self = Self {
        min: Vec2::new(f32::MAX, f32::MAX),
        max: Vec2::new(f32::MIN, f32::MIN),
    };

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self::new(center - half, center + half)
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn union(&self, other: &Self) -> Self {
        Self::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self::new(self.min.max(other.min), self.max.min(other.max))
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::EMPTY
    }
}
