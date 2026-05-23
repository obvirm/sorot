use sorot_core::color::Color;
use sorot_core::math::{Rect, Vec2};
use sorot_gpu::sdf::compute_sdf;
use sorot_path::{flatten_path, Path};
use sorot_raster::TriMesh;

pub struct DemoScene {
    pub meshes: Vec<(TriMesh, Color)>,
    pub sdf: Option<(Vec<u8>, u32, u32, Rect, Color)>,
}

pub fn demo_scene() -> DemoScene {
    let meshes = vec![
        {
            let tri = make_triangle();
            let flat = flatten_path(&tri, 0.5);
            let mesh = sorot_raster::triangulate(&flat);
            (mesh, Color::from_rgba(0.9, 0.3, 0.2, 0.7))
        },
        {
            let rect = Path::rounded_rect(
                Vec2::new(520.0, 350.0),
                Vec2::new(750.0, 500.0),
                20.0,
                20.0,
            );
            let flat = flatten_path(&rect, 0.5);
            let mesh = sorot_raster::triangulate(&flat);
            (mesh, Color::from_rgba(0.9, 0.7, 0.1, 0.9))
        },
    ];

    let circle_path = Path::circle(Vec2::new(220.0, 300.0), 140.0);
    let flat_circle = flatten_path(&circle_path, 0.5);
    let (sdf_pixels, sdf_rect, sdf_w, sdf_h) = compute_sdf(&flat_circle, 128, 0.15);
    let sdf = Some((sdf_pixels, sdf_w, sdf_h, sdf_rect, Color::from_rgba(0.2, 0.6, 0.9, 0.95)));

    DemoScene { meshes, sdf }
}

fn make_triangle() -> Path {
    let mut p = Path::new();
    p.move_to(Vec2::new(480.0, 120.0));
    p.line_to(Vec2::new(680.0, 450.0));
    p.line_to(Vec2::new(280.0, 450.0));
    p.close();
    p
}
