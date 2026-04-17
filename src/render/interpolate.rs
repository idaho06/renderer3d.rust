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
