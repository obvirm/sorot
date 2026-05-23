use srcore::color::Color;
use srcore::math::{Rect, Vec2};
use srcore::paint::Paint;
use srcore::path::{flatten_path, Path};
use srraster::sdf::compute_sdf;
use srrender::render_ir::{RenderFrame, SdfOp};
use srscene::canvas::Canvas;
use srscene::pipeline::Pipeline;

pub fn build_frame() -> RenderFrame {
    let mut canvas = Canvas::new();

    canvas.set_paint(Paint::fill(Color::from_rgba(0.2, 0.6, 0.9, 0.9)));
    canvas.draw_oval(Vec2::new(200.0, 300.0), 140.0, 140.0);

    canvas.set_paint(Paint::fill(Color::from_rgba(0.9, 0.3, 0.2, 0.7)));
    canvas.translate(100.0, -50.0);
    canvas.draw_rect(Rect::new(Vec2::new(280.0, 120.0), Vec2::new(480.0, 450.0)));
    canvas.translate(-100.0, 50.0);

    canvas.set_paint(Paint::fill(Color::from_rgba(0.9, 0.7, 0.1, 0.9)));
    let rr = Path::rounded_rect(Vec2::new(520.0, 350.0), Vec2::new(750.0, 500.0), 20.0, 20.0);
    canvas.draw_path(&rr);

    let dl = canvas.finalize();
    let graph = canvas.graph();

    let mut pipeline = Pipeline::new();
    let mut frame = pipeline.build_frame(graph, &dl, 800, 600);

    let circle = Path::circle(Vec2::new(200.0, 300.0), 140.0);
    let flat = flatten_path(&circle, 0.5);
    let (sdf_pixels, sdf_rect, sdf_w, sdf_h) = compute_sdf(&flat, 128, 0.15);

    frame.sdf_ops.push(SdfOp {
        sdf_data: sdf_pixels.into_boxed_slice(),
        sdf_width: sdf_w,
        sdf_height: sdf_h,
        bounds: sdf_rect,
        paint: Paint::fill(Color::from_rgba(0.2, 0.6, 0.9, 0.95)),
    });

    frame
}
