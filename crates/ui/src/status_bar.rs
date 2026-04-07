//! Bottom status bar for the editor.
//!
//! Displays a status message, FPS counter, 3-D cursor position, selection count,
//! and an agent activity indicator.

use egui::{Ui, Vec2};

use crate::theme::ThemeColors;

/// Snapshot of data displayed in the status bar each frame.
pub struct StatusBarState {
    /// Free-form status message (e.g. "Saved project.").
    pub message: String,
    /// Frames per second.
    pub fps: f32,
    /// 3-D world-space cursor position, if any.
    pub cursor_position: Option<(f32, f32, f32)>,
    /// Number of currently selected entities.
    pub selection_count: usize,
    /// Whether the AI agent is currently running a task.
    pub agent_active: bool,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            message: String::new(),
            fps: 0.0,
            cursor_position: None,
            selection_count: 0,
            agent_active: false,
        }
    }
}

/// The editor status bar rendered at the very bottom of the window.
pub struct StatusBar;

impl StatusBar {
    /// Draw the status bar.
    pub fn show(ui: &mut Ui, state: &StatusBarState) {
        let colors = ThemeColors::dark_default();

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);

                    // Status message
                    if !state.message.is_empty() {
                        ui.label(
                            egui::RichText::new(&state.message)
                                .size(11.0)
                                .color(colors.text_dim),
                        );
                    }

                    // Agent indicator
                    if state.agent_active {
                        ui.label(
                            egui::RichText::new("\u{2022} Agent")
                                .size(11.0)
                                .color(colors.accent),
                        );
                    }

                    // Spacer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // FPS
                        ui.label(
                            egui::RichText::new(format!("{:.0} FPS", state.fps))
                                .size(11.0)
                                .color(colors.text_dim),
                        );

                        // Selection count
                        if state.selection_count > 0 {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} selected",
                                    state.selection_count
                                ))
                                .size(11.0)
                                .color(colors.text_dim),
                            );
                        }

                        // Cursor position
                        if let Some((x, y, z)) = state.cursor_position {
                            ui.label(
                                egui::RichText::new(format!(
                                    "X:{:.1}  Y:{:.1}  Z:{:.1}",
                                    x, y, z
                                ))
                                .size(11.0)
                                .color(colors.text_dim),
                            );
                        }
                    });
                });
            });
    }
}
