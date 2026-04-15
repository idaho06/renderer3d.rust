use crate::triangle::Triangle;
use glam::{Vec2, Vec4};
//use smallvec::SmallVec;

pub enum TriangleClipResult {
    OneTriangle(Triangle),
    TwoTriangles(Triangle, Triangle),
    NoTriangle,
}

// Clip-space frustum planes using signed-distance functions.
// Inside when d(v) > 0. Intersection factor: t = d_a / (d_a - d_b).

pub fn clip_triangle_w_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - 0.001)
}

pub fn clip_triangle_x_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.x + v.w)
}

pub fn clip_triangle_y_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.y + v.w)
}

pub fn clip_triangle_z_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.z)
}

pub fn clip_triangle_nx_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.x)
}

pub fn clip_triangle_ny_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.y)
}

pub fn clip_triangle_nz_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.z)
}

fn clip_triangle_on_plane<F>(
    triangle: Triangle,
    signed_distance: F,
) -> TriangleClipResult
where
    F: Fn(&Vec4) -> f32,
{
    let mut inside_points: Vec<(&Vec4, &Vec2)> = Vec::new();
    let mut outside_points: Vec<(&Vec4, &Vec2)> = Vec::new();

    for i in 0..3 {
        if signed_distance(&triangle.vertices[i]) > 0.0 {
            inside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        } else {
            outside_points.push((&triangle.vertices[i], &triangle.uvs[i]));
        }
    }

    match inside_points.len() {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            let a = inside_points[0];
            let b = outside_points[0];
            let c = outside_points[1];
            let d_a = signed_distance(a.0);
            let d_b = signed_distance(b.0);
            let d_c = signed_distance(c.0);
            let factor_ab = d_a / (d_a - d_b);
            let factor_ac = d_a / (d_a - d_c);
            let b_prime = (a.0.lerp(*b.0, factor_ab), a.1.lerp(*b.1, factor_ab));
            let c_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [*a.0, b_prime.0, c_prime.0],
                [*a.1, b_prime.1, c_prime.1],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            let a = inside_points[0];
            let b = inside_points[1];
            let c = outside_points[0];
            let d_a = signed_distance(a.0);
            let d_b = signed_distance(b.0);
            let d_c = signed_distance(c.0);
            let factor_ac = d_a / (d_a - d_c);
            let factor_bc = d_b / (d_b - d_c);
            let a_prime = (a.0.lerp(*c.0, factor_ac), a.1.lerp(*c.1, factor_ac));
            let b_prime = (b.0.lerp(*c.0, factor_bc), b.1.lerp(*c.1, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [*a.0, *b.0, a_prime.0],
                [*a.1, *b.1, a_prime.1],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0, *b.0, b_prime.0],
                [a_prime.1, *b.1, b_prime.1],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_points.len() is not 0, 1, 2, or 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, Vec4};
    use sdl2::pixels::Color;

    fn make_triangle(verts: [Vec4; 3], uvs: [Vec2; 3]) -> Triangle {
        Triangle::from_vertices_uvs_normal_color(
            verts,
            uvs,
            Vec3::new(0.0, 0.0, 1.0),
            Color::WHITE,
        )
    }

    #[test]
    fn test_w_clip_all_inside() {
        let tri = make_triangle(
            [
                Vec4::new(0.0, 0.0, 0.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_w_axis(tri) {
            TriangleClipResult::OneTriangle(t) => {
                assert_eq!(t.vertices[0], Vec4::new(0.0, 0.0, 0.0, 1.0));
                assert_eq!(t.vertices[1], Vec4::new(1.0, 0.0, 0.0, 1.0));
                assert_eq!(t.vertices[2], Vec4::new(0.0, 1.0, 0.0, 1.0));
            }
            _ => panic!("Expected OneTriangle"),
        }
    }

    #[test]
    fn test_w_clip_one_inside() {
        let tri = make_triangle(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_w_axis(tri) {
            TriangleClipResult::OneTriangle(_) => {}
            _ => panic!("Expected OneTriangle"),
        }
    }

    #[test]
    fn test_w_clip_two_inside() {
        let tri = make_triangle(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_w_axis(tri) {
            TriangleClipResult::TwoTriangles(_, _) => {}
            _ => panic!("Expected TwoTriangles"),
        }
    }

    #[test]
    fn test_w_clip_all_outside() {
        let tri = make_triangle(
            [
                Vec4::new(0.0, 0.0, 0.0, -1.0),
                Vec4::new(1.0, 0.0, 0.0, -1.0),
                Vec4::new(0.0, 1.0, 0.0, -1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_w_axis(tri) {
            TriangleClipResult::NoTriangle => {}
            _ => panic!("Expected NoTriangle"),
        }
    }

    #[test]
    fn test_left_clip_all_outside() {
        // x + w < 0 for all: (-3+1=-2), (-2+1=-1), (-2.5+1=-1.5)
        let tri = make_triangle(
            [
                Vec4::new(-3.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.5, 1.0, 0.0, 1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_x_axis(tri) {
            TriangleClipResult::NoTriangle => {}
            _ => panic!("Expected NoTriangle"),
        }
    }

    #[test]
    fn test_left_clip_one_inside() {
        // x + w: (-3+1=-2 out), (-2+1=-1 out), (0+1=1 in)
        let tri = make_triangle(
            [
                Vec4::new(-3.0, 0.0, 0.0, 1.0),
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_x_axis(tri) {
            TriangleClipResult::OneTriangle(t) => {
                // Intersection on the plane x + w = 0 (x = -w = -1 when w=1)
                // A=(0,1,0,1) d_a=1, B=(-3,0,0,1) d_b=-2, t=1/(1+2)=1/3
                // B' = lerp(A, B, 1/3) = (0-1, 1-1/3, 0, 1) = (-1, 2/3, 0, 1)
                let b_prime = t.vertices[1];
                assert!((b_prime.x - (-1.0)).abs() < 1e-5);
                assert!((b_prime.x + b_prime.w).abs() < 1e-5); // on the plane
            }
            _ => panic!("Expected OneTriangle"),
        }
    }

    #[test]
    fn test_w_clip_intersection_correctness() {
        // A behind camera (w=-1), B in front (w=2)
        let tri = make_triangle(
            [
                Vec4::new(1.0, 0.0, 0.5, -1.0),
                Vec4::new(1.0, 0.0, 0.5, 2.0),
                Vec4::new(0.0, 1.0, 0.5, 2.0),
            ],
            [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)],
        );
        match clip_triangle_w_axis(tri) {
            TriangleClipResult::TwoTriangles(t1, _t2) => {
                // B (w=2) and C (w=2) are inside, A (w=-1) is outside
                // Intersection on A-B: d_b = 2-0.001=1.999, d_a = -1-0.001=-1.001
                // factor = 1.999 / (1.999+1.001) = 1.999/3.0
                // W at intersection = lerp(2, -1, 1.999/3.0) ≈ 0.001
                // All clipped vertices should have w ≈ 0.001
                for v in &t1.vertices {
                    assert!(v.w > 0.0, "Clipped vertex W must be positive, got {}", v.w);
                }
            }
            _ => panic!("Expected TwoTriangles"),
        }
    }
}
