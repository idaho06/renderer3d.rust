//use core::slice::SlicePattern;

use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
//use sdl2::keyboard::Keycode;
//use sdl2::pixels::Color;

use crate::{
    clipping::{
        TriangleClipResult, clip_triangle_nx_axis, clip_triangle_ny_axis, clip_triangle_nz_axis,
        clip_triangle_w_axis, clip_triangle_x_axis, clip_triangle_y_axis, clip_triangle_z_axis,
    },
    display::Display,
    mesh::Mesh,
    render::{self, Render},
    scene::{Scene, Sequence},
    triangle::Triangle,
};

const CLIP_BUFFER_CAPACITY: usize = 128;

pub struct Cube {
    render: Render,
    mesh: Mesh,
    transformed_triangles: Vec<Triangle>,
    screen_triangles: Vec<Triangle>,
    clip_buffer_ping: Vec<Triangle>,
    clip_buffer_pong: Vec<Triangle>,
    camera_pos: Vec3,
    camera_up: Vec3,
    camera_target: Vec3,
    camera_speed: f32,
    light_dir: Vec3,
    z_buffer: Box<[f32]>,
    //z_buffer_clear: Box<[f32]>,
    color_buffer: Box<[u8]>,
    //color_buffer_clear: Box<[u8]>,
    //buffer_name: String,
    width: u32,
    height: u32,
    now_time: u32,
}

impl Cube {
    pub fn new(display: &mut Display) -> Self {
        // let vertices = vec![
        //     Vec3::new(-1.0, -1.0, -1.0), // 0
        //     Vec3::new(-1.0, 1.0, -1.0),  // 1
        //     Vec3::new(1.0, 1.0, -1.0),   // 2
        //     Vec3::new(1.0, -1.0, -1.0),  // 3
        //     Vec3::new(1.0, 1.0, 1.0),    // 4
        //     Vec3::new(1.0, -1.0, 1.0),   // 5
        //     Vec3::new(-1.0, 1.0, 1.0),   // 6
        //     Vec3::new(-1.0, -1.0, 1.0),  // 7
        // ];
        // let uvs = vec![
        //     Vec2::new(0.0, 1.0), // 0
        //     Vec2::new(0.0, 0.0), // 1
        //     Vec2::new(1.0, 0.0), // 2
        //     Vec2::new(1.0, 1.0), // 3
        // ];

        // // face front
        // let face1 = Face {
        //     vertices: [0, 1, 2],
        //     uvs: [0, 1, 2],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, 0.0, -1.0),
        //     color: Color::RED,
        // };
        // let face2 = Face {
        //     vertices: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     uvs: [0, 2, 3],
        //     normal: Vec3::new(0.0, 0.0, -1.0),
        //     color: Color::RED,
        // };
        // // face right
        // let face3 = Face {
        //     vertices: [3, 2, 4],
        //     normals: [0, 0, 0],
        //     uvs: [0, 1, 2],
        //     normal: Vec3::new(1.0, 0.0, 0.0),
        //     color: Color::GREEN,
        // };
        // let face4 = Face {
        //     vertices: [3, 4, 5],
        //     uvs: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(1.0, 0.0, 0.0),
        //     color: Color::GREEN,
        // };
        // // face back
        // let face5 = Face {
        //     vertices: [5, 4, 6],
        //     uvs: [0, 1, 2],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, 0.0, 1.0),
        //     color: Color::BLUE,
        // };
        // let face6 = Face {
        //     vertices: [5, 6, 7],
        //     uvs: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, 0.0, 1.0),
        //     color: Color::BLUE,
        // };
        // // face left
        // let face7 = Face {
        //     vertices: [7, 6, 1],
        //     uvs: [0, 1, 2],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(-1.0, 0.0, 0.0),
        //     color: Color::YELLOW,
        // };
        // let face8 = Face {
        //     vertices: [7, 1, 0],
        //     uvs: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(-1.0, 0.0, 0.0),
        //     color: Color::YELLOW,
        // };
        // //face top
        // let face9 = Face {
        //     vertices: [1, 6, 4],
        //     uvs: [0, 1, 2],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, 1.0, 0.0),
        //     color: Color::MAGENTA,
        // };
        // let face10 = Face {
        //     vertices: [1, 4, 2],
        //     uvs: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, 1.0, 0.0),
        //     color: Color::MAGENTA,
        // };
        // // face bottom
        // let face11 = Face {
        //     vertices: [5, 7, 0],
        //     uvs: [0, 1, 2],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, -1.0, 0.0),
        //     color: Color::CYAN,
        // };
        // let face12 = Face {
        //     vertices: [5, 0, 3],
        //     uvs: [0, 2, 3],
        //     normals: [0, 0, 0],
        //     normal: Vec3::new(0.0, -1.0, 0.0),
        //     color: Color::CYAN,
        // };

        let mut mesh = Mesh::new();
        mesh.load_obj("assets/lexus.obj", "assets/lexus.png");
        //let buffer_name = String::from("cube");
        // mesh.vertices = vertices;
        // mesh.uvs = uvs;
        // mesh.faces = vec![
        //     face1, face2, face3, face4, face5, face6, face7, face8, face9, face10, face11, face12,
        // ];
        let transformed_triangles = Vec::<Triangle>::with_capacity(mesh.faces.len());
        let screen_triangles = Vec::<Triangle>::with_capacity(mesh.faces.len());
        let clip_buffer_ping = Vec::<Triangle>::with_capacity(CLIP_BUFFER_CAPACITY);
        let clip_buffer_pong = Vec::<Triangle>::with_capacity(CLIP_BUFFER_CAPACITY);
        let width = 640 * 2;
        let height = 360 * 2;
        display.add_streaming_buffer("cube", width, height);
        let z_buffer = vec![0.0_f32; (width * height) as usize].into_boxed_slice();
        //let z_buffer_clear = vec![0.0_f32; (width * height) as usize].into_boxed_slice();
        let color_buffer = vec![0_u8; (width * height * 4) as usize].into_boxed_slice();
        //let color_buffer_clear = vec![0_u8; (width * height * 4) as usize].into_boxed_slice();
        let now_time = 0_u32;
        let camera_pos = Vec3::new(0.0, 25.0, 55.0);
        let camera_up = Vec3::new(0.0, 1.0, 0.0);
        let camera_target = Vec3::new(0.0, 0.0, 0.0);
        let camera_speed = 20.0_f32;
        let light_dir = Vec3::new(1.0, -1.0, 1.0).normalize();
        Self {
            render: Render::new(width, height),
            mesh,
            transformed_triangles,
            screen_triangles,
            clip_buffer_ping,
            clip_buffer_pong,
            //buffer_name,
            width,
            height,
            z_buffer,
            //z_buffer_clear,
            color_buffer,
            //color_buffer_clear,
            now_time,
            camera_pos,
            camera_up,
            camera_target,
            camera_speed,
            light_dir,
        }
    }

