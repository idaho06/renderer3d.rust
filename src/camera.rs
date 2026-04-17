//! First-person camera with WASD movement.
//!
//! [`Camera`] stores position, target, and projection parameters.
//! [`Camera::update`] reads [`crate::userinput::UserInput`] each frame and moves the camera.
//!
//! See book chapter: _Camera and view matrix_ (TODO: link when mdBook is set up).

use glam::Vec3;

use crate::userinput::UserInput;

/// First-person camera for the 3D scene.
pub struct Camera {
    /// Camera position in world space.
    pub pos: Vec3,
    /// Point the camera is looking at in world space.
    pub target: Vec3,
    /// World-space up vector (usually `Vec3::Y`).
    pub up: Vec3,
    /// Movement speed in world units per second.
    pub speed: f32,
    /// Vertical field of view in radians.
    pub fov: f32,
    /// Near clip plane distance.
    pub near: f32,
    /// Far clip plane distance.
    pub far: f32,
}

impl Camera {
    #[must_use]
    /// Creates a camera looking from `pos` toward `target` with default projection settings.
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

    /// Moves the camera based on WASD keys held this frame.
    ///
    /// `dt` is the elapsed time in milliseconds since the last frame.
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
