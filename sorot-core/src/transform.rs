use crate::math::{Matrix3x2, Vec2};

#[inline]
pub fn translate(dx: f32, dy: f32) -> Matrix3x2 {
    Matrix3x2::translate(dx, dy)
}

#[inline]
pub fn scale(sx: f32, sy: f32) -> Matrix3x2 {
    Matrix3x2::scale(sx, sy)
}

#[inline]
pub fn rotate(radians: f32) -> Matrix3x2 {
    Matrix3x2::rotate(radians)
}

#[inline]
pub fn skew_x(radians: f32) -> Matrix3x2 {
    Matrix3x2::skew_x(radians)
}

#[inline]
pub fn skew_y(radians: f32) -> Matrix3x2 {
    Matrix3x2::skew_y(radians)
}

#[inline]
pub fn identity() -> Matrix3x2 {
    Matrix3x2::identity()
}

/// Create a rotation about a pivot point.
#[inline]
pub fn rotate_about(radians: f32, pivot: Vec2) -> Matrix3x2 {
    translate(pivot.x, pivot.y)
        .then(rotate(radians))
        .then(translate(-pivot.x, -pivot.y))
}

/// Create a scale about a pivot point.
#[inline]
pub fn scale_about(sx: f32, sy: f32, pivot: Vec2) -> Matrix3x2 {
    translate(pivot.x, pivot.y)
        .then(scale(sx, sy))
        .then(translate(-pivot.x, -pivot.y))
}
