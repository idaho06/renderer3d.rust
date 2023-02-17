use byte_slice_cast::AsMutSliceOf;
use sdl2::pixels::Color;

use crate::{
    display::Display,
    scene::{Scene, Sequence},
};

pub struct Fire {
    width: u32,
    height: u32,
    color_buffer: Box<[u8]>,
    fire_buffer: Box<[u8]>,
    palette: Vec<Color>,
    rng: fastrand::Rng,
}

impl Fire {
    pub fn new(display: &mut Display) -> Self {
        let width = 320_u32;
        let height = 90_u32;
        let color_buffer = vec![0_u8; (width * height * 4) as usize];
        let fire_buffer = vec![0_u8; (width * height) as usize];
        let palette = vec![
            Color::RGB(7, 7, 7),
            Color::RGB(31, 7, 7),
            Color::RGB(47, 15, 7),
            Color::RGB(71, 15, 7),
            Color::RGB(87, 23, 7),
            Color::RGB(103, 31, 7),
            Color::RGB(119, 31, 7),
            Color::RGB(143, 39, 7),
            Color::RGB(159, 47, 7),
            Color::RGB(175, 63, 7),
            Color::RGB(191, 71, 7),
            Color::RGB(199, 71, 7),
            Color::RGB(223, 79, 7),
            Color::RGB(223, 87, 7),
            Color::RGB(223, 87, 7),
            Color::RGB(215, 95, 7),
            Color::RGB(215, 95, 7),
            Color::RGB(215, 103, 15),
            Color::RGB(207, 111, 15),
            Color::RGB(207, 119, 15),
            Color::RGB(207, 127, 15),
            Color::RGB(207, 135, 23),
            Color::RGB(199, 135, 23),
            Color::RGB(199, 143, 23),
            Color::RGB(199, 151, 31),
            Color::RGB(191, 159, 31),
            Color::RGB(191, 159, 31),
            Color::RGB(191, 167, 39),
            Color::RGB(191, 167, 39),
            Color::RGB(191, 175, 47),
            Color::RGB(183, 175, 47),
            Color::RGB(183, 183, 47),
            Color::RGB(183, 183, 55),
            Color::RGB(207, 207, 111),
            Color::RGB(223, 223, 159),
            Color::RGB(239, 239, 199),
            Color::RGB(255, 255, 255),
        ];
        display.add_streaming_buffer("fire", width, height);
        Self {
            width,
            height,
            color_buffer: color_buffer.into_boxed_slice(),
            fire_buffer: fire_buffer.into_boxed_slice(),
            palette,
            rng: fastrand::Rng::new(),
        }
    }

    // method to seed the fire_buffer randomly in the bottom row
    fn seed(&mut self) {
        optick::event!();
        for x in 0..self.width {
            let index = (x + (self.width * (self.height - 1))) as usize;
            //self.fire_buffer[index] = rand::random::<u8>() % 36;
            self.fire_buffer[index] = self.rng.u8(..36);
        }
    }

    fn update_fire_buffer(&mut self) {
        optick::event!();
        for x in 0..self.width {
            for y in 0..self.height {
                let index = (x + (self.width * y)) as usize;
                let decay = self.rng.u8(..3);
                //let decay = 1;
                let below = if y < self.height - 1 {
                    (x + (self.width * (y + 1))) as usize
                } else {
                    0
                };
                let new_value = if self.fire_buffer[below] > decay {
                    self.fire_buffer[below] - decay
                } else {
                    0
                };
                self.fire_buffer[index] = new_value;
            }
        }
    }
    fn update_color_buffer(&mut self) {
        optick::event!();
        //let pixelformat = PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
        let color_buffer_u32 = self.color_buffer.as_mut_slice_of::<u32>().unwrap();
        // for index in 0..self.fire_buffer.len() {
        //     let color_index = self.fire_buffer[index] as usize;
        //     let color = self.palette[color_index];
        //     //let color = color.to_u32(&pixelformat); // this is slow
        //     let color = u32::from_be_bytes([
        //         color.rgba().3, // alpha
        //         color.rgba().0, // red
        //         color.rgba().1, // green
        //         color.rgba().2, // blue
        //     ]);
        //     //self.color_buffer.as_mut_slice_of::<u32>().unwrap()[index] = color;
        //     color_buffer_u32[index] = color;
        // }
        for (index, item) in color_buffer_u32.iter_mut().enumerate()
        //.take(self.fire_buffer.len())
        {
            let color_index = self.fire_buffer[index] as usize;
            let color = self.palette[color_index];
            //let color = color.to_u32(&pixelformat); // this is slow
            let color = u32::from_be_bytes([
                color.rgba().3, // alpha
                color.rgba().0, // red
                color.rgba().1, // green
                color.rgba().2, // blue
            ]);
            //self.color_buffer.as_mut_slice_of::<u32>().unwrap()[index] = color;
            *item = color;
        }
    }
}

// implement Scene for Fire
impl Scene for Fire {
    fn update(&mut self, _t: u32, _display: &Display, _scene: &Option<Sequence>) {
        optick::event!();
        self.seed();
        // update fire_buffer
        self.update_fire_buffer();
        // update color_buffer
        self.update_color_buffer();
    }

    fn render(&self, display: &mut Display) {
        optick::event!();
        display.color_buffer_to_canvas("fire", &self.color_buffer);
    }
}
