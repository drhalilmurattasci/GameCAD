//! Keyboard shortcut handling for the editor.
//!
//! Maps global key bindings (1-7 for tabs, Q/W/E/R for tools, Ctrl+T for
//! theme toggle, Z for render-style cycling, F for focus, etc.) into
//! state mutations on [`ForgeEditorApp`].

use eframe::egui;
use glam::Vec3;

use crate::state::ForgeEditorApp;
use crate::state::types::*;

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
                i.key_pressed(egui::Key::M),
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
            self.settings.grid.visible = !self.settings.grid.visible;
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Grid {}", if self.settings.grid.visible { "ON" } else { "OFF" }),
            });
        }

        // L (no modifiers): Cycle active layer
        if ctx.input(|i| {
            i.key_pressed(egui::Key::L) && !i.modifiers.command && !i.modifiers.shift
        }) {
            // Cycle through top-level layers
            let current = self.active_layer.first().copied().unwrap_or(0);
            let next = (current + 1) % self.layers.len();
            self.active_layer = vec![next];
            let name = self.layers[next].name.clone();
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Active layer: {}", name),
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
            let mut cmd_ctx = forge_core::commands::CommandContext::new(
                &mut self.world, &self.event_bus,
            );
            match self.command_history.undo(&mut cmd_ctx) {
                Ok(true) => self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Undo".into(),
                }),
                Ok(false) => self.console_log.push(LogEntry {
                    level: LogLevel::Warn,
                    message: "Nothing to undo".into(),
                }),
                Err(e) => self.console_log.push(LogEntry {
                    level: LogLevel::Error,
                    message: format!("Undo failed: {e}"),
                }),
            }
        }
        // Redo: Ctrl+Shift+Z
        if ctx.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift) {
            let mut cmd_ctx = forge_core::commands::CommandContext::new(
                &mut self.world, &self.event_bus,
            );
            match self.command_history.redo(&mut cmd_ctx) {
                Ok(true) => self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Redo".into(),
                }),
                Ok(false) => self.console_log.push(LogEntry {
                    level: LogLevel::Warn,
                    message: "Nothing to redo".into(),
                }),
                Err(e) => self.console_log.push(LogEntry {
                    level: LogLevel::Error,
                    message: format!("Redo failed: {e}"),
                }),
            }
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
        // Ctrl+D: Duplicate (skip if any selected entity is locked)
        if ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.command) {
            let any_locked = self.selected_entities.iter().any(|&i| self.is_entity_locked(i));
            if !any_locked {
                self.duplicate_selected();
            } else {
                self.console_log.push(LogEntry {
                    level: LogLevel::Warn,
                    message: "Cannot duplicate: selection contains locked entities".into(),
                });
            }
        }
        // Ctrl+G: Group
        if ctx.input(|i| i.key_pressed(egui::Key::G) && i.modifiers.command) {
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Group selected entities".into(),
            });
        }
        // Delete: Delete selected (skip if any selected entity is locked)
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            let any_locked = self.selected_entities.iter().any(|&i| self.is_entity_locked(i));
            if !any_locked {
                self.delete_selected();
            } else {
                self.console_log.push(LogEntry {
                    level: LogLevel::Warn,
                    message: "Cannot delete: selection contains locked entities".into(),
                });
            }
        }
    }
}
