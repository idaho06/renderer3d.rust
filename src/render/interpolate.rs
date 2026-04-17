//! Scanline interpolation helpers for the rasterizer.
//!
//! Two variants are provided:
//! - [`map_interpolate_float_vec4_iter`] — allocation-free iterator used in the hot path.
//! - [`map_interpolate_int`] — `Vec`-returning integer version used for 2D line drawing.
//!
//! See book chapter: _Interpolation iterations_ (TODO: link when mdBook is set up).

use glam::Vec4;

/// Linear interpolation iterator for scanline rasterization.
///
/// `i` steps by ±1.0 from `i0` to `i1`. `d` tracks a `Vec4` payload
/// (x, `reciprocal_w`, u/w, v/w) that advances by a constant step each tick.
/// Allocation-free: no heap allocation, unlike the earlier `Vec`-based versions.
#[inline]
pub fn map_interpolate_float_vec4_iter(
    i0: f32,
    d0: Vec4,
    i1: f32,
    d1: Vec4,
) -> impl Iterator<Item = (f32, Vec4)> {
    let forward = i1 > i0;
    #[allow(clippy::float_cmp)]
    let mut a = if i1 == i0 {
        Vec4::ZERO
    } else {
        (d1 - d0) / (i1 - i0)
    };
    if !forward {
        a = Vec4::ZERO - a;
    }
    let step = if forward { 1.0 } else { -1.0 };
    let mut i = i0;
    let mut d = d0;
    std::iter::from_fn(move || {
        let result = Some((i, d));
        if (forward && i > i1) || (!forward && i < i1) {
            None
        } else {
            d += a;
            i += step;
            result
        }
    })
}

/// Integer linear interpolation — returns a `Vec` of (i, d) pairs stepping by ±1.
/// Kept for use in 2D line drawing where the alloc cost is acceptable.
#[inline]
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::missing_panics_doc)]
pub fn map_interpolate_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<(i32, i32)> {
    if i0 == i1 {
        return vec![(i0, d0)];
    }
    let distance = (i1 - i0).unsigned_abs();
    let mut values: Vec<(i32, i32)> = Vec::with_capacity(distance.try_into().unwrap());
    let mut a: f32 = (d1 as f32 - d0 as f32) / (i1 as f32 - i0 as f32);
    let step: i32 = if i1 > i0 { 1 } else { -1 };
    if step == -1 {
        a = -a;
    }
    let mut i = i0;
    let mut d = d0 as f32;
    loop {
        values.push((i, d as i32));
        d += a;
        i += step;
        if step == 1 && i > i1 {
            break;
        }
        if step == -1 && i < i1 {
            break;
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    const EPS: f32 = 1e-5;

    // ── map_interpolate_float_vec4_iter ──────────────────────────────────────

    #[test]
    fn midpoint_is_lerp_half() {
        let d0 = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let d1 = Vec4::new(4.0, 8.0, 2.0, 1.0);
        let vals: Vec<_> = map_interpolate_float_vec4_iter(0.0, d0, 4.0, d1).collect();
        // i steps 0, 1, 2, 3, 4 → 5 values; midpoint is index 2 (i=2.0)
        assert_eq!(vals.len(), 5);
        let (i_mid, d_mid) = vals[2];
        assert!((i_mid - 2.0).abs() < EPS);
        let expected = (d0 + d1) / 2.0;
        assert!((d_mid - expected).length() < EPS, "midpoint d should be (d0+d1)/2");
    }

    #[test]
    fn empty_range_yields_one_value() {
        let d0 = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let vals: Vec<_> = map_interpolate_float_vec4_iter(5.0, d0, 5.0, d0).collect();
        assert_eq!(vals.len(), 1, "equal i0/i1 must yield exactly one element");
        let (i, d) = vals[0];
        assert!((i - 5.0).abs() < EPS);
        assert!((d - d0).length() < EPS);
    }

    #[test]
    fn reverse_range_correct_order() {
        // i0=2, i1=0 → should step 2, 1, 0 (three values in descending order)
        let d0 = Vec4::new(2.0, 0.0, 0.0, 0.0);
        let d1 = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let vals: Vec<_> = map_interpolate_float_vec4_iter(2.0, d0, 0.0, d1).collect();
        assert_eq!(vals.len(), 3);
        assert!((vals[0].0 - 2.0).abs() < EPS);
        assert!((vals[1].0 - 1.0).abs() < EPS);
        assert!((vals[2].0 - 0.0).abs() < EPS);
    }

    #[test]
    fn reverse_range_midpoint() {
        let d0 = Vec4::new(8.0, 0.0, 0.0, 0.0);
        let d1 = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let vals: Vec<_> = map_interpolate_float_vec4_iter(2.0, d0, 0.0, d1).collect();
        // middle step is i=1 (index 1); d should be midpoint (4,0,0,0)
        let (_, d_mid) = vals[1];
        let expected = (d0 + d1) / 2.0;
        assert!((d_mid - expected).length() < EPS, "reverse midpoint d should be (d0+d1)/2");
    }
}
