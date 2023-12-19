use crate::triangle::Triangle;
use glam::{Vec2, Vec4};
//use smallvec::SmallVec;

pub enum TriangleClipResult {
    OneTriangle(Triangle),
    TwoTriangles(Triangle, Triangle),
    NoTriangle,
}

pub fn clip_triangle_w_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, 0.001, |v| v.w, |w, plane| w > plane)
}

pub fn clip_triangle_x_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, -1.0, |v| v.x, |x, plane| x > plane)
}

pub fn clip_triangle_y_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, -1.0, |v| v.y, |y, plane| y > plane)
}

pub fn clip_triangle_z_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, -1.0, |v| v.z, |z, plane| z > plane)
}

pub fn clip_triangle_nx_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, 1.0, |v| v.x, |x, plane| x < plane)
}

pub fn clip_triangle_ny_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, 1.0, |v| v.y, |y, plane| y < plane)
}

pub fn clip_triangle_nz_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_axis(triangle, 1.0, |v| v.z, |z, plane| z < plane)
}

fn clip_triangle_on_w_axis(triangle: Triangle) -> TriangleClipResult {
    // TODO: Use smallvec for inside_points and outside_points
    let mut inside_points: Vec<(&Vec4, &Vec2)> = Vec::new();
    let mut outside_points: Vec<(&Vec4, &Vec2)> = Vec::new();

    for i in 0..3 {
        if triangle.vertices[i].w > 0.0 {
            inside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        } else {
            outside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        }
    }

    match inside_points.len() {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            // The inside vertex is A, and the outside vertices are B and C
            // We are going to create a new Triangle with vertices A, B', and C'
            // where B' and C' are the intersection points of the line AB and AC
            // with the plane w = 0
            let a = inside_points[0];
            let b = outside_points[0];
            let c = outside_points[1];
            let factor_ab = (0.0 - a.0.w) / (b.0.w - a.0.w);
            let factor_ac = (0.0 - a.0.w) / (c.0.w - a.0.w);
            let b_prime = (a.0.lerp(*b.0, factor_ab), a.1.lerp(*b.1, factor_ab));
            let c_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b_prime.0.clone(), c_prime.0.clone()],
                [a.1.clone(), b_prime.1.clone(), c_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            // The inside vertices are A and B, and the outside vertex is C
            // We are going to create two new Triangles with vertices A, B, and A'
            // and A', B, and B' where A' and B' are the intersection points of the
            // line AC and BC with the plane w = 0
            let a = inside_points[0];
            let b = inside_points[1];
            let c = outside_points[0];
            let factor_ac = (0.0 - a.0.w) / (c.0.w - a.0.w);
            let factor_bc = (0.0 - b.0.w) / (c.0.w - b.0.w);
            let a_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));
            let b_prime = (b.0.lerp(*c.0, factor_bc), b.1.lerp(*c.1, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b.0.clone(), a_prime.0.clone()],
                [a.1.clone(), b.1.clone(), a_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0.clone(), b.0.clone(), b_prime.0.clone()],
                [a_prime.1.clone(), b.1.clone(), b_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_points.len() is not 0, 1, 2, or 3"),
    }
}

fn clip_triangle_on_plus_x_axis(triangle: Triangle) -> TriangleClipResult {
    let mut inside_points: Vec<(&Vec4, &Vec2)> = Vec::new();
    let mut outside_points: Vec<(&Vec4, &Vec2)> = Vec::new();

    for i in 0..3 {
        if triangle.vertices[i].x > -1.0 {
            inside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        } else {
            outside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        }
    }

    match inside_points.len() {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            // The inside vertex is A, and the outside vertices are B and C
            // We are going to create a new Triangle with vertices A, B', and C'
            // where B' and C' are the intersection points of the line AB and AC
            // with the plane x = -1
            let a = inside_points[0];
            let b = outside_points[0];
            let c = outside_points[1];
            let factor_ab = (-1.0 - a.0.x) / (b.0.x - a.0.x);
            let factor_ac = (-1.0 - a.0.x) / (c.0.x - a.0.x);
            let b_prime = (a.0.lerp(*b.0, factor_ab), a.1.lerp(*b.1, factor_ab));
            let c_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b_prime.0.clone(), c_prime.0.clone()],
                [a.1.clone(), b_prime.1.clone(), c_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            // The inside vertices are A and B, and the outside vertex is C
            // We are going to create two new Triangles with vertices A, B, and A'
            // and A', B, and B' where A' and B' are the intersection points of the
            // line AC and BC with the plane x = -1
            let a = inside_points[0];
            let b = inside_points[1];
            let c = outside_points[0];
            let factor_ac = (-1.0 - a.0.x) / (c.0.x - a.0.x);
            let factor_bc = (-1.0 - b.0.x) / (c.0.x - b.0.x);
            let a_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));
            let b_prime = (b.0.lerp(*c.0, factor_bc), b.1.lerp(*c.1, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b.0.clone(), a_prime.0.clone()],
                [a.1.clone(), b.1.clone(), a_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0.clone(), b.0.clone(), b_prime.0.clone()],
                [a_prime.1.clone(), b.1.clone(), b_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_points.len() is not 0, 1, 2, or 3"),
    }
}

