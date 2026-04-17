//! # Iteration 4: Power-of-2 dimensions, bitwise AND wrap, `get_unchecked`
//!
//! **Context:** Texture dimensions are always powers of two (enforced at load
//! time with a `debug_assert!`). This unlocks two micro-optimisations:
//!
//! 1. `x % n` → `x & (n - 1)`: single AND instruction vs integer divide.
//! 2. `.get(index..)` → `get_unchecked(index + k)`: eliminates the bounds
//!    check; safe because `x & (n - 1)` can never exceed `n - 1`, so
//!    `index + 3 < texture.len()` is guaranteed by construction.
//!
//! **Measurement:** Together with the integer color multiply (see
//! `color_multiply_v2_int_shift.rs`), this iteration was the dominant win in
//! the profiling session. The texture sampler dropped off the flame graph.
//!
//! **Problem:** None — this is the final version.
//!
//! **Next step:** Color multiply still uses `Vec4` float arithmetic. See
//! `color_multiply_v2_int_shift.rs`.
//!
//! **Commit:** 63bb8a0 (bounds-check removal pass); finalised alongside
//! `29108d1` (integer color multiply)

/// Returns the texel colour at UV coordinates `(u, v)` as `[a, r, g, b]`
/// (ARGB byte order, matching the ARGB8888 framebuffer).
///
/// # Invariants
///
/// - `width` and `height` must be powers of two.
/// - `texture.len() == width as usize * height as usize * 4`.
///
/// These are checked with `debug_assert!` in debug builds; violated invariants
/// cause undefined behaviour in release builds.
#[inline]
fn get_texture_color_argb_pow2_unchecked(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    debug_assert!(width.is_power_of_two());
    debug_assert!(height.is_power_of_two());
    debug_assert_eq!(texture.len(), width as usize * height as usize * 4);

    // Bitwise AND wrap: equivalent to `% width` when width is a power of two,
    // but compiles to a single AND instruction instead of an integer divide.
    let u = ((u * width as f32) as u32) & (width - 1);
    let v = ((v * height as f32) as u32) & (height - 1);
    let index = ((v * width + u) * 4) as usize;

    // SAFETY: `u <= width - 1` and `v <= height - 1` by the AND above, so
    // `index + 3 == (v * width + u) * 4 + 3 <= (width * height - 1) * 4 + 3
    //             == texture.len() - 1`. Bounds cannot be exceeded.
    unsafe {
        [
            *texture.get_unchecked(index + 3), // a
            *texture.get_unchecked(index + 2), // r
            *texture.get_unchecked(index + 1), // g
            *texture.get_unchecked(index),     // b
        ]
    }
}
