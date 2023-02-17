// this module defines the struct mesh,
// which contains Vec of Vec3 for the vertices,
// Vec of Vec2 for the uvs,
// Vec of Vec3 for the normals,
// a buffer of bytes for the color texture,
// a Vec3 for the position,
// a Vec3 for the rotation,
// and a Vec3 for the scale

use crate::triangle::Face;
use glam::{Vec2, Vec3};
use sdl2::pixels::Color;
//use sdl2::pixels::Color;

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub texture: Option<Box<[u8]>>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Mesh {
    pub fn new() -> Self {
        let vertices = Vec::new();
        let uvs = Vec::new();
        let normals = Vec::new();
        let faces = Vec::new();
        let texture = None;
        let texture_width = 0_u32;
        let texture_height = 0_u32;
        let position = Vec3::new(0.0, 0.0, 0.0);
        let rotation = Vec3::new(0.0, 0.0, 0.0);
        let scale = Vec3::new(1.0, 1.0, 1.0);
        Self {
            vertices,
            uvs,
            normals,
            faces,
            texture,
            texture_width,
            texture_height,
            position,
            rotation,
            scale,
        }
    }

    // open obj file and load the vertices, uvs, normals, and faces
    // open image file and load the texture
    pub(crate) fn load_obj(&mut self, obj_file: &str, image_file: &str) {
        // we assume there is only one mesh per obj file
        // we assume the obj file is well formed
        // we assume the obj file is triangulated
        // we assume the obj file has vertices, uvs, normals, and faces

        // open the obj file and read each line
        let obj_file = std::fs::read_to_string(obj_file)
            .unwrap_or_else(|_| panic!("Failed to load object: {obj_file}"));
        let obj_file = obj_file.lines();
        // for each line, check if it is a vertex, uv, normal, or face
        for line in obj_file {
            // if it is a vertex, add it to the vertices vector
            if line.starts_with("v ") {
                let line = line.split_whitespace();
                let mut line = line.skip(1);
                let x = line.next().unwrap().parse::<f32>().unwrap();
                let y = line.next().unwrap().parse::<f32>().unwrap();
                let z = line.next().unwrap().parse::<f32>().unwrap();
                self.vertices.push(Vec3::new(x, y, z));
            }
            // if it is a uv, add it to the uvs vector
            if line.starts_with("vt ") {
                let line = line.split_whitespace();
                let mut line = line.skip(1);
                let u = line.next().unwrap().parse::<f32>().unwrap();
                let v = 1.0 - line.next().unwrap().parse::<f32>().unwrap(); // flip v
                self.uvs.push(Vec2::new(u, v));
            }
            // if it is a normal, add it to the normals vector
            if line.starts_with("vn ") {
                let line = line.split_whitespace();
                let mut line = line.skip(1);
                let x = line.next().unwrap().parse::<f32>().unwrap();
                let y = line.next().unwrap().parse::<f32>().unwrap();
                let z = line.next().unwrap().parse::<f32>().unwrap();
                self.normals.push(Vec3::new(x, y, z));
            }
            // if it is a face, add it to the faces vector
            if line.starts_with("f ") {
                let line = line.split_whitespace();
                let mut line = line.skip(1);
                let mut vertices = [0_usize; 3];
                let mut uvs = [0_usize; 3];
                let mut normals = [0_usize; 3];
                for i in 0..3 {
                    let face = line.next().unwrap().split('/');
                    let mut face = face.skip(0);
                    vertices[i] = face.next().unwrap().parse::<usize>().unwrap() - 1;
                    uvs[i] = face.next().unwrap().parse::<usize>().unwrap() - 1;
                    normals[i] = face.next().unwrap().parse::<usize>().unwrap() - 1;
                }
                let normal = self.normals[normals[0]];
                self.faces.push(Face {
                    vertices,
                    uvs,
                    normals,
                    normal,
                    color: Color::WHITE,
                });
            }
        }
        // open the image file and load the texture into a buffer using the image crate
        let image = image::open(image_file)
            .unwrap_or_else(|_| panic!("Failed to load texture: {image_file}"))
            .into_rgba8();
        let (width, height) = image.dimensions();
        // get rgba values from the image buffer and put them into a buffer of bytes with the order argb
        let texture = image.into_raw();
        let mut texture = texture.into_boxed_slice();
        for i in 0..texture.len() / 4 {
            let r = texture[i * 4];
            let g = texture[i * 4 + 1];
            let b = texture[i * 4 + 2];
            let a = texture[i * 4 + 3];
            texture[i * 4] = b; // b
            texture[i * 4 + 1] = g; // g
            texture[i * 4 + 2] = r; // r
            texture[i * 4 + 3] = a; // a
        }
        self.texture = Some(texture);
        self.texture_width = width;
        self.texture_height = height;
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
