//! Low-level 3D-to-2D projection and line-drawing helpers.
//!
//! `project_3d` transforms a world-space point through the view-projection
//! matrix into screen-space coordinates.  `draw_line_3d` projects both
//! endpoints and draws a painter line segment.

use egui::{Pos2, Rect, Stroke};
use glam::{Mat4, Vec3, Vec4};

use crate::state::ForgeEditorApp;

impl ForgeEditorApp {
    /// Project a world-space point to screen-space, returning `None` if behind the camera.
    pub(crate) fn project_3d(vp: &Mat4, rect: &Rect, world_pos: Vec3) -> Option<Pos2> {
        let clip = *vp * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
        if clip.w <= 0.0001 {
            return None;
        }
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        let sx = rect.left() + (ndc_x + 1.0) * 0.5 * rect.width();
        let sy = rect.top() + (1.0 - ndc_y) * 0.5 * rect.height();
        Some(Pos2::new(sx, sy))
    }

    /// Project two world-space points and draw a line segment between them.
    pub(crate) fn draw_line_3d(
        painter: &egui::Painter,
        vp: &Mat4,
        rect: &Rect,
        a: Vec3,
        b: Vec3,
        stroke: Stroke,
    ) {
        if let (Some(pa), Some(pb)) = (
            Self::project_3d(vp, rect, a),
            Self::project_3d(vp, rect, b),
        ) {
            painter.line_segment([pa, pb], stroke);
        }
    }
}
