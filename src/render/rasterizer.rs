//! Scanline rasterizer — the innermost hot path of the graphics pipeline.
//!
//! [`draw_3dtriangle_to_color_buffer`] rasterizes a single clipped, projected triangle
//! with perspective-correct UV interpolation, z-buffering, and integer color multiplication.
//!
//! ## Key design decisions
//!
//! - **`1/w` z-buffer**: reciprocal W interpolates linearly in screen space; higher = closer.
//! - **`(a * b) >> 8` color multiply**: integer multiply + shift is multiple cycles faster
//!   than float multiply/divide and visually equivalent for 8-bit channel modulation.
//! - **`unsafe` accesses**: bounds are proven before entering the pixel loop; the `unsafe`
//!   blocks call `get_unchecked` / `to_int_unchecked` to avoid redundant bounds checks.
//!
//! See book chapter: _Pipeline Stage 8 — Rasterize_ (TODO: link when mdBook is set up).

use byte_slice_cast::AsMutSliceOf;
use glam::{Vec3, Vec4};

use crate::triangle::{Triangle, TriangleScreenPixel};
use super::interpolate::map_interpolate_float_vec4_iter;
use super::texture::get_texture_color_argb_pow2_unchecked;

#[inline]
#[allow(clippy::cast_lossless, clippy::cast_possible_truncation)]
/// Multiplies two 8-bit color channel values as if they were in `[0, 1]`.
///
/// `(a as u16 * b as u16) >> 8` — integer multiply + right shift is multiple cycles faster
/// than float division by 255 and visually equivalent for 8-bit channel modulation.
fn mul_u8(a: u8, b: u8) -> u8 {
    ((u16::from(a) * u16::from(b)) >> 8) as u8
}

