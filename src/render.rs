use byte_slice_cast::{AsMutSliceOf, AsSliceOf};
use glam::{Vec2, Vec3, Vec4};
use sdl2::pixels::Color;

use crate::triangle::{Triangle, TriangleScreenPixel};

pub struct Render {
    corner_segments_x: Vec<(f32, f32)>,
    //short_segment2_x: Vec<(f32, f32)>, 
    straight_segment_x: Vec<(f32, f32)>,
    corner_segments_w: Vec<(f32, f32)>,
    //short_segment2_w: Vec<(f32, f32)>, 
    straight_segment_w: Vec<(f32, f32)>,
    corner_segments_u: Vec<(f32, f32)>,
    //short_segment2_u: Vec<(f32, f32)>, 
    straight_segment_u: Vec<(f32, f32)>,
    corner_segments_v: Vec<(f32, f32)>,
    //short_segment2_v: Vec<(f32, f32)>, 
    straight_segment_v: Vec<(f32, f32)>,
    //corner_segments: Vec<TriangleScreenPixel>,
    //straight_segment: Vec<TriangleScreenPixel>,
    horizontal_segment_w: Vec<(f32, f32)>,
    horizontal_segment_u: Vec<(f32, f32)>,
    horizontal_segment_v: Vec<(f32, f32)>,
    //horizontal_segment: Vec<TriangleScreenPixel>,
}

