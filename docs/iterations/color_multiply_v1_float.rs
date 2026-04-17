//! # Iteration 1: `Vec4` float multiply with division by 255
//!
//! **Context:** The natural first approach to blending a face colour with a
//! texture colour: normalise both to `[0.0, 1.0]`, multiply component-wise,
//! then convert back to `u8`. `Vec4` from `glam` made this a single multiply
//! expression with SIMD potential.
//!
//! **Measurement:** Despite the apparent SIMD opportunity, profiling showed
//! this on the hot path. The `/ 255.0` (or pre-dividing the face colour) and
//! the four `to_int_unchecked()` casts per pixel added up at 800k+ pixels/frame.
//! The `Vec4` construct/deconstruct cycle also prevented scalar optimisation
//! in some compiler versions.
//!
//! **Problem:** Float arithmetic, float-to-integer conversion, and unnecessary
//! `Vec4` boxing for a purely per-channel operation. The range is always
//! `u8 × u8 → u8`, which is a fixed-point problem.
//!
//! **Next step:** Iteration 2 — integer multiply with a right shift:
//! `(a as u16 * b as u16) >> 8` stays in integer registers and avoids all
//! float conversion.
//!
//! **Commit:** before `29108d1` (visible as commented-out code in the current
//! `src/render.rs` inside `draw_3dtriangle_to_color_buffer`)

use glam::Vec4;

/// Multiplies two ARGB colours represented as `Vec4` components in `[0, 255]`.
///
/// Each channel: `(texture_channel / 255.0) * face_channel`.
/// Returns the result as `[a, r, g, b]` floats in `[0.0, 255.0]`, which the
/// caller converts to `u8` with `to_int_unchecked`.
///
/// Called inside the per-pixel rasterizer loop.
#[inline]
fn color_multiply_v1_float(texture_color: Vec4, face_color: Vec4) -> Vec4 {
    // face_color channels are already divided by 255 at the scanline level:
    //   let a = face.a as f32 / 255.0;
    //   let r = face.r as f32 / 255.0;
    //   ...
    //   let face_color = Vec4::new(a, r, g, b);
    texture_color * face_color
}

// --- Call site (inside the per-pixel rasterizer loop) ---
//
//  let triangle_color = Vec4::new(
//      triangle3d.color.a as f32,
//      triangle3d.color.r as f32,
//      triangle3d.color.g as f32,
//      triangle3d.color.b as f32,
//  ) / 255.0;
//
//  // ... per pixel ...
//  let [tr, tg, tb, ta] = get_texture_color_rgba(texture, u, v, t_width, t_height);
//  let texture_color = Vec4::new(ta as f32, tr as f32, tg as f32, tb as f32);
//  let color: Vec4 = texture_color * triangle_color;
//  let [a, r, g, b] = color.to_array();
//  let color: u32;
//  unsafe {
//      color = u32::from_be_bytes([
//          a.to_int_unchecked(),
//          r.to_int_unchecked(),
//          g.to_int_unchecked(),
//          b.to_int_unchecked(),
//      ]);
//      *color_buffer_u32.get_unchecked_mut(z_index) = color;
//  }
