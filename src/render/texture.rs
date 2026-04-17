//! Texture sampling with power-of-2 dimension requirement.
//!
//! The single public function [`get_texture_color_argb_pow2_unchecked`] is the hot-path
//! texture lookup called once per visible pixel in the rasterizer.
//!
//! See book chapter: _Texture sampling iterations_ (TODO: link when mdBook is set up).

/// Texture lookup with power-of-2 dimensions.
///
/// Uses bitwise AND instead of modulo for UV wrapping — requires that both
/// `width` and `height` are powers of two.
/// Returns `[a, r, g, b]` (ARGB byte order matching the framebuffer u32 layout).
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
pub fn get_texture_color_argb_pow2_unchecked(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    debug_assert!(width.is_power_of_two());
    debug_assert!(height.is_power_of_two());
    debug_assert_eq!(texture.len(), width as usize * height as usize * 4);

    let u = ((u * width as f32) as u32) & (width - 1);
    let v = ((v * height as f32) as u32) & (height - 1);
    let index = ((v * width + u) * 4) as usize;

    // SAFETY: `index` is derived from `u` and `v` masked by `(width-1)` and `(height-1)`.
    // Because `width` and `height` are powers of two (asserted above), the bitwise AND
    // guarantees `u < width` and `v < height`, so `index + 3 < texture.len()`.
    unsafe {
        [
            *texture.get_unchecked(index + 3),
            *texture.get_unchecked(index + 2),
            *texture.get_unchecked(index + 1),
            *texture.get_unchecked(index),
        ]
    }
}
