//! CPU-side color buffer and z-buffer pair.
//!
//! [`Framebuffer`] holds the two buffers that the rasterizer writes into each frame.
//! After rasterization, the color buffer is handed to [`crate::display::Display::color_buffer_to_canvas`].
//!
//! See book chapter: _Framebuffer and z-buffer_ (TODO: link when mdBook is set up).

/// CPU-side color buffer (ARGB8888) and z-buffer (`1/w`) for one frame.
pub struct Framebuffer {
    /// Pixel color data, ARGB8888, row-major. Length = `width * height * 4` bytes.
    pub color_buffer: Box<[u8]>,
    /// Reciprocal-W per pixel. Length = `width * height`. Higher = closer.
    pub z_buffer: Box<[f32]>,
    /// Buffer width in pixels.
    pub width: u32,
    /// Buffer height in pixels.
    pub height: u32,
}

impl Framebuffer {
    #[must_use]
    /// Allocates a framebuffer of `width × height` pixels (all zeroed).
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            color_buffer: vec![0u8; pixel_count * 4].into_boxed_slice(),
            z_buffer: vec![0.0f32; pixel_count].into_boxed_slice(),
            width,
            height,
        }
    }

    /// Zeroes both buffers, ready for a new frame.
    pub fn clear(&mut self) {
        self.color_buffer.fill(0u8);
        self.z_buffer.fill(0.0f32);
    }
}
