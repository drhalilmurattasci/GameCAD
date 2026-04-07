//! Orbit camera implementation for the 3D viewport.

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// Minimum distance the camera can be from the target.
const MIN_DISTANCE: f32 = 0.01;

/// Pitch epsilon to prevent gimbal lock at the poles.
const PITCH_EPSILON: f32 = 0.001;

/// Preset axis-aligned views for the camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisView {
    /// Looking along -Z.
    Front,
    /// Looking along +Z.
    Back,
    /// Looking along +X.
    Left,
    /// Looking along -X.
    Right,
    /// Looking along -Y (top-down).
    Top,
    /// Looking along +Y (bottom-up).
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
            distance: distance.max(MIN_DISTANCE),
            yaw: 0.0,
            pitch: 0.3,
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Computes the world-space position of the camera.
    #[inline]
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    /// Builds a view matrix (world-to-camera transform).
    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        let pos = self.position();
        // Guard against degenerate case where position equals target
        if pos.distance_squared(self.target) < 1e-10 {
            return Mat4::IDENTITY;
        }
        Mat4::look_at_rh(pos, self.target, Vec3::Y)
    }

    /// Builds a perspective projection matrix for the given aspect ratio.
    ///
    /// `aspect` is clamped to a minimum of `0.001` to prevent division by zero
    /// or degenerate projections.
    #[inline]
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        let safe_aspect = aspect.max(0.001);
        Mat4::perspective_rh(self.fov, safe_aspect, self.near, self.far)
    }

    /// Orbits the camera by the given horizontal and vertical deltas (in radians).
    #[inline]
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        // Keep yaw in [-PI, PI] to avoid precision loss
        self.yaw = wrap_angle(self.yaw);
        self.pitch = (self.pitch + dy).clamp(
            -std::f32::consts::FRAC_PI_2 + PITCH_EPSILON,
            std::f32::consts::FRAC_PI_2 - PITCH_EPSILON,
        );
    }

    /// Pans the camera (translates both target and eye) in the view plane.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view_matrix();
        // In a row-major convention, the right and up vectors of the camera
        // are stored in the first two rows of the 3x3 rotation portion.
        // glam stores matrices column-major, so row i is col(j)[i].
        let right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);
        let up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);

        let pan_speed = self.distance * 0.002;
        self.target += right * (-dx * pan_speed) + up * (dy * pan_speed);
    }

    /// Zooms by adjusting the distance from the target. Positive delta zooms out.
    #[inline]
    pub fn zoom(&mut self, delta: f32) {
        let factor = delta * self.distance * 0.1;
        self.distance = (self.distance + factor).max(MIN_DISTANCE);
    }

    /// Focuses the camera on a target point, fitting an object of `bounds_size` in view.
    pub fn focus_on(&mut self, target: Vec3, bounds_size: f32) {
        self.target = target;
        let half_fov_tan = (self.fov * 0.5).tan();
        if half_fov_tan.abs() < 1e-7 {
            // Degenerate FOV -- use a reasonable default
            self.distance = bounds_size * 2.0;
        } else {
            self.distance = ((bounds_size * 0.5) / half_fov_tan).max(MIN_DISTANCE);
        }
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
                self.pitch = FRAC_PI_2 - PITCH_EPSILON;
            }
            AxisView::Bottom => {
                self.yaw = 0.0;
                self.pitch = -(FRAC_PI_2 - PITCH_EPSILON);
            }
        }
    }

    /// Returns the view-projection matrix for the given aspect ratio.
    #[inline]
    pub fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }
}

