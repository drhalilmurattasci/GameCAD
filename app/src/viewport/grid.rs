//! Perspective ground grid and world-origin axis lines.
//!
//! Draws a planar grid on Y=0 with minor / major line distinction,
//! plus red (X), green (Y), and blue (Z) world-origin axis indicators.

use egui::{Color32, Rect, Stroke};
use glam::{Mat4, Vec3};

use crate::app::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the ground grid and world-origin axes using 3D line projection.
    pub(crate) fn draw_perspective_grid(
        painter: &egui::Painter,
        vp: &Mat4,
        rect: &Rect,
        grid_color: Color32,
        major_color: Color32,
        grid_spacing: f32,
    ) {
        let grid_extent: i32 = (20.0 / grid_spacing.max(0.1)) as i32;
        let spacing = grid_spacing.max(0.1);

        let minor_stroke = Stroke::new(1.0, grid_color);
        let major_stroke = Stroke::new(1.2, major_color);

        for i in -grid_extent..=grid_extent {
            let f = i as f32 * spacing;
            let is_major = i % 5 == 0;
            let stroke = if is_major { major_stroke } else { minor_stroke };
            let ext = grid_extent as f32 * spacing;

            Self::draw_line_3d(
                painter,
                vp,
                rect,
                Vec3::new(-ext, 0.0, f),
                Vec3::new(ext, 0.0, f),
                stroke,
            );
            Self::draw_line_3d(
                painter,
                vp,
                rect,
                Vec3::new(f, 0.0, -ext),
                Vec3::new(f, 0.0, ext),
                stroke,
            );
        }

        // World origin axes
        let axis_len = 3.0;
        Self::draw_line_3d(
            painter,
            vp,
            rect,
            Vec3::ZERO,
            Vec3::new(axis_len, 0.0, 0.0),
            Stroke::new(2.5, Color32::from_rgb(0xe7, 0x4c, 0x3c)),
        );
        Self::draw_line_3d(
            painter,
            vp,
            rect,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, axis_len),
            Stroke::new(2.5, Color32::from_rgb(0x3e, 0x55, 0xff)),
        );
        Self::draw_line_3d(
            painter,
            vp,
            rect,
            Vec3::ZERO,
            Vec3::new(0.0, axis_len, 0.0),
            Stroke::new(2.5, Color32::from_rgb(0x2e, 0xcc, 0x71)),
        );
    }
}
