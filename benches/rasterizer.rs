use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use glam::Vec4;

// ---------------------------------------------------------------------------
// Historical implementations — preserved here so the benchmarks can import
// them without adding dead code to the main library.
// ---------------------------------------------------------------------------
mod bench_archive {
    use glam::Vec4;

    // --- Texture sampling ---

    /// V1: builds an `sdl2::pixels::Color` struct per pixel.
    /// (Reconstructed from commit ff55eab — the SDL2 dep is not available in
    /// benchmarks, so this version uses the equivalent byte extraction logic
    /// with the same modulo wrap and assert.)
    #[inline]
    pub fn texture_v1_sdl2_color(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> u32 {
        let u = (u * width as f32) as u32 % width;
        let v = (v * height as f32) as u32 % height;
        let index = (v * width + u) as usize;
        assert!(index * 4 + 3 < texture.len());
        let b = texture[index * 4];
        let g = texture[index * 4 + 1];
        let r = texture[index * 4 + 2];
        let a = texture[index * 4 + 3];
        // Simulates the round-trip through sdl2::pixels::Color fields.
        u32::from_be_bytes([a, r, g, b])
    }

    /// V4: power-of-2 dimensions, bitwise AND wrap, `get_unchecked`. Final version.
    #[inline]
    pub fn texture_v4_pow2_unchecked(
        texture: &[u8],
        u: f32,
        v: f32,
        width: u32,
        height: u32,
    ) -> [u8; 4] {
        debug_assert!(width.is_power_of_two());
        debug_assert!(height.is_power_of_two());
        let u = ((u * width as f32) as u32) & (width - 1);
        let v = ((v * height as f32) as u32) & (height - 1);
        let index = ((v * width + u) * 4) as usize;
        // SAFETY: u <= width-1 and v <= height-1 by AND above.
        unsafe {
            [
                *texture.get_unchecked(index + 3),
                *texture.get_unchecked(index + 2),
                *texture.get_unchecked(index + 1),
                *texture.get_unchecked(index),
            ]
        }
    }

    // --- Scanline interpolation ---

    /// V1: returns `Vec<(i32, i32)>` — one heap allocation per edge.
    pub fn interpolate_v1_vec_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<(i32, i32)> {
        if i0 == i1 {
            return vec![(i0, d0)];
        }
        let distance = (i1 - i0).unsigned_abs();
        let mut values: Vec<(i32, i32)> = Vec::with_capacity(distance as usize);
        let mut a: f32 = (d1 as f32 - d0 as f32) / (i1 as f32 - i0 as f32);
        let step: i32 = if i1 > i0 { 1 } else { -1 };
        if step == -1 {
            a = -a;
        }
        let mut i = i0;
        let mut d = d0 as f32;
        loop {
            values.push((i, d as i32));
            d += a;
            i += step;
            if step == 1 && i > i1 {
                break;
            }
            if step == -1 && i < i1 {
                break;
            }
        }
        values
    }

    /// V3: `std::iter::from_fn` — zero allocation, Vec4 payload. Final version.
    pub fn interpolate_v3_iterator(
        i0: f32,
        d0: Vec4,
        i1: f32,
        d1: Vec4,
    ) -> impl Iterator<Item = (f32, Vec4)> {
        let forward = i1 > i0;
        let mut a = if i1 == i0 {
            Vec4::ZERO
        } else {
            (d1 - d0) / (i1 - i0)
        };
        if !forward {
            a = Vec4::ZERO - a;
        }
        let step = if forward { 1.0_f32 } else { -1.0_f32 };
        let mut i = i0;
        let mut d = d0;
        std::iter::from_fn(move || {
            let result = Some((i, d));
            if (forward && i > i1) || (!forward && i < i1) {
                None
            } else {
                d += a;
                i += step;
                result
            }
        })
    }

    // --- Color multiply ---

    /// V1: Vec4 float multiply.
    #[inline]
    pub fn color_multiply_v1_float(texture: Vec4, face: Vec4) -> Vec4 {
        texture * face
    }

    /// V2: integer multiply + right shift. Final version.
    #[inline]
    pub fn color_multiply_v2_int_shift(a: u8, b: u8) -> u8 {
        ((a as u16 * b as u16) >> 8) as u8
    }

    // --- Buffer clear ---

    /// V1: `iter_mut().for_each`.
    pub fn clear_v1_itermut(buf: &mut [u8]) {
        buf.iter_mut().for_each(|x| *x = 0_u8);
    }

    /// V2: `.fill(0)`. Final version.
    pub fn clear_v2_fill(buf: &mut [u8]) {
        buf.fill(0_u8);
    }

    /// V3: `copy_from_slice` from a pre-allocated zero buffer.
    pub fn clear_v3_copy_from_slice(buf: &mut [u8], zero: &[u8]) {
        buf.copy_from_slice(zero);
    }
}

// ---------------------------------------------------------------------------
// Benchmark: texture sampling
// ---------------------------------------------------------------------------
fn bench_texture_sampling(c: &mut Criterion) {
    const W: u32 = 512;
    const H: u32 = 512;
    let texture: Vec<u8> = (0..W * H * 4).map(|i| (i % 256) as u8).collect();

    let mut group = c.benchmark_group("texture_sampling");
    group.bench_function("v1_sdl2_color_modulo", |b| {
        b.iter(|| {
            bench_archive::texture_v1_sdl2_color(
                black_box(&texture),
                black_box(0.37),
                black_box(0.61),
                black_box(W),
                black_box(H),
            )
        })
    });
    group.bench_function("v4_pow2_unchecked", |b| {
        b.iter(|| {
            bench_archive::texture_v4_pow2_unchecked(
                black_box(&texture),
                black_box(0.37),
                black_box(0.61),
                black_box(W),
                black_box(H),
            )
        })
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: scanline interpolation
// ---------------------------------------------------------------------------
fn bench_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpolation");

    // Simulate a ~720px tall edge.
    group.bench_function("v1_vec_int", |b| {
        b.iter(|| {
            bench_archive::interpolate_v1_vec_int(
                black_box(0),
                black_box(100),
                black_box(720),
                black_box(900),
            )
            .len() // consume the Vec
        })
    });
    group.bench_function("v3_iterator", |b| {
        b.iter(|| {
            bench_archive::interpolate_v3_iterator(
                black_box(0.0),
                black_box(Vec4::new(100.0, 1.0, 0.0, 0.0)),
                black_box(720.0),
                black_box(Vec4::new(900.0, 0.5, 1.0, 1.0)),
            )
            .count() // consume the iterator
        })
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: color multiply (per-channel, called 4× per pixel)
// ---------------------------------------------------------------------------
fn bench_color_multiply(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_multiply");

    // Simulate multiplying a full scanline of pixels (1280 pixels × 4 channels).
    const PIXELS: usize = 1280;
    let tex: Vec<u8> = (0..PIXELS * 4).map(|i| (i % 256) as u8).collect();
    let face: Vec<u8> = vec![200u8; PIXELS * 4];
    let tex_vec4: Vec<Vec4> = (0..PIXELS)
        .map(|i| Vec4::new(tex[i * 4] as f32, tex[i * 4 + 1] as f32, tex[i * 4 + 2] as f32, tex[i * 4 + 3] as f32))
        .collect();
    let face_vec4: Vec<Vec4> = (0..PIXELS)
        .map(|_| Vec4::new(200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 255.0 / 255.0))
        .collect();

    group.bench_function("v1_float_vec4", |b| {
        b.iter(|| {
            tex_vec4
                .iter()
                .zip(face_vec4.iter())
                .map(|(&t, &f)| bench_archive::color_multiply_v1_float(black_box(t), black_box(f)))
                .fold(Vec4::ZERO, |acc, x| acc + x)
        })
    });
    group.bench_function("v2_int_shift", |b| {
        b.iter(|| {
            tex.iter()
                .zip(face.iter())
                .map(|(&t, &f)| bench_archive::color_multiply_v2_int_shift(black_box(t), black_box(f)))
                .fold(0u32, |acc, x| acc + x as u32)
        })
    });
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: buffer clear (1280×720 color buffer = ~3.5 MB)
// ---------------------------------------------------------------------------
fn bench_buffer_clear(c: &mut Criterion) {
    const SIZE: usize = 1280 * 720 * 4;
    let zero_buf: Vec<u8> = vec![0u8; SIZE];

    let mut group = c.benchmark_group("buffer_clear");

    group.bench_function("v1_iter_mut", |b| {
        let mut buf = vec![0xFFu8; SIZE];
        b.iter(|| bench_archive::clear_v1_itermut(black_box(&mut buf)))
    });
    group.bench_function("v2_fill", |b| {
        let mut buf = vec![0xFFu8; SIZE];
        b.iter(|| bench_archive::clear_v2_fill(black_box(&mut buf)))
    });
    group.bench_function("v3_copy_from_slice", |b| {
        let mut buf = vec![0xFFu8; SIZE];
        b.iter(|| bench_archive::clear_v3_copy_from_slice(black_box(&mut buf), black_box(&zero_buf)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_texture_sampling,
    bench_interpolation,
    bench_color_multiply,
    bench_buffer_clear
);
criterion_main!(benches);
