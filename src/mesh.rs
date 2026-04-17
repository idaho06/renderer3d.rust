use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};

use crate::{
    camera::Camera,
    clipping::{
        TriangleClipResult, clip_triangle_nx_axis, clip_triangle_ny_axis, clip_triangle_nz_axis,
        clip_triangle_w_axis, clip_triangle_x_axis, clip_triangle_y_axis, clip_triangle_z_axis,
    },
    display::Display,
    framebuffer::Framebuffer,
    model::Model,
    render,
    scene::Scene,
    triangle::Triangle,
};

static CLIP_PLANES: [fn(Triangle) -> TriangleClipResult; 7] = [
    clip_triangle_w_axis,
    clip_triangle_x_axis,
    clip_triangle_y_axis,
    clip_triangle_z_axis,
    clip_triangle_nx_axis,
    clip_triangle_ny_axis,
    clip_triangle_nz_axis,
];

const CLIP_BUFFER_CAPACITY: usize = 128;

pub enum ModelSource {
    BuiltinCube,
    Obj { obj_path: String, png_path: String },
}

pub struct Mesh {
    model: Model,
    camera: Camera,
    framebuffer: Framebuffer,
    transformed_triangles: Vec<Triangle>,
    projected_triangles: Vec<Triangle>,
    screen_triangles: Vec<Triangle>,
    clip_buffer_ping: Vec<Triangle>,
    clip_buffer_pong: Vec<Triangle>,
    light_dir: Vec3,
    now_time: u32,
}

impl Mesh {
    pub fn new(display: &mut Display, source: ModelSource) -> Self {
        let model = match source {
            ModelSource::BuiltinCube => Model::builtin_cube(),
            ModelSource::Obj { obj_path, png_path } => {
                Model::from_obj_and_png(&obj_path, &png_path)
            }
        };

        let face_count = model.faces.len();
        let width = 640 * 2;
        let height = 360 * 2;
        display.add_streaming_buffer("cube", width, height);

        let camera = Camera::new(Vec3::new(0.0, 25.0, 55.0), Vec3::ZERO);

        Self {
            transformed_triangles: Vec::with_capacity(face_count),
            projected_triangles: Vec::with_capacity(face_count),
            screen_triangles: Vec::with_capacity(face_count * 2),
            clip_buffer_ping: Vec::with_capacity(CLIP_BUFFER_CAPACITY),
            clip_buffer_pong: Vec::with_capacity(CLIP_BUFFER_CAPACITY),
            framebuffer: Framebuffer::new(width, height),
            model,
            camera,
            light_dir: Vec3::new(1.0, -1.0, 1.0).normalize(),
            now_time: 0,
        }
    }

    fn run_input(&mut self, display: &Display, dt: u32) {
        self.camera.update(&display.user_input, dt);
    }

    fn build_matrices(&self, display: &Display) -> (Mat4, Mat4, Mat4) {
        let rotation_quat = Quat::from_euler(
            EulerRot::ZYX,
            self.model.rotation.z,
            self.model.rotation.y,
            self.model.rotation.x,
        );
        let world_matrix = Mat4::from_scale_rotation_translation(
            self.model.scale,
            rotation_quat,
            self.model.position,
        );
        let view_matrix = Mat4::look_at_lh(self.camera.pos, self.camera.target, self.camera.up);
        let proj_matrix = Mat4::perspective_lh(
            self.camera.fov,
            display.get_aspect_ratio(),
            self.camera.near,
            self.camera.far,
        );
        (world_matrix, view_matrix, proj_matrix)
    }

