//! # Iteration 3: Zero-allocation lazy iterator with `std::iter::from_fn`
//!
//! **Context:** The `Vec`-returning versions (v1, v2) always allocated and then
//! iterated exactly once before dropping the allocation. `std::iter::from_fn`
//! lets us produce the same sequence lazily, with state captured in a closure,
//! eliminating the heap allocation entirely.
//!
//! The final version also bundles four `f32` values (x, 1/w, u/w, v/w) into a
//! `Vec4` so that a single interpolation call drives the full perspective-
//! correct rasterizer rather than separate passes.
//!
//! **Measurement:** Allocation profiling showed the per-edge `Vec` allocs were
//! gone. Frame time improved noticeably. The iterator version also chains
//! cleanly with `.map()`, `.zip()`, and `.skip()` at zero cost.
//!
//! **Problem:** None — this is the final version.
//!
//! **Next step:** See `texture_v4_pow2_unchecked.rs` and
//! `color_multiply_v2_int_shift.rs` for the remaining hot-path optimisations.
//!
//! **Commit:** 8599a87 (initial iterator attempt); a87689c (micro-optimisation
//! — replace `step == 1.0 && ...` with a `forward` bool precomputed before
//! the closure)

use glam::Vec4;

/// Interpolates a `Vec4` displacement `d` over the float interval `[i0, i1]`.
///
/// Returns an iterator yielding one `(i, d)` pair per unit step (both
/// endpoints included). The step direction is inferred from `i1 > i0`.
///
/// Zero heap allocation — state is held in the closure captured by `from_fn`.
#[inline]
fn map_interpolate_float_vec4_iter(
    i0: f32,
    d0: Vec4,
    i1: f32,
    d1: Vec4,
) -> impl Iterator<Item = (f32, Vec4)> {
    // Precompute the direction once so the closure body avoids a float compare
    // on every iteration (commit a87689c).
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
        // Return None *before* advancing so the endpoint is included.
        if (forward && i > i1) || (!forward && i < i1) {
            None
        } else {
            d += a;
            i += step;
            result
        }
    })
}

// --- Call site (inside the 3D triangle rasterizer) ---
//
//  let corner_segment1_iter = map_interpolate_float_vec4_iter(
//      v0.y,
//      Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
//      v1.y,
//      Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w),
//  )
//  .map(|(y, xwuv)| TriangleScreenPixel { x: xwuv.x.round(), y, ... });
