//! # Iteration 3: Pre-allocated zero buffer + `copy_from_slice`
//!
//! **Context:** A common systems-programming trick for clearing a buffer is to
//! keep a pre-allocated "clean" copy and `memcpy` it over the working buffer
//! each frame. The hypothesis was that reading from a warm cache line and
//! writing to the destination might be faster than generating zeros on the fly.
//!
//! **Measurement:** Commit `8d6e3a0` ("avoid L1 cache trash when updating the
//! buffer") investigated cache-access patterns. The pre-allocated buffer
//! doubles the memory footprint of the color/z-buffer pair (2× 1280×720×4 =
//! ~7 MB for color alone). At that size, the "clean" source buffer is cold in
//! L1/L2 on each frame. Reading it to copy into the working buffer caused cache
//! misses, making this *slower* than `.fill()`, which can zero memory without
//! loading cache lines from a source.
//!
//! **Problem:** L1/L2 pollution. The `.fill(0)` path writes zeros without
//! needing a source load, so it has half the memory bandwidth of
//! `copy_from_slice`. On 7 MB buffers, bandwidth is the bottleneck.
//!
//! **Next step:** `.fill()` won. See `clear_v2_fill.rs`.
//!
//! **Commit:** `8d6e3a0` — the loop-order fix in the fire buffer renderer was
//! another instance of the same cache-access lesson applied to iteration order
//! (column-major vs row-major traversal).

/// Clears a buffer by copying from a pre-allocated zero buffer.
///
/// Requires `zero_buf.len() == buf.len()`. The zero buffer must be allocated
/// once at startup and never modified.
///
/// Do NOT use — benchmarks showed this is slower than `.fill(0)` for large
/// buffers because the source buffer is cold in cache each frame.
fn clear_buffer_copy_from_slice(buf: &mut [u8], zero_buf: &[u8]) {
    buf.copy_from_slice(zero_buf);
}

fn clear_zbuffer_copy_from_slice(buf: &mut [f32], zero_buf: &[f32]) {
    buf.copy_from_slice(zero_buf);
}

// --- What the struct would have looked like ---
//
//  struct Cube {
//      color_buffer: Box<[u8]>,
//      color_buffer_clear: Box<[u8]>,  // pre-allocated zero copy
//      z_buffer: Box<[f32]>,
//      z_buffer_clear: Box<[f32]>,     // pre-allocated zero copy
//      ...
//  }
//
// --- Call site (in `Cube::update`) ---
//
//  self.color_buffer.copy_from_slice(&self.color_buffer_clear);
//  self.z_buffer.copy_from_slice(&self.z_buffer_clear);
//
// (Visible as commented-out code in `src/cube.rs` lines 34–36 and 587–589)
