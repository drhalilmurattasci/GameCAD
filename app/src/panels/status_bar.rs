//! Status bar at the bottom of the window.
//!
//! Shows the active tool, estimated FPS, entity count, and memory usage.

use eframe::egui;
use egui::{FontId, RichText};

use crate::state::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the thin status bar with tool mode, FPS, and entity count.
    pub(crate) fn draw_status_bar(&mut self, ctx: &egui::Context) {
        let entity_count = self.entity_count();
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(22.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    let fps = 1.0 / ctx.input(|i| i.predicted_dt).max(0.001);
                    ui.label(
                        RichText::new(format!(
                            "{} Tool  |  FPS: {:.0}  |  Entities: {}",
                            self.tool_mode.label(),
                            fps,
                            entity_count,
                        ))
                        .font(FontId::proportional(11.0))
                        .color(tc!(self, text_dim)),
                    );
                });
            });
    }
}
