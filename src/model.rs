//! 3D model geometry and texture data.
//!
//! [`Model`] owns the vertex buffer, UV buffer, face list, and (optionally) a packed texture.
//! Two constructors are provided: [`Model::builtin_cube`] for the procedural test model and
//! [`Model::from_obj_and_png`] for loading assets from disk.
//!
//! ## Why the BGRA → ARGB swap at load time?
//!
//! The `image` crate delivers PNG pixels in RGBA byte order.  The rasterizer reads the texture
//! as a `u32` in big-endian byte order, expecting **ARGB** (`[a, r, g, b]`).  Swapping once at
//! load time (RGBA → BGRA so that big-endian interpretation gives ARGB) means **zero extra work
//! in the hot path** — each pixel lookup just reads four bytes, with no per-pixel reordering.
//!
//! See book chapter: _Model loading_ (TODO: link when mdBook is set up).

use crate::triangle::Face;
use glam::{Vec2, Vec3};
use sdl2::pixels::Color;

/// Holds all geometry and texture data for a single 3D object.
pub struct Model {
    /// Vertex positions in model space.
    pub vertices: Vec<Vec3>,
    /// UV texture coordinates (one per UV index, not per vertex).
    pub uvs: Vec<Vec2>,
    /// Surface normals (one per normal index, not per vertex).
    pub normals: Vec<Vec3>,
    /// Triangulated face list referencing `vertices`, `uvs`, and `normals` by index.
    pub faces: Vec<Face>,
    /// Optional packed texture in BGRA byte order (interpreted as ARGB big-endian).
    pub texture: Option<Box<[u8]>>,
    /// Texture width in pixels. Must be a power of two for the rasterizer's fast lookup.
    pub texture_width: u32,
    /// Texture height in pixels. Must be a power of two for the rasterizer's fast lookup.
    pub texture_height: u32,
    /// World-space translation applied each frame.
    pub position: Vec3,
    /// Per-axis Euler rotation (radians) applied each frame.
    pub rotation: Vec3,
    /// Per-axis scale factor applied each frame.
    pub scale: Vec3,
}

impl Model {
    #[must_use]
    /// Creates an empty model with no geometry or texture.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            texture: None,
            texture_width: 0,
            texture_height: 0,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    #[must_use]
    /// Loads a model from an OBJ file and its companion PNG texture.
    ///
    /// Quad faces are triangulated automatically. The texture is swapped from RGBA to BGRA
    /// so that the rasterizer can interpret it as ARGB8888 with a simple big-endian read.
    pub fn from_obj_and_png(obj_file: &str, image_file: &str) -> Self {
        let mut model = Self::new();
        model.load_from_files(obj_file, image_file);
        model
    }

