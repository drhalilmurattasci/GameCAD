//! Central viewport panel and its sub-modules.
//!
//! The viewport renders a 3D scene using painter-based rasterisation:
//! gradient background, perspective ground grid, projected mesh / light /
//! camera objects, tool gizmos, axis indicator, box-select overlay, and
//! HUD text (tab, render style, camera info, navigation hints).

pub(crate) mod axis_indicator;
pub(crate) mod camera_input;
pub(crate) mod context_menu;
pub(crate) mod gizmo;
pub(crate) mod grid;
pub(crate) mod objects;
pub(crate) mod picking;
pub(crate) mod projection;
pub(crate) mod styles;

use eframe::egui;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect,
    Sense, Stroke, StrokeKind,
};

use crate::app::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the central viewport panel with the 3D scene.
    pub(crate) fn draw_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(tc!(self, bg))
                    .inner_margin(Margin::same(0)),
            )
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();

                // Allocate interactive area for mouse input
                let response = ui.allocate_rect(rect, Sense::click_and_drag());
                let painter = ui.painter_at(rect);

                // ---- Read input state ----
                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

                // ---- Mouse controls (Unreal Engine style) ----
                self.handle_camera_input(ctx, &response, pointer_pos);

                // ---- Right-click context menu ----
                self.draw_context_menu(&response);

                // ---- Draw viewport contents with 3D projection ----

                // Gradient background (8-band, adapts to dark/light theme)
                let gradient = self.theme_manager.viewport_gradient();
                let bands = gradient.len();
                for (i, rgb) in gradient.iter().enumerate() {
                    let t0 = i as f32 / bands as f32;
                    let t1 = (i + 1) as f32 / bands as f32;
                    let c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                    let band_rect = Rect::from_min_max(
                        Pos2::new(rect.left(), rect.top() + t0 * rect.height()),
                        Pos2::new(rect.right(), rect.top() + t1 * rect.height()),
                    );
                    painter.rect_filled(band_rect, CornerRadius::ZERO, c);
                }

                // Build view-projection matrix from OrbitCamera
                let aspect = if rect.height() > 0.0 {
                    rect.width() / rect.height()
                } else {
                    16.0 / 9.0
                };
                let view = self.orbit_camera.view_matrix();
                let proj = self.orbit_camera.projection_matrix(aspect);
                let vp = proj * view;

                // ---- Perspective ground grid (toggle with show_grid) ----
                let wire_c = self.theme_manager.wireframe_color();
                if self.show_grid {
                    let grid_c = self.theme_manager.grid_color();
                    let grid_mc = self.theme_manager.grid_major_color();
                    Self::draw_perspective_grid(&painter, &vp, &rect, grid_c, grid_mc);
                }

                // ---- Draw 3D scene objects with projection ----
                let cam_pos = self.orbit_camera.position();
                self.draw_projected_objects(&painter, &vp, &rect, wire_c, cam_pos);

                // ---- Tool gizmo indicators at selected entity ----
                self.draw_tool_gizmo(&painter, &vp, &rect);

                // ---- Axis gizmo indicator (bottom-left corner) ----
                Self::draw_axis_gizmo(&painter, &view, &rect);

                // Draw box selection rectangle
                if let (Some(start), Some(end)) = (self.box_select_start, self.box_select_end) {
                    let sel_rect = Rect::from_two_pos(start, end);
                    painter.rect_filled(
                        sel_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_premultiplied(0x4e, 0xff, 0x93, 20),
                    );
                    painter.rect_stroke(
                        sel_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, tc!(self, accent).linear_multiply(0.6)),
                        StrokeKind::Outside,
                    );
                }

                // Active tab + render style + grid/snap status overlay
                let grid_status = if self.show_grid { "Grid:ON" } else { "Grid:OFF" };
                let snap_status = if self.snap_enabled {
                    format!("Snap:{:.2}", self.snap_size)
                } else {
                    "Snap:OFF".to_string()
                };
                painter.text(
                    Pos2::new(rect.left() + 12.0, rect.top() + 12.0),
                    Align2::LEFT_TOP,
                    format!(
                        "{} | {} | {} | {}",
                        self.active_tab.label(),
                        self.render_style.label(),
                        grid_status,
                        snap_status,
                    ),
                    FontId::proportional(13.0),
                    tc!(self, accent),
                );

                // Move tool hint
                if self.box_select_key_held {
                    painter.text(
                        Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
                        Align2::LEFT_TOP,
                        "S + Left-Drag: Box Select (release to apply)",
                        FontId::proportional(11.0),
                        tc!(self, accent),
                    );
                } else if self.tool_mode == crate::types::ToolMode::Move && self.selected_entity > 0 {
                    painter.text(
                        Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
                        Align2::LEFT_TOP,
                        "Drag: X+Z | Shift+Drag: Y only | Ctrl+Drag: X+Y",
                        FontId::proportional(11.0),
                        tc!(self, text_dim),
                    );
                }

                // Shortcut reference (top-right corner, tiny font)
                let shortcut_text = "\
RMB Drag: Orbit  |  MMB Drag: Pan  |  Scroll: Zoom
LMB+RMB Drag: Pan  |  Alt+LMB: Orbit  |  Alt+RMB: Dolly
Q: Select  M: Move  E: Rotate  R: Scale  |  S+LMB: Box Select
Move: Drag=XZ  Shift=Y  Ctrl=XY  |  Z: Render Style
G: Grid  F: Focus  |  Ctrl+T: Theme  |  Ctrl+Shift+P: Commands
Del: Delete  Ctrl+A: Select All  Ctrl+D: Duplicate  Ctrl+G: Group
1-7: Tabs  |  Ctrl+Z: Undo  Ctrl+Shift+Z: Redo";
                // Use contrasting color: light text on dark bg, dark text on light bg
                let ref_color = if self.theme_manager.is_dark() {
                    Color32::from_rgba_premultiplied(255, 255, 255, 180)
                } else {
                    Color32::from_rgba_premultiplied(0, 0, 0, 180)
                };
                let line_h = 14.0;
                for (i, line) in shortcut_text.lines().enumerate() {
                    painter.text(
                        Pos2::new(rect.right() - 10.0, rect.top() + 10.0 + i as f32 * line_h),
                        Align2::RIGHT_TOP,
                        line,
                        FontId::proportional(11.0),
                        ref_color,
                    );
                }

                // Camera info overlay
                let cam = &self.orbit_camera;
                let cam_info = format!(
                    "Yaw: {:.1}  Pitch: {:.1}  Dist: {:.1}  Target: ({:.1}, {:.1}, {:.1})",
                    cam.yaw.to_degrees(),
                    cam.pitch.to_degrees(),
                    cam.distance,
                    cam.target.x,
                    cam.target.y,
                    cam.target.z,
                );
                painter.text(
                    Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
                    Align2::LEFT_TOP,
                    cam_info,
                    FontId::proportional(10.0),
                    Color32::from_rgba_premultiplied(0x9b, 0x9b, 0xa1, 160),
                );

                // Interaction state indicator
                let state_text = if self.is_orbiting {
                    "Orbiting"
                } else if self.is_panning {
                    "Panning"
                } else {
                    ""
                };
                if !state_text.is_empty() {
                    painter.text(
                        Pos2::new(rect.left() + 12.0, rect.top() + 44.0),
                        Align2::LEFT_TOP,
                        state_text,
                        FontId::proportional(11.0),
                        Color32::from_rgb(0xff, 0xd7, 0x00),
                    );
                }

                // Navigation hint
                painter.text(
                    Pos2::new(rect.right() - 12.0, rect.top() + 12.0),
                    Align2::RIGHT_TOP,
                    "RMB: Orbit | MMB: Pan | Scroll: Zoom | Alt+LMB: Orbit | Right-click: Menu",
                    FontId::proportional(10.0),
                    Color32::from_rgba_premultiplied(0x9b, 0x9b, 0xa1, 120),
                );
            });
    }
}
