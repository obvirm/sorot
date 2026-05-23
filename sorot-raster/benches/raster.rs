use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sorot_core::math::Vec2;
use sorot_path::{flatten_path, FlattenVerb, FlattenedPath, Path};
use sorot_raster::{scanline, triangulate, TriMesh};
use sorot_core::color::Color;

fn bench_flatten_circle(c: &mut Criterion) {
    let path = Path::circle(Vec2::new(200.0, 200.0), 150.0);

    c.bench_function("flatten_circle_tol_05", |b| {
        b.iter(|| {
            let p = black_box(&path);
            black_box(flatten_path(p, 0.5));
        })
    });

    c.bench_function("flatten_circle_tol_10", |b| {
        b.iter(|| {
            let p = black_box(&path);
            black_box(flatten_path(p, 1.0));
        })
    });
}

fn bench_triangulate(c: &mut Criterion) {
    let path = Path::circle(Vec2::new(200.0, 200.0), 150.0);
    let flat = flatten_path(&path, 0.5);

    c.bench_function("triangulate_circle", |b| {
        b.iter(|| {
            let f = black_box(&flat);
            black_box(triangulate(f));
        })
    });

    let tri = make_triangle();
    c.bench_function("triangulate_triangle", |b| {
        b.iter(|| {
            let f = black_box(&tri);
            black_box(triangulate(f));
        })
    });
}

fn bench_scanline(c: &mut Criterion) {
    let tri = make_triangle();
    let mut buf = vec![0u8; 400 * 400 * 4];
    let color = Color::from_rgba(1.0, 0.0, 0.0, 1.0);

    c.bench_function("scanline_triangle", |b| {
        b.iter(|| {
            let t = black_box(&tri);
            let b = black_box(&mut buf);
            let c = black_box(color);
            scanline::rasterize_path(t, c, b, 400, 400, true);
        })
    });
}

fn make_triangle() -> FlattenedPath {
    let mut path = FlattenedPath::new();
    path.verbs.push(FlattenVerb::MoveTo);
    path.points.push(Vec2::new(200.0, 50.0));
    path.verbs.push(FlattenVerb::LineTo);
    path.points.push(Vec2::new(350.0, 350.0));
    path.verbs.push(FlattenVerb::LineTo);
    path.points.push(Vec2::new(50.0, 350.0));
    path.verbs.push(FlattenVerb::Close);
    path
}

criterion_group!(benches, bench_flatten_circle, bench_triangulate, bench_scanline);
criterion_main!(benches);
