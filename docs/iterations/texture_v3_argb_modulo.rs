//! # Iteration 3: ARGB byte order, modulo wrap, bounds-checked slice
//!
//! **Context:** The framebuffer is ARGB8888 (stored as `u32::from_be_bytes([a, r, g, b])`).
//! Returning RGBA from the texture sampler meant the call site had to manually
//! reorder bytes on every pixel. Moving the swap into the sampler removes that
//! burden and makes the intent explicit.
//!
//! **Measurement:** The byte-swap itself is free — it is just a different index
//! order in the return array. The win is cleaner call-site code. The `%`
//! modulo wrap and the `.get()` bounds check are still present.
//!
//! **Problem:** `u % width` compiles to an integer divide on most targets.
//! For a 1280×720 scene rendering 800k+ pixels per frame this is measurable.
//! The texture dimensions are always powers of two (256×256, 512×512, etc.),
//! so `x % n` can be replaced by `x & (n - 1)`.
//!
//! **Next step:** Iteration 4 — require power-of-2 dimensions, replace `%`
//! with `& (n - 1)`, and remove the bounds check with `get_unchecked`.
//!
//! **Commit:** (intermediate; visible as commented-out `get_texture_color_argb`
//! in the current `src/render.rs`)

/// Returns the texel colour at UV coordinates `(u, v)` as `[a, r, g, b]`
/// (ARGB byte order, matching the ARGB8888 framebuffer).
///
/// UV coordinates wrap via modulo. Out-of-bounds accesses return yellow
/// (debug sentinel).
#[allow(dead_code)]
#[inline]
fn get_texture_color_argb(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    let u = (u * width as f32) as u32 % width;
    let v = (v * height as f32) as u32 % height;
    let index = ((v * width + u) * 4) as usize;
    // Texture is stored BGRA (as loaded by the `image` crate, then left as-is).
    // We read [b, g, r, a] and return [a, r, g, b] to match ARGB8888.
    if let Some([b, g, r, a]) = texture.get(index..(index + 4)) {
        [*a, *r, *g, *b]
    } else {
        [255, 255, 0, 255] // yellow sentinel — means texture coords are broken
    }
}

// --- Call site (inside the per-pixel rasterizer loop) ---
//
//  let texture_color_array_u8 = get_texture_color_argb(texture, u, v, t_width, t_height);
//  // No channel reorder needed — [a, r, g, b] maps directly to ARGB8888.
