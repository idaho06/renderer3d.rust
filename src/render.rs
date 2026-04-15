use byte_slice_cast::AsMutSliceOf;
use glam::{Vec2, Vec3, Vec4};
use sdl2::pixels::Color;

use crate::triangle::{Triangle, TriangleScreenPixel};

pub struct Render {}

impl Render {
    pub fn new(cb_width: u32, cb_height: u32) -> Self {
        let _cb_height = cb_height as usize;
        let _cb_width = cb_width as usize;
        Self {}
    }

    // implement draw_3dtriangle_to_color_buffer
    #[allow(clippy::too_many_arguments)]
    pub fn draw_3dtriangle_to_color_buffer(
        &mut self,
        triangle3d: &Triangle,
        color_buffer: &mut [u8],
        cb_width: u32,
        cb_height: u32,
        texture: &[u8],
        t_width: u32,
        t_height: u32,
        z_buffer: &mut [f32],
    ) {
        //optick::event!();
        if cb_width == 0 || cb_height == 0 {
            return;
        }

        let color_buffer_u32 = color_buffer.as_mut_slice_of::<u32>().unwrap();
        let cb_width_usize = cb_width as usize;
        let cb_height_usize = cb_height as usize;
        let buffer_len = cb_width_usize * cb_height_usize;
        debug_assert_eq!(color_buffer_u32.len(), buffer_len);
        debug_assert_eq!(z_buffer.len(), buffer_len);

        let cb_width_f32 = cb_width as f32;
        let cb_height_f32 = cb_height as f32;
        let max_x = cb_width_f32 - 1.0;
        // reorder the triangle vertices by the y coordinate
        let triangle3d = triangle3d.reorder_vertices_by_y();
        // let triangle_color = Vec4::new(
        //     triangle3d.color.a as f32,
        //     triangle3d.color.r as f32,
        //     triangle3d.color.g as f32,
        //     triangle3d.color.b as f32,
        // ) / 255.0;
        let triangle_color_array_u8 = [
            triangle3d.color.a,
            triangle3d.color.r,
            triangle3d.color.g,
            triangle3d.color.b,
        ];
        // assuming the longest segment is v0 to v2
        // we calculate the segments:
        // v0 to v1 and v1 to v2
        // v0 to v2
        let v0 = triangle3d.vertices[0];
        let v1 = triangle3d.vertices[1];
        let v2 = triangle3d.vertices[2];
        let uv0 = triangle3d.uvs[0];
        let uv1 = triangle3d.uvs[1];
        let uv2 = triangle3d.uvs[2];

        // round x and y coordinates to integers for v0, v1 and v2 and let the rest be floats
        let v0 = Vec4::new(v0.x.round(), v0.y.round(), v0.z, v0.w);
        let v1 = Vec4::new(v1.x.round(), v1.y.round(), v1.z, v1.w);
        let v2 = Vec4::new(v2.x.round(), v2.y.round(), v2.z, v2.w);

        let corner_segment1_iter = map_interpolate_float_vec4_iter(
            v0.y,
            Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
            v1.y,
            Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w),
        )
        .map(|(y, xwuv)| TriangleScreenPixel {
            x: xwuv.x.round(),
            y,
            reciprocal_w: xwuv.y,
            u_divided_w: xwuv.z,
            v_divided_w: xwuv.w,
        });
        let corner_segment2_iter = map_interpolate_float_vec4_iter(
            v1.y,
            Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w),
            v2.y,
            Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w),
        )
        .map(|(y, xwuv)| TriangleScreenPixel {
            x: xwuv.x.round(),
            y,
            reciprocal_w: xwuv.y,
            u_divided_w: xwuv.z,
            v_divided_w: xwuv.w,
        });
        let corner_segments_iter = corner_segment1_iter.chain(corner_segment2_iter.skip(1));

        let straight_segment_iter = map_interpolate_float_vec4_iter(
            v0.y,
            Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
            v2.y,
            Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w),
        )
        .map(|(y, xwuv)| TriangleScreenPixel {
            x: xwuv.x.round(),
            y,
            reciprocal_w: xwuv.y,
            u_divided_w: xwuv.z,
            v_divided_w: xwuv.w,
        });

        for (straight_segment_pixel, corner_segment_pixel) in
            straight_segment_iter.zip(corner_segments_iter)
        {
            let scanline_y = straight_segment_pixel.y;
            if scanline_y < 0.0 || scanline_y >= cb_height_f32 {
                continue;
            }

            let start_x = straight_segment_pixel.x;
            let end_x = corner_segment_pixel.x;
            let start_values = Vec3::new(
                straight_segment_pixel.reciprocal_w,
                straight_segment_pixel.u_divided_w,
                straight_segment_pixel.v_divided_w,
            );
            let end_values = Vec3::new(
                corner_segment_pixel.reciprocal_w,
                corner_segment_pixel.u_divided_w,
                corner_segment_pixel.v_divided_w,
            );

            let (raw_start_x, raw_end_x, raw_start_values, raw_end_values) = if start_x <= end_x {
                (start_x, end_x, start_values, end_values)
            } else {
                (end_x, start_x, end_values, start_values)
            };

            if raw_end_x < 0.0 || raw_start_x > max_x {
                continue;
            }

            let clamped_start_x = raw_start_x.max(0.0);
            let clamped_end_x = raw_end_x.min(max_x);
            if clamped_start_x > clamped_end_x {
                continue;
            }

            let span_step = if raw_start_x == raw_end_x {
                Vec3::ZERO
            } else {
                (raw_end_values - raw_start_values) / (raw_end_x - raw_start_x)
            };
            let mut span_values =
                raw_start_values + span_step * (clamped_start_x - raw_start_x);

            let start_x: i32;
            let end_x: i32;
            let y: i32;
            unsafe {
                start_x = clamped_start_x.to_int_unchecked();
                end_x = clamped_end_x.to_int_unchecked();
                y = scanline_y.to_int_unchecked();
            }

            let row_base = y as usize * cb_width_usize;

            for x in start_x..=end_x {
                let reciprocal_w = span_values.x;
                let z_index = row_base + x as usize;

                let z_buffer_w = unsafe { *z_buffer.get_unchecked(z_index) };

                if reciprocal_w > z_buffer_w {
                    unsafe {
                        *z_buffer.get_unchecked_mut(z_index) = reciprocal_w;
                    }

                    let u_divided_w = span_values.y;
                    let v_divided_w = span_values.z;
                    let w = 1.0 / reciprocal_w;
                    let u = u_divided_w * w;
                    let v = v_divided_w * w;
                    //let texture_color = get_texture_color_sdl2(texture, u, v, t_width, t_height); // <== this is slow!!
                    //unsafe {
                    //    (tr, tg, tb, ta) = get_texture_color_rgba_unsafe(texture, u, v, t_width, t_height);
                    //}
                    //let [tr, tg, tb, ta] = get_texture_color_u32(texture, u, v, t_width, t_height).to_be_bytes();
                    // let [tr, tg, tb, ta] =
                    //     get_texture_color_rgba(texture, u, v, t_width, t_height);
                    // let texture_color: Vec4 =
                    //     Vec4::new(ta as f32, tr as f32, tg as f32, tb as f32);
                    let texture_color_array_u8 =
                        get_texture_color_argb_pow2_unchecked(texture, u, v, t_width, t_height);
                    // multiply color by triangle3d color
                    // let a = triangle3d.color.a as f32 / 255.0;
                    // let r = triangle3d.color.r as f32 / 255.0;
                    // let g = triangle3d.color.g as f32 / 255.0;
                    // let b = triangle3d.color.b as f32 / 255.0;
                    // let color = Color::RGBA( //<== This is probably also slow
                    //     (tr as f32 * r ) as u8,
                    //     (tg as f32 * g ) as u8,
                    //     (tb as f32 * b ) as u8,
                    //     (ta as f32 * a) as u8,
                    // );
                    // let color: u32 = u32::from_be_bytes([
                    //     (*ta as f32 * a) as u8,
                    //     (*tr as f32 * r ) as u8,
                    //     (*tg as f32 * g ) as u8,
                    //     (*tb as f32 * b ) as u8]); //ARGB8888
                    //let color: Vec4 = texture_color * triangle_color;

                    
                    fn mul_u8_and_shift_right_8(a: u8, b: u8) -> u8 {
                        ((a as u16 * b as u16) >> 8) as u8
                    }

                    let a = mul_u8_and_shift_right_8(texture_color_array_u8[0], triangle_color_array_u8[0]);
                    let r = mul_u8_and_shift_right_8(texture_color_array_u8[1], triangle_color_array_u8[1]);
                    let g = mul_u8_and_shift_right_8(texture_color_array_u8[2], triangle_color_array_u8[2]);
                    let b = mul_u8_and_shift_right_8(texture_color_array_u8[3], triangle_color_array_u8[3]);
                    let color: u32;
                    unsafe {
                        color = u32::from_be_bytes([
                            a,
                            r,
                            g,
                            b,
                        ]); //ARGB8888
                        *color_buffer_u32.get_unchecked_mut(z_index) = color;
                    }
                    //color = u32::from_be_bytes([a as u8, r as u8, g as u8, b as u8]); //ARGB8888
                }

                span_values += span_step;
            }
        }
    }
}

