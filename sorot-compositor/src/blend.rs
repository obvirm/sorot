use sorot_core::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcOut,
    DstOut,
    SrcAtop,
    DstAtop,
    Xor,
    Plus,
    Multiply,
    Screen,
    Darken,
    Lighten,
}

impl BlendMode {
    pub fn blend(self, src: Color, dst: Color) -> Color {
        match self {
            BlendMode::SrcOver => src.over(dst),
            BlendMode::DstOver => dst.over(src),
            BlendMode::SrcIn => Color {
                r: src.r * dst.a,
                g: src.g * dst.a,
                b: src.b * dst.a,
                a: src.a * dst.a,
            },
            BlendMode::DstIn => Color {
                r: dst.r * src.a,
                g: dst.g * src.a,
                b: dst.b * src.a,
                a: dst.a * src.a,
            },
            BlendMode::SrcOut => {
                let inv = 1.0 - dst.a;
                Color {
                    r: src.r * inv,
                    g: src.g * inv,
                    b: src.b * inv,
                    a: src.a * inv,
                }
            }
            BlendMode::DstOut => {
                let inv = 1.0 - src.a;
                Color {
                    r: dst.r * inv,
                    g: dst.g * inv,
                    b: dst.b * inv,
                    a: dst.a * inv,
                }
            }
            BlendMode::SrcAtop => Color {
                r: src.r * dst.a + dst.r * (1.0 - src.a),
                g: src.g * dst.a + dst.g * (1.0 - src.a),
                b: src.b * dst.a + dst.b * (1.0 - src.a),
                a: dst.a,
            },
            BlendMode::DstAtop => Color {
                r: dst.r * src.a + src.r * (1.0 - dst.a),
                g: dst.g * src.a + src.g * (1.0 - dst.a),
                b: dst.b * src.a + src.b * (1.0 - dst.a),
                a: src.a,
            },
            BlendMode::Xor => {
                let f1 = 1.0 - dst.a;
                let f2 = 1.0 - src.a;
                Color {
                    r: src.r * f1 + dst.r * f2,
                    g: src.g * f1 + dst.g * f2,
                    b: src.b * f1 + dst.b * f2,
                    a: src.a * f1 + dst.a * f2,
                }
            }
            BlendMode::Plus => Color {
                r: (src.r + dst.r).min(dst.a.max(src.a)),
                g: (src.g + dst.g).min(dst.a.max(src.a)),
                b: (src.b + dst.b).min(dst.a.max(src.a)),
                a: (src.a + dst.a).min(1.0),
            },
            BlendMode::Multiply => Color {
                r: src.r * dst.r + src.r * (1.0 - dst.a) + dst.r * (1.0 - src.a),
                g: src.g * dst.g + src.g * (1.0 - dst.a) + dst.g * (1.0 - src.a),
                b: src.b * dst.b + src.b * (1.0 - dst.a) + dst.b * (1.0 - src.a),
                a: src.a + dst.a - src.a * dst.a,
            },
            BlendMode::Screen => Color {
                r: src.r + dst.r - src.r * dst.r,
                g: src.g + dst.g - src.g * dst.g,
                b: src.b + dst.b - src.b * dst.b,
                a: src.a + dst.a - src.a * dst.a,
            },
            BlendMode::Darken => Color {
                r: src.r.min(dst.r) + src.r * (1.0 - dst.a) + dst.r * (1.0 - src.a),
                g: src.g.min(dst.g) + src.g * (1.0 - dst.a) + dst.g * (1.0 - src.a),
                b: src.b.min(dst.b) + src.b * (1.0 - dst.a) + dst.b * (1.0 - src.a),
                a: src.a + dst.a - src.a * dst.a,
            },
            BlendMode::Lighten => Color {
                r: src.r.max(dst.r) + src.r * (1.0 - dst.a) + dst.r * (1.0 - src.a),
                g: src.g.max(dst.g) + src.g * (1.0 - dst.a) + dst.g * (1.0 - src.a),
                b: src.b.max(dst.b) + src.b * (1.0 - dst.a) + dst.b * (1.0 - src.a),
                a: src.a + dst.a - src.a * dst.a,
            },
        }
    }
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::SrcOver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_over() {
        let src = Color::from_rgba(1.0, 0.0, 0.0, 0.5);
        let dst = Color::from_rgba(0.0, 0.0, 1.0, 1.0);
        let result = BlendMode::SrcOver.blend(src, dst);
        assert!(result.r > 0.0);
        assert!(result.b > 0.0);
    }

    #[test]
    fn test_multiply() {
        let src = Color::from_rgba(0.8, 0.8, 0.8, 1.0);
        let dst = Color::from_rgba(0.5, 0.5, 0.5, 1.0);
        let result = BlendMode::Multiply.blend(src, dst);
        assert!(result.r < 0.5);
    }
}
