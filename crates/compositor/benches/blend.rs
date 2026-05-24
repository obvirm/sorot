use criterion::{black_box, criterion_group, criterion_main, Criterion};
use blend::BlendMode;
use srsimd;
use color::Color;

fn bench_blend_src_over(c: &mut Criterion) {
    let color = Color::from_rgba(1.0, 0.0, 0.0, 0.5);
    let rgba = color.to_premultiplied_u8();

    c.bench_function("blend_src_over_scalar", |b| {
        let mut dst = vec![0u8; 4096];
        let src = vec![rgba[0], rgba[1], rgba[2], rgba[3]].repeat(1024);
        b.iter(|| {
            let s = black_box(&src);
            let d = black_box(&mut dst);
            for i in (0..d.len()).step_by(4) {
                let sa = s[i + 3] as f32 / 255.0;
                let inv = 1.0 - sa;
                d[i] = (s[i] as f32 + d[i] as f32 * inv).min(255.0) as u8;
                d[i + 1] = (s[i + 1] as f32 + d[i + 1] as f32 * inv).min(255.0) as u8;
                d[i + 2] = (s[i + 2] as f32 + d[i + 2] as f32 * inv).min(255.0) as u8;
                d[i + 3] = ((sa * 255.0) + d[i + 3] as f32 * inv).min(255.0) as u8;
            }
        })
    });

    c.bench_function("blend_src_over_simd", |b| {
        let mut dst = vec![0u8; 4096];
        let src = vec![rgba[0], rgba[1], rgba[2], rgba[3]].repeat(1024);
        b.iter(|| {
            let s = black_box(&src);
            let d = black_box(&mut dst);
            simd::blend_src_over(d, s);
        })
    });
}

fn bench_blend_modes(c: &mut Criterion) {
    let modes = [
        BlendMode::SrcOver,
        BlendMode::Multiply,
        BlendMode::Screen,
    ];

    for mode in &modes {
        let name = format!("blend_mode_{:?}", mode);
        c.bench_function(&name, |b| {
            let src = Color::from_rgba(0.8, 0.4, 0.2, 0.7);
            let dst = Color::from_rgba(0.1, 0.3, 0.9, 0.8);
            b.iter(|| {
                let s = black_box(src);
                let d = black_box(dst);
                black_box(mode.blend(s, d));
            })
        });
    }
}

criterion_group!(benches, bench_blend_src_over, bench_blend_modes);
criterion_main!(benches);
