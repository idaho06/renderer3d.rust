use byte_slice_cast::{AsMutSliceOf, AsSliceOf};
use glam::{Vec2, Vec3, Vec4};
use sdl2::pixels::Color;

use crate::triangle::{Triangle, TriangleScreenPixel};

pub struct Render {
}

impl Render {
    pub fn new(cb_width: u32, cb_height: u32) -> Self {
        let _cb_height = cb_height as usize;
        let _cb_width = cb_width as usize;
        Self {
        }
    }

    // implement draw_3dtriangle_to_color_buffer
    pub fn draw_3dtriangle_to_color_buffer(
        &mut self,
        triangle3d: &Triangle,
        color_buffer: &mut [u8],
        cb_width: u32,
        cb_height: u32,
        texture: &[u8],
        t_width: u32,
        t_height: u32,
        z_buffer: &mut [f32]
        ) {
            optick::event!();
            let color_buffer_u32 = color_buffer.as_mut_slice_of::<u32>().unwrap();
            // reorder the triangle vertices by the y coordinate
            let triangle3d = triangle3d.reorder_vertices_by_y();
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
                Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w))
                .map(|(y, xwuv)| TriangleScreenPixel{x: xwuv.x.round(), y, reciprocal_w: xwuv.y, u_divided_w: xwuv.z, v_divided_w: xwuv.w});
            let corner_segment2_iter = map_interpolate_float_vec4_iter(
                v1.y, 
                Vec4::new(v1.x, 1.0 / v1.w, uv1.x / v1.w, uv1.y / v1.w), 
                v2.y, 
                Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w))
                .map(|(y, xwuv)| TriangleScreenPixel{x: xwuv.x.round(), y, reciprocal_w: xwuv.y, u_divided_w: xwuv.z, v_divided_w: xwuv.w});
            let corner_segments_iter = corner_segment1_iter.chain(corner_segment2_iter.skip(1));
        
            let straight_segment_iter = map_interpolate_float_vec4_iter(
                v0.y, 
                Vec4::new(v0.x, 1.0 / v0.w, uv0.x / v0.w, uv0.y / v0.w),
                v2.y, 
                Vec4::new(v2.x, 1.0 / v2.w, uv2.x / v2.w, uv2.y / v2.w))
                .map(|(y, xwuv)| TriangleScreenPixel{x: xwuv.x.round(), y, reciprocal_w: xwuv.y, u_divided_w: xwuv.z, v_divided_w: xwuv.w});

            straight_segment_iter
                .zip(corner_segments_iter)
                .for_each(|(straight_segment_pixel, corner_segment_pixel)| {
                    let start_x = straight_segment_pixel.x;
                    let end_x = corner_segment_pixel.x;
                    let start_w = straight_segment_pixel.reciprocal_w;
                    let end_w = corner_segment_pixel.reciprocal_w;
                    let start_u = straight_segment_pixel.u_divided_w;
                    let end_u = corner_segment_pixel.u_divided_w;
                    let start_v = straight_segment_pixel.v_divided_w;
                    let end_v = corner_segment_pixel.v_divided_w;
                    let y = straight_segment_pixel.y;
        
                    let horizontal_segment_iter = map_interpolate_float_vec4_iter(
                        start_x, 
                        Vec4::new(y, start_w, start_u, start_v), 
                        end_x, 
                        Vec4::new(y, end_w, end_u, end_v))
                        .map(|(x, v)| TriangleScreenPixel{x, y, reciprocal_w: v.y, u_divided_w: v.z, v_divided_w: v.w});

                    horizontal_segment_iter
                    .filter(|horizontal_segment_pixel| 
                        horizontal_segment_pixel.x >= 0.0_f32 && 
                        horizontal_segment_pixel.x < cb_width as f32 &&
                        horizontal_segment_pixel.y >= 0.0_f32 &&
                        horizontal_segment_pixel.y < cb_height as f32)
                    .for_each(|horizontal_segment_pixel| {
                        let x = horizontal_segment_pixel.x;
                        let y = horizontal_segment_pixel.y;
                        let reciprocal_w = horizontal_segment_pixel.reciprocal_w;
                        if reciprocal_w > z_buffer[(y as usize * cb_width as usize + x as usize)] {
                            z_buffer[(y as usize * cb_width as usize + x as usize)] = reciprocal_w;
                            let u_divided_w = horizontal_segment_pixel.u_divided_w;
                            let v_divided_w = horizontal_segment_pixel.v_divided_w;
                            let w = 1.0 / reciprocal_w;
                            let u = u_divided_w * w;
                            let v = v_divided_w * w;
                            //let texture_color = get_texture_color_sdl2(texture, u, v, t_width, t_height); // <== this is slow!!
                            let (tr, tg, tb, ta) = get_texture_color_rgba(texture, u, v, t_width, t_height);
                            //let [tr, tg, tb, ta] = get_texture_color_u32(texture, u, v, t_width, t_height).to_be_bytes();
                            // multiply color by triangle3d color
                            let a = triangle3d.color.a as f32 / 255.0;
                            let r = triangle3d.color.r as f32 / 255.0;
                            let g = triangle3d.color.g as f32 / 255.0;
                            let b = triangle3d.color.b as f32 / 255.0;
                            // let color = Color::RGBA( //<== This is probably also slow
                            //     (tr as f32 * r ) as u8,
                            //     (tg as f32 * g ) as u8,
                            //     (tb as f32 * b ) as u8,
                            //     (ta as f32 * a) as u8,
                            // );
                            let color: u32 = u32::from_be_bytes([
                                (ta as f32 * a) as u8, 
                                (tr as f32 * r ) as u8, 
                                (tg as f32 * g ) as u8, 
                                (tb as f32 * b ) as u8]); //ARGB8888
                            
                            put_pixel_to_color_buffer(x as i32, y as i32, color, color_buffer_u32, cb_width, cb_height)
                        }
                    });
                });
        }
}


