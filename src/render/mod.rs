pub mod interpolate;
pub mod line;
pub mod rasterizer;
pub mod texture;

pub use line::{draw_2dtriangle_to_color_buffer, draw_2dline_to_color_buffer, put_pixel_to_color_buffer};
pub use rasterizer::draw_3dtriangle_to_color_buffer;
pub use texture::get_texture_color_argb_pow2_unchecked;

use glam::Vec3;
use sdl2::pixels::Color;

#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn calculate_face_color(light_dir: Vec3, normal: Vec3, color: Color) -> Color {
    let light_dir = light_dir * -1.0;
    let intensity = light_dir.dot(normal).clamp(0.0, 1.0);
    Color {
        r: (f32::from(color.r) * intensity) as u8,
        g: (f32::from(color.g) * intensity) as u8,
        b: (f32::from(color.b) * intensity) as u8,
        a: color.a,
    }
}