    #[must_use]
    #[allow(clippy::items_after_statements, clippy::cast_possible_truncation)]
    /// Returns a hard-coded unit cube with a procedural 8×8 checkerboard texture.
    ///
    /// Useful for testing the pipeline without loading files from disk.
    pub fn builtin_cube() -> Self {
        let vertices = vec![
            Vec3::new(-1.0, -1.0, -1.0), // 0
            Vec3::new(-1.0, 1.0, -1.0),  // 1
            Vec3::new(1.0, 1.0, -1.0),   // 2
            Vec3::new(1.0, -1.0, -1.0),  // 3
            Vec3::new(1.0, 1.0, 1.0),    // 4
            Vec3::new(1.0, -1.0, 1.0),   // 5
            Vec3::new(-1.0, 1.0, 1.0),   // 6
            Vec3::new(-1.0, -1.0, 1.0),  // 7
        ];
        use glam::Vec2;
        let uvs = vec![
            Vec2::new(0.0, 1.0), // 0
            Vec2::new(0.0, 0.0), // 1
            Vec2::new(1.0, 0.0), // 2
            Vec2::new(1.0, 1.0), // 3
        ];
        let normals = vec![
            Vec3::new(0.0, 0.0, -1.0), // front
            Vec3::new(1.0, 0.0, 0.0),  // right
            Vec3::new(0.0, 0.0, 1.0),  // back
            Vec3::new(-1.0, 0.0, 0.0), // left
            Vec3::new(0.0, 1.0, 0.0),  // top
            Vec3::new(0.0, -1.0, 0.0), // bottom
        ];
        let faces = vec![
            // front
            Face { vertices: [0, 1, 2], uvs: [0, 1, 2], normals: [0, 0, 0], normal: normals[0], color: Color::RED },
            Face { vertices: [0, 2, 3], uvs: [0, 2, 3], normals: [0, 0, 0], normal: normals[0], color: Color::RED },
            // right
            Face { vertices: [3, 2, 4], uvs: [0, 1, 2], normals: [1, 1, 1], normal: normals[1], color: Color::GREEN },
            Face { vertices: [3, 4, 5], uvs: [0, 2, 3], normals: [1, 1, 1], normal: normals[1], color: Color::GREEN },
            // back
            Face { vertices: [5, 4, 6], uvs: [0, 1, 2], normals: [2, 2, 2], normal: normals[2], color: Color::BLUE },
            Face { vertices: [5, 6, 7], uvs: [0, 2, 3], normals: [2, 2, 2], normal: normals[2], color: Color::BLUE },
            // left
            Face { vertices: [7, 6, 1], uvs: [0, 1, 2], normals: [3, 3, 3], normal: normals[3], color: Color::YELLOW },
            Face { vertices: [7, 1, 0], uvs: [0, 2, 3], normals: [3, 3, 3], normal: normals[3], color: Color::YELLOW },
            // top
            Face { vertices: [1, 6, 4], uvs: [0, 1, 2], normals: [4, 4, 4], normal: normals[4], color: Color::MAGENTA },
            Face { vertices: [1, 4, 2], uvs: [0, 2, 3], normals: [4, 4, 4], normal: normals[4], color: Color::MAGENTA },
            // bottom
            Face { vertices: [5, 7, 0], uvs: [0, 1, 2], normals: [5, 5, 5], normal: normals[5], color: Color::CYAN },
            Face { vertices: [5, 0, 3], uvs: [0, 2, 3], normals: [5, 5, 5], normal: normals[5], color: Color::CYAN },
        ];

        // 8×8 procedural checkerboard texture (ARGB byte order)
        const TEX_SIZE: usize = 8;
        let mut tex_bytes = vec![0u8; TEX_SIZE * TEX_SIZE * 4];
        for row in 0..TEX_SIZE {
            for col in 0..TEX_SIZE {
                let idx = (row * TEX_SIZE + col) * 4;
                let white = (row + col) % 2 == 0;
                let (a, r, g, b): (u8, u8, u8, u8) = if white {
                    (255, 220, 220, 220)
                } else {
                    (255, 40, 40, 40)
                };
                // stored BGRA on disk → swapped to ARGB at load: store as ARGB directly
                tex_bytes[idx] = b;
                tex_bytes[idx + 1] = g;
                tex_bytes[idx + 2] = r;
                tex_bytes[idx + 3] = a;
            }
        }

        Self {
            vertices,
            uvs,
            normals,
            faces,
            texture: Some(tex_bytes.into_boxed_slice()),
            texture_width: TEX_SIZE as u32,
            texture_height: TEX_SIZE as u32,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    #[allow(
        clippy::items_after_statements,
        clippy::cast_possible_truncation,
        clippy::missing_panics_doc
    )]
    fn load_from_files(&mut self, obj_file: &str, image_file: &str) {
        let obj_contents = std::fs::read_to_string(obj_file)
            .unwrap_or_else(|_| panic!("Failed to load object: {obj_file}"));
        for line in obj_contents.lines() {
            if line.starts_with("v ") {
                let mut parts = line.split_whitespace().skip(1);
                let x = parts.next().unwrap().parse::<f32>().unwrap();
                let y = parts.next().unwrap().parse::<f32>().unwrap();
                let z = parts.next().unwrap().parse::<f32>().unwrap();
                self.vertices.push(Vec3::new(x, y, z));
            } else if line.starts_with("vt ") {
                let mut parts = line.split_whitespace().skip(1);
                let u = parts.next().unwrap().parse::<f32>().unwrap();
                let v = 1.0 - parts.next().unwrap().parse::<f32>().unwrap();
                self.uvs.push(Vec2::new(u, v));
            } else if line.starts_with("vn ") {
                let mut parts = line.split_whitespace().skip(1);
                let x = parts.next().unwrap().parse::<f32>().unwrap();
                let y = parts.next().unwrap().parse::<f32>().unwrap();
                let z = parts.next().unwrap().parse::<f32>().unwrap();
                self.normals.push(Vec3::new(x, y, z));
            } else if line.starts_with("f ") {
                let tokens: Vec<&str> = line.split_whitespace().skip(1).collect();
                if tokens.len() == 4 {
                    // quad — triangulate: [0,1,2] and [0,2,3]
                    let [v0, v1, v2, v3] = Self::parse_four_face_tokens(&tokens);
                    let n0 = self.normals[v0.2];
                    let n1 = self.normals[v2.2];
                    self.faces.push(Face {
                        vertices: [v0.0, v1.0, v2.0],
                        uvs: [v0.1, v1.1, v2.1],
                        normals: [v0.2, v1.2, v2.2],
                        normal: n0,
                        color: Color::WHITE,
                    });
                    self.faces.push(Face {
                        vertices: [v0.0, v2.0, v3.0],
                        uvs: [v0.1, v2.1, v3.1],
                        normals: [v0.2, v2.2, v3.2],
                        normal: n1,
                        color: Color::WHITE,
                    });
                } else {
                    let mut vertices = [0usize; 3];
                    let mut uvs = [0usize; 3];
                    let mut normals_idx = [0usize; 3];
                    for (i, token) in tokens.iter().enumerate().take(3) {
                        let mut parts = token.split('/');
                        vertices[i] = parts.next().unwrap().parse::<usize>().unwrap() - 1;
                        uvs[i] = parts.next().unwrap().parse::<usize>().unwrap() - 1;
                        normals_idx[i] = parts.next().unwrap().parse::<usize>().unwrap() - 1;
                    }
                    let normal = self.normals[normals_idx[0]];
                    self.faces.push(Face {
                        vertices,
                        uvs,
                        normals: normals_idx,
                        normal,
                        color: Color::WHITE,
                    });
                }
            }
        }

        let image = image::open(image_file)
            .unwrap_or_else(|_| panic!("Failed to load texture: {image_file}"))
            .into_rgba8();
        let (width, height) = image.dimensions();
        let raw = image.into_raw();
        let mut texture = raw.into_boxed_slice();
        // Why RGBA → BGRA? The `image` crate gives us RGBA bytes. The rasterizer reads 4 bytes
        // as a big-endian u32 and expects ARGB: bytes [a, r, g, b]. Reordering to BGRA here means
        // that big-endian layout becomes ARGB, so the hot path does a plain 4-byte read with no
        // per-pixel branch or swap.
        for i in 0..texture.len() / 4 {
            let r = texture[i * 4];
            let g = texture[i * 4 + 1];
            let b = texture[i * 4 + 2];
            let a = texture[i * 4 + 3];
            texture[i * 4] = b;
            texture[i * 4 + 1] = g;
            texture[i * 4 + 2] = r;
            texture[i * 4 + 3] = a;
        }
        self.texture = Some(texture);
        self.texture_width = width;
        self.texture_height = height;
    }

    fn parse_four_face_tokens(tokens: &[&str]) -> [(usize, usize, usize); 4] {
        let mut result = [(0usize, 0usize, 0usize); 4];
        for (i, token) in tokens.iter().enumerate().take(4) {
            let mut parts = token.split('/');
            result[i].0 = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            result[i].1 = parts.next().unwrap().parse::<usize>().unwrap() - 1;
            result[i].2 = parts.next().unwrap().parse::<usize>().unwrap() - 1;
        }
        result
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
