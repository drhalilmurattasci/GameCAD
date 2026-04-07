//! 3D manipulation gizmo for translate, rotate, and scale operations.

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

/// The active transformation mode for the gizmo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
    None,
}

/// The coordinate space in which gizmo operations are performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GizmoSpace {
    Local,
    World,
}

/// An axis or plane that a drag operation is constrained to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
}

impl Axis {
    /// Returns the unit direction vector for single-axis variants, or the
    /// plane normal for dual-axis (plane) variants.
    #[inline]
    pub fn direction(&self) -> Vec3 {
        match self {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
            Axis::XY => Vec3::Z, // normal of the XY plane
            Axis::XZ => Vec3::Y, // normal of the XZ plane
            Axis::YZ => Vec3::X, // normal of the YZ plane
        }
    }

    /// Returns `true` if this is a single-axis constraint.
    #[inline]
    pub fn is_single(&self) -> bool {
        matches!(self, Axis::X | Axis::Y | Axis::Z)
    }
}

/// State for an active gizmo drag operation.
#[derive(Debug, Clone)]
struct DragState {
    /// The axis or plane the drag is constrained to.
    axis: Axis,
    /// World-space position where the drag started.
    start_point: Vec3,
    /// Most recent world-space position during the drag.
    current_point: Vec3,
    /// Gizmo position when the drag began, used to compute total delta.
    original_position: Vec3,
}

/// A 3D manipulation gizmo for translate, rotate, and scale operations.
#[derive(Debug, Clone)]
pub struct Gizmo {
    pub mode: GizmoMode,
    pub space: GizmoSpace,
    pub position: Vec3,
    pub orientation: Mat4,
    pub scale: f32,
    drag: Option<DragState>,
}

impl Gizmo {
    /// Creates a new gizmo in translate mode at the origin.
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            space: GizmoSpace::World,
            position: Vec3::ZERO,
            orientation: Mat4::IDENTITY,
            scale: 1.0,
            drag: None,
        }
    }

    /// Tests whether a ray (given as origin + direction) hits one of the gizmo
    /// handles. Returns the axis that was hit, if any.
    ///
    /// `ray_origin` and `ray_dir` are in world space. `ray_dir` must be normalized.
    /// `handle_length` controls the size of the hit region.
    pub fn hit_test(
        &self,
        ray_origin: Vec3,
        ray_dir: Vec3,
        handle_length: f32,
    ) -> Option<Axis> {
        if self.mode == GizmoMode::None {
            return None;
        }

        let ray_dir_norm = if ray_dir.length_squared() < 1e-10 {
            return None;
        } else {
            ray_dir.normalize()
        };

        let threshold = handle_length * 0.1 * self.scale;
        let axes = [Axis::X, Axis::Y, Axis::Z];
        let dirs = [Vec3::X, Vec3::Y, Vec3::Z];

        let mut best: Option<(Axis, f32)> = None;

        for (axis, dir) in axes.iter().zip(dirs.iter()) {
            let handle_end = self.position + *dir * handle_length * self.scale;
            let dist = ray_segment_distance(ray_origin, ray_dir_norm, self.position, handle_end);
            if dist < threshold
                && best.is_none_or(|(_, prev_dist)| dist < prev_dist)
            {
                best = Some((*axis, dist));
            }
        }

        best.map(|(axis, _)| axis)
    }

    /// Begins a drag operation along the given axis.
    pub fn begin_drag(&mut self, axis: Axis, start_point: Vec3) {
        self.drag = Some(DragState {
            axis,
            start_point,
            current_point: start_point,
            original_position: self.position,
        });
    }

    /// Updates the current drag, returning the delta from the drag start.
    #[inline]
    pub fn update_drag(&mut self, current_point: Vec3) -> Option<Vec3> {
        if let Some(ref mut drag) = self.drag {
            drag.current_point = current_point;
            let raw_delta = current_point - drag.start_point;

            let constrained = match drag.axis {
                Axis::X => Vec3::new(raw_delta.x, 0.0, 0.0),
                Axis::Y => Vec3::new(0.0, raw_delta.y, 0.0),
                Axis::Z => Vec3::new(0.0, 0.0, raw_delta.z),
                Axis::XY => Vec3::new(raw_delta.x, raw_delta.y, 0.0),
                Axis::XZ => Vec3::new(raw_delta.x, 0.0, raw_delta.z),
                Axis::YZ => Vec3::new(0.0, raw_delta.y, raw_delta.z),
            };

            self.position = drag.original_position + constrained;
            Some(constrained)
        } else {
            None
        }
    }

    /// Ends the current drag operation, returning the total delta.
    pub fn end_drag(&mut self) -> Option<Vec3> {
        if let Some(drag) = self.drag.take() {
            Some(self.position - drag.original_position)
        } else {
            None
        }
    }

    /// Returns `true` if a drag is currently active.
    #[inline]
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Cancels the current drag, restoring the original position.
    pub fn cancel_drag(&mut self) {
        if let Some(drag) = self.drag.take() {
            self.position = drag.original_position;
        }
    }
}

