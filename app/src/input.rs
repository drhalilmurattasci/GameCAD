//! Keyboard shortcut handling for the editor.
//!
//! Maps global key bindings (1-7 for tabs, Q/W/E/R for tools, Ctrl+T for
//! theme toggle, Z for render-style cycling, F for focus, etc.) into
//! state mutations on [`ForgeEditorApp`].

use eframe::egui;
use glam::Vec3;

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Process all global keyboard shortcuts.
    ///
    /// Called once per frame when the command palette is closed.
    pub(crate) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Num1),
                i.key_pressed(egui::Key::Num2),
                i.key_pressed(egui::Key::Num3),
                i.key_pressed(egui::Key::Num4),
                i.key_pressed(egui::Key::Num5),
                i.key_pressed(egui::Key::Num6),
                i.key_pressed(egui::Key::Num7),
                i.key_pressed(egui::Key::Q),
                i.key_pressed(egui::Key::W),
                i.key_pressed(egui::Key::E),
                i.key_pressed(egui::Key::R),
                i.modifiers.ctrl
                    && i.modifiers.shift
                    && i.key_pressed(egui::Key::P),
            )
        });

        if input.0 {
            self.active_tab = MainTab::ALL[0];
        }
        if input.1 {
            self.active_tab = MainTab::ALL[1];
        }
        if input.2 {
            self.active_tab = MainTab::ALL[2];
        }
        if input.3 {
            self.active_tab = MainTab::ALL[3];
        }
        if input.4 {
            self.active_tab = MainTab::ALL[4];
        }
        if input.5 {
            self.active_tab = MainTab::ALL[5];
        }
        if input.6 {
            self.active_tab = MainTab::ALL[6];
        }
        if input.7 {
            self.tool_mode = ToolMode::Select;
        }
        if input.8 {
            self.tool_mode = ToolMode::Move;
        }
        if input.9 {
            self.tool_mode = ToolMode::Rotate;
        }
        if input.10 {
            self.tool_mode = ToolMode::Scale;
        }
        if input.11 {
            self.show_command_palette = !self.show_command_palette;
            self.command_query.clear();
        }

        // Toggle theme: Ctrl+T
        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.command && !i.modifiers.shift)
        {
            self.theme_manager.toggle_theme();
            let mode = if self.theme_manager.is_dark() {
                "Dark"
            } else {
                "Light"
            };
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Theme switched to {}", mode),
            });
        }

        // G (no modifiers): Toggle grid
        if ctx.input(|i| {
            i.key_pressed(egui::Key::G) && !i.modifiers.command && !i.modifiers.shift
        }) {
            self.show_grid = !self.show_grid;
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Grid {}", if self.show_grid { "ON" } else { "OFF" }),
            });
        }

        // Z (no modifiers): Cycle render style
        if ctx.input(|i| {
            i.key_pressed(egui::Key::Z) && !i.modifiers.command && !i.modifiers.shift
        }) {
            self.render_style = self.render_style.next();
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Render style: {}", self.render_style.label()),
            });
        }
        // Undo: Ctrl+Z
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift)
        {
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Undo".into(),
            });
        }
        // Redo: Ctrl+Shift+Z
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift) {
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Redo".into(),
            });
        }
        // F: Focus selection
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers == egui::Modifiers::NONE) {
            // Focus on the selected entity's position if available
            let focus_pos = if self.selected_entity < self.transforms.len() {
                let t = &self.transforms[self.selected_entity];
                Vec3::new(t[0], t[1], t[2])
            } else {
                Vec3::ZERO
            };
            self.orbit_camera.focus_on(focus_pos, 5.0);
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Focus selection".into(),
            });
        }
        // Ctrl+A: Select all
        if ctx.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.command && !i.modifiers.shift)
        {
            self.select_all();
        }
        // Ctrl+D: Duplicate
        if ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.command) {
            self.duplicate_selected();
        }
        // Ctrl+G: Group
        if ctx.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.command) {
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Group selected entities".into(),
            });
        }
        // Delete: Delete selected
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.delete_selected();
        }
    }
}
