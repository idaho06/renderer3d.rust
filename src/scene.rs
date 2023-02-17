use crate::display::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sequence {
    Quit,
    Fire,
}

pub trait Scene {
    fn update(&mut self, t: u32, display: &Display, scene: &Option<Sequence>);
    //fn update(&mut self, t: u32, display: &Display);
    fn render(&self, display: &mut Display);
}
