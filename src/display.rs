//! SDL2 window, canvas, and streaming texture management.
//!
//! [`Display`] owns the SDL2 context and a map of named streaming buffers.
//! Each buffer is a CPU-side ARGB8888 `Box<[u8]>` that scenes write into; `Display` uploads
//! them to the canvas as SDL2 streaming textures on each call to
//! [`Display::color_buffer_to_canvas`].
//!
//! [`DisplayConfig`] lets callers opt into vsync without constructing a display first.
//!
//! See book chapter: _Display and SDL2 setup_ (TODO: link when mdBook is set up).

use crate::userinput::UserInput;
use byte_slice_cast::AsMutSliceOf;
use rustc_hash::FxHashMap;
use sdl2::{
    EventPump, Sdl,
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    render::{Canvas, Texture},
    video::Window,
};

use crate::pixel::Pixel;

extern crate sdl2;

struct StreamingBuffer {
    color_buffer: Box<[u8]>,
    texture: Texture,
}

/// SDL2 window + canvas + named streaming buffers.
///
/// Scenes never call SDL2 directly; they write into a named color buffer and call
/// [`Display::color_buffer_to_canvas`] to blit it.
pub struct Display {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    streaming_buffers: FxHashMap<String, StreamingBuffer>,
    w_width: u32,
    w_height: u32,
    pub user_input: UserInput,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration passed to [`Display::with_config`] before the SDL2 window is created.
#[derive(Default)]
pub struct DisplayConfig {
    /// Enable vsync on the SDL2 canvas presenter.
    pub vsync: bool,
}

#[allow(clippy::missing_panics_doc)]
impl Display {
    #[must_use]
    /// Creates a display with default config (vsync off).
    pub fn new() -> Self {
        Self::with_config(DisplayConfig::default())
    }

    /// # Panics
    /// Panics if SDL2 initialization fails.
    #[must_use]
    #[allow(clippy::needless_pass_by_value, clippy::cast_sign_loss, clippy::missing_panics_doc)]
    /// Creates a display with the given config, querying the current desktop resolution.
    pub fn with_config(config: DisplayConfig) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let mut dm = sdl2::sys::SDL_DisplayMode {
            format: 0,
            w: 0,
            h: 0,
            refresh_rate: 0,
            driverdata: std::ptr::null_mut::<std::ffi::c_void>(),
        };
        // SAFETY: `SDL_GetCurrentDisplayMode` writes into `dm` which we own on the stack.
        // Display index 0 is always valid when SDL2 video is initialised.
        // `&raw mut dm` produces a raw pointer without creating an intermediate reference,
        // satisfying SDL2's `*mut SDL_DisplayMode` parameter.
        unsafe {
            sdl2::sys::SDL_GetCurrentDisplayMode(0_i32, &raw mut dm);
        }

        println!("Current display w: {} h: {}", dm.w, dm.h);

        #[allow(clippy::cast_sign_loss)]
        let w_width: u32 = dm.w as u32;
        #[allow(clippy::cast_sign_loss)]
        let w_height: u32 = dm.h as u32;

        let window = video_subsystem
            .window("Renderer 3D in rust", dm.w as u32, dm.h as u32)
            .position_centered()
            .borderless()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas_builder = window.into_canvas().accelerated();
        if config.vsync {
            canvas_builder = canvas_builder.present_vsync();
        }
        let canvas = canvas_builder
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.info().texture_formats.iter().for_each(|&format| {
            println!("texture format: {format:?}");
        });

        println!(
            "canvas default pixel format: {:?}",
            canvas.default_pixel_format()
        );

        Self {
            sdl_context,
            canvas,
            w_width,
            w_height,
            streaming_buffers: FxHashMap::default(),
            user_input: UserInput::new(),
        }
    }

    /// Fills the canvas with black.
    pub fn cls(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    }
    /// Presents the canvas to the screen (flips the back buffer).
    pub fn present_canvas(&mut self) {
        self.canvas.present();
    }
    #[must_use]
    /// Returns the SDL2 event pump. Call once per frame to drain the event queue.
    pub fn get_event_pump(&self) -> EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    #[must_use]
    /// Returns the number of milliseconds since SDL2 was initialized (monotonic clock).
    pub fn get_frame_time(&self) -> u32 {
        self.sdl_context.timer().unwrap().ticks()
    }

    #[must_use]
    /// Returns the window width in pixels (current desktop resolution).
    pub fn get_width(&self) -> u32 {
        self.w_width
    }
    #[must_use]
    /// Returns the window height in pixels.
    pub fn get_height(&self) -> u32 {
        self.w_height
    }

    /// Allocates a new named ARGB8888 streaming buffer and its backing SDL2 texture.
    pub fn add_streaming_buffer(&mut self, name: &str, width: u32, height: u32) {
        let mut texture = self
            .canvas
            .create_texture_streaming(sdl2::pixels::PixelFormatEnum::ARGB8888, width, height)
            .unwrap();
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        let color_buffer = vec![0; (width * height * 4) as usize].into_boxed_slice();
        self.streaming_buffers.insert(
            name.to_string(),
            StreamingBuffer {
                color_buffer,
                texture,
            },
        );
    }

