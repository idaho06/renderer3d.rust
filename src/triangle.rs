// This module defines two structs:
// 1. face - a struct that containst the indices to the vertices and uvs and the normal of triangle from a mesh
// 2. triangle - an array of 3 Vec4 for the vertices, an array of 3 Vec2 for the uvs, a Vec4 for the normal, and a Vec4 for the color

use glam::{Vec2, Vec3, Vec4};
use sdl2::pixels::Color;

pub struct Face {
    pub vertices: [usize; 3],
    pub uvs: [usize; 3],
    pub normals: [usize; 3],
    pub normal: Vec3,
    pub color: Color,
}

#[derive(Debug, PartialEq)]
pub struct Triangle {
    pub vertices: [Vec4; 3],
    pub center: Vec4,
    pub uvs: [Vec2; 3],
    pub normal: Vec3,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TriangleScreenPixel {
    pub x: f32,
    pub y: f32,
    pub reciprocal_w: f32,
    pub u_divided_w: f32,
    pub v_divided_w: f32,
}
impl TriangleScreenPixel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Triangle {
    // triangle constructor with only three Vec4 vertices and uvs. No normal calculated
    #[must_use]
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
    // triangle constructor with three Vec4 vertices, three Vec2 uvs, Vec3 normal, and Color color
    #[must_use]
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
    // triangle constructor "new". No parameters. Returns a triangle with all zeros
    #[must_use]
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