// i = interval. Always change in +1 or -1 steps
// d = displacement. Fractional increment, rounded to integer.
#[inline]
fn map_interpolate_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<(i32, i32)> {
    //
    //optick::event!();
    if i0 == i1 {
        return vec![(i0, d0)];
    }
    let distance = (i1 - i0).unsigned_abs(); // clippy change as usize to .unsigned_abs()
    let mut values: Vec<(i32, i32)> = Vec::with_capacity(distance.try_into().unwrap()); // clippy convert to usize and panic in case of error
    let mut a: f32 = (d1 as f32 - d0 as f32) / (i1 as f32 - i0 as f32);
    // clippy warning: unneeded late initialization
    let step: i32 = if i1 > i0 { 1 } else { -1 };
    if step == -1 {
        a = -a;
    } // change sign of a if we are going backwards
    let mut i = i0;
    let mut d = d0 as f32;
    loop {
        values.push((i, d as i32));

        d += a;
        i += step;

        if step == 1 && i > i1 {
            break;
        }
        if step == -1 && i < i1 {
            break;
        }
    }

    values
}

// function interpolate_float
// same as interpolate_int but with f32 instead of i32
// i = interval. Always change in +1.0 or -1.0 steps
// d = displacement. Fractional increment
// #[inline(always)]
// fn map_interpolate_float(i0: f32, d0: f32, i1: f32, d1: f32) -> Vec<(f32, f32)> {
//     //optick::event!();
//     if i0 == i1 {
//         return vec![(i0, d0)];
//     }
//     let distance = (i1 - i0).abs();
//     let mut values: Vec<(f32, f32)> = Vec::with_capacity(distance as usize);
//     let mut a: f32 = (d1 - d0) / (i1 - i0);
//     let step: f32 = if i1 > i0 { 1.0 } else { -1.0 };
//     if step == -1.0 {
//         a = -a;
//     } // change sign of a if we are going backwards
//     let mut i = i0;
//     let mut d = d0;
//     loop {
//         values.push((i, d));

