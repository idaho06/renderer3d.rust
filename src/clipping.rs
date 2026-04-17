use crate::triangle::Triangle;
use glam::Vec4;

pub enum TriangleClipResult {
    OneTriangle(Triangle),
    TwoTriangles(Triangle, Triangle),
    NoTriangle,
}

// Clip-space frustum planes using signed-distance functions.
// Inside when d(v) > 0. Intersection factor: t = d_a / (d_a - d_b).

#[must_use]
pub fn clip_triangle_w_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - 0.001)
}

#[must_use]
pub fn clip_triangle_x_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.x + v.w)
}

#[must_use]
pub fn clip_triangle_y_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.y + v.w)
}

#[must_use]
pub fn clip_triangle_z_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.z)
}

#[must_use]
pub fn clip_triangle_nx_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.x)
}

#[must_use]
pub fn clip_triangle_ny_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.y)
}

#[must_use]
pub fn clip_triangle_nz_axis(triangle: Triangle) -> TriangleClipResult {
    clip_triangle_on_plane(triangle, |v| v.w - v.z)
}

#[allow(clippy::similar_names)]
fn clip_triangle_on_plane<F>(triangle: Triangle, signed_distance: F) -> TriangleClipResult
where
    F: Fn(&Vec4) -> f32,
{
    let mut inside_indices = [0_usize; 3];
    let mut outside_indices = [0_usize; 3];
    let mut inside_count = 0_usize;
    let mut outside_count = 0_usize;
    let mut distances = [0.0_f32; 3];

    #[allow(clippy::needless_range_loop)]
    for i in 0..3 {
        let distance = signed_distance(&triangle.vertices[i]);
        distances[i] = distance;

        if distance > 0.0 {
            inside_indices[inside_count] = i;
            inside_count += 1;
        } else {
            outside_indices[outside_count] = i;
            outside_count += 1;
        }
    }

    match inside_count {
        0 => TriangleClipResult::NoTriangle,
        1 => {
            let a_index = inside_indices[0];
            let b_index = outside_indices[0];
            let c_index = outside_indices[1];
            let a = triangle.vertices[a_index];
            let b = triangle.vertices[b_index];
            let c = triangle.vertices[c_index];
            let uv_a = triangle.uvs[a_index];
            let uv_b = triangle.uvs[b_index];
            let uv_c = triangle.uvs[c_index];
            let d_a = distances[a_index];
            let d_b = distances[b_index];
            let d_c = distances[c_index];
            let factor_ab = d_a / (d_a - d_b);
            let factor_ac = d_a / (d_a - d_c);
            let b_prime = (a.lerp(b, factor_ab), uv_a.lerp(uv_b, factor_ab));
            let c_prime = (a.lerp(c, factor_ac), uv_a.lerp(uv_c, factor_ac));

            let new_triangle = Triangle::from_vertices_uvs_normal_color(
                [a, b_prime.0, c_prime.0],
                [uv_a, b_prime.1, c_prime.1],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::OneTriangle(new_triangle)
        }
        2 => {
            let a_index = inside_indices[0];
            let b_index = inside_indices[1];
            let c_index = outside_indices[0];
            let a = triangle.vertices[a_index];
            let b = triangle.vertices[b_index];
            let c = triangle.vertices[c_index];
            let uv_a = triangle.uvs[a_index];
            let uv_b = triangle.uvs[b_index];
            let uv_c = triangle.uvs[c_index];
            let d_a = distances[a_index];
            let d_b = distances[b_index];
            let d_c = distances[c_index];
            let factor_ac = d_a / (d_a - d_c);
            let factor_bc = d_b / (d_b - d_c);
            let a_prime = (a.lerp(c, factor_ac), uv_a.lerp(uv_c, factor_ac));
            let b_prime = (b.lerp(c, factor_bc), uv_b.lerp(uv_c, factor_bc));

            let triangle1 = Triangle::from_vertices_uvs_normal_color(
                [a, b, a_prime.0],
                [uv_a, uv_b, a_prime.1],
                triangle.normal,
                triangle.color,
            );

            let triangle2 = Triangle::from_vertices_uvs_normal_color(
                [a_prime.0, b, b_prime.0],
                [a_prime.1, uv_b, b_prime.1],
                triangle.normal,
                triangle.color,
            );

            TriangleClipResult::TwoTriangles(triangle1, triangle2)
        }
        3 => TriangleClipResult::OneTriangle(triangle),
        _ => panic!("inside_count is not 0, 1, 2, or 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, Vec4};
    use sdl2::pixels::Color;

    fn make_triangle(verts: [Vec4; 3], uvs: [Vec2; 3]) -> Triangle {
        Triangle::from_vertices_uvs_normal_color(verts, uvs, Vec3::new(0.0, 0.0, 1.0), Color::WHITE)
    }

    #[test]
    fn test_w_clip_all_inside() {
        let tri = make_triangle(
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],
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

    #[test]
    fn test_x_clip_preserves_vertex_order_and_uvs() {
        let tri = make_triangle(
            [
                Vec4::new(-2.0, 0.0, 0.0, 1.0),
                Vec4::new(0.5, 0.0, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.5, 1.0),
            ],
        );

        match clip_triangle_x_axis(tri) {
            TriangleClipResult::TwoTriangles(t1, t2) => {
                assert_eq!(t1.vertices[0], Vec4::new(0.5, 0.0, 0.0, 1.0));
                assert_eq!(t1.vertices[1], Vec4::new(0.0, 1.0, 0.0, 1.0));
                assert_eq!(t1.uvs[0], Vec2::new(1.0, 0.0));
                assert_eq!(t1.uvs[1], Vec2::new(0.5, 1.0));

                let first_intersection = t1.vertices[2];
                let second_intersection = t2.vertices[2];
                assert!((first_intersection.x + first_intersection.w).abs() < 1e-5);
                assert!((second_intersection.x + second_intersection.w).abs() < 1e-5);

                let first_uv = t1.uvs[2];
                let second_uv = t2.uvs[2];
                assert!((first_uv.x - 0.4).abs() < 1e-5);
                assert!((first_uv.y - 0.0).abs() < 1e-5);
                assert!((second_uv.x - 0.25).abs() < 1e-5);
                assert!((second_uv.y - 0.5).abs() < 1e-5);
            }
            _ => panic!("Expected TwoTriangles"),
        }
    }
}
