pub struct Framebuffer {
    pub color_buffer: Box<[u8]>,
    pub z_buffer: Box<[f32]>,
    pub width: u32,
    pub height: u32,
}

impl Framebuffer {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            color_buffer: vec![0u8; pixel_count * 4].into_boxed_slice(),
            z_buffer: vec![0.0f32; pixel_count].into_boxed_slice(),
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.fill(0u8);
        self.z_buffer.fill(0.0f32);
    }
}