    fn transform_to_camera_space(&mut self, world_matrix: Mat4, view_matrix: Mat4) {
        self.transformed_triangles.clear();
        let light_dir = self.light_dir;

        for face in &self.model.faces {
            let v = [0, 1, 2].map(|i| {
                let p = self.model.vertices[face.vertices[i]];
                view_matrix * world_matrix * Vec4::new(p.x, p.y, p.z, 1.0)
            });
            let uvs = [0, 1, 2].map(|i| self.model.uvs[face.uvs[i]]);

            let normal = {
                let a = Vec3::new(v[0].x, v[0].y, v[0].z);
                let b = Vec3::new(v[1].x, v[1].y, v[1].z);
                let c = Vec3::new(v[2].x, v[2].y, v[2].z);
                (b - a).cross(c - a).normalize()
            };

            let lit_color = render::calculate_face_color(light_dir, normal, face.color);
            self.transformed_triangles.push(Triangle::from_vertices_uvs_normal_color(
                v, uvs, normal, lit_color,
            ));
        }
    }

    fn backface_cull(&mut self) {
        self.transformed_triangles.retain(Triangle::is_visible);
    }

    fn project(&mut self, proj_matrix: Mat4) {
        self.projected_triangles.clear();
        for tri in &self.transformed_triangles {
            let v = tri.vertices.map(|v| {
                let mut p = proj_matrix * v;
                p.y *= -1.0;
                p
            });
            self.projected_triangles.push(Triangle::from_vertices_uvs_normal_color(
                v, tri.uvs, tri.normal, tri.color,
            ));
        }
    }

    fn clip_against_frustum(&mut self) {
        self.screen_triangles.clear();
        for projected in self.projected_triangles.drain(..) {
            self.clip_buffer_ping.clear();
            self.clip_buffer_ping.push(projected);

            let mut survived = true;
            for &clip_fn in &CLIP_PLANES {
                if !Self::run_clip_stage(
                    &mut self.clip_buffer_ping,
                    &mut self.clip_buffer_pong,
                    clip_fn,
                ) {
                    survived = false;
                    break;
                }
                std::mem::swap(&mut self.clip_buffer_ping, &mut self.clip_buffer_pong);
            }

            if survived {
                self.screen_triangles.append(&mut self.clip_buffer_ping);
            }
        }
    }

    fn perspective_divide_and_map(&mut self) {
        #[allow(clippy::cast_precision_loss)]
        let width = self.framebuffer.width as f32;
        #[allow(clippy::cast_precision_loss)]
        let height = self.framebuffer.height as f32;

        for tri in &mut self.screen_triangles {
            for v in &mut tri.vertices {
                let w = v.w;
                v.x /= w;
                v.y /= w;
                v.z /= w;
                v.x = (v.x + 1.0) * width / 2.0;
                v.y = (v.y + 1.0) * height / 2.0;
            }
        }
    }

    fn rasterize_all(&mut self) {
        self.framebuffer.clear();
        let width = self.framebuffer.width;
        let height = self.framebuffer.height;

        for tri in &self.screen_triangles {
            if let Some(texture) = &self.model.texture {
                render::draw_3dtriangle_to_color_buffer(
                    tri,
                    &mut self.framebuffer.color_buffer,
                    width,
                    height,
                    texture,
                    self.model.texture_width,
                    self.model.texture_height,
                    &mut self.framebuffer.z_buffer,
                );
            } else {
                render::draw_2dtriangle_to_color_buffer(
                    tri,
                    &mut self.framebuffer.color_buffer,
                    width,
                    height,
                );
            }
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
                TriangleClipResult::OneTriangle(t) => destination.push(t),
                TriangleClipResult::TwoTriangles(t1, t2) => {
                    destination.push(t1);
                    destination.push(t2);
                }
                TriangleClipResult::NoTriangle => {}
            }
        }
        !destination.is_empty()
    }
}

impl Scene for Mesh {
    fn update(&mut self, delta_time: u32, display: &Display) {
        self.now_time += delta_time;
        self.run_input(display, delta_time);
        let (world_matrix, view_matrix, proj_matrix) = self.build_matrices(display);
        self.transform_to_camera_space(world_matrix, view_matrix);
        self.backface_cull();
        self.project(proj_matrix);
        self.clip_against_frustum();
        self.perspective_divide_and_map();
        self.rasterize_all();
    }

    fn render(&self, display: &mut Display) {
        display.color_buffer_to_canvas("cube", &self.framebuffer.color_buffer);
    }
}
