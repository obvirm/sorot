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

/// A single color stop in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub position: f32, // 0.0 to 1.0
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GradientType {
    Linear {
        x0: f32, y0: f32, // start point
        x1: f32, y1: f32, // end point
    },
    Radial {
        cx: f32, cy: f32, // center
        radius: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: GradientType,
    pub stops: Vec<GradientStop>,
}

/// What fills the shape: solid color or gradient.
#[derive(Debug, Clone, PartialEq)]
pub enum FillKind {
    Solid(Color),
    Gradient(Gradient),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    pub fill_kind: FillKind,
    pub style: PaintStyle,
    pub anti_alias: bool,
}

impl Paint {
    pub fn fill(color: Color) -> Self {
        Self {
            fill_kind: FillKind::Solid(color),
            style: PaintStyle::Fill,
            anti_alias: true,
        }
    }

    pub fn fill_gradient(gradient: Gradient) -> Self {
        Self {
            fill_kind: FillKind::Gradient(gradient),
            style: PaintStyle::Fill,
            anti_alias: true,
        }
    }

    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            fill_kind: FillKind::Solid(color),
            style: PaintStyle::Stroke {
                width,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            },
            anti_alias: true,
        }
    }

    pub fn stroke_gradient(gradient: Gradient, width: f32) -> Self {
        Self {
            fill_kind: FillKind::Gradient(gradient),
            style: PaintStyle::Stroke {
                width,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
            },
            anti_alias: true,
        }
    }

    /// Resolve the color at a gradient t value (0..1). Falls back to solid color for non-gradient fills.
    pub fn color_at(&self, t: f32) -> Color {
        match &self.fill_kind {
            FillKind::Solid(c) => *c,
            FillKind::Gradient(g) => sample_gradient(&g.stops, t),
        }
    }

    /// Convenience: get the solid color (or first gradient stop color).
    pub fn color(&self) -> Color {
        match &self.fill_kind {
            FillKind::Solid(c) => *c,
            FillKind::Gradient(g) => g.stops.first().map(|s| s.color).unwrap_or(Color::BLACK),
        }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            fill_kind: FillKind::Solid(Color::BLACK),
            style: PaintStyle::Fill,
            anti_alias: true,
        }
    }
}

/// Sample a gradient color ramp at position t (0..1).
fn sample_gradient(stops: &[GradientStop], t: f32) -> Color {
    if stops.is_empty() {
        return Color::TRANSPARENT;
    }
    if stops.len() == 1 {
        return stops[0].color;
    }

    let t = t.clamp(0.0, 1.0);

    // Find the two stops we're between
    for i in 0..stops.len() - 1 {
        if t >= stops[i].position && t <= stops[i + 1].position {
            let range = stops[i + 1].position - stops[i].position;
            if range <= 0.0 {
                return stops[i].color;
            }
            let local_t = (t - stops[i].position) / range;
            return stops[i].color.lerp(stops[i + 1].color, local_t);
        }
    }

    if t <= stops[0].position {
        stops[0].color
    } else {
        stops[stops.len() - 1].color
    }
}

/// Build a gradient with evenly spaced stops from a list of colors.
pub fn gradient_from_colors(colors: &[Color]) -> Gradient {
    let n = colors.len();
    if n == 0 {
        return Gradient {
            kind: GradientType::Linear { x0: 0.0, y0: 0.0, x1: 1.0, y1: 0.0 },
            stops: vec![GradientStop { position: 0.0, color: Color::BLACK }],
        };
    }
    let stops: Vec<GradientStop> = colors.iter().enumerate().map(|(i, c)| {
        GradientStop {
            position: if n == 1 { 0.0 } else { i as f32 / (n - 1) as f32 },
            color: *c,
        }
    }).collect();
    Gradient {
        kind: GradientType::Linear { x0: 0.0, y0: 0.0, x1: 1.0, y1: 0.0 },
        stops,
    }
}
