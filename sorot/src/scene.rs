use sorot_core::color::Color;
use sorot_core::math::Vec2;
use sorot_path::{flatten_path, Path};
use sorot_raster::TriMesh;

pub fn demo_scene() -> Vec<(TriMesh, Color)> {
    let circle = Path::circle(Vec2::new(220.0, 280.0), 120.0);
    let flat = flatten_path(&circle, 0.5);
    let mesh = sorot_raster::triangulate(&flat);

    let triangle = {
        let mut p = Path::new();
        p.move_to(Vec2::new(480.0, 120.0));
        p.line_to(Vec2::new(680.0, 450.0));
        p.line_to(Vec2::new(280.0, 450.0));
        p.close();
        p
    };
    let flat_tri = flatten_path(&triangle, 0.5);
    let mesh_tri = sorot_raster::triangulate(&flat_tri);

    let rect = Path::rounded_rect(Vec2::new(520.0, 80.0), Vec2::new(750.0, 220.0), 20.0, 20.0);
    let flat_rect = flatten_path(&rect, 0.5);
    let mesh_rect = sorot_raster::triangulate(&flat_rect);

    vec![
        (mesh, Color::from_rgba(0.2, 0.6, 0.9, 0.9)),
        (mesh_tri, Color::from_rgba(0.9, 0.3, 0.2, 0.7)),
        (mesh_rect, Color::from_rgba(0.9, 0.7, 0.1, 0.9)),
    ]
}
