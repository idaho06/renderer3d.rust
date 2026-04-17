//! # Iteration 2: Integer multiply with right shift — `(a as u16 * b as u16) >> 8`
//!
//! **Context:** Color channel values are `u8` in `[0, 255]`. The "multiply two
//! colours" operation is conceptually `(a / 255.0) * (b / 255.0) * 255.0`,
//! which simplifies to `a * b / 255`. In fixed-point, this is done entirely in
//! integer registers.
//!
//! **Measurement:** Replacing the `Vec4` float path with four `u16` multiplies
//! and right shifts (one per channel) dropped the color-multiply cost
//! significantly. The `>> 8` is a slight approximation (`/ 256` vs `/ 255`),
//! but the one-LSB difference is visually indistinguishable.
//!
//! **Problem:** None — this is the final version.
//!
//! **Next step:** The texture sampler bottleneck was already addressed in
//! `texture_v4_pow2_unchecked.rs`. With both hot-path items fixed, the
//! rasterizer dropped off the top of the flame graph.
//!
//! **Commit:** 29108d1

/// Multiplies two colour channel values using integer arithmetic.
///
/// Equivalent to `(a as f32 * b as f32 / 255.0) as u8` but faster:
/// promotes to `u16`, multiplies, then right-shifts by 8 (≈ divide by 256).
///
/// The `>> 8` approximation introduces at most a 1-LSB error, which is
/// visually indistinguishable for color modulation.
#[inline(always)]
fn mul_u8_channel(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) >> 8) as u8
}

// --- Call site (inside the per-pixel rasterizer loop) ---
//
//  // texture_color_array_u8: [a, r, g, b] from get_texture_color_argb_pow2_unchecked
//  // triangle_color_array_u8: [a, r, g, b] from the face colour
//
//  let a = mul_u8_channel(texture_color_array_u8[0], triangle_color_array_u8[0]);
//  let r = mul_u8_channel(texture_color_array_u8[1], triangle_color_array_u8[1]);
//  let g = mul_u8_channel(texture_color_array_u8[2], triangle_color_array_u8[2]);
//  let b = mul_u8_channel(texture_color_array_u8[3], triangle_color_array_u8[3]);
//  unsafe {
//      let color = u32::from_be_bytes([a, r, g, b]); // ARGB8888
//      *color_buffer_u32.get_unchecked_mut(z_index) = color;
//  }