//         d += a;
//         i += step;

//         if step == 1.0 && i > i1 {
//             break;
//         }
//         if step == -1.0 && i < i1 {
//             break;
//         }
//     }

//     values
// }

#[inline]
fn map_interpolate_float_vec4_iter(
    i0: f32,
    d0: Vec4,
    i1: f32,
    d1: Vec4,
) -> impl Iterator<Item = (f32, Vec4)> {
    //optick::event!();
    //let mut a = (d1 - d0) / (i1 - i0);
    let forward = i1 > i0;
    let mut a = if i1 == i0 {
        Vec4::ZERO
    } else {
        (d1 - d0) / (i1 - i0)
    };
    if !forward {
        a = Vec4::ZERO - a;
    } // change sign of a if we are going backwards
    let step = if forward { 1.0 } else { -1.0 };
    let mut i = i0;
    let mut d = d0;
    std::iter::from_fn(move || {
        let result = Some((i, d));
        if (forward && i > i1) || (!forward && i < i1) {
            None
        } else {
            d += a;
            i += step;
            result
        }
    })
}

// draw a 2d line to the passed color buffer
pub fn draw_2dline_to_color_buffer(
    point1: &Vec2,
    point2: &Vec2,
    color_buffer: &mut [u8],
    width: u32,
    height: u32,
    color: &Color,
) {
    // get integer coordinates from point1 and point2
    let x0 = point1.x as i32;
    let y0 = point1.y as i32;
    let x1 = point2.x as i32;
    let y1 = point2.y as i32;
    // get delta x and delta y
    let dx = x1 - x0;
    let dy = y1 - y0;

    // fast transform color to u32
    let color_u32: u32 = u32::from_be_bytes([color.a, color.r, color.g, color.b]); //ARGB8888
    // cast color buffer to u32
    let color_buffer_u32 = color_buffer.as_mut_slice_of::<u32>().unwrap();

    if dx.abs() > dy.abs() {
        // horizontal-ish
        /* for (x, y) in self.interpolate_int(x0, y0, x1, y1).iter() {
            self.put_pixel(name, *x, *y, r, g, b);
        } */
        map_interpolate_int(x0, y0, x1, y1)
            .iter()
            .for_each(|(x, y)| 
                //self.put_pixel(name, *x, *y, r, g, b)
                // if *x < 0 || *x > (width - 1) as i32 || *y < 0 || *y > (height - 1) as i32 {
                    
                // } else {
                //     let index = (*y * width as i32 + *x) as usize;
                //     color_buffer_u32[index] = color_u32;
                // }
                put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height));
    } else {
        // vertical-ish
        /* for (y, x) in self.interpolate_int(y0, x0, y1, x1).iter() {
            self.put_pixel(name, *x, *y, r, g, b);
        } */
        map_interpolate_int(y0, x0, y1, x1)
            .iter()
            .for_each(|(y, x)| 
                //self.put_pixel(name, *x, *y, r, g, b)
                // if *x < 0 || *x > (width - 1) as i32 || *y < 0 || *y > (height - 1) as i32 {
                    
                // } else {
                //     let index = (*y * width as i32 + *x) as usize;
                //     color_buffer_u32[index] = color_u32;
                // }
                put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height));
    }
}

