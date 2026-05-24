#![no_main]

use libfuzzer_sys::fuzz_target;
use vector::{Matrix, Vec2};

fuzz_target!(|data: ([f32; 6], [f32; 2])| {
    let ([a, c, e, b, d, f], [x, y]) = data;
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() || !e.is_finite() || !f.is_finite() { return; }
    if !x.is_finite() || !y.is_finite() { return; }
    if a.abs() > 1e6 || b.abs() > 1e6 || c.abs() > 1e6 || d.abs() > 1e6 || e.abs() > 1e6 || f.abs() > 1e6 { return; }

    let m = Matrix {
        a, c, e, b, d, f,
        p0: 0.0, p1: 0.0, p2: 1.0,
        mask: Default::default(),
    };
    let m = Matrix { mask: matrix::Matrix::TypeMask::compute(a, c, e, b, d, f, 0.0, 0.0, 1.0), ..m };
    let p = Vec2::new(x, y);

    let r1 = m.map_point(p);
    let r2 = m.transform_point(p);
    assert!((r1.x - r2.x).abs() < 1e-5, "map_point != transform_point");

    let v = m.map_vector(p);
    let m2 = Matrix { e: 0.0, f: 0.0, ..m };
    let v2 = m2.map_point(p);
    assert!((v.x - v2.x).abs() < 1e-5, "map_vector != map_point(zero_translate)");

    if m.is_affine() {
        if let Some(inv) = m.inverse() {
            let rt = inv.map_point(r1);
            if rt.x.is_finite() && rt.y.is_finite() {
                assert!((rt.x - x).abs() < 0.5, "inverse roundtrip {x},{y} -> {},{rt}", rt.x, rt.y);
            }
        }
    }
});
