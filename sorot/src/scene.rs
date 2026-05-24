use vector::Vec2;
use rect::Rect;
use color::Color;
use paint::{Paint, Gradient, GradientType, GradientStop};
use pathbuilder::Path;
use flatten::flatten_path;
use raster::sdf::compute_sdf;
use renderir::render_ir::{RenderFrame, SdfOp};
use canvas::Canvas;
use pipeline::Pipeline;

pub fn build_frame() -> RenderFrame {
    let mut canvas = Canvas::new();

    // ── Background ──
    canvas.set_paint(Paint::fill(Color::from_rgba(0.12, 0.12, 0.15, 1.0)));
    canvas.draw_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(800.0, 600.0)));

    // ── 1. Solid fill shapes ──
    canvas.set_paint(Paint::fill(Color::from_rgba(0.2, 0.6, 0.9, 0.95)));
    canvas.draw_oval(Vec2::new(160.0, 160.0), 120.0, 120.0);

    canvas.set_paint(Paint::fill(Color::from_rgba(0.9, 0.3, 0.2, 0.85)));
    let p = Path::rounded_rect(Vec2::new(40.0, 40.0), Vec2::new(220.0, 280.0), 15.0, 15.0);
    canvas.translate(100.0, 20.0);
    canvas.draw_path(&p);
    canvas.translate(-100.0, -20.0);

    // ── 2. Gradient fills ──
    let grad = Gradient {
        kind: GradientType::Linear { x0: 300.0, y0: 50.0, x1: 520.0, y1: 250.0 },
        stops: vec![
            GradientStop { position: 0.0, color: Color::from_rgba(1.0, 0.2, 0.3, 1.0) },
            GradientStop { position: 0.5, color: Color::from_rgba(1.0, 0.8, 0.1, 1.0) },
            GradientStop { position: 1.0, color: Color::from_rgba(0.2, 0.9, 0.4, 1.0) },
        ],
    };
    canvas.set_paint(Paint::fill_gradient(grad.clone()));
    canvas.draw_rect(Rect::new(Vec2::new(300.0, 50.0), Vec2::new(520.0, 250.0)));

    let rad_grad = Gradient {
        kind: GradientType::Radial { cx: 680.0, cy: 150.0, radius: 100.0 },
        stops: vec![
            GradientStop { position: 0.0, color: Color::from_rgba(1.0, 1.0, 1.0, 1.0) },
            GradientStop { position: 0.4, color: Color::from_rgba(0.4, 0.3, 0.9, 1.0) },
            GradientStop { position: 1.0, color: Color::from_rgba(0.1, 0.1, 0.3, 0.0) },
        ],
    };
    canvas.set_paint(Paint::fill_gradient(rad_grad));
    canvas.draw_oval(Vec2::new(680.0, 150.0), 100.0, 100.0);

    // ── 3. Stroke rendering ──
    canvas.set_paint(Paint::stroke(Color::from_rgba(0.9, 0.2, 0.5, 0.95), 4.0));
    let star = build_star_path(Vec2::new(160.0, 420.0), 50.0, 100.0, 5);
    canvas.draw_path(&star);

    canvas.set_paint(Paint::stroke(Color::from_rgba(0.3, 0.8, 0.9, 0.95), 3.0));
    let spiral = build_spiral_path(Vec2::new(400.0, 420.0), 80.0, 4.0);
    canvas.draw_path(&spiral);

    canvas.set_paint(Paint::stroke(Color::from_rgba(0.2, 0.9, 0.4, 0.9), 6.0));
    canvas.draw_line(Vec2::new(540.0, 360.0), Vec2::new(740.0, 480.0));

    // ── 4. Gradient stroke ──
    let stroke_grad = Gradient {
        kind: GradientType::Linear { x0: 520.0, y0: 360.0, x1: 760.0, y1: 500.0 },
        stops: vec![
            GradientStop { position: 0.0, color: Color::from_rgba(1.0, 0.5, 0.0, 1.0) },
            GradientStop { position: 1.0, color: Color::from_rgba(0.8, 0.2, 1.0, 1.0) },
        ],
    };
    canvas.set_paint(Paint::stroke_gradient(stroke_grad, 8.0));
    let squiggle = build_squiggle_path(Vec2::new(520.0, 380.0), Vec2::new(760.0, 500.0));
    canvas.draw_path(&squiggle);

    // ── 5. SDF text rendering ──
    // Build an SDF from the word "SOROT"
    let text_path = build_text_path(Vec2::new(80.0, 460.0), 60.0);
    let flat = flatten_path(&text_path, 0.5);
    let (sdf_pixels, sdf_rect, sdf_w, sdf_h) = compute_sdf(&flat, 128, 0.12);

    let dl = canvas.finalize();
    let graph = canvas.graph();

    let mut pipeline = Pipeline::new();
    let mut frame = pipeline.build_frame(graph, &dl, 800, 600);

    frame.sdf_ops.push(SdfOp {
        sdf_data: sdf_pixels.into_boxed_slice(),
        sdf_width: sdf_w,
        sdf_height: sdf_h,
        bounds: sdf_rect,
        paint: Paint::fill(Color::from_rgba(1.0, 0.7, 0.1, 1.0)),
    });

    frame
}

