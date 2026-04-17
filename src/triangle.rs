//! Triangle and face primitives used throughout the pipeline.
//!
//! - [`Face`] — index record pointing into a [`crate::model::Model`]'s vertex/UV/normal arrays.
//! - [`Triangle`] — fully expanded triangle (3 × `Vec4` vertices, 3 × `Vec2` UVs, normal, color).
//! - [`TriangleScreenPixel`] — one interpolated scanline endpoint carrying perspective-correct data.

use glam::{Vec2, Vec3, Vec4};
use sdl2::pixels::Color;

/// Index record for one face of a [`crate::model::Model`].
///
/// All fields are indices into the model's corresponding arrays.
pub struct Face {
    /// Indices into `Model::vertices`.
    pub vertices: [usize; 3],
    /// Indices into `Model::uvs`.
    pub uvs: [usize; 3],
    /// Indices into `Model::normals`.
    pub normals: [usize; 3],
    /// Pre-computed face normal (model space).
    pub normal: Vec3,
    /// Base face color before lighting (multiplied by diffuse intensity in the pipeline).
    pub color: Color,
}

#[derive(Debug, PartialEq)]
/// A fully expanded, self-contained triangle ready for clipping and rasterization.
pub struct Triangle {
    /// Three clip-space or screen-space vertices (homogeneous coordinates).
    pub vertices: [Vec4; 3],
    /// Centroid used for back-face culling.
    pub center: Vec4,
    /// UV texture coordinates at each vertex.
    pub uvs: [Vec2; 3],
    /// Surface normal in view space.
    pub normal: Vec3,
    /// Lit face color (ARGB, baked during `transform_to_camera_space`).
    pub color: Color,
}

#[derive(Debug, Clone, Copy, Default)]
/// One interpolated endpoint on a triangle scanline edge.
///
/// Carries perspective-correct data for the per-pixel rasterization inner loop.
pub struct TriangleScreenPixel {
    /// Screen-space X coordinate.
    pub x: f32,
    /// Screen-space Y coordinate.
    pub y: f32,
    /// `1/w` — used for z-buffering and perspective correction.
    pub reciprocal_w: f32,
    /// `u/w` — perspective-divided U coordinate.
    pub u_divided_w: f32,
    /// `v/w` — perspective-divided V coordinate.
    pub v_divided_w: f32,
}
impl TriangleScreenPixel {
    #[must_use]
    /// Returns a zeroed `TriangleScreenPixel` (same as `Default::default()`).
    pub fn new() -> Self {
        Self::default()
    }
}

