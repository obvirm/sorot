use sorot_core::color::Color;
use sorot_core::math::{Rect, Vec2};
use sorot_core::paint::Paint;
use sorot_gpu::sdf::compute_sdf;
use sorot_path::{flatten_path, Path};
use sorot_raster::TriMesh;
use sorot_scene::canvas::Canvas;
use sorot_scene::display_list::{DisplayList, DrawCommand};

pub fn build_demo() -> (Vec<(TriMesh, Color)>, Option<(Vec<u8>, u32, u32, Rect, Color)>) {
    let mut canvas = Canvas::new();

    canvas.set_paint(Paint::fill(Color::from_rgba(0.2, 0.6, 0.9, 0.9)));
    canvas.draw_oval(Vec2::new(200.0, 300.0), 140.0, 140.0);

    canvas.set_paint(Paint::fill(Color::from_rgba(0.9, 0.3, 0.2, 0.7)));
    canvas.translate(100.0, -50.0);
    canvas.draw_rect(Rect::new(Vec2::new(280.0, 120.0), Vec2::new(480.0, 450.0)));
    canvas.translate(-100.0, 50.0);

    canvas.set_paint(Paint::fill(Color::from_rgba(0.9, 0.7, 0.1, 0.9)));
    let rr = Path::rounded_rect(
        Vec2::new(520.0, 350.0),
        Vec2::new(750.0, 500.0),
        20.0,
        20.0,
    );
    canvas.draw_path(&rr);

    let dl = canvas.finalize();
    process_display_list(&dl)
}

fn process_display_list(
    dl: &DisplayList,
) -> (Vec<(TriMesh, Color)>, Option<(Vec<u8>, u32, u32, Rect, Color)>) {
    let mut meshes: Vec<(TriMesh, Color)> = Vec::new();
    let mut sdf: Option<(Vec<u8>, u32, u32, Rect, Color)> = None;

    for cmd in &dl.commands {
        match cmd {
            DrawCommand::Rect(draw_rect) => {
                let path = Path::rect(draw_rect.rect.min, draw_rect.rect.max);
                let flat = flatten_path(&path, 0.5);
                let mesh = sorot_raster::triangulate(&flat);
                meshes.push((mesh, draw_rect.paint.color));
            }
            DrawCommand::Oval(draw_oval) => {
                let path = Path::oval(draw_oval.center, draw_oval.rx, draw_oval.ry);
                let flat = flatten_path(&path, 0.5);
                let (data, bounds, w, h) = compute_sdf(&flat, 128, 0.15);
                sdf = Some((data, w, h, bounds, draw_oval.paint.color));
            }
            DrawCommand::Path(draw_path) => {
                let path = Path::rect(
                    Vec2::new(520.0, 350.0),
                    Vec2::new(750.0, 500.0),
                );
                let flat = flatten_path(&path, 0.5);
                let mesh = sorot_raster::triangulate(&flat);
                meshes.push((mesh, draw_path.paint.color));
            }
        }
    }

    (meshes, sdf)
}
