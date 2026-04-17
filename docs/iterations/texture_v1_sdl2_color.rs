//! # Iteration 1: SDL2 Color struct per pixel
//!
//! **Context:** First working textured triangle. The natural instinct was to
//! reach for the SDL2 color type since the rest of the display layer already
//! used it.
//!
//! **Measurement:** Profiling showed `get_texture_color_sdl2` near the top of
//! the flame graph. Constructing an `sdl2::pixels::Color` struct for every
//! pixel — and then destructuring it again — added measurable overhead in the
//! hot rasterization loop.
//!
//! **Problem:** `sdl2::pixels::Color` is an opaque struct with RGBA field
//! accessors. The round-trip (construct → read `.r`/`.g`/`.b`/`.a`) is
//! unnecessary; we only need the raw bytes. Also uses `%` for UV wrap (full
//! integer divide) and an `assert!` bounds check on every call.
//!
//! **Next step:** Iteration 2 — return `[u8; 4]` directly (RGBA byte order),
//! keep the `%` wrap and `.get()` bounds check.
//!
//! **Commit:** ff55eab

/// Returns the texel colour at UV coordinates `(u, v)` as an
/// `sdl2::pixels::Color` struct.
///
/// The texture is stored in BGRA byte order (as loaded by the `image` crate).
/// UV coordinates wrap with `%` (modulo). Bounds are asserted.
#[inline(always)]
fn get_texture_color_sdl2(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> sdl2::pixels::Color {
    let u = u * width as f32;
    let v = v * height as f32;
    let u = u as u32;
    let v = v as u32;
    let u = u % width;
    let v = v % height;
    let index = (v * width + u) as usize;
    assert!(index * 4 + 3 < texture.len());
    let b = texture[index * 4];
    let g = texture[index * 4 + 1];
    let r = texture[index * 4 + 2];
    let a = texture[index * 4 + 3];
    sdl2::pixels::Color::RGBA(r, g, b, a)
}

// --- Call site (inside the per-pixel rasterizer loop) ---
//
//  let texture_color = get_texture_color_sdl2(texture, u, v, t_width, t_height);
//  let a = triangle3d.color.a as f32 / 255.0;
//  let r = triangle3d.color.r as f32 / 255.0;
//  let g = triangle3d.color.g as f32 / 255.0;
//  let b = triangle3d.color.b as f32 / 255.0;
//  let color = sdl2::pixels::Color::RGBA(        // <== another Color alloc!
//      (texture_color.r as f32 * r) as u8,
//      (texture_color.g as f32 * g) as u8,
//      (texture_color.b as f32 * b) as u8,
//      (texture_color.a as f32 * a) as u8,
//  );
//  let color: u32 = u32::from_be_bytes([color.a, color.r, color.g, color.b]);