// put pixel in the color buffer using x, y, color in u32, color_buffer in &[u32], width and height
#[inline]
pub fn put_pixel_to_color_buffer(
    x: i32,
    y: i32,
    color: u32,
    color_buffer: &mut [u32],
    width: u32,
    height: u32,
) {
    if x < 0 || x > (width - 1) as i32 || y < 0 || y > (height - 1) as i32 {
    } else {
        let index = (y * width as i32 + x) as usize;
        //assert!(index<color_buffer.len());
        color_buffer[index] = color;
    }
}

// draw a filled 2d triangle to the passed color buffer
pub fn draw_2dtriangle_to_color_buffer(
    triangle2d: &Triangle,
    color_buffer: &mut [u8],
    width: u32,
    height: u32,
) {
    // extract x and y of the vertices of the triangle
    let x0 = triangle2d.vertices[0].x as i32;
    let y0 = triangle2d.vertices[0].y as i32;
    let x1 = triangle2d.vertices[1].x as i32;
    let y1 = triangle2d.vertices[1].y as i32;
    let x2 = triangle2d.vertices[2].x as i32;
    let y2 = triangle2d.vertices[2].y as i32;

    // order the vertices by y
    let (x0, y0, x1, y1, x2, y2) = if y0 < y1 {
        if y1 < y2 {
            (x0, y0, x1, y1, x2, y2)
        } else if y0 < y2 {
            (x0, y0, x2, y2, x1, y1)
        } else {
            (x2, y2, x0, y0, x1, y1)
        }
    } else if y0 < y2 {
        (x1, y1, x0, y0, x2, y2)
    } else if y1 < y2 {
        (x1, y1, x2, y2, x0, y0)
    } else {
        (x2, y2, x1, y1, x0, y0)
    };

    // assuming that the longest segment is x0,y0 to x2,y2
    // we calculate the segments:
    // x0,y0 to x1,y1 and x1,y1 to x2,y2
    // x0,y0 to x2,y2
    let straigt_segment = map_interpolate_int(y0, x0, y2, x2);
    let mut corner_segments = map_interpolate_int(y0, x0, y1, x1);
    let mut short_segment2 = map_interpolate_int(y1, x1, y2, x2);

    // remove the last point of the first segment and add the short segment
    corner_segments.pop();
    corner_segments.append(&mut short_segment2);

    // iterate the long segment, zip with the corner segment and draw a line for each couple of points
    straigt_segment
        .iter()
        .zip(corner_segments.iter())
        .for_each(|((y1,x1),(y2,x2))| 
            // TODO: change to draw an horizontal line 2d
            draw_2dline_to_color_buffer(
                &Vec2::new(*x1 as f32, *y1 as f32),
                &Vec2::new(*x2 as f32, *y2 as f32),
                color_buffer,
                width,
                height,
                &triangle2d.color
            ));
}

