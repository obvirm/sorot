use color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaintStyle {
    Fill,
    Stroke { width: f32, cap: LineCap, join: LineJoin },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    pub color: Color,
    pub style: PaintStyle,
    pub anti_alias: bool,
}

impl Paint {
    pub fn fill(color: Color) -> Self {
        Self {
            color,
            style: PaintStyle::Fill,
            anti_alias: true,
        }
    }

    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            color,
            style: PaintStyle::Stroke {
                width,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            },
            anti_alias: true,
        }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            style: PaintStyle::Fill,
            anti_alias: true,
        }
    }
}
