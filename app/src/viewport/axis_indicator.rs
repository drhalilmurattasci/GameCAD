//! Axis gizmo indicator drawn in the bottom-left corner of the viewport.
//!
//! Shows the camera-relative orientation of the X (red), Y (green), and
//! Z (blue) world axes, sorted back-to-front for correct overlap.

use egui::{Color32, FontId, Pos2, Rect, Stroke};
use glam::Mat4;

use crate::state::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the orientation axis gizmo in the bottom-left corner of `rect`.
    pub(crate) fn draw_axis_gizmo(painter: &egui::Painter, view: &Mat4, rect: &Rect) {
        let center = Pos2::new(rect.left() + 50.0, rect.bottom() - 50.0);
        let gizmo_len = 30.0;

        let axes: [(glam::Vec3, Color32, &str); 3] = [
            (glam::Vec3::X, Color32::from_rgb(0xe7, 0x4c, 0x3c), "X"),
            (glam::Vec3::Y, Color32::from_rgb(0x2e, 0xcc, 0x71), "Y"),
            (glam::Vec3::Z, Color32::from_rgb(0x3e, 0x55, 0xff), "Z"),
        ];

        // Sort by depth (back-to-front), using total_cmp to avoid panic on NaN
        let mut sorted: Vec<(glam::Vec3, Color32, &str, f32)> = axes
            .iter()
            .map(|(dir, color, label)| {
                let rotated = view.transform_vector3(*dir);
                (*dir, *color, *label, rotated.z)
            })
            .collect();
        sorted.sort_by(|a, b| a.3.total_cmp(&b.3));

        for (dir, color, label, _) in &sorted {
            let rotated = view.transform_vector3(*dir);
            let screen_end = Pos2::new(
                center.x + rotated.x * gizmo_len,
                center.y - rotated.y * gizmo_len,
            );
            painter.line_segment([center, screen_end], Stroke::new(2.5, *color));
            painter.text(
                Pos2::new(
                    screen_end.x + (screen_end.x - center.x).signum() * 6.0,
                    screen_end.y + (screen_end.y - center.y).signum() * 6.0,
                ),
                egui::Align2::CENTER_CENTER,
                *label,
                FontId::proportional(11.0),
                *color,
            );
        }

        painter.circle_stroke(
            center,
            gizmo_len + 8.0,
            Stroke::new(
                1.0,
                Color32::from_rgba_premultiplied(0x3a, 0x3a, 0x5a, 60),
            ),
        );
    }
}
