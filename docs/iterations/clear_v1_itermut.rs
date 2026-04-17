//! # Iteration 1: `iter_mut().for_each(|x| *x = 0)`
//!
//! **Context:** First implementation of buffer clearing. Before each frame,
//! both the color buffer (ARGB8888, `Box<[u8]>`) and the z-buffer (`Box<[f32]>`)
//! must be reset to zero. The iterator version felt idiomatic.
//!
//! **Measurement:** Functional but not optimal. The compiler cannot always
//! prove that the iterator loop is equivalent to a `memset`, so LLVM may emit
//! a scalar loop rather than a vectorised one. In practice, `.fill()` is more
//! reliably optimised to `memset`.
//!
//! **Problem:** Leaving LLVM to figure out the `memset` equivalence is fragile.
//! `.fill()` is explicit about the intent and consistently maps to `memset` or
//! the platform's fastest zero-fill intrinsic.
//!
//! **Next step:** Iteration 2 — use `.fill(0)` directly.
//!
//! **Commit:** visible as commented-out code in `src/cube.rs` lines 583–584

/// Clears a byte slice to zero using `iter_mut`.
fn clear_buffer_iter_mut(buf: &mut [u8]) {
    buf.iter_mut().for_each(|x| *x = 0_u8);
}

/// Clears an f32 slice to zero using `iter_mut`.
fn clear_zbuffer_iter_mut(buf: &mut [f32]) {
    buf.iter_mut().for_each(|x| *x = 0.0_f32);
}

// --- Call site (in `Cube::update`, before rasterizing triangles) ---
//
//  self.color_buffer.iter_mut().for_each(|x| *x = 0_u8);
//  self.z_buffer.iter_mut().for_each(|x| *x = 0.0_f32);
