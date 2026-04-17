use crate::display::Display;

pub trait Scene {
    fn update(&mut self, delta_time: u32, display: &Display);
    fn render(&self, display: &mut Display);
}
