use byte_slice_cast::AsMutSliceOf;
use rustc_hash::FxHashMap;
use sdl2::{
    pixels::{Color, PixelFormat},
    render::{Canvas, Texture},
    video::Window,
    EventPump, Sdl,
};

use crate::point::Pixel;

extern crate sdl2;

struct StreamingBuffer {
    color_buffer: Box<[u8]>,
    texture: Texture,
}

pub struct Display {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    streaming_buffers: FxHashMap<String, StreamingBuffer>,
    w_width: u32,
    w_height: u32,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        //let timer = sdl_context.timer().unwrap();

        let mut dm = sdl2::sys::SDL_DisplayMode {
            format: 0,
            w: 0,
            h: 0,
            refresh_rate: 0,
            // clippy warning: `0 as *mut _` detected
            //driverdata: 0 as *mut c_void,
            driverdata: std::ptr::null_mut::<std::ffi::c_void>(),
        };
        unsafe {
            // clippy warning: casting integer literal to `i32` is unnecessary
            //SDL_GetCurrentDisplayMode(0 as i32, &mut dm);
            sdl2::sys::SDL_GetCurrentDisplayMode(0_i32, &mut dm);
        }

        println!("Current display w: {} h: {}", dm.w, dm.h);

        //dm.w = 640;
        //dm.h = 360;
        let w_width: u32 = dm.w as u32;
        let w_height: u32 = dm.h as u32;

        let window = video_subsystem
            .window("Renderer 3D in rust", dm.w as u32, dm.h as u32)
            .position_centered()
            //.vulkan()
            //.opengl()
            .borderless()
            //.fullscreen()
            //.resizable()
            //.maximized()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let canvas = window
            .into_canvas()
            .accelerated()
            //.software()
            .present_vsync()
            //.target_texture()
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
        }
    }

    pub fn cls(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        //self.canvas.present();
    }
    pub fn present_canvas(&mut self) {
        optick::event!();
        self.canvas.present();
    }
    pub fn get_event_pump(&self) -> EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    pub fn get_frame_time(&self) -> u32 {
        self.sdl_context.timer().unwrap().ticks()
    }

    pub fn get_width(&self) -> u32 {
        self.w_width
    }
    pub fn get_height(&self) -> u32 {
        self.w_height
    }

    pub fn add_streaming_buffer(&mut self, name: &str, width: u32, height: u32) {
        optick::event!();
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

    pub fn clear_streaming_buffer(&mut self, name: &str, color: Color) {
        optick::event!();
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let pixelformat =
                PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
            let color = color.to_u32(&pixelformat);
            streaming_buffer
                .color_buffer
                .as_mut_slice_of::<u32>()
                .unwrap()
                .fill(color);
        }
    }

    pub fn streaming_buffer_to_canvas(&mut self, name: &str) {
        optick::event!();
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            // let width = streaming_buffer.texture.query().width;
            // streaming_buffer
            //     .texture
            //     .update(
            //         None,
            //         streaming_buffer.color_buffer.as_byte_slice(),
            //         width as usize * 4,
            //     )
            //     .unwrap();

            streaming_buffer
                .texture
                //streaming_texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    optick::event!();
                    buffer.copy_from_slice(&streaming_buffer.color_buffer);
                })
                .unwrap();

            self.canvas
                .copy(&streaming_buffer.texture, None, None)
                .unwrap();
        }
    }

    pub fn color_buffer_to_canvas(&mut self, name: &str, color_buffer: &[u8]) {
        optick::event!();
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            // let width = streaming_buffer.texture.query().width;
            // streaming_buffer
            //     .texture
            //     .update(None, color_buffer, width as usize * 4)
            //     .unwrap();
            streaming_buffer
                .texture
                //streaming_texture
                .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                    buffer.copy_from_slice(color_buffer);
                })
                .unwrap();
            self.canvas
                .copy(&streaming_buffer.texture, None, None)
                .unwrap();
        }
    }

    pub fn put_pixel_queue(&mut self, name: &str, pixel_queue: &[Pixel]) {
        optick::event!();
        // replaced "&Vec<Pixel>" with "&[Pixel]" ==> huge performance gain!!
        if let Some(streaming_buffer) = self.streaming_buffers.get_mut(name) {
            let width = streaming_buffer.texture.query().width;
            let height = streaming_buffer.texture.query().height;
            //let pixelformat =
            //    PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
            pixel_queue.iter().for_each(|pixel| {
                let x = pixel.x;
                let y = pixel.y;
                if x < 0 || x > (width - 1) as i32 || y < 0 || y > (height - 1) as i32 {
                    //()
                } else {
                    let a = pixel.color.a;
                    let r = pixel.color.r;
                    let g = pixel.color.g;
                    let b = pixel.color.b;
                    //let color: u32 = pixel.color.to_u32(&pixelformat); //ARGB8888
                    let color: u32 = u32::from_be_bytes([a, r, g, b]); //ARGB8888
                    let offset = ((y * width as i32) + (x)) as usize;
                    let pixel_data_u32 = streaming_buffer
                        .color_buffer
                        .as_mut_slice_of::<u32>()
                        .unwrap();
                    pixel_data_u32[offset] = color;
                }
            });
        }
    }

    pub(crate) fn get_aspect_ratio(&self) -> f32 {
        self.get_width() as f32 / self.get_height() as f32
    }
}
