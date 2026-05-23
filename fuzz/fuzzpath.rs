#![no_main]

use libfuzzer_sys::fuzz_target;
use srcore::math::Vec2;
use srcore::path::{flatten_path, Path};

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 { return; }
    let cx = f32::from_le_bytes([data[0], data[1], data[2], data[3]]).clamp(-500.0, 500.0);
    let cy = f32::from_le_bytes([data[4], data[5], data[6], data[7]]).clamp(-500.0, 500.0);
    let r = (cx.abs() + cy.abs()).clamp(1.0, 400.0);

    let mut path = Path::new();
    let k = 0.5522847498 * r;
    path.move_to(Vec2::new(cx + r, cy));
    path.cubic_to(Vec2::new(cx + r, cy + k), Vec2::new(cx + k, cy + r), Vec2::new(cx, cy + r));
    path.cubic_to(Vec2::new(cx - k, cy + r), Vec2::new(cx - r, cy + k), Vec2::new(cx - r, cy));
    path.cubic_to(Vec2::new(cx - r, cy - k), Vec2::new(cx - k, cy - r), Vec2::new(cx, cy - r));
    path.cubic_to(Vec2::new(cx + k, cy - r), Vec2::new(cx + r, cy - k), Vec2::new(cx + r, cy));
    path.close();

    let flat = flatten_path(&path, 0.5);
    assert!(!flat.points.is_empty(), "flattened empty");

    for p in &flat.points {
        assert!(p.x.is_finite() && p.y.is_finite(), "non-finite point");
    }
});
