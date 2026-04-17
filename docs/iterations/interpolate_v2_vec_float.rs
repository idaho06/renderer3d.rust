//! # Iteration 2: Float interpolation returning `Vec<(f32, f32)>`
//!
//! **Context:** To carry floating-point UV coordinates and reciprocal-W through
//! the scanline interpolation, the integer version was rewritten with `f32`.
//!
//! **Measurement:** The allocation pattern is identical to iteration 1 — one
//! `Vec` heap allocation per edge per frame. Profile showed no regression vs
//! v1, but also no improvement in allocation pressure.
//!
//! **Problem:** Still allocates. The `Vec` lifetime is always a single frame;
//! it is pushed, iterated once, then dropped. The memory never escapes the
//! function call stack. This is the ideal shape for a zero-allocation lazy
//! iterator.
//!
//! **Next step:** Iteration 3 — replace the `Vec` with `std::iter::from_fn`,
//! producing values lazily with zero heap allocation.
//!
//! **Commit:** commented out in `src/render.rs` at lines 285–315 (visible in
//! the final source as a preserved comparison)

use glam::Vec4;

/// Interpolates a float displacement `d` over the float interval `[i0, i1]`.
///
/// Returns one `(i, d)` pair per unit step. Both endpoints are included.
/// The step direction is inferred from the sign of `i1 - i0`.
///
/// Allocates a `Vec` on every call.
#[allow(dead_code)]
#[inline(always)]
fn map_interpolate_float(i0: f32, d0: f32, i1: f32, d1: f32) -> Vec<(f32, f32)> {
    if i0 == i1 {
        return vec![(i0, d0)];
    }
    let distance = (i1 - i0).abs();
    let mut values: Vec<(f32, f32)> = Vec::with_capacity(distance as usize);
    let mut a: f32 = (d1 - d0) / (i1 - i0);
    let step: f32 = if i1 > i0 { 1.0 } else { -1.0 };
    if step == -1.0 {
        a = -a;
    }
    let mut i = i0;
    let mut d = d0;
    loop {
        values.push((i, d));
        d += a;
        i += step;
        if step == 1.0 && i > i1 {
            break;
        }
        if step == -1.0 && i < i1 {
            break;
        }
    }
    values
}

// --- Call site (inside the triangle rasterizer) ---
//
//  let straight_segment = map_interpolate_float(v0.y, v0.x, v2.y, v2.x);
//  // ...then iterate and zip