fn clip_triangle_on_minus_x_axis(triangle: Triangle) -> TriangleClipResult {
    let mut inside_points: Vec<(&Vec4, &Vec2)> = Vec::new();
    let mut outside_points: Vec<(&Vec4, &Vec2)> = Vec::new();

    for i in 0..3 {
        if triangle.vertices[i].x < 1.0 {
            inside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        } else {
            outside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        }
    }

    match inside_points.len() {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            // The inside vertex is A, and the outside vertices are B and C
            // We are going to create a new Triangle with vertices A, B', and C'
            // where B' and C' are the intersection points of the line AB and AC
            // with the plane x = +1
            let a = inside_points[0];
            let b = outside_points[0];
            let c = outside_points[1];
            let factor_ab = (1.0 - a.0.x) / (b.0.x - a.0.x);
            let factor_ac = (1.0 - a.0.x) / (c.0.x - a.0.x);
            let b_prime = (a.0.lerp(*b.0, factor_ab), a.1.lerp(*b.1, factor_ab));
            let c_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b_prime.0.clone(), c_prime.0.clone()],
                [a.1.clone(), b_prime.1.clone(), c_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            // The inside vertices are A and B, and the outside vertex is C
            // We are going to create two new Triangles with vertices A, B, and A'
            // and A', B, and B' where A' and B' are the intersection points of the
            // line AC and BC with the plane x = +1
            let a = inside_points[0];
            let b = inside_points[1];
            let c = outside_points[0];
            let factor_ac = (1.0 - a.0.x) / (c.0.x - a.0.x);
            let factor_bc = (1.0 - b.0.x) / (c.0.x - b.0.x);
            let a_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));
            let b_prime = (b.0.lerp(*c.0, factor_bc), b.1.lerp(*c.1, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b.0.clone(), a_prime.0.clone()],
                [a.1.clone(), b.1.clone(), a_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0.clone(), b.0.clone(), b_prime.0.clone()],
                [a_prime.1.clone(), b.1.clone(), b_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_points.len() is not 0, 1, 2, or 3"),
    }
}

fn clip_triangle_on_axis<F>(
    triangle: Triangle,
    clip_plane: f32,
    axis_selector: F,
    comparator: fn(f32, f32) -> bool,
) -> TriangleClipResult
where
    F: Fn(&Vec4) -> f32,
{
    let mut inside_points: Vec<(&Vec4, &Vec2)> = Vec::new();
    let mut outside_points: Vec<(&Vec4, &Vec2)> = Vec::new();

    for i in 0..3 {
        if comparator(axis_selector(&triangle.vertices[i]), clip_plane) {
            inside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        } else {
            outside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        }
    }

    match inside_points.len() {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            // The inside vertex is A, and the outside vertices are B and C
            // We are going to create a new Triangle with vertices A, B', and C'
            // where B' and C' are the intersection points of the line AB and AC
            // with the plane x = +1
            let a = inside_points[0];
            let b = outside_points[0];
            let c = outside_points[1];
            let factor_ab =
                (clip_plane - axis_selector(a.0)) / (axis_selector(b.0) - axis_selector(a.0));
            let factor_ac =
                (clip_plane - axis_selector(a.0)) / (axis_selector(c.0) - axis_selector(a.0));
            let b_prime = (a.0.lerp(*b.0, factor_ab), a.1.lerp(*b.1, factor_ab));
            let c_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b_prime.0.clone(), c_prime.0.clone()],
                [a.1.clone(), b_prime.1.clone(), c_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            // The inside vertices are A and B, and the outside vertex is C
            // We are going to create two new Triangles with vertices A, B, and A'
            // and A', B, and B' where A' and B' are the intersection points of the
            // line AC and BC with the plane x = +1
            let a = inside_points[0];
            let b = inside_points[1];
            let c = outside_points[0];
            let factor_ac =
                (clip_plane - axis_selector(a.0)) / (axis_selector(c.0) - axis_selector(a.0));
            let factor_bc =
                (clip_plane - axis_selector(b.0)) / (axis_selector(c.0) - axis_selector(b.0));
            let a_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));
            let b_prime = (b.0.lerp(*c.0, factor_bc), b.1.lerp(*c.1, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [a.0.clone(), b.0.clone(), a_prime.0.clone()],
                [a.1.clone(), b.1.clone(), a_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0.clone(), b.0.clone(), b_prime.0.clone()],
                [a_prime.1.clone(), b.1.clone(), b_prime.1.clone()],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_points.len() is not 0, 1, 2, or 3"),
    }
}

