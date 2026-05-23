use crate::math::{Matrix, Vec2};

#[inline] pub fn translate(dx: f32, dy: f32) -> Matrix { Matrix::translate(dx, dy) }
#[inline] pub fn scale(sx: f32, sy: f32) -> Matrix { Matrix::scale(sx, sy) }
#[inline] pub fn rotate(radians: f32) -> Matrix { Matrix::rotate(radians) }
#[inline] pub fn skew_x(radians: f32) -> Matrix { Matrix::skew_x(radians) }
#[inline] pub fn skew_y(radians: f32) -> Matrix { Matrix::skew_y(radians) }
#[inline] pub fn identity() -> Matrix { Matrix::identity() }

#[inline]
pub fn rotate_about(radians: f32, pivot: Vec2) -> Matrix {
    Matrix::translate(pivot.x, pivot.y).then(Matrix::rotate(radians)).then(Matrix::translate(-pivot.x, -pivot.y))
}

#[inline]
pub fn scale_about(sx: f32, sy: f32, pivot: Vec2) -> Matrix {
    Matrix::translate(pivot.x, pivot.y).then(Matrix::scale(sx, sy)).then(Matrix::translate(-pivot.x, -pivot.y))
}
