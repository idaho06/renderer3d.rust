//! # Iteration 2: `.fill(0)` — standard library slice fill
//!
//! **Context:** `<[T]>::fill(value)` is the idiomatic way to set all elements
//! of a slice to the same value. It is a one-liner, clearly expresses intent,
//! and gives LLVM the best signal to emit `memset` or a platform-optimised
//! fill intrinsic.
//!
//! **Measurement:** Equivalent or faster than the `iter_mut` loop in all
//! tested configurations. Benchmarks confirmed no regression vs v1; in some
//! builds `.fill()` was measurably faster because LLVM vectorised it.
//!
//! **Problem:** None — this is the final version. A pre-allocated "template"
//! buffer + `copy_from_slice` was also tried (see `clear_v3_copy_from_slice.rs`)
//! and was slower due to L1 cache pressure.
//!
//! **Next step:** See `clear_v3_copy_from_slice.rs` for the failed alternative.
//!
//! **Commit:** visible as active code in `src/cube.rs` lines 581–582

/// Clears a byte slice to zero using `.fill()`.
///
/// This is the final version. LLVM reliably lowers `.fill(0)` to `memset`.
fn clear_buffer_fill(buf: &mut [u8]) {
    buf.fill(0_u8);
}

/// Clears an f32 slice to zero using `.fill()`.
fn clear_zbuffer_fill(buf: &mut [f32]) {
    buf.fill(0.0_f32);
}

// --- Call site (in `Cube::update`, before rasterizing triangles) ---
//
//  self.color_buffer.fill(0_u8);
//  self.z_buffer.fill(0.0_f32);