/*
 _______ ______  _____ _______ _____
|__   __|  ____|/ ____|__   __/ ____|
   | |  | |__  | (___    | | | (___
   | |  |  __|  \___ \   | |  \___ \
   | |  | |____ ____) |  | |  ____) |
   |_|  |______|_____/   |_| |_____/

                                      */

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, Vec4};
    use sdl2::pixels::Color;

    #[test]
    fn test01_clip_triangle_on_w_axis() {
        let triangle = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(0.0, 0.0, 0.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result = clip_triangle_on_w_axis(triangle);

        match result {
            TriangleClipResult::OneTriangle(triangle) => {
                assert_eq!(triangle.vertices[0], Vec4::new(0.0, 0.0, 0.0, 1.0));
                assert_eq!(triangle.vertices[1], Vec4::new(1.0, 0.0, 0.0, 1.0));
                assert_eq!(triangle.vertices[2], Vec4::new(0.0, 1.0, 0.0, 1.0));
                assert_eq!(triangle.uvs[0], Vec2::new(0.0, 0.0));
                assert_eq!(triangle.uvs[1], Vec2::new(1.0, 0.0));
                assert_eq!(triangle.uvs[2], Vec2::new(0.0, 1.0));
            }
            _ => panic!("Expected TriangleClipResult::OneTriangle"),
        }
    }

    #[test]
    fn test02_clip_triangle_on_w_axis() {
        let triangle = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result = clip_triangle_on_w_axis(triangle);

        match result {
            TriangleClipResult::OneTriangle(triangle) => {
                println!("{:?}", triangle);
                assert!(true);
            }
            _ => panic!("Expected TriangleClipResult::OneTriangle"),
        }
    }

    #[test]
    fn test03_clip_triangle_on_w_axis() {
        let triangle = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result = clip_triangle_on_w_axis(triangle);

        match result {
            TriangleClipResult::TwoTriangles(triangle1, triangle2) => {
                println!("triangle 1: {:?}", triangle1);
                println!("triangle 2: {:?}", triangle2);
                assert!(true);
            }
            _ => panic!("Expected TriangleClipResult::TwoTriangles"),
        }
    }

    #[test]
    fn test04_clip_triangle_on_plus_x_axis() {
        let triangle = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(-3.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result = clip_triangle_on_plus_x_axis(triangle);

        match result {
            TriangleClipResult::OneTriangle(triangle) => {
                println!("{:?}", triangle);
                assert!(true);
            }
            _ => panic!("Expected TriangleClipResult::OneTriangle"),
        }
    }

    #[test]
    fn test05_plus_x_vs_generic_axis() {
        let triangle1 = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(-3.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );
        let triangle2 = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(-3.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result1 = clip_triangle_on_plus_x_axis(triangle1);
        let result2 = clip_triangle_on_axis(triangle2, -1.0, |v| v.x, |x, plane| x > plane);

        // The two results should be the same
        match (result1, result2) {
            (
                TriangleClipResult::OneTriangle(triangle1),
                TriangleClipResult::OneTriangle(triangle2),
            ) => {
                println!("triangle1: {:?}", triangle1);
                println!("triangle2: {:?}", triangle2);
                assert_eq!(triangle1, triangle2);
            }
            _ => panic!("Expected TriangleClipResult::OneTriangle"),
        }
    }

    #[test]
    fn test06_clip_w_vs_generic_axis() {
        let triangle1 = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );
        let triangle2 = Triangle::from_vertices_uvs_normal_color(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
            Vec3::new(0.0, 0.0, 1.0),
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );

        let result1 = clip_triangle_on_w_axis(triangle1);
        let result2 = clip_triangle_on_axis(triangle2, 0.0, |v| v.w, |w, plane| w > plane);

        // The two results should be the same
        match (result1, result2) {
            (
                TriangleClipResult::OneTriangle(triangle1),
                TriangleClipResult::OneTriangle(triangle2),
            ) => {
                println!("triangle1: {:?}", triangle1);
                println!("triangle2: {:?}", triangle2);
                assert_eq!(triangle1, triangle2);
            }
            _ => panic!("Expected TriangleClipResult::OneTriangle"),
        }
    }
}
