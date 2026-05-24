#![no_main]

use libfuzzer_sys::fuzz_target;
use matrix::Matrix;
use vector::Vec2;

fuzz_target!(|data: ([f32; 6], [f32; 2])| {
    let ([a, c, e, b, d, f], [x, y]) = data;
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() || !e.is_finite() || !f.is_finite() { return; }
    if !x.is_finite() || !y.is_finite() { return; }
    if a.abs() > 1e6 || b.abs() > 1e6 || c.abs() > 1e6 || d.abs() > 1e6 || e.abs() > 1e6 || f.abs() > 1e6 { return; }

    let m = matrix::Matrix::from_raw(a, c, e, b, d, f);
    let p = Vec2::new(x, y);

    let r1 = m.map_point(p);
    let r2 = m.transform_point(p);
    assert!((r1.x - r2.x).abs() < 1e-5, "map_point != transform_point");

    let v = m.map_vector(p);
    // Use from_raw with zero translation to test vector mapping
    let m2 = Matrix::from_raw(a, c, 0.0, b, d, 0.0);
    let v2 = m2.map_point(p);
    assert!((v.x - v2.x).abs() < 1e-5, "map_vector != map_point(zero_translate)");

    if m.is_affine() {
        if let Some(inv) = m.inverse() {
            let rt = inv.map_point(r1);
            if rt.x.is_finite() && rt.y.is_finite() {
                assert!((rt.x - x).abs() < 0.5, "inverse roundtrip {:?} -> {:?}", (x, y), (rt.x, rt.y));
            }
        }
    }
});