    fn run_clip_stage(
        source: &mut Vec<Triangle>,
        destination: &mut Vec<Triangle>,
        clip_fn: fn(Triangle) -> TriangleClipResult,
    ) -> bool {
        destination.clear();

        while let Some(triangle) = source.pop() {
            match clip_fn(triangle) {
                TriangleClipResult::OneTriangle(triangle) => {
                    destination.push(triangle);
                }
                TriangleClipResult::TwoTriangles(triangle1, triangle2) => {
                    destination.push(triangle1);
                    destination.push(triangle2);
                }
                TriangleClipResult::NoTriangle => {}
            }
        }

        !destination.is_empty()
    }
}

// implement the scene trait for the cube
impl Scene for Cube {
    fn update(&mut self, delta_time: u32, display: &Display, _scene: &Option<Sequence>) {
        self.now_time += delta_time;
        let time_factor = delta_time as f32 / 1000.0;

        // input handling
        if display.user_input.key_w.pressed {
            let camera_direction = (self.camera_target - self.camera_pos).normalize();
            self.camera_pos += camera_direction * self.camera_speed * time_factor;
        }
        if display.user_input.key_s.pressed {
            let camera_direction = (self.camera_target - self.camera_pos).normalize();
            self.camera_pos -= camera_direction * self.camera_speed * time_factor;
        }
        if display.user_input.key_a.pressed {
            let camera_direction = (self.camera_target - self.camera_pos).normalize();
            let camera_right = camera_direction.cross(self.camera_up).normalize();
            self.camera_pos -= camera_right * self.camera_speed * time_factor;
        }
        if display.user_input.key_d.pressed {
            let camera_direction = (self.camera_target - self.camera_pos).normalize();
            let camera_right = camera_direction.cross(self.camera_up).normalize();
            self.camera_pos += camera_right * self.camera_speed * time_factor;
        }

        // update mesh
        //self.mesh.rotation.x = -(PI / 2.0);
        //self.mesh.rotation.y += 0.25 * time_factor;
        //self.mesh.rotation.z += 0.5 * time_factor;

        // get world matrix, view matrix, projection matrix
        // let scale_matrix = Mat4::from_scale(self.mesh.scale);
        // let translation_matrix = Mat4::from_translation(self.mesh.position);
        // let rotation_matrix = Mat4::from_euler(
        //     glam::EulerRot::ZYX,
        //     self.mesh.rotation.z,
        //     self.mesh.rotation.y,
        //     self.mesh.rotation.x,
        // );
        // let mut world_matrix = Mat4::IDENTITY;
        // world_matrix = world_matrix * scale_matrix;
        // world_matrix = world_matrix * rotation_matrix;
        // world_matrix = world_matrix * translation_matrix;
        let rotation_quat = Quat::from_euler(
            // TODO: Move this to the mesh struct
            EulerRot::ZYX,
            self.mesh.rotation.z,
            self.mesh.rotation.y,
            self.mesh.rotation.x,
        );
        let world_matrix = Mat4::from_scale_rotation_translation(
            self.mesh.scale,
            rotation_quat,
            self.mesh.position,
        );
        let view_matrix = Mat4::look_at_lh(self.camera_pos, self.camera_target, self.camera_up);
        let projection_matrix = Mat4::perspective_lh(
            90.0_f32.to_radians(),
            display.get_aspect_ratio(),
            0.1,
            100.0,
        );

        // We will use distances from the center of the model to the frustum planes
        // to decide if the mesh is inside the frustum or not.
        // TODO: Define the frustum planes
        // TODO: Calculate mesh center and maximum radius
        // TODO: Transform the center of the mesh and the radius to camera space
        // TODO: Check if the mesh is inside the frustum

        ///////////////////////////////////////////////////////////////////////////////
        // Process the graphics pipeline stages for all the mesh triangles
        ///////////////////////////////////////////////////////////////////////////////
        // +-------------+
        // | Model space |  <-- original mesh vertices
        // +-------------+
        // |   +-------------+
        // `-> | World space |  <-- multiply by world matrix
        //     +-------------+
        //     |   +--------------+
        //     `-> | Camera space |  <-- multiply by view matrix
        //         +--------------+
        //         |    +--------------+
        //         `--> |  Projection  |  <-- multiply by projection matrix
        //              +--------------+
        //              |    +-------------+
        //              `--> | Image space |  <-- apply perspective divide
        //                   +-------------+
        //                   |    +-------------------+
        //                   `--> | Triangle clipping |  <-- Homogeneous coordinate clipping
        //                        +-------------------+
        //                        |    +--------------+
        //                        `--> | Screen space |  <-- ready to render
        //                             +--------------+
        ///////////////////////////////////////////////////////////////////////////////

        // clear the projected triangles vector
        self.transformed_triangles.clear(); // this is equivalent to .truncate(0)
        self.screen_triangles.clear(); // this is equivalent to .truncate(0)

        // loop faces to transform them to screen space and clip them
        for face in self.mesh.faces.iter() {
            // get the three vertices of the face
            let vertex1 = self.mesh.vertices[face.vertices[0]];
            let vertex2 = self.mesh.vertices[face.vertices[1]];
            let vertex3 = self.mesh.vertices[face.vertices[2]];

            //Get Vec4 vertices
            let vertex1 = Vec4::new(vertex1.x, vertex1.y, vertex1.z, 1.0);
            let vertex2 = Vec4::new(vertex2.x, vertex2.y, vertex2.z, 1.0);
            let vertex3 = Vec4::new(vertex3.x, vertex3.y, vertex3.z, 1.0);
            // world transformation (model space -> world space)
            let world_vertex1 = world_matrix * vertex1;
            let world_vertex2 = world_matrix * vertex2;
            let world_vertex3 = world_matrix * vertex3;
            // view transformation (world space -> camera space)
            let transformed_vertex1 = view_matrix * world_vertex1;
            let transformed_vertex2 = view_matrix * world_vertex2;
            let transformed_vertex3 = view_matrix * world_vertex3;

            // create triangle
            let transformed_triangle = Triangle::from_vertices_color(
                [
                    transformed_vertex1,
                    transformed_vertex2,
                    transformed_vertex3,
                ],
                face.color,
            ); // this calculates normal and center of the triangle
            let triangle_normal = transformed_triangle.normal;

            // decide if the triangle is visible or not
            // if the triangle is not visible, do not add it to the transformed triangles vector
            // as the triangle is transformed to camera space, we can use the z component to decide
            if transformed_triangle.is_visible() {
                self.transformed_triangles.push(transformed_triangle);
            } else {
                continue;
            }

            // reorder transformed_triangles by depth
            //self.transformed_triangles
            //    .sort_unstable_by(|a, b| b.center.z.partial_cmp(&a.center.z).unwrap());

            let mut projected_vertex1 = projection_matrix * transformed_vertex1;
            let mut projected_vertex2 = projection_matrix * transformed_vertex2;
            let mut projected_vertex3 = projection_matrix * transformed_vertex3;
            // flip y component to match the screen coordinate system
            projected_vertex1.y *= -1.0;
            projected_vertex2.y *= -1.0;
            projected_vertex3.y *= -1.0;

            // Clip in clip space (before perspective divide) to avoid
            // divide-by-zero for vertices behind the camera and to get
            // correct W interpolation at clip boundaries.
            let clip_triangle = Triangle::from_vertices_uv(
                [projected_vertex1, projected_vertex2, projected_vertex3],
                [
                    self.mesh.uvs[face.uvs[0]],
                    self.mesh.uvs[face.uvs[1]],
                    self.mesh.uvs[face.uvs[2]],
                ],
            );
            self.clip_buffer_ping.clear();
            self.clip_buffer_pong.clear();
            self.clip_buffer_ping.push(clip_triangle);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_w_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_x_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_y_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_z_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_nx_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_ny_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            if !Self::run_clip_stage(
                &mut self.clip_buffer_ping,
                &mut self.clip_buffer_pong,
                clip_triangle_nz_axis,
            ) {
                continue;
            }
            std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);

            // 4.- Perspective divide (after clipping, so W is always positive)
            //     then transform to screen space
            let width = self.width as f32;
            let height = self.height as f32;
            let light_dir = self.light_dir;
            let clipped_triangles = &self.clip_buffer_ping;
            let screen_triangles = &mut self.screen_triangles;

            for clipped_triangle in clipped_triangles.iter() {
                let mut screen_vertex1 = clipped_triangle.vertices[0];
                let mut screen_vertex2 = clipped_triangle.vertices[1];
                let mut screen_vertex3 = clipped_triangle.vertices[2];

                // Perspective divide: x,y,z /= w (keep w for rasterizer)
                let w1 = screen_vertex1.w;
                let w2 = screen_vertex2.w;
                let w3 = screen_vertex3.w;
                screen_vertex1.x /= w1;
                screen_vertex1.y /= w1;
                screen_vertex1.z /= w1;
                screen_vertex2.x /= w2;
                screen_vertex2.y /= w2;
                screen_vertex2.z /= w2;
                screen_vertex3.x /= w3;
                screen_vertex3.y /= w3;
                screen_vertex3.z /= w3;

                // Transform x and y to screen space
                screen_vertex1.x = (screen_vertex1.x + 1.0) * width / 2.0;
                screen_vertex1.y = (screen_vertex1.y + 1.0) * height / 2.0;
                screen_vertex2.x = (screen_vertex2.x + 1.0) * width / 2.0;
                screen_vertex2.y = (screen_vertex2.y + 1.0) * height / 2.0;
                screen_vertex3.x = (screen_vertex3.x + 1.0) * width / 2.0;
                screen_vertex3.y = (screen_vertex3.y + 1.0) * height / 2.0;

                // calculate face color based on the light direction and the normal of the face
                let face_color = render::calculate_face_color(
                    light_dir,
                    triangle_normal,
                    clipped_triangle.color,
                );

                screen_triangles.push(Triangle::from_vertices_uvs_normal_color(
                    [screen_vertex1, screen_vertex2, screen_vertex3],
                    [
                        clipped_triangle.uvs[0],
                        clipped_triangle.uvs[1],
                        clipped_triangle.uvs[2],
                    ],
                    triangle_normal,
                    face_color,
                ));
            }

            // // Transform x and y to screen space
            // let screen_vertex1 = Vec4::new(
            //     (perspective_vertex1.x + 1.0) * self.width as f32 / 2.0,
            //     (perspective_vertex1.y + 1.0) * self.height as f32 / 2.0,
            //     perspective_vertex1.z,
            //     perspective_vertex1.w,
            // );
            // let screen_vertex2 = Vec4::new(
            //     (perspective_vertex2.x + 1.0) * self.width as f32 / 2.0,
            //     (perspective_vertex2.y + 1.0) * self.height as f32 / 2.0,
            //     perspective_vertex2.z,
            //     perspective_vertex2.w,
            // );
            // let screen_vertex3 = Vec4::new(
            //     (perspective_vertex3.x + 1.0) * self.width as f32 / 2.0,
            //     (perspective_vertex3.y + 1.0) * self.height as f32 / 2.0,
            //     perspective_vertex3.z,
            //     perspective_vertex3.w,
            // );

            // // do perspective division and screen mapping
            // // let screen_vertex1 = Vec4::new(
            // //     (projected_vertex1.x / projected_vertex1.w) * self.width as f32 / 2.0
            // //         + self.width as f32 / 2.0,
            // //     (projected_vertex1.y / projected_vertex1.w) * self.height as f32 / 2.0
            // //         + self.height as f32 / 2.0,
            // //     projected_vertex1.z / projected_vertex1.w,
            // //     projected_vertex1.w,
            // // );
            // // let screen_vertex2 = Vec4::new(
            // //     (projected_vertex2.x / projected_vertex2.w) * self.width as f32 / 2.0
            // //         + self.width as f32 / 2.0,
            // //     (projected_vertex2.y / projected_vertex2.w) * self.height as f32 / 2.0
            // //         + self.height as f32 / 2.0,
            // //     projected_vertex2.z / projected_vertex2.w,
            // //     projected_vertex2.w,
            // // );
            // // let screen_vertex3 = Vec4::new(
            // //     (projected_vertex3.x / projected_vertex3.w) * self.width as f32 / 2.0
            // //         + self.width as f32 / 2.0,
            // //     (projected_vertex3.y / projected_vertex3.w) * self.height as f32 / 2.0
            // //         + self.height as f32 / 2.0,
            // //     projected_vertex3.z / projected_vertex3.w,
            // //     projected_vertex3.w,
            // // );

            // // calculate face color based on the light direction and the normal of the face
            // let face_color = render::calculate_face_color(
            //     self.light_dir,
            //     self.transformed_triangles.last().unwrap().normal,
            //     face.color,
            // );

            // // push screen space vertices
            // let screen_triangle = Triangle::from_vertices_uvs_normal_color(
            //     [screen_vertex1, screen_vertex2, screen_vertex3],
            //     [
            //         self.mesh.uvs[face.uvs[0]],
            //         self.mesh.uvs[face.uvs[1]],
            //         self.mesh.uvs[face.uvs[2]],
            //     ],
            //     self.transformed_triangles.last().unwrap().normal,
            //     face_color,
            // );
            // self.screen_triangles.push(screen_triangle);
        }
        // sort screen triangles by depth
        self.screen_triangles
            .sort_unstable_by(|a, b| a.center.z.partial_cmp(&b.center.z).unwrap());