pub fn calculate_face_color(light_dir: Vec3, normal: Vec3, color: Color) -> Color {
    //optick::event!();
    //let light_dir = light_dir.normalize();
    //let normal = normal.normalize();
    // invert light direction
    let light_dir = light_dir * -1.0;
    let intensity = light_dir.dot(normal);
    // clamp intensity to 0.0 - 1.0
    let intensity = intensity.clamp(0.0, 1.0);
    let r = (color.r as f32 * intensity) as u8;
    let g = (color.g as f32 * intensity) as u8;
    let b = (color.b as f32 * intensity) as u8;
    Color {
        r,
        g,
        b,
        a: color.a,
    }
}

// returns the color in u32 format from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height
// #[inline]
// fn get_texture_color_u32(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> u32 {
//     let u = u * width as f32;
//     let v = v * height as f32;
//     let u = u as u32;
//     let v = v as u32;
//     let u = u % width;
//     let v = v % height;
//     let index = (v * width + u) as usize;
//     //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
//     let texture_u32 = texture.as_slice_of::<u32>().unwrap();
//     assert!(index<texture_u32.len());
//     texture_u32[index]
//     //u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]])
// }

// returns the color in sdl2::pixels::Color type from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height
// #[inline]
// fn get_texture_color_sdl2(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> sdl2::pixels::Color {
//     let u = u * width as f32;
//     let v = v * height as f32;
//     let u = u as u32;
//     let v = v as u32;
//     let u = u % width;
//     let v = v % height;
//     let index = (v * width + u) as usize;
//     //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
//     //let color = color.to_be_bytes();
//     assert!(index * 4 + 3 < texture.len());
//     let b = texture[index * 4];
//     let g = texture[index * 4 + 1];
//     let r = texture[index * 4 + 2];
//     let a = texture[index * 4 + 3];
//     sdl2::pixels::Color::RGBA(r, g, b, a)
// }

// returns the color in sdl2::pixels::Color type from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height

#[allow(dead_code)]
#[inline]
fn get_texture_color_rgba(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    let u = (u * width as f32) as u32 % width;
    let v = (v * height as f32) as u32 % height;
    let index = ((v * width + u) * 4) as usize;
    if let Some([b, g, r, a]) = texture.get(index..(index + 4)) {
        [*r, *g, *b, *a]
    } else {
        [255, 0, 255, 255]
    }
}

#[allow(dead_code)]
#[inline]
fn get_texture_color_argb(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    let u = (u * width as f32) as u32 % width;
    let v = (v * height as f32) as u32 % height;
    let index = ((v * width + u) * 4) as usize;
    if let Some([b, g, r, a]) = texture.get(index..(index + 4)) {
        [*a, *r, *g, *b]
    } else {
        [255, 255, 0, 255]
    }
}

#[inline]
fn get_texture_color_argb_pow2_unchecked(
    texture: &[u8],
    u: f32,
    v: f32,
    width: u32,
    height: u32,
) -> [u8; 4] {
    debug_assert!(width.is_power_of_two());
    debug_assert!(height.is_power_of_two());
    debug_assert_eq!(texture.len(), width as usize * height as usize * 4);

    let u = ((u * width as f32) as u32) & (width - 1);
    let v = ((v * height as f32) as u32) & (height - 1);
    let index = ((v * width + u) * 4) as usize;

    unsafe {
        [
            *texture.get_unchecked(index + 3),
            *texture.get_unchecked(index + 2),
            *texture.get_unchecked(index + 1),
            *texture.get_unchecked(index),
        ]
    }
}


// #[inline]
// unsafe fn get_texture_color_rgba_unsafe(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> (u8,u8,u8,u8) {
//     let u = (u * width as f32) as u32 % width;
//     let v = (v * height as f32) as u32 % height;
//     let index = ((v * width + u) * 4) as usize;

//     let [b,g,r,a] = texture.get_unchecked(index..(index+4)) else { std::hint::unreachable_unchecked() };

//     (*r, *g, *b, *a)
// }
