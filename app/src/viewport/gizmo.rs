//! Tool gizmo overlays for Move, Rotate, and Scale modes.
//!
//! Draws coloured axis indicators at the projected position of the
//! primary selected entity: arrows for Move, concentric circles for
//! Rotate, and axis lines with square handles for Scale.

use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};
use glam::{Mat4, Vec3};

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Draw tool gizmo indicators at the selected entity's projected position.
    pub(crate) fn draw_tool_gizmo(
        &self,
        painter: &egui::Painter,
        vp: &Mat4,
        rect: &Rect,
    ) {
        if self.selected_entity > 0
            && self.selected_entity < self.transforms.len()
            && self.tool_mode != ToolMode::Select
        {
            let sel = self.selected_entity;
            let ent_pos = Vec3::new(
                self.transforms[sel][0],
                self.transforms[sel][1],
                self.transforms[sel][2],
            );
            if let Some(sp) = Self::project_3d(vp, rect, ent_pos) {
                let red = Color32::from_rgb(0xe7, 0x4c, 0x3c);
                let green = Color32::from_rgb(0x2e, 0xcc, 0x71);
                let blue = Color32::from_rgb(0x3e, 0x55, 0xff);
                match self.tool_mode {
                    ToolMode::Move => {
                        // Draw XYZ arrows
                        let arrow_len = 40.0;
                        // X arrow (right)
                        let x_end = Pos2::new(sp.x + arrow_len, sp.y);
                        painter.line_segment([sp, x_end], Stroke::new(2.5, red));
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                x_end,
                                Pos2::new(x_end.x - 6.0, x_end.y - 4.0),
                                Pos2::new(x_end.x - 6.0, x_end.y + 4.0),
                            ],
                            red,
                            Stroke::NONE,
                        ));
                        // Y arrow (up)
                        let y_end = Pos2::new(sp.x, sp.y - arrow_len);
                        painter.line_segment([sp, y_end], Stroke::new(2.5, green));
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                y_end,
                                Pos2::new(y_end.x - 4.0, y_end.y + 6.0),
                                Pos2::new(y_end.x + 4.0, y_end.y + 6.0),
                            ],
                            green,
                            Stroke::NONE,
                        ));
                        // Z arrow (diagonal)
                        let z_end = Pos2::new(sp.x - 28.0, sp.y + 28.0);
                        painter.line_segment([sp, z_end], Stroke::new(2.5, blue));
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                z_end,
                                Pos2::new(z_end.x + 7.0, z_end.y - 1.0),
                                Pos2::new(z_end.x + 1.0, z_end.y - 7.0),
                            ],
                            blue,
                            Stroke::NONE,
                        ));
                    }
                    ToolMode::Rotate => {
                        // Draw rotation circles
                        let r = 35.0;
                        painter.circle_stroke(sp, r, Stroke::new(1.5, red));
                        painter.circle_stroke(sp, r + 5.0, Stroke::new(1.5, green));
                        painter.circle_stroke(sp, r + 10.0, Stroke::new(1.5, blue));
                    }
                    ToolMode::Scale => {
                        // Draw scale handles (small squares on axes)
                        let handle_len = 35.0;
                        let sq = 4.0;
                        // X
                        let xp = Pos2::new(sp.x + handle_len, sp.y);
                        painter.line_segment([sp, xp], Stroke::new(2.0, red));
                        painter.rect_filled(
                            Rect::from_center_size(xp, Vec2::new(sq * 2.0, sq * 2.0)),
                            CornerRadius::ZERO,
                            red,
                        );
                        // Y
                        let yp = Pos2::new(sp.x, sp.y - handle_len);
                        painter.line_segment([sp, yp], Stroke::new(2.0, green));
                        painter.rect_filled(
                            Rect::from_center_size(yp, Vec2::new(sq * 2.0, sq * 2.0)),
                            CornerRadius::ZERO,
                            green,
                        );
                        // Z
                        let zp = Pos2::new(sp.x - 25.0, sp.y + 25.0);
                        painter.line_segment([sp, zp], Stroke::new(2.0, blue));
                        painter.rect_filled(
                            Rect::from_center_size(zp, Vec2::new(sq * 2.0, sq * 2.0)),
                            CornerRadius::ZERO,
                            blue,
                        );
                    }
                    ToolMode::Select => {}
                }
            }
        }
    }
}