// i = interval. Always change in +1 or -1 steps
// d = displacement. Fractional increment, rounded to integer.
#[inline(always)]
fn map_interpolate_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<(i32, i32)> {
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
#[inline(always)]
fn map_interpolate_float(i0: f32, d0: f32, i1: f32, d1: f32) -> Vec<(f32, f32)> {
    //optick::event!();
    if i0 == i1 {
        return vec![(i0, d0)];
    }
    let distance = (i1 - i0).abs();
    let mut values: Vec<(f32, f32)> = Vec::with_capacity(distance as usize);
    let mut a: f32 = (d1 - d0) / (i1 - i0);
    let step: f32 = if i1 > i0 { 1.0 } else { -1.0 };
    if step == -1.0 {
        a = -a;
    } // change sign of a if we are going backwards
    let mut i = i0;
    let mut d = d0;
    loop {
        values.push((i, d));

        d += a;
        i += step;

        if step == 1.0 && i > i1 {
            break;
        }
        if step == -1.0 && i < i1 {
            break;
        }
    }

    values
}

#[inline]
fn map_interpolate_float_vec4_iter(
    i0: f32,
    d0: Vec4,
    i1: f32,
    d1: Vec4,
) -> impl Iterator<Item = (f32, Vec4)> {
    //optick::event!();
    //let mut a = (d1 - d0) / (i1 - i0);
    let mut a = if i1 == i0 {
        Vec4::ZERO
    } else {
        (d1 - d0) / (i1 - i0)
    };
    let step: f32 = if i1 > i0 { 1.0 } else { -1.0 };
    if step == -1.0 {
        a = Vec4::ZERO - a;
    } // change sign of a if we are going backwards
    let mut i = i0;
    let mut d = d0;
    std::iter::from_fn(move || {
        let result = Some((i, d)); 
        if (step == 1.0 && i > i1) || (step == -1.0 && i < i1) {
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
    //optick::event!();
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
                put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height)
            );
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
                put_pixel_to_color_buffer(*x, *y, color_u32, color_buffer_u32, width, height)
            );
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
        assert!(index<color_buffer.len());
        color_buffer[index] = color;
    }
}



// draw a filled 2d triangle to the passed color buffer
pub fn draw_2dtriangle_to_color_buffer(
    triangle2d: &Triangle,
    color_buffer: &mut [u8],
    width: u32,
    height: u32
) {
    optick::event!();
    // extract x and y of the vertices of the triangle
    let x0 = triangle2d.vertices[0].x as i32;
    let y0 = triangle2d.vertices[0].y as i32;
    let x1 = triangle2d.vertices[1].x as i32;
    let y1 = triangle2d.vertices[1].y as i32;
    let x2 = triangle2d.vertices[2].x as i32;
    let y2 = triangle2d.vertices[2].y as i32;

    // order the vertices by y
    let (x0,y0,x1,y1,x2,y2) = if y0 < y1 {
        if y1 < y2 {
            (x0,y0,x1,y1,x2,y2)
        } else if y0 < y2 {
            (x0,y0,x2,y2,x1,y1)
        } else {
            (x2,y2,x0,y0,x1,y1)
        }
    } else if y0 < y2 {
        (x1,y1,x0,y0,x2,y2)
    } else if y1 < y2 {
        (x1,y1,x2,y2,x0,y0)
    } else {
        (x2,y2,x1,y1,x0,y0)
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
            )
        ); 

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
    Color{r,g,b,a:color.a}
}
        
// returns the color in u32 format from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height
#[inline]
fn get_texture_color_u32(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> u32 {
    let u = u * width as f32;
    let v = v * height as f32;
    let u = u as u32;
    let v = v as u32;
    let u = u % width;
    let v = v % height;
    let index = (v * width + u) as usize;
    //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
    let texture_u32 = texture.as_slice_of::<u32>().unwrap();
    assert!(index<texture_u32.len());
    texture_u32[index]
    //u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]])
}

// returns the color in sdl2::pixels::Color type from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height
#[inline]
fn get_texture_color_sdl2(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> sdl2::pixels::Color {
    let u = u * width as f32;
    let v = v * height as f32;
    let u = u as u32;
    let v = v as u32;
    let u = u % width;
    let v = v % height;
    let index = (v * width + u) as usize;
    //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
    //let color = color.to_be_bytes();
    assert!(index * 4 + 3 < texture.len());
    let b = texture[index * 4];
    let g = texture[index * 4 + 1];
    let r = texture[index * 4 + 2];
    let a = texture[index * 4 + 3];
    sdl2::pixels::Color::RGBA(r, g, b, a)
}

// returns the color in sdl2::pixels::Color type from a texture in &[u8] format
// using coordinates u and v
// and texture size width and height
#[inline]
fn get_texture_color_rgba(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> (u8,u8,u8,u8) {
    let u = u * width as f32;
    let v = v * height as f32;
    let u = u as u32;
    let v = v as u32;
    let u = u % width;
    let v = v % height;
    let index = (((v * width + u) * 4) + 3) as usize;
    //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
    //let color = color.to_be_bytes();
    assert!(index < texture.len());
    assert!(index > 2);
    let b = texture[index - 3];
    let g = texture[index - 2];
    let r = texture[index - 1];
    let a = texture[index];
    (r, g, b, a)
}