impl Default for Gizmo {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the minimum distance between a ray and a line segment.
///
/// Uses the standard closest-point-between-two-lines algorithm with
/// proper handling of degenerate cases (parallel lines, zero-length segments).
fn ray_segment_distance(
    ray_origin: Vec3,
    ray_dir: Vec3,
    seg_start: Vec3,
    seg_end: Vec3,
) -> f32 {
    let u = ray_dir;
    let v = seg_end - seg_start;
    let w = ray_origin - seg_start;

    let a = u.dot(u); // always >= 0
    let b = u.dot(v);
    let c = v.dot(v); // always >= 0
    let d = u.dot(w);
    let e = v.dot(w);

    let denom = a * c - b * b;

    // If segment is degenerate (zero-length), just compute point-to-ray distance
    if c < 1e-10 {
        // Segment is a point; find closest point on ray
        let s = if a > 1e-10 { (-d / a).max(0.0) } else { 0.0 };
        let closest_ray = ray_origin + u * s;
        return closest_ray.distance(seg_start);
    }

    if denom < 1e-7 {
        // Nearly parallel: use the segment midpoint heuristic
        let t = (e / c).clamp(0.0, 1.0);
        let seg_point = seg_start + v * t;
        // Find closest point on ray to seg_point
        let s = u.dot(seg_point - ray_origin);
        let s = if a > 1e-10 { (s / a).max(0.0) } else { 0.0 };
        let ray_point = ray_origin + u * s;
        return ray_point.distance(seg_point);
    }

    // General case: compute initial unclamped parameters
    let _s0 = (b * e - c * d) / denom;
    let t0 = (a * e - b * d) / denom;

    // Clamp t to segment [0, 1], then re-derive s
    let mut t = t0.clamp(0.0, 1.0);

    // Re-compute s for the clamped t
    let mut s = if a > 1e-10 {
        ((b * t - d) / a).max(0.0)
    } else {
        0.0
    };

    // Re-compute t for the clamped s
    let t_for_s = if c > 1e-10 {
        (b * s + e) / c
    } else {
        0.0
    };
    t = t_for_s.clamp(0.0, 1.0);

    // Final s computation
    let s_final = if a > 1e-10 {
        (b * t - d) / a
    } else {
        0.0
    };
    s = s_final.max(0.0);

    let closest_ray = ray_origin + u * s;
    let closest_seg = seg_start + v * t;
    closest_ray.distance(closest_seg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gizmo_defaults() {
        let g = Gizmo::new();
        assert_eq!(g.mode, GizmoMode::Translate);
        assert_eq!(g.space, GizmoSpace::World);
        assert!(!g.is_dragging());
    }

    #[test]
    fn drag_lifecycle() {
        let mut g = Gizmo::new();
        g.begin_drag(Axis::X, Vec3::ZERO);
        assert!(g.is_dragging());

        let delta = g.update_drag(Vec3::new(5.0, 3.0, 2.0));
        assert!(delta.is_some());
        let d = delta.unwrap();
        assert!((d.x - 5.0).abs() < 1e-5);
        assert_eq!(d.y, 0.0); // constrained to X
        assert_eq!(d.z, 0.0);

        let total = g.end_drag();
        assert!(total.is_some());
        assert!(!g.is_dragging());
    }

    #[test]
    fn drag_on_plane_xy() {
        let mut g = Gizmo::new();
        g.begin_drag(Axis::XY, Vec3::ZERO);
        let delta = g.update_drag(Vec3::new(3.0, 4.0, 5.0)).unwrap();
        assert!((delta.x - 3.0).abs() < 1e-5);
        assert!((delta.y - 4.0).abs() < 1e-5);
        assert_eq!(delta.z, 0.0); // Z constrained out
    }

    #[test]
    fn cancel_drag_restores_position() {
        let mut g = Gizmo::new();
        g.position = Vec3::new(1.0, 2.0, 3.0);
        let original = g.position;
        g.begin_drag(Axis::X, Vec3::ZERO);
        g.update_drag(Vec3::new(10.0, 0.0, 0.0));
        g.cancel_drag();
        assert_eq!(g.position, original);
        assert!(!g.is_dragging());
    }

    #[test]
    fn end_drag_without_begin_returns_none() {
        let mut g = Gizmo::new();
        assert!(g.end_drag().is_none());
    }

    #[test]
    fn cancel_drag_without_begin_is_noop() {
        let mut g = Gizmo::new();
        let pos = g.position;
        g.cancel_drag();
        assert_eq!(g.position, pos);
    }

    #[test]
    fn hit_test_returns_none_when_mode_none() {
        let mut g = Gizmo::new();
        g.mode = GizmoMode::None;
        assert!(g.hit_test(Vec3::ZERO, Vec3::X, 1.0).is_none());
    }

    #[test]
    fn hit_test_returns_none_for_zero_direction() {
        let g = Gizmo::new();
        assert!(g.hit_test(Vec3::ZERO, Vec3::ZERO, 1.0).is_none());
    }

    #[test]
    fn hit_test_detects_x_axis() {
        let mut g = Gizmo::new();
        g.position = Vec3::ZERO;
        // Ray passes very close to the X axis handle (at y=0, offset along X)
        // Shooting from z=-1 toward z=+1, offset slightly in X to be near the X handle
        let result = g.hit_test(
            Vec3::new(0.5, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
        );
        assert_eq!(result, Some(Axis::X));
    }

    #[test]
    fn hit_test_misses_far_ray() {
        let g = Gizmo::new();
        let result = g.hit_test(
            Vec3::new(100.0, 100.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
        );
        assert!(result.is_none());
    }

    #[test]
    fn ray_segment_distance_perpendicular() {
        let dist = ray_segment_distance(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::X,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
        );
        assert!((dist - 1.0).abs() < 1e-4);
    }

    #[test]
    fn ray_segment_distance_zero_length_segment() {
        let dist = ray_segment_distance(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::X,
            Vec3::ZERO,
            Vec3::ZERO, // degenerate segment
        );
        assert!((dist - 1.0).abs() < 1e-4);
    }

    #[test]
    fn ray_segment_distance_parallel() {
        // Ray along X at y=1, segment along X at y=0
        let dist = ray_segment_distance(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::X,
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
        );
        assert!((dist - 1.0).abs() < 1e-4);
    }

    #[test]
    fn axis_direction_single() {
        assert_eq!(Axis::X.direction(), Vec3::X);
        assert_eq!(Axis::Y.direction(), Vec3::Y);
        assert_eq!(Axis::Z.direction(), Vec3::Z);
    }

    #[test]
    fn axis_direction_plane() {
        assert_eq!(Axis::XY.direction(), Vec3::Z);
        assert_eq!(Axis::XZ.direction(), Vec3::Y);
        assert_eq!(Axis::YZ.direction(), Vec3::X);
    }

    #[test]
    fn axis_is_single() {
        assert!(Axis::X.is_single());
        assert!(Axis::Y.is_single());
        assert!(Axis::Z.is_single());
        assert!(!Axis::XY.is_single());
        assert!(!Axis::XZ.is_single());
        assert!(!Axis::YZ.is_single());
    }
}
