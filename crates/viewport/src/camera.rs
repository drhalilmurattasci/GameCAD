//! Orbit camera implementation for the 3D viewport.

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Preset axis-aligned views for the camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisView {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

/// An orbit camera that revolves around a target point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitCamera {
    /// The point the camera orbits around.
    pub target: Vec3,
    /// Distance from the target.
    pub distance: f32,
    /// Horizontal rotation angle in radians.
    pub yaw: f32,
    /// Vertical rotation angle in radians (clamped to avoid gimbal lock).
    pub pitch: f32,
    /// Vertical field of view in radians.
    pub fov: f32,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
}

impl OrbitCamera {
    /// Creates a new orbit camera looking at `target` from `distance` away.
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            yaw: 0.0,
            pitch: 0.3,
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Computes the world-space position of the camera.
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    /// Builds a view matrix (world-to-camera transform).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }

    /// Builds a perspective projection matrix for the given aspect ratio.
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect, self.near, self.far)
    }

    /// Orbits the camera by the given horizontal and vertical deltas (in radians).
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch = (self.pitch + dy).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }

    /// Pans the camera (translates both target and eye) in the view plane.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view_matrix();
        let right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);
        let up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);

        let pan_speed = self.distance * 0.002;
        self.target += right * (-dx * pan_speed) + up * (dy * pan_speed);
    }

    /// Zooms by adjusting the distance from the target. Positive delta zooms out.
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta * self.distance * 0.1).max(0.01);
    }

    /// Focuses the camera on a target point, fitting an object of `bounds_size` in view.
    pub fn focus_on(&mut self, target: Vec3, bounds_size: f32) {
        self.target = target;
        self.distance = (bounds_size * 0.5) / (self.fov * 0.5).tan();
    }

    /// Snaps the camera to a preset axis-aligned view.
    pub fn set_axis_view(&mut self, view: AxisView) {
        use std::f32::consts::{FRAC_PI_2, PI};
        match view {
            AxisView::Front => {
                self.yaw = 0.0;
                self.pitch = 0.0;
            }
            AxisView::Back => {
                self.yaw = PI;
                self.pitch = 0.0;
            }
            AxisView::Left => {
                self.yaw = -FRAC_PI_2;
                self.pitch = 0.0;
            }
            AxisView::Right => {
                self.yaw = FRAC_PI_2;
                self.pitch = 0.0;
            }
            AxisView::Top => {
                self.yaw = 0.0;
                self.pitch = FRAC_PI_2 - 0.01;
            }
            AxisView::Bottom => {
                self.yaw = 0.0;
                self.pitch = -(FRAC_PI_2 - 0.01);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_camera_has_sane_defaults() {
        let cam = OrbitCamera::new(Vec3::ZERO, 10.0);
        assert_eq!(cam.target, Vec3::ZERO);
        assert_eq!(cam.distance, 10.0);
        assert!(cam.fov > 0.0);
    }

    #[test]
    fn position_is_correct_distance() {
        let cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let pos = cam.position();
        let dist = pos.distance(cam.target);
        assert!((dist - 5.0).abs() < 1e-4);
    }

    #[test]
    fn zoom_clamps_min() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 0.1);
        cam.zoom(-1000.0);
        assert!(cam.distance >= 0.01);
    }

    #[test]
    fn orbit_clamps_pitch() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.orbit(0.0, 100.0);
        assert!(cam.pitch < std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn axis_view_front() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.set_axis_view(AxisView::Front);
        assert_eq!(cam.yaw, 0.0);
        assert_eq!(cam.pitch, 0.0);
    }
}