impl Triangle {
    #[must_use]
    /// Creates a triangle from vertices and UVs, with no normal and white color.
    pub fn from_vertices_uv(vertices: [Vec4; 3], uvs: [Vec2; 3]) -> Self {
        let normal = Vec3::ZERO;
        let center = (vertices[0] + vertices[1] + vertices[2]) / 3.0;
        Self {
            vertices,
            center,
            uvs,
            normal,
            color: Color::WHITE,
        }
    }
    #[must_use]
    /// Creates a triangle from vertices, UVs, a surface normal, and a lit color.
    pub fn from_vertices_uvs_normal_color(
        vertices: [Vec4; 3],
        uvs: [Vec2; 3],
        normal: Vec3,
        color: Color,
    ) -> Self {
        let center = (vertices[0] + vertices[1] + vertices[2]) / 3.0;
        Self {
            vertices,
            center,
            uvs,
            normal,
            color,
        }
    }
    #[must_use]
    /// Creates a degenerate all-zero triangle with white color.
    pub fn new() -> Self {
        Self {
            vertices: [Vec4::ZERO; 3],
            center: Vec4::ZERO,
            uvs: [Vec2::ZERO; 3],
            normal: Vec3::ZERO,
            color: Color::WHITE,
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        let camera_to_triangle = Vec3::ZERO - self.center.truncate();
        self.normal.dot(camera_to_triangle) > 0.0
    }

    // returns the vertices of the triangle in order of y
    pub(crate) fn reorder_vertices_by_y(&self) -> Triangle {
        let mut vertices = self.vertices;
        let mut uvs = self.uvs;
        let normal = self.normal;
        let color = self.color;
        if vertices[0].y > vertices[1].y {
            vertices.swap(0, 1);
            uvs.swap(0, 1);
        }
        if vertices[1].y > vertices[2].y {
            vertices.swap(1, 2);
            uvs.swap(1, 2);
        }
        if vertices[0].y > vertices[1].y {
            vertices.swap(0, 1);
            uvs.swap(0, 1);
        }
        Triangle {
            vertices,
            center: self.center,
            uvs,
            normal,
            color,
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, Vec4};
    use sdl2::pixels::Color;

    fn make_tri(verts: [Vec4; 3], uvs: [Vec2; 3], normal: Vec3) -> Triangle {
        Triangle::from_vertices_uvs_normal_color(verts, uvs, normal, Color::WHITE)
    }

    // ── is_visible ───────────────────────────────────────────────────────────
    //
    // In view space the camera sits at the origin.
    // `camera_to_triangle = ZERO - center` (vector from triangle toward camera).
    // A face is visible when `normal · camera_to_triangle > 0`.

    #[test]
    fn is_visible_front_facing() {
        // Normal points toward camera (-Z); triangle is in front (+Z).
        // dot((0,0,-1), (0,0,-1)) = 1 > 0 → visible.
        let tri = make_tri(
            [Vec4::new(0.0, 0.0, 1.0, 1.0); 3],
            [Vec2::ZERO; 3],
            Vec3::new(0.0, 0.0, -1.0),
        );
        assert!(tri.is_visible());
    }

    #[test]
    fn is_visible_back_facing() {
        // Normal points away from camera (+Z).
        // dot((0,0,1), (0,0,-1)) = -1 ≤ 0 → not visible.
        let tri = make_tri(
            [Vec4::new(0.0, 0.0, 1.0, 1.0); 3],
            [Vec2::ZERO; 3],
            Vec3::new(0.0, 0.0, 1.0),
        );
        assert!(!tri.is_visible());
    }

    #[test]
    fn is_visible_edge_on() {
        // Normal is perpendicular to view direction → dot == 0, not > 0 → not visible.
        let tri = make_tri(
            [Vec4::new(0.0, 0.0, 1.0, 1.0); 3],
            [Vec2::ZERO; 3],
            Vec3::new(1.0, 0.0, 0.0),
        );
        assert!(!tri.is_visible());
    }

    // ── reorder_vertices_by_y ────────────────────────────────────────────────

    fn tri_with_y(y0: f32, y1: f32, y2: f32) -> Triangle {
        make_tri(
            [
                Vec4::new(0.0, y0, 0.0, 1.0),
                Vec4::new(1.0, y1, 0.0, 1.0),
                Vec4::new(2.0, y2, 0.0, 1.0),
            ],
            [
                Vec2::new(0.0, y0),
                Vec2::new(1.0, y1),
                Vec2::new(2.0, y2),
            ],
            Vec3::Z,
        )
    }

    fn sorted_ys(tri: &Triangle) -> [f32; 3] {
        [tri.vertices[0].y, tri.vertices[1].y, tri.vertices[2].y]
    }

    #[test]
    fn reorder_already_sorted() {
        let t = tri_with_y(0.0, 1.0, 2.0).reorder_vertices_by_y();
        assert_eq!(sorted_ys(&t), [0.0, 1.0, 2.0]);
    }

    #[test]
    fn reorder_reversed() {
        let t = tri_with_y(2.0, 1.0, 0.0).reorder_vertices_by_y();
        assert_eq!(sorted_ys(&t), [0.0, 1.0, 2.0]);
    }

    #[test]
    fn reorder_partial() {
        let t = tri_with_y(1.0, 0.0, 2.0).reorder_vertices_by_y();
        assert_eq!(sorted_ys(&t), [0.0, 1.0, 2.0]);
    }

    #[test]
    fn reorder_duplicate_y() {
        let t = tri_with_y(1.0, 0.0, 1.0).reorder_vertices_by_y();
        assert_eq!(sorted_ys(&t), [0.0, 1.0, 1.0]);
    }

    #[test]
    fn reorder_uvs_follow_vertices() {
        // Each UV's x encodes the original vertex slot (0.0, 1.0, 2.0).
        // After reordering by Y the UV must travel with its vertex.
        let t = tri_with_y(2.0, 0.0, 1.0).reorder_vertices_by_y();
        for i in 0..3 {
            assert!(
                (t.uvs[i].x - t.vertices[i].x).abs() < 1e-5,
                "UV[{i}].x should match vertex[{i}].x after reorder"
            );
        }
    }
}
