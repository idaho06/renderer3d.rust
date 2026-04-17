//! # Iteration 2: Raw RGBA bytes, modulo wrap, bounds-checked slice
//!
//! **Context:** After profiling showed `sdl2::pixels::Color` construction was
//! expensive, the return type was changed to a plain `(u8, u8, u8, u8)` tuple
//! (later `[u8; 4]`).
//!
//! **Measurement:** Removing the SDL2 struct helped. The `assert!` hot-path
//! panics were replaced by a safe `.get()` call that returns a magenta fallback
//! for out-of-bounds accesses. Bounds check still present on every call.
//!
//! **Problem:** UV wrap is still `u % width` — a full integer divide (or
//! compiler-emitted `idiv`/`udiv` sequence). The byte layout is RGBA, but the
//! framebuffer is ARGB8888, so the caller had to manually reorder the channels.
//!
//! **Next step:** Iteration 3 — return ARGB byte order directly, matching the
//! framebuffer so the call site needs no reordering.
//!
//! **Commit:** 53a6295

/// Returns the texel colour at UV coordinates `(u, v)` as `[r, g, b, a]`
/// (RGBA byte order).
///
/// UV coordinates wrap via modulo. Out-of-bounds accesses return magenta.
#[allow(dead_code)]
#[inline]
fn get_texture_color_rgba(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    let u = (u * width as f32) as u32 % width;
    let v = (v * height as f32) as u32 % height;
    let index = ((v * width + u) * 4) as usize;
    // Safe slice read — returns magenta on any out-of-bounds access.
    if let Some([b, g, r, a]) = texture.get(index..(index + 4)) {
        [*r, *g, *b, *a]
    } else {
        [255, 0, 255, 255]
    }
}

// --- Call site (inside the per-pixel rasterizer loop) ---
//
//  let [tr, tg, tb, ta] = get_texture_color_rgba(texture, u, v, t_width, t_height);
//  let texture_color = Vec4::new(ta as f32, tr as f32, tg as f32, tb as f32);
//  // note: channel reorder needed because the return is RGBA but the buffer is ARGB
