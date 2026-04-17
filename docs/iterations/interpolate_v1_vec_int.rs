//! # Iteration 1: Integer interpolation returning `Vec<(i32, i32)>`
//!
//! **Context:** First working scanline rasterizer. The interpolation helper was
//! written to walk along a triangle edge (the "interval" axis `i`) and produce
//! a displacement value `d` for each step — one entry per scanline row.
//!
//! **Measurement:** This version allocates a new `Vec` for every triangle edge
//! on every frame. With 800+ triangles and two edge pairs per triangle, that is
//! thousands of short-lived heap allocations per frame. The allocator showed up
//! prominently in profiling.
//!
//! **Problem:** Integer arithmetic limits the values to `x` coordinates only.
//! Texture UV coordinates and reciprocal-W (needed for perspective-correct
//! interpolation) are floating-point; this version cannot carry them.
//! It also allocates.
//!
//! **Next step:** Iteration 2 — switch to `f32` and still allocate `Vec`, as a
//! stepping stone before the zero-allocation iterator version.
//!
//! **Commit:** present in `src/render.rs` throughout the project history;
//! still used for 2D line drawing as of the final version.

use glam::Vec4;

/// Interpolates a displacement value `d` over the integer interval `[i0, i1]`.
///
/// Returns one `(i, d)` pair per integer step. Both endpoints are included.
/// The step direction is inferred from the sign of `i1 - i0`.
///
/// Allocates a `Vec` on every call.
#[inline]
fn map_interpolate_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<(i32, i32)> {
    if i0 == i1 {
        return vec![(i0, d0)];
    }
    let distance = (i1 - i0).unsigned_abs();
    let mut values: Vec<(i32, i32)> = Vec::with_capacity(distance.try_into().unwrap());
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

// --- Call site (inside the triangle rasterizer) ---
//
//  let straight_segment  = map_interpolate_int(y0, x0, y2, x2);
//  let corner_segments   = map_interpolate_int(y0, x0, y1, x1);
//  // ...then .iter().zip(corner_segments.iter()).for_each(...)