        // clear color buffer and z buffer
        // TODO: move buffer clearing to a separate function
        // TODO: Call clear function in parallel to the present_canvas() function
        self.color_buffer.fill(0_u8);
        self.z_buffer.fill(0.0_f32);
        //self.color_buffer.iter_mut().for_each(|x| *x = 0_u8);
        //self.z_buffer.iter_mut().for_each(|x| *x = 0.0_f32);

        // fast copy color_buffer_clear to color_buffer
        //self.color_buffer.copy_from_slice(&self.color_buffer_clear);
        // fast copy z_buffer_clear to z_buffer
        //self.z_buffer.copy_from_slice(&self.z_buffer_clear);

        // for each triangle in the transformed triangles vector
        // draw the triangle to the color buffer
        for screen_triangle in self.screen_triangles.iter() {
            // draw triangle to color buffer
            // render::draw_2dtriangle_to_color_buffer(
            //     screen_triangle,
            //     &mut self.color_buffer,
            //     self.width,
            //     self.height,
            // );
            if let Some(texture) = &self.mesh.texture {
                self.render.draw_3dtriangle_to_color_buffer(
                    //render::draw_3dtriangle_to_color_buffer(
                    screen_triangle,
                    &mut self.color_buffer,
                    self.width,
                    self.height,
                    texture,
                    self.mesh.texture_width,
                    self.mesh.texture_height,
                    &mut self.z_buffer,
                );
            } else {
                render::draw_2dtriangle_to_color_buffer(
                    screen_triangle,
                    &mut self.color_buffer,
                    self.width,
                    self.height,
                );
            }
        }
    }

    fn render(&self, display: &mut Display) {
        //display.streaming_buffer_to_canvas(self.buffer_name.as_str());
        display.color_buffer_to_canvas("cube", &self.color_buffer);
    }
}
