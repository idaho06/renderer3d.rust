use sdl2::pixels::Color;

pub struct Pixel {
    pub x: i32,
    pub y: i32,
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
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Self { x, y, color }
    }
}
