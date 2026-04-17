//! A single screen pixel with a color.
//!
//! [`Pixel`] is used by [`crate::display::Display::put_pixel_queue`] for batched 2D pixel writes.
//!
//! See book chapter: _Display and SDL2 setup_ (TODO: link when mdBook is set up).

use sdl2::pixels::Color;

/// A screen-space pixel coordinate and ARGB color.
pub struct Pixel {
    /// Horizontal pixel coordinate (screen space, origin top-left).
    pub x: i32,
    /// Vertical pixel coordinate (screen space, origin top-left).
    pub y: i32,
    /// ARGB color of this pixel.
    pub color: Color,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            color: Color::RGBA(0, 0, 0, 255),
        }
    }
}

impl Pixel {
    #[must_use]
    /// Creates a pixel at `(x, y)` with the given `color`.
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self { x, y, color }
    }
}
