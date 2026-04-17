//! # Iteration 2: `&[Pixel]` — slice parameter
//!
//! **Context:** Changing `&Vec<Pixel>` to `&[Pixel]` is a one-character edit
//! at the function signature. The Rust compiler automatically coerces `&Vec<T>`
//! to `&[T]` at every call site (deref coercion), so no call-site changes were
//! needed.
//!
//! **Measurement:** The comment in `src/display.rs` records the result:
//! "replaced `&Vec<Pixel>` with `&[Pixel]` ==> huge performance gain!!"
//!
//! Why? `&[T]` is a fat pointer (data pointer + length). The compiler can
//! inline the slice pointer directly and eliminate the extra indirection
//! through the `Vec` struct. More critically, `&[Pixel]` enables the compiler
//! to inline and auto-vectorise the loop body more aggressively because the
//! aliasing rules are tighter.
//!
//! **Problem:** None — this is the final version. The `Pixel` type itself
//! (containing an `sdl2::pixels::Color`) is renamed to `pixel.rs` in Phase 2,
//! but the slice parameter pattern is kept.
//!
//! **Next step:** Phase 2 renames `Pixel` → `Pixel` in `src/pixel.rs` and
//! the `Display` acquires a `DisplayConfig` for vsync. The slice pattern stays.
//!
//! **Commit:** `ff55eab` (earliest available; the change predates repository
//! history but the comment documents the before/after)

use sdl2::pixels::Color;

// Reproduced here for context — defined in src/point.rs.
struct Pixel {
    x: i32,
    y: i32,
    color: Color,
}

/// Writes a batch of pixels into a streaming color buffer.
///
/// `pixel_queue: &[Pixel]` — accepts any contiguous slice, including `&Vec<T>`
/// (via deref coercion), arrays, and stack slices.
fn put_pixel_queue_v2(
    streaming_buffer_color: &mut [u8],
    streaming_buffer_width: u32,
    streaming_buffer_height: u32,
    pixel_queue: &[Pixel], // <== the fast parameter type
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
