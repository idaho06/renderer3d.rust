use glam::Vec3;

use crate::userinput::UserInput;

pub struct Camera {
    pub pos: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub speed: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    #[must_use]
    pub fn new(pos: Vec3, target: Vec3) -> Self {
        Self {
            pos,
            target,
            up: Vec3::Y,
            speed: 20.0,
            fov: 90.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn update(&mut self, input: &UserInput, dt: u32) {
        #[allow(clippy::cast_precision_loss)]
        let time_factor = dt as f32 / 1000.0;
        let direction = (self.target - self.pos).normalize();
        let right = direction.cross(self.up).normalize();

        if input.key_w.pressed {
            self.pos += direction * self.speed * time_factor;
        }
        if input.key_s.pressed {
            self.pos -= direction * self.speed * time_factor;
        }
        if input.key_a.pressed {
            self.pos -= right * self.speed * time_factor;
        }
        if input.key_d.pressed {
            self.pos += right * self.speed * time_factor;
        }
    }
}
