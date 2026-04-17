use byte_slice_cast::AsMutSliceOf;
use glam::Vec2;
use sdl2::pixels::Color;

use crate::triangle::Triangle;
use super::interpolate::map_interpolate_int;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::needless_for_each,
    clippy::missing_panics_doc
)]
pub fn draw_2dline_to_color_buffer(
    point1: &Vec2,
    point2: &Vec2,
    color_buffer: &mut [u8],
    width: u32,
    height: u32,
    color: &Color,
) {
    let x0 = point1.x as i32;
    let y0 = point1.y as i32;
    let x1 = point2.x as i32;
    let y1 = point2.y as i32;
    let dx = x1 - x0;
    let dy = y1 - y0;

    let color_u32: u32 = u32::from_be_bytes([color.a, color.r, color.g, color.b]);
    let color_buffer_u32 = color_buffer.as_mut_slice_of::<u32>().unwrap();

    if dx.abs() > dy.abs() {
        map_interpolate_int(x0, y0, x1, y1)
            .iter()
            .for_each(|(x, y)| put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height));
    } else {
        map_interpolate_int(y0, x0, y1, x1)
            .iter()
            .for_each(|(y, x)| put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height));
    }
}

#[inline]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub fn put_pixel_to_color_buffer(
    x: i32,
    y: i32,
    color: u32,
    color_buffer: &mut [u32],
    width: u32,
    height: u32,
) {
    if x >= 0 && x <= (width - 1) as i32 && y >= 0 && y <= (height - 1) as i32 {
        color_buffer[(y * width as i32 + x) as usize] = color;
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::needless_for_each
)]
pub fn draw_2dtriangle_to_color_buffer(
    triangle2d: &Triangle,
    color_buffer: &mut [u8],
    width: u32,
    height: u32,
) {
    let x0 = triangle2d.vertices[0].x as i32;
    let y0 = triangle2d.vertices[0].y as i32;
    let x1 = triangle2d.vertices[1].x as i32;
    let y1 = triangle2d.vertices[1].y as i32;
    let x2 = triangle2d.vertices[2].x as i32;
    let y2 = triangle2d.vertices[2].y as i32;

    let (x0, y0, x1, y1, x2, y2) = if y0 < y1 {
        if y1 < y2 { (x0, y0, x1, y1, x2, y2) }
        else if y0 < y2 { (x0, y0, x2, y2, x1, y1) }
        else { (x2, y2, x0, y0, x1, y1) }
    } else if y0 < y2 {
        (x1, y1, x0, y0, x2, y2)
    } else if y1 < y2 {
        (x1, y1, x2, y2, x0, y0)
    } else {
        (x2, y2, x1, y1, x0, y0)
    };

    let straight_segment = map_interpolate_int(y0, x0, y2, x2);
    let mut corner_segments = map_interpolate_int(y0, x0, y1, x1);
    let mut short_segment2 = map_interpolate_int(y1, x1, y2, x2);
    corner_segments.pop();
    corner_segments.append(&mut short_segment2);

    straight_segment
        .iter()
        .zip(corner_segments.iter())
        .for_each(|((y1, x1), (y2, x2))| {
            draw_2dline_to_color_buffer(
                &Vec2::new(*x1 as f32, *y1 as f32),
                &Vec2::new(*x2 as f32, *y2 as f32),
                color_buffer,
                width,
                height,
                &triangle2d.color,
            );
        });
}