    /// Fills the named buffer's CPU-side color buffer with `color` (ARGB8888).
    pub fn clear_streaming_buffer(&mut self, name: &str, color: Color) {
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let a = color.a;
            let r = color.r;
            let g = color.g;
            let b = color.b;
            let color: u32 = u32::from_be_bytes([a, r, g, b]); //ARGB8888
            streaming_buffer
                .color_buffer
                .as_mut_slice_of::<u32>()
                .unwrap()
                .fill(color);
        }
    }

    /// Uploads the named buffer's internal color buffer to its SDL2 texture and blits to canvas.
    pub fn streaming_buffer_to_canvas(&mut self, name: &str) {
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let width = streaming_buffer.texture.query().width;
            let pitch = width as usize * std::mem::size_of::<u32>();
            streaming_buffer
                .texture
                .update(None, &streaming_buffer.color_buffer, pitch)
                .unwrap();

            self.canvas
                .copy(&streaming_buffer.texture, None, None)
                .unwrap();
        }
    }

    /// Uploads an externally-owned ARGB8888 `color_buffer` to the named SDL2 texture and blits.
    ///
    /// The scene writes into its own `Framebuffer::color_buffer`; this method is the handoff
    /// from CPU memory to the SDL2 canvas.
    pub fn color_buffer_to_canvas(&mut self, name: &str, color_buffer: &[u8]) {
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let width = streaming_buffer.texture.query().width;
            let pitch = width as usize * std::mem::size_of::<u32>();
            streaming_buffer
                .texture
                .update(None, color_buffer, pitch)
                .unwrap();
            self.canvas
                .copy(&streaming_buffer.texture, None, None)
                .unwrap();
        }
    }

    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::many_single_char_names
    )]
    /// Writes a batch of pixels into the named streaming buffer.
    ///
    /// Out-of-bounds pixels are silently skipped.
    pub fn put_pixel_queue(&mut self, name: &str, pixel_queue: &[Pixel]) {
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let width = streaming_buffer.texture.query().width;
            let height = streaming_buffer.texture.query().height;
            for pixel in pixel_queue {
                let x = pixel.x;
                let y = pixel.y;
                if x >= 0 && x <= (width - 1) as i32 && y >= 0 && y <= (height - 1) as i32 {
                    let a = pixel.color.a;
                    let r = pixel.color.r;
                    let g = pixel.color.g;
                    let b = pixel.color.b;
                    let color: u32 = u32::from_be_bytes([a, r, g, b]); //ARGB8888
                    let offset = ((y * width as i32) + x) as usize;
                    let pixel_data_u32 = streaming_buffer
                        .color_buffer
                        .as_mut_slice_of::<u32>()
                        .unwrap();
                    pixel_data_u32[offset] = color;
                }
            }
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn get_aspect_ratio(&self) -> f32 {
        self.get_width() as f32 / self.get_height() as f32
    }

    /// Toggles SDL2 relative mouse mode (captured vs. free cursor).
    pub fn switch_relative_mouse_mode(&mut self) {
        self.user_input.mouse.is_relative = !self.user_input.mouse.is_relative;
        self.sdl_context
            .mouse()
            .set_relative_mouse_mode(self.user_input.mouse.is_relative);
    }

    /// Drains the SDL2 event queue and updates [`Display::user_input`] for this frame.
    pub fn update_user_input(&mut self) {
        let mut event_pump = self.get_event_pump();
        self.user_input.reset();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.user_input.quit = true;
                }

                // W key
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    self.user_input.key_w.changed = true;
                    self.user_input.key_w.pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    self.user_input.key_w.changed = true;
                    self.user_input.key_w.pressed = false;
                }

                // A key
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.user_input.key_a.changed = true;
                    self.user_input.key_a.pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.user_input.key_a.changed = true;
                    self.user_input.key_a.pressed = false;
                }

                // S key
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.user_input.key_s.changed = true;
                    self.user_input.key_s.pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.user_input.key_s.changed = true;
                    self.user_input.key_s.pressed = false;
                }

                // D key
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    self.user_input.key_d.changed = true;
                    self.user_input.key_d.pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    self.user_input.key_d.changed = true;
                    self.user_input.key_d.pressed = false;
                }

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    self.user_input.mouse.left.changed = true;
                    self.user_input.mouse.left.pressed = true;
                }

                Event::MouseMotion { xrel, yrel, .. } if self.user_input.mouse.is_relative => {
                    self.user_input.mouse.x = xrel;
                    self.user_input.mouse.y = yrel;
                }
                Event::MouseMotion { x, y, .. } if !self.user_input.mouse.is_relative => {
                    self.user_input.mouse.x = x;
                    self.user_input.mouse.y = y;
                }

                _ => {}
            }
        }
    }
}
