use slotmap::new_key_type;
use srmath::{Color, Vec2};

new_key_type! {
    pub struct ShapeId;
}

#[derive(Debug, Clone, Copy)]
pub enum Fill {
    Solid(Color),
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Rect {
        id: ShapeId,
        position: Vec2,
        size: Vec2,
        fill: Fill,
        stroke: Option<Stroke>,
        corner_radius: f32,
    },
    Circle {
        id: ShapeId,
        center: Vec2,
        radius: f32,
        fill: Fill,
        stroke: Option<Stroke>,
    },
    Triangle {
        id: ShapeId,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        fill: Fill,
        stroke: Option<Stroke>,
    },
}

impl Shape {
    pub fn id(&self) -> ShapeId {
        match self {
            Shape::Rect { id, .. }
            | Shape::Circle { id, .. }
            | Shape::Triangle { id, .. } => *id,
        }
    }

    pub fn bounding_box(&self) -> Option<(Vec2, Vec2)> {
        match self {
            Shape::Rect { position, size, .. } => {
                let half = *size * 0.5;
                Some((*position - half, *position + half))
            }
            Shape::Circle { center, radius, .. } => {
                let r = Vec2::splat(*radius);
                Some((*center - r, *center + r))
            }
            Shape::Triangle { a, b, c, .. } => {
                let min = a.min(*b).min(*c);
                let max = a.max(*b).max(*c);
                Some((min, max))
            }
        }
    }
}
