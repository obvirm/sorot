#![no_main]

use libfuzzer_sys::fuzz_target;
use srcolor::Color;
use srblend::BlendMode;

fuzz_target!(|data: [u8; 16]| {
    let r1 = data[0] as f32 / 255.0;
    let g1 = data[1] as f32 / 255.0;
    let b1 = data[2] as f32 / 255.0;
    let a1 = data[3] as f32 / 255.0;
    let r2 = data[4] as f32 / 255.0;
    let g2 = data[5] as f32 / 255.0;
    let b2 = data[6] as f32 / 255.0;
    let a2 = data[7] as f32 / 255.0;
    let mode = data[8] % 14;

    let c1 = Color::from_rgba(r1, g1, b1, a1);
    let c2 = Color::from_rgba(r2, g2, b2, a2);

    let modes = [
        BlendMode::SrcOver, BlendMode::DstOver, BlendMode::SrcIn, BlendMode::DstIn,
        BlendMode::SrcOut, BlendMode::DstOut, BlendMode::SrcAtop, BlendMode::DstAtop,
        BlendMode::Xor, BlendMode::Plus, BlendMode::Multiply, BlendMode::Screen,
        BlendMode::Darken, BlendMode::Lighten,
    ];
    let result = modes[mode as usize].blend(c1, c2);

    assert!(result.r >= 0.0 && result.r <= 1.1);
    assert!(result.g >= 0.0 && result.g <= 1.1);
    assert!(result.b >= 0.0 && result.b <= 1.1);
    assert!(result.a >= 0.0 && result.a <= 1.1);
});