impl Render {
    pub fn new(cb_width: u32, cb_height: u32) -> Self {
        let cb_height = cb_height as usize;
        let cb_width = cb_width as usize;
        Self {
            corner_segments_x: vec![(0.0_f32, 0.0_f32); cb_height],
            //short_segment2_x: vec![(0.0_f32, 0.0_f32); cb_height],
            straight_segment_x: vec![(0.0_f32, 0.0_f32); cb_height],
            corner_segments_w: vec![(0.0_f32, 0.0_f32); cb_height],
            //short_segment2_w: vec![(0.0_f32, 0.0_f32); cb_height],
            straight_segment_w: vec![(0.0_f32, 0.0_f32); cb_height],
            corner_segments_u: vec![(0.0_f32, 0.0_f32); cb_height],
            //short_segment2_u: vec![(0.0_f32, 0.0_f32); cb_height],
            straight_segment_u: vec![(0.0_f32, 0.0_f32); cb_height],
            corner_segments_v: vec![(0.0_f32, 0.0_f32); cb_height],
            //short_segment2_v: vec![(0.0_f32, 0.0_f32); cb_height],
            straight_segment_v: vec![(0.0_f32, 0.0_f32); cb_height],
            //corner_segments: vec![TriangleScreenPixel::default(); cb_height],
            //straight_segment: vec![TriangleScreenPixel::default(); cb_height],
            horizontal_segment_w: vec![(0.0_f32, 0.0_f32); cb_width],
            horizontal_segment_u: vec![(0.0_f32, 0.0_f32); cb_width],
            horizontal_segment_v: vec![(0.0_f32, 0.0_f32); cb_width],
            //horizontal_segment: vec![TriangleScreenPixel::default(); cb_width],
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
        
            // invert v coordinate
            // let uv0 = Vec2::new(uv0.x, 1.0 - uv0.y);
            // let uv1 = Vec2::new(uv1.x, 1.0 - uv1.y);
            // let uv2 = Vec2::new(uv2.x, 1.0 - uv2.y);
            self.corner_segments_x.clear();
            self.straight_segment_x.clear();
            map_interpolate_float_mut(v0.y, v0.x, v1.y, v1.x, &mut self.corner_segments_x);
            self.corner_segments_x.pop();            
            map_interpolate_float_mut(v1.y, v1.x, v2.y, v2.x, &mut self.corner_segments_x);
            map_interpolate_float_mut(v0.y, v0.x, v2.y, v2.x, &mut self.straight_segment_x);
        
            // round x,y values because they represent pixel coordinates
            self.corner_segments_x.iter_mut().for_each(|(y, x)| {
                *x = x.round();
                *y = y.round();
            }
            );
            self.straight_segment_x.iter_mut().for_each(|(y, x)| {
                *x = x.round();
                *y = y.round();
            }
            );
        
        
            // now we calculate same segment for the reciprocal of the z in camera space (w) coordinate
            // to calculate the z coordinate of the interpolated points
            self.corner_segments_w.clear();
            self.straight_segment_w.clear();
            map_interpolate_float_mut(v0.y, 1.0 / v0.w, v1.y, 1.0 / v1.w, &mut self.corner_segments_w);
            self.corner_segments_w.pop();
            map_interpolate_float_mut(v1.y, 1.0 / v1.w, v2.y, 1.0 / v2.w, &mut self.corner_segments_w);
            map_interpolate_float_mut(v0.y, 1.0 / v0.w, v2.y, 1.0 / v2.w, &mut self.straight_segment_w);
        
            // now we calculate same segments for the u divided by the z in camera space (w) coordinate
            // to calculate the u coordinate of the interpolated points
            self.corner_segments_u.clear();
            self.straight_segment_u.clear();
            map_interpolate_float_mut(v0.y, uv0.x / v0.w, v1.y, uv1.x / v1.w, &mut self.corner_segments_u);
            self.corner_segments_u.pop();
            map_interpolate_float_mut(v1.y, uv1.x / v1.w, v2.y, uv2.x / v2.w, &mut self.corner_segments_u);
            map_interpolate_float_mut(v0.y, uv0.x / v0.w, v2.y, uv2.x / v2.w, &mut self.straight_segment_u);
        
            // now we calculate same segments for the v divided by the z in camera space (w) coordinate
            // to calculate the v coordinate of the interpolated points
            self.corner_segments_v.clear();
            self.straight_segment_v.clear();
            map_interpolate_float_mut(v0.y, uv0.y / v0.w, v1.y, uv1.y / v1.w, &mut self.corner_segments_v);
            self.corner_segments_v.pop();
            map_interpolate_float_mut(v1.y, uv1.y / v1.w, v2.y, uv2.y / v2.w, &mut self.corner_segments_v);
            map_interpolate_float_mut(v0.y, uv0.y / v0.w, v2.y, uv2.y / v2.w, &mut self.straight_segment_v);
        
            // corner_segments as vector of TriangleScreenPixel zipping corner_segments_x, corner_segments_w, corner_segments_u, corner_segments_v
            // self.corner_segments.clear();
            // self.corner_segments = self.corner_segments_x
            //     .iter()
            //     .zip(self.corner_segments_w.iter())
            //     .zip(self.corner_segments_u.iter())
            //     .zip(self.corner_segments_v.iter())
            //     .map(|((((y, x), (_, reciprocal_w)), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y: *y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w})
            //     .collect();
            let corner_segments_iter = self.corner_segments_x
            .iter()
            .zip(self.corner_segments_w.iter())
            .zip(self.corner_segments_u.iter())
            .zip(self.corner_segments_v.iter())
            .map(|((((y, x), (_, reciprocal_w)), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y: *y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w});
        
            // straight_segment as vector of TriangleScreenPixel zipping straight_segment_x, straight_segment_w, straight_segment_u, straight_segment_v
            // self.straight_segment.clear();
            // self.straight_segment = self.straight_segment_x
            //     .iter()
            //     .zip(self.straight_segment_w.iter())
            //     .zip(self.straight_segment_u.iter())
            //     .zip(self.straight_segment_v.iter())
            //     .map(|((((y, x), (_, reciprocal_w)), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y: *y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w})
            //     .collect();
            let straight_segment_iter = self.straight_segment_x
            .iter()
            .zip(self.straight_segment_w.iter())
            .zip(self.straight_segment_u.iter())
            .zip(self.straight_segment_v.iter())
            .map(|((((y, x), (_, reciprocal_w)), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y: *y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w});
                        
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
                    // let horizontal_segment_w = map_interpolate_float(start_x, start_w, end_x, end_w);
                    // let horizontal_segment_u = map_interpolate_float(start_x, start_u, end_x, end_u);
                    // let horizontal_segment_v = map_interpolate_float(start_x, start_v, end_x, end_v);
                    self.horizontal_segment_w.clear();
                    map_interpolate_float_mut(start_x, start_w, end_x, end_w, &mut self.horizontal_segment_w);
                    self.horizontal_segment_u.clear();
                    map_interpolate_float_mut(start_x, start_u, end_x, end_u, &mut self.horizontal_segment_u);
                    self.horizontal_segment_v.clear();
                    map_interpolate_float_mut(start_x, start_v, end_x, end_v, &mut self.horizontal_segment_v);
        
                    // self.horizontal_segment.clear();
                    // self.horizontal_segment = self.horizontal_segment_w
                    //     .iter()
                    //     .zip(self.horizontal_segment_u.iter())
                    //     .zip(self.horizontal_segment_v.iter())
                    //     .map(|(((x, reciprocal_w), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w})
                    //     .collect();
                    let horizontal_segment_iter = self.horizontal_segment_w
                    .iter()
                    .zip(self.horizontal_segment_u.iter())
                    .zip(self.horizontal_segment_v.iter())
                    .map(|(((x, reciprocal_w), (_, u_divided_w)), (_, v_divided_w))| TriangleScreenPixel{x: *x, y, reciprocal_w: *reciprocal_w, u_divided_w: *u_divided_w, v_divided_w: *v_divided_w});
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
                            let z = 1.0 / reciprocal_w;
                            let u = u_divided_w * z;
                            let v = v_divided_w * z;
                            let color = get_texture_color(texture, u, v, t_width, t_height);
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


// same as map_interpolate_float but using a mut Vec<f32, f32> instead of returning a new Vec
#[inline(always)]
fn map_interpolate_float_mut(i0: f32, d0: f32, i1: f32, d1: f32, values: &mut Vec<(f32, f32)>) {
    //optick::event!();
    if i0 == i1 {
        values.push((i0, d0));
        return;
    }
    let distance = (i1 - i0).abs();
    values.reserve(distance as usize);
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
#[inline(always)]
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
#[inline(always)]
fn get_texture_color(texture: &[u8], u: f32, v: f32, width: u32, height: u32) -> u32 {
    let u = u * width as f32;
    let v = v * height as f32;
    let u = u as u32;
    let v = v as u32;
    let u = u % width;
    let v = v % height;
    let index = (v * width + u) as usize;
    //let color = u32::from_le_bytes([texture[index * 4], texture[index * 4 + 1], texture[index * 4 + 2], texture[index * 4 + 3]]);
    let texture_u32 = texture.as_slice_of::<u32>().unwrap();
    texture_u32[index]
}
