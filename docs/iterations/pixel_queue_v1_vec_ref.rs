//! # Iteration 1: `&Vec<Pixel>` parameter
//!
//! **Context:** The `put_pixel_queue` function on `Display` accepted a
//! `&Vec<Pixel>` — the natural type when the call site builds a `Vec` and
//! passes it in.
//!
//! **Measurement:** Passing `&Vec<T>` forces the caller to have a `Vec` on the
//! heap. More importantly, `&Vec<T>` carries a fat pointer (pointer + length +
//! capacity), and Rust must go through an extra indirection to obtain the slice
//! for iteration. The Rust API guidelines and Clippy both warn against
//! `&Vec<T>` in function signatures for this reason.
//!
//! **Problem:** `&Vec<T>` is strictly less general than `&[T]`: it cannot
//! accept a fixed-size array, a stack slice, or any other slice-compatible
//! type. And in this hot path (called every frame for every pixel), the extra
//! pointer indirection was measurably expensive — hence the comment
//! "huge performance gain!!" in the source.
//!
//! **Next step:** Iteration 2 — change the parameter to `&[Pixel]`.
//!
//! **Commit:** predates the earliest commit in the repository; reconstructed
//! from the comment in `src/display.rs`: "replaced `&Vec<Pixel>` with
//! `&[Pixel]` ==> huge performance gain!!"

use sdl2::pixels::Color;

// Reproduced here for context — defined in src/point.rs.
struct Pixel {
    x: i32,
    y: i32,
    color: Color,
}

// The original signature — the `&Vec<Pixel>` anti-pattern.
//
// Clippy lint: `clippy::ptr_arg` — "writing `&Vec<_>` instead of `&[_]`
// involves a new object where a slice will do."
fn put_pixel_queue_v1(
    // display fields inlined for clarity:
    streaming_buffer_color: &mut [u8],
    streaming_buffer_width: u32,
    streaming_buffer_height: u32,
    pixel_queue: &Vec<Pixel>, // <== the slow parameter type
) {
    use byte_slice_cast::AsMutSliceOf;
    let width = streaming_buffer_width;
    let height = streaming_buffer_height;
    let pixel_data_u32 = streaming_buffer_color
        .as_mut_slice_of::<u32>()
        .unwrap();
    pixel_queue.iter().for_each(|pixel| {
        let x = pixel.x;
        let y = pixel.y;
        if x < 0 || x > (width - 1) as i32 || y < 0 || y > (height - 1) as i32 {
            // out of bounds — skip
        } else {
            let color: u32 =
                u32::from_be_bytes([pixel.color.a, pixel.color.r, pixel.color.g, pixel.color.b]);
            let offset = (y * width as i32 + x) as usize;
            pixel_data_u32[offset] = color;
        }
    });
}
