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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 2×2 texture in BGRA byte order with known pixel values.
    ///
    /// Pixel layout (row-major, each pixel is 4 bytes BGRA):
    /// - (u=0, v=0) → index 0: B=10, G=20, R=30, A=255
    /// - (u=1, v=0) → index 1: B=40, G=50, R=60, A=255
    /// - (u=0, v=1) → index 2: B=70, G=80, R=90, A=255
    /// - (u=1, v=1) → index 3: B=100, G=110, R=120, A=255
    fn test_texture_2x2() -> Vec<u8> {
        vec![
            10, 20, 30, 255, // pixel (0,0): B G R A
            40, 50, 60, 255, // pixel (1,0)
            70, 80, 90, 255, // pixel (0,1)
            100, 110, 120, 255, // pixel (1,1)
        ]
    }

    #[test]
    fn known_coord_u0_v0() {
        let tex = test_texture_2x2();
        // (u=0, v=0) → index 0 → BGRA [10,20,30,255] → returned as ARGB [255,30,20,10]
        let color = get_texture_color_argb_pow2_unchecked(&tex, 0.0, 0.0, 2, 2);
        assert_eq!(color, [255, 30, 20, 10]);
    }

    #[test]
    fn known_coord_u_half() {
        let tex = test_texture_2x2();
        // u=0.5 → (0.5 * 2) as u32 = 1 → pixel (1,0) → BGRA [40,50,60,255] → ARGB [255,60,50,40]
        let color = get_texture_color_argb_pow2_unchecked(&tex, 0.5, 0.0, 2, 2);
        assert_eq!(color, [255, 60, 50, 40]);
    }

    #[test]
    fn u_wrap_at_1_equals_u_0() {
        let tex = test_texture_2x2();
        // u=1.0 → (1.0 * 2) as u32 = 2 → 2 & (2-1) = 2 & 1 = 0 → same as u=0
        let at_0 = get_texture_color_argb_pow2_unchecked(&tex, 0.0, 0.0, 2, 2);
        let at_1 = get_texture_color_argb_pow2_unchecked(&tex, 1.0, 0.0, 2, 2);
        assert_eq!(at_0, at_1, "u=1.0 should wrap to u=0.0");
    }

    #[test]
    fn u_wrap_at_2_equals_u_0() {
        let tex = test_texture_2x2();
        // u=2.0 → (2.0 * 2) as u32 = 4 → 4 & 1 = 0 → same as u=0
        let at_0 = get_texture_color_argb_pow2_unchecked(&tex, 0.0, 0.0, 2, 2);
        let at_2 = get_texture_color_argb_pow2_unchecked(&tex, 2.0, 0.0, 2, 2);
        assert_eq!(at_0, at_2, "u=2.0 should wrap to u=0.0");
    }
}