/// Build a star path centered at `center` with inner/outer radius and `points` points.
fn build_star_path(center: Vec2, inner_r: f32, outer_r: f32, points: u32) -> Path {
    let mut path = Path::new();
    for i in 0..points * 2 {
        let angle = std::f32::consts::PI * -0.5 + std::f32::consts::PI * i as f32 / points as f32;
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let p = Vec2::new(center.x + angle.cos() * r, center.y + angle.sin() * r);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close();
    path
}

/// Build an approximate spiral path.
fn build_spiral_path(center: Vec2, max_r: f32, turns: f32) -> Path {
    let mut path = Path::new();
    let steps = (turns * 24.0) as u32;
    let mut first = true;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = t * turns * 2.0 * std::f32::consts::PI;
        let r = t * max_r;
        let p = Vec2::new(center.x + angle.cos() * r, center.y + angle.sin() * r);
        if first {
            path.move_to(p);
            first = false;
        } else {
            path.line_to(p);
        }
    }
    path
}

/// Build a squiggle (bezier curve) path.
fn build_squiggle_path(p0: Vec2, p1: Vec2) -> Path {
    let mut path = Path::new();
    path.move_to(p0);
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    path.cubic_to(
        Vec2::new(p0.x + dx * 0.25, p0.y - dy * 0.5),
        Vec2::new(p0.x + dx * 0.75, p1.y + dy * 0.5),
        p1,
    );
    path
}

/// Build a simple "SOROT" text outline using path operations (simulated glyph shapes).
fn build_text_path(origin: Vec2, size: f32) -> Path {
    let s = size * 0.6;
    let x = origin.x;
    let y = origin.y;
    let mut path = Path::new();

    // 'S'
    let cx = x;
    path.move_to(Vec2::new(cx + s, y));
    path.line_to(Vec2::new(cx, y));
    path.cubic_to(Vec2::new(cx - s * 0.3, y), Vec2::new(cx - s * 0.3, y + s * 0.5), Vec2::new(cx, y + s * 0.5));
    path.cubic_to(Vec2::new(cx + s * 0.3, y + s * 0.5), Vec2::new(cx + s * 0.3, y + s), Vec2::new(cx + s, y + s));

    // Move to 'O'
    let cx2 = x + s * 1.6;
    path.move_to(Vec2::new(cx2, y));
    path.cubic_to(
        Vec2::new(cx2 - s * 0.4, y),
        Vec2::new(cx2 - s * 0.5, y + s),
        Vec2::new(cx2, y + s),
    );
    path.cubic_to(
        Vec2::new(cx2 + s * 0.4, y + s),
        Vec2::new(cx2 + s * 0.5, y),
        Vec2::new(cx2, y),
    );

    // 'R'
    let cx3 = x + s * 3.0;
    path.move_to(Vec2::new(cx3, y));
    path.line_to(Vec2::new(cx3, y + s));
    path.move_to(Vec2::new(cx3, y));
    path.line_to(Vec2::new(cx3 + s * 0.6, y));
    path.line_to(Vec2::new(cx3 + s * 0.3, y + s * 0.5));
    path.line_to(Vec2::new(cx3 + s * 0.7, y + s));

    // 'O'
    let cx4 = x + s * 4.4;
    path.move_to(Vec2::new(cx4, y));
    path.cubic_to(
        Vec2::new(cx4 - s * 0.4, y),
        Vec2::new(cx4 - s * 0.5, y + s),
        Vec2::new(cx4, y + s),
    );
    path.cubic_to(
        Vec2::new(cx4 + s * 0.4, y + s),
        Vec2::new(cx4 + s * 0.5, y),
        Vec2::new(cx4, y),
    );

    // 'T'
    let cx5 = x + s * 5.8;
    path.move_to(Vec2::new(cx5 - s * 0.5, y));
    path.line_to(Vec2::new(cx5 + s * 0.5, y));
    path.move_to(Vec2::new(cx5, y));
    path.line_to(Vec2::new(cx5, y + s));

    path
}
