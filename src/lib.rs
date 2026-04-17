//! `render3d` — CPU software rasterizer library.
//!
//! All rendering happens on the CPU; SDL2 is used only for windowing, event handling,
//! and blitting the finished ARGB8888 color buffer to the screen each frame.
//!
//! ## Module overview
//!
//! | Module | Role |
//! |--------|------|
//! | [`camera`] | First-person camera with WASD movement |
//! | [`cli`] | Command-line argument parsing |
//! | [`clipping`] | Homogeneous-space frustum clipping |
//! | [`display`] | SDL2 window, canvas, and streaming buffers |
//! | [`fire`] | 2D fire-effect scene |
//! | [`framebuffer`] | CPU color buffer + z-buffer |
//! | [`mesh`] | 3D scene: pipeline orchestration |
//! | [`model`] | OBJ loader and built-in cube |
//! | [`pixel`] | Single pixel helper |
//! | [`render`] | Rasterizer, interpolation, texture lookup, 2D helpers |
//! | [`scene`] | `Scene` trait |
//! | [`triangle`] | `Triangle`, `Face`, `TriangleScreenPixel` types |
//! | [`userinput`] | Keyboard / mouse input state |

pub mod camera;
pub mod cli;
pub mod clipping;
pub mod display;
pub mod fire;
pub mod framebuffer;
pub mod mesh;
pub mod model;
pub mod pixel;
pub mod render;
pub mod scene;
pub mod triangle;
pub mod userinput;