/// Scanline rasterizer with perspective-correct UV interpolation and z-buffering.
///
/// ## Z-buffer: why `1/w`
///
/// The z-buffer stores reciprocal W (`1/w`) rather than depth directly.
/// `1/w` interpolates **linearly in screen space** (whereas depth `z/w` does not),
/// which means we compute one reciprocal per vertex and interpolate cheaply across the
/// scanline. Higher values of `1/w` mean closer to the camera, so the test is `>`.
///
/// ## Pixel format
///
/// ARGB8888 stored as `u32::from_be_bytes([a, r, g, b])`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::missing_panics_doc
)]
pub fn draw_3dtriangle_to_color_buffer(
    triangle3d: &Triangle,
    color_buffer: &mut [u8],
    cb_width: u32,
    cb_height: u32,
    texture: &[u8],
    t_width: u32,
    t_height: u32,
    z_buffer: &mut [f32],
) {
    if cb_width == 0 || cb_height == 0 {
        return;
    }

    let color_buffer_u32 = color_buffer.as_mut_slice_of::<u32>().unwrap();
    let cb_width_usize = cb_width as usize;
    let cb_height_usize = cb_height as usize;
    let buffer_len = cb_width_usize * cb_height_usize;
    debug_assert_eq!(color_buffer_u32.len(), buffer_len);
    debug_assert_eq!(z_buffer.len(), buffer_len);

    let cb_width_f32 = cb_width as f32;
    let cb_height_f32 = cb_height as f32;
    let max_x = cb_width_f32 - 1.0;

    let triangle3d = triangle3d.reorder_vertices_by_y();
    let triangle_color_array_u8 = [
        triangle3d.color.a,
        triangle3d.color.r,
        triangle3d.color.g,
        triangle3d.color.b,
    ];

    let v0 = triangle3d.vertices[0];
    let v1 = triangle3d.vertices[1];
    let v2 = triangle3d.vertices[2];
    let uv0 = triangle3d.uvs[0];
    let uv1 = triangle3d.uvs[1];
    let uv2 = triangle3d.uvs[2];

    let v0 = Vec4::new(v0.x.round(), v0.y.round(), v0.z, v0.w);
    let v1 = Vec4::new(v1.x.round(), v1.y.round(), v1.z, v1.w);
    let v2 = Vec4::new(v2.x.round(), v2.y.round(), v2.z, v2.w);

    let corner_segment1_iter = map_interpolate_float_vec4_iter(
        v0.y,
        Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
        v1.y,
        Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w),
    )
    .map(|(y, xwuv)| TriangleScreenPixel {
        x: xwuv.x.round(),
        y,
        reciprocal_w: xwuv.y,
        u_divided_w: xwuv.z,
        v_divided_w: xwuv.w,
    });
    let corner_segment2_iter = map_interpolate_float_vec4_iter(
        v1.y,
        Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w),
        v2.y,
        Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w),
    )
    .map(|(y, xwuv)| TriangleScreenPixel {
        x: xwuv.x.round(),
        y,
        reciprocal_w: xwuv.y,
        u_divided_w: xwuv.z,
        v_divided_w: xwuv.w,
    });
    let corner_segments_iter = corner_segment1_iter.chain(corner_segment2_iter.skip(1));

    let straight_segment_iter = map_interpolate_float_vec4_iter(
        v0.y,
        Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
        v2.y,
        Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w),
    )
    .map(|(y, xwuv)| TriangleScreenPixel {
        x: xwuv.x.round(),
        y,
        reciprocal_w: xwuv.y,
        u_divided_w: xwuv.z,
        v_divided_w: xwuv.w,
    });

    for (straight_pixel, corner_pixel) in straight_segment_iter.zip(corner_segments_iter) {
        let scanline_y = straight_pixel.y;
        if scanline_y < 0.0 || scanline_y >= cb_height_f32 {
            continue;
        }

        let start_x = straight_pixel.x;
        let end_x = corner_pixel.x;
        let start_values = Vec3::new(
            straight_pixel.reciprocal_w,
            straight_pixel.u_divided_w,
            straight_pixel.v_divided_w,
        );
        let end_values = Vec3::new(
            corner_pixel.reciprocal_w,
            corner_pixel.u_divided_w,
            corner_pixel.v_divided_w,
        );

        let (raw_start_x, raw_end_x, raw_start_values, raw_end_values) = if start_x <= end_x {
            (start_x, end_x, start_values, end_values)
        } else {
            (end_x, start_x, end_values, start_values)
        };

        if raw_end_x < 0.0 || raw_start_x > max_x {
            continue;
        }

        let clamped_start_x = raw_start_x.max(0.0);
        let clamped_end_x = raw_end_x.min(max_x);
        if clamped_start_x > clamped_end_x {
            continue;
        }

        #[allow(clippy::float_cmp)]
        let span_step = if raw_start_x == raw_end_x {
            Vec3::ZERO
        } else {
            (raw_end_values - raw_start_values) / (raw_end_x - raw_start_x)
        };
        let mut span_values = raw_start_values + span_step * (clamped_start_x - raw_start_x);

        let start_x: i32;
        let end_x: i32;
        let y: i32;
        // SAFETY: `clamped_start_x` and `clamped_end_x` are both clamped to `[0, max_x]`
        // (i.e. `[0.0, cb_width - 1.0]`) by the `.max(0.0)` / `.min(max_x)` calls above.
        // `scanline_y` passed the `< cb_height_f32` guard, so it is in `[0, cb_height - 1)`.
        // All three values are therefore exact non-negative integers fitting in `i32`.
        unsafe {
            start_x = clamped_start_x.to_int_unchecked();
            end_x = clamped_end_x.to_int_unchecked();
            y = scanline_y.to_int_unchecked();
        }

        let row_base = y as usize * cb_width_usize;

        for x in start_x..=end_x {
            let reciprocal_w = span_values.x;
            let z_index = row_base + x as usize;

            // SAFETY: `z_index = y * cb_width + x`. `y` is in `[0, cb_height)` and
            // `x` is in `[start_x, end_x]` ⊆ `[0, cb_width - 1]`, so
            // `z_index < cb_width * cb_height == z_buffer.len()`.
            let z_buffer_w = unsafe { *z_buffer.get_unchecked(z_index) };

            if reciprocal_w > z_buffer_w {
                // SAFETY: same index bounds as the read above.
                unsafe {
                    *z_buffer.get_unchecked_mut(z_index) = reciprocal_w;
                }

                let u_divided_w = span_values.y;
                let v_divided_w = span_values.z;
                let w = 1.0 / reciprocal_w;
                let u = u_divided_w * w;
                let v = v_divided_w * w;

                let texture_color =
                    get_texture_color_argb_pow2_unchecked(texture, u, v, t_width, t_height);

                let color: u32 = u32::from_be_bytes([
                    mul_u8(texture_color[0], triangle_color_array_u8[0]),
                    mul_u8(texture_color[1], triangle_color_array_u8[1]),
                    mul_u8(texture_color[2], triangle_color_array_u8[2]),
                    mul_u8(texture_color[3], triangle_color_array_u8[3]),
                ]);
                // SAFETY: `z_index < color_buffer_u32.len()` (same bounds as z_buffer).
                unsafe {
                    *color_buffer_u32.get_unchecked_mut(z_index) = color;
                }
            }

            span_values += span_step;
        }
    }
}
