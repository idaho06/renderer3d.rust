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
        // we assume the obj faces are triangles or quads
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
                let line: Vec<&str> = line.skip(1).collect();
                // if the face has more than 3 vertices, triangulate it
                if line.len() > 3 && line.len() < 5 {
                    // 0   1
                    // 3   2

                    let mut vertices1 = [0_usize; 3];
                    let mut uvs1 = [0_usize; 3];
                    let mut normals1 = [0_usize; 3];
                    let mut vertices2 = [0_usize; 3];
                    let mut uvs2 = [0_usize; 3];
                    let mut normals2 = [0_usize; 3];
                    let mut line = line.iter();

                    let mut vertex1 = line.next().unwrap().split('/'); // 0
                    vertices1[0] = vertex1.next().unwrap().parse::<usize>().unwrap() - 1;
                    uvs1[0] = vertex1.next().unwrap().parse::<usize>().unwrap() - 1;
                    normals1[0] = vertex1.next().unwrap().parse::<usize>().unwrap() - 1;
                    vertices2[0] = vertices1[0];
                    uvs2[0] = uvs1[0];
                    normals2[0] = normals1[0];

                    let mut vertex2 = line.next().unwrap().split('/'); // 1
                    vertices1[1] = vertex2.next().unwrap().parse::<usize>().unwrap() - 1;
                    uvs1[1] = vertex2.next().unwrap().parse::<usize>().unwrap() - 1;
                    normals1[1] = vertex2.next().unwrap().parse::<usize>().unwrap() - 1;

                    let mut vertex3 = line.next().unwrap().split('/'); // 2
                    vertices1[2] = vertex3.next().unwrap().parse::<usize>().unwrap() - 1;
                    uvs1[2] = vertex3.next().unwrap().parse::<usize>().unwrap() - 1;
                    normals1[2] = vertex3.next().unwrap().parse::<usize>().unwrap() - 1;
                    vertices2[1] = vertices1[2];
                    uvs2[1] = uvs1[2];
                    normals2[1] = normals1[2];

                    let mut vertex4 = line.next().unwrap().split('/'); // 3
                    vertices2[2] = vertex4.next().unwrap().parse::<usize>().unwrap() - 1;
                    uvs2[2] = vertex4.next().unwrap().parse::<usize>().unwrap() - 1;
                    normals2[2] = vertex4.next().unwrap().parse::<usize>().unwrap() - 1;
                    let normal1 = self.normals[normals1[0]];
                    let normal2 = self.normals[normals2[0]];
                    self.faces.push(Face {
                        vertices: vertices1,
                        uvs: uvs1,
                        normals: normals1,
                        normal: normal1,
                        color: Color::WHITE,
                    });
                    self.faces.push(Face {
                        vertices: vertices2,
                        uvs: uvs2,
                        normals: normals2,
                        normal: normal2,
                        color: Color::WHITE,
                    });
                } else {
                    let mut line = line.iter();
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
