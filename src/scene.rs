//! The [`Scene`] trait — the interface between the frame loop and renderable objects.
//!
//! Each scene is updated once per frame (logic, input, pipeline) and then rendered
//! (blitting its finished framebuffer to the display).
//!
//! See book chapter: _Scene trait and frame loop_ (TODO: link when mdBook is set up).

use crate::display::Display;

/// A renderable scene that participates in the frame loop.
pub trait Scene {
    /// Advances simulation by `delta_time` milliseconds and runs the full pipeline.
    fn update(&mut self, delta_time: u32, display: &Display);
    /// Blits the finished framebuffer to `display`.
    fn render(&self, display: &mut Display);
}