/// Wraps an angle to the range [-PI, PI].
#[inline]
fn wrap_angle(angle: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let mut a = angle % (2.0 * pi);
    if a > pi {
        a -= 2.0 * pi;
    } else if a < -pi {
        a += 2.0 * pi;
    }
    a
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
    fn new_camera_clamps_zero_distance() {
        let cam = OrbitCamera::new(Vec3::ZERO, 0.0);
        assert!(cam.distance >= MIN_DISTANCE);
    }

    #[test]
    fn new_camera_clamps_negative_distance() {
        let cam = OrbitCamera::new(Vec3::ZERO, -5.0);
        assert!(cam.distance >= MIN_DISTANCE);
    }

    #[test]
    fn position_is_correct_distance() {
        let cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let pos = cam.position();
        let dist = pos.distance(cam.target);
        assert!((dist - 5.0).abs() < 1e-4);
    }

    #[test]
    fn position_with_nonzero_target() {
        let target = Vec3::new(10.0, 20.0, 30.0);
        let cam = OrbitCamera::new(target, 5.0);
        let pos = cam.position();
        let dist = pos.distance(cam.target);
        assert!((dist - 5.0).abs() < 1e-4);
    }

    #[test]
    fn zoom_clamps_min() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 0.1);
        cam.zoom(-1000.0);
        assert!(cam.distance >= MIN_DISTANCE);
    }

    #[test]
    fn zoom_out_increases_distance() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let old_dist = cam.distance;
        cam.zoom(1.0);
        assert!(cam.distance > old_dist);
    }

    #[test]
    fn zoom_in_decreases_distance() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let old_dist = cam.distance;
        cam.zoom(-1.0);
        assert!(cam.distance < old_dist);
    }

    #[test]
    fn orbit_clamps_pitch() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.orbit(0.0, 100.0);
        assert!(cam.pitch < std::f32::consts::FRAC_PI_2);
        assert!(cam.pitch > 0.0);

        cam.orbit(0.0, -200.0);
        assert!(cam.pitch > -std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn orbit_wraps_yaw() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.orbit(100.0, 0.0);
        assert!(cam.yaw.abs() <= std::f32::consts::PI + 0.001);
    }

    #[test]
    fn axis_view_front() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.set_axis_view(AxisView::Front);
        assert_eq!(cam.yaw, 0.0);
        assert_eq!(cam.pitch, 0.0);
    }

    #[test]
    fn axis_view_top_not_exactly_90() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.set_axis_view(AxisView::Top);
        assert!(cam.pitch < std::f32::consts::FRAC_PI_2);
        assert!(cam.pitch > std::f32::consts::FRAC_PI_2 - 0.01);
    }

    #[test]
    fn axis_view_bottom_not_exactly_neg_90() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.set_axis_view(AxisView::Bottom);
        assert!(cam.pitch > -std::f32::consts::FRAC_PI_2);
        assert!(cam.pitch < -std::f32::consts::FRAC_PI_2 + 0.01);
    }

    #[test]
    fn projection_matrix_clamps_aspect() {
        let cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        // Should not panic with zero or negative aspect
        let _m = cam.projection_matrix(0.0);
        let _m = cam.projection_matrix(-1.0);
    }

    #[test]
    fn view_matrix_identity_fallback_at_zero_distance() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, MIN_DISTANCE);
        cam.distance = 0.0; // Force zero
        let _m = cam.view_matrix(); // should not panic
    }

    #[test]
    fn focus_on_sets_target_and_distance() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        cam.focus_on(Vec3::new(10.0, 0.0, 0.0), 4.0);
        assert_eq!(cam.target, Vec3::new(10.0, 0.0, 0.0));
        assert!(cam.distance > 0.0);
    }

    #[test]
    fn view_projection_matrix_is_product() {
        let cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let vp = cam.view_projection_matrix(1.5);
        let expected = cam.projection_matrix(1.5) * cam.view_matrix();
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (vp.col(j)[i] - expected.col(j)[i]).abs() < 1e-5,
                    "Mismatch at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn pan_moves_target() {
        let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
        let old_target = cam.target;
        cam.pan(100.0, 0.0);
        assert_ne!(cam.target, old_target);
    }

    #[test]
    fn wrap_angle_stays_in_range() {
        assert!((wrap_angle(0.0)).abs() < 1e-5);
        assert!((wrap_angle(std::f32::consts::PI) - std::f32::consts::PI).abs() < 1e-5);
        let wrapped = wrap_angle(7.0);
        assert!(wrapped >= -std::f32::consts::PI && wrapped <= std::f32::consts::PI);
    }

    #[test]
    fn all_axis_views_produce_valid_view_matrix() {
        let views = [
            AxisView::Front,
            AxisView::Back,
            AxisView::Left,
            AxisView::Right,
            AxisView::Top,
            AxisView::Bottom,
        ];
        for view in views {
            let mut cam = OrbitCamera::new(Vec3::ZERO, 5.0);
            cam.set_axis_view(view);
            let m = cam.view_matrix();
            // The view matrix should be finite
            for i in 0..4 {
                for j in 0..4 {
                    assert!(m.col(j)[i].is_finite(), "Non-finite value in view matrix for {view:?}");
                }
            }
        }
    }
}
