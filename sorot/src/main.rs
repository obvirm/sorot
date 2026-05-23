use sorot_core::color::Color;
use sorot_path::{flatten_path, Path};
use sorot_raster::scanline::rasterize_path;

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;
    let mut buffer = vec![0u8; (width * height * 4) as usize];

    let bg = Color::from_rgba(0.05, 0.05, 0.08, 1.0);
    for chunk in buffer.chunks_exact_mut(4) {
        chunk[0] = (bg.r * 255.0) as u8;
        chunk[1] = (bg.g * 255.0) as u8;
        chunk[2] = (bg.b * 255.0) as u8;
        chunk[3] = (bg.a * 255.0) as u8;
    }

    let circle = Path::circle(
        sorot_core::math::Vec2::new(200.0, 300.0),
        120.0,
    );
    let flat_circle = flatten_path(&circle, 0.5);
    rasterize_path(
        &flat_circle,
        Color::from_rgba(0.9, 0.2, 0.3, 0.8),
        &mut buffer,
        width,
        height,
        true,
    );

    let triangle = {
        let mut p = Path::new();
        p.move_to(sorot_core::math::Vec2::new(400.0, 120.0));
        p.line_to(sorot_core::math::Vec2::new(550.0, 420.0));
        p.line_to(sorot_core::math::Vec2::new(250.0, 420.0));
        p.close();
        p
    };
    let flat_tri = flatten_path(&triangle, 0.5);
    rasterize_path(
        &flat_tri,
        Color::from_rgba(0.2, 0.7, 0.9, 0.7),
        &mut buffer,
        width,
        height,
        true,
    );

    let rect = Path::rounded_rect(
        sorot_core::math::Vec2::new(520.0, 80.0),
        sorot_core::math::Vec2::new(750.0, 200.0),
        20.0,
        20.0,
    );
    let flat_rect = flatten_path(&rect, 0.5);
    rasterize_path(
        &flat_rect,
        Color::from_rgba(0.9, 0.7, 0.1, 0.9),
        &mut buffer,
        width,
        height,
        true,
    );

    let oval = Path::oval(
        sorot_core::math::Vec2::new(600.0, 400.0),
        100.0,
        60.0,
    );
    let flat_oval = flatten_path(&oval, 0.5);
    rasterize_path(
        &flat_oval,
        Color::from_rgba(0.4, 0.9, 0.3, 0.6),
        &mut buffer,
        width,
        height,
        true,
    );

    image::save_buffer(
        "output.png",
        &buffer,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .expect("failed to save output.png");

    println!("saved output.png ({}x{})", width, height);
}
