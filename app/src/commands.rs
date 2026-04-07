//! Command palette overlay.
//!
//! Provides a searchable popup (Ctrl+Shift+P) that lists available editor
//! commands. Typing filters the list and pressing Enter executes the first
//! match.

use eframe::egui;
use egui::{
    Align, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Pos2, RichText, Sense,
    Stroke,
};

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Draw the command palette overlay and handle input/execution.
    pub(crate) fn draw_command_palette(&mut self, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        let palette_width = 450.0_f32.min(screen.width() - 40.0);
        let palette_x = (screen.width() - palette_width) / 2.0;
        let palette_y = screen.height() * 0.2;

        // Dim overlay
        egui::Area::new(Id::new("cmd_overlay"))
            .fixed_pos(Pos2::ZERO)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let resp = ui.allocate_rect(screen, Sense::click());
                ui.painter().rect_filled(
                    screen,
                    CornerRadius::ZERO,
                    Color32::from_rgba_premultiplied(0, 0, 0, 140),
                );
                if resp.clicked() {
                    self.show_command_palette = false;
                }
            });

        egui::Area::new(Id::new("cmd_palette"))
            .fixed_pos(Pos2::new(palette_x, palette_y))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::default()
                    .fill(Color32::from_rgb(0x22, 0x22, 0x26))
                    .stroke(Stroke::new(
                        1.5,
                        tc!(self, accent).linear_multiply(0.5),
                    ))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(palette_width - 24.0);
                        ui.label(
                            RichText::new("Command Palette")
                                .font(FontId::proportional(13.0))
                                .color(tc!(self, accent)),
                        );
                        ui.add_space(4.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.command_query)
                                .hint_text("Type a command...")
                                .desired_width(palette_width - 36.0)
                                .font(FontId::proportional(13.0)),
                        );
                        resp.request_focus();

                        ui.add_space(4.0);

                        // Command suggestions
                        let commands: &[(&str, &str)] = &[
                            ("Switch to Map Editor", "Tab 1"),
                            ("Switch to Gameplay", "Tab 2"),
                            ("Switch to Object Editor", "Tab 3"),
                            ("Switch to Script Editor", "Tab 4"),
                            ("Switch to Material Editor", "Tab 5"),
                            ("Switch to Animation", "Tab 6"),
                            ("Switch to Physics", "Tab 7"),
                            ("Toggle Wireframe", "Render"),
                            ("Select Tool", "Q"),
                            ("Move Tool", "W"),
                            ("Rotate Tool", "E"),
                            ("Scale Tool", "R"),
                            ("New Entity", ""),
                            ("Delete Entity", ""),
                            ("Save Scene", "Ctrl+S"),
                            ("Build Lighting", "Agent"),
                            ("Clear Console", ""),
                            ("Toggle Theme", "Ctrl+T"),
                            ("Select All", "Ctrl+A"),
                            ("Deselect All", ""),
                        ];
                        let query_lower = self.command_query.to_lowercase();
                        let filtered: Vec<_> = commands
                            .iter()
                            .filter(|(name, _)| {
                                query_lower.is_empty()
                                    || name.to_lowercase().contains(&query_lower)
                            })
                            .take(8)
                            .collect();

                        let mut executed_command: Option<&str> = None;

                        for &&(name, shortcut) in &filtered {
                            let resp = ui.horizontal(|ui| {
                                let btn = ui.add(
                                    egui::Button::new(
                                        RichText::new(name)
                                            .font(FontId::proportional(12.0))
                                            .color(tc!(self, text)),
                                    )
                                    .frame(false),
                                );
                                ui.with_layout(
                                    Layout::right_to_left(Align::Center),
                                    |ui| {
                                        if !shortcut.is_empty() {
                                            ui.label(
                                                RichText::new(shortcut)
                                                    .font(FontId::proportional(10.0))
                                                    .color(tc!(self, text_dim)),
                                            );
                                        }
                                    },
                                );
                                btn
                            });
                            if resp.inner.clicked() {
                                executed_command = Some(name);
                            }
                        }

                        // Enter key executes first filtered command
                        if ctx.input(|i| i.key_pressed(egui::Key::Enter))
                            && let Some(&&(name, _)) = filtered.first()
                        {
                            executed_command = Some(name);
                        }

                        // Execute the selected command
                        if let Some(cmd) = executed_command {
                            match cmd {
                                "Switch to Map Editor" => {
                                    self.active_tab = MainTab::MapEditor
                                }
                                "Switch to Gameplay" => {
                                    self.active_tab = MainTab::Gameplay
                                }
                                "Switch to Object Editor" => {
                                    self.active_tab = MainTab::ObjectEditor
                                }
                                "Switch to Script Editor" => {
                                    self.active_tab = MainTab::ScriptEditor
                                }
                                "Switch to Material Editor" => {
                                    self.active_tab = MainTab::MaterialEditor
                                }
                                "Switch to Animation" => {
                                    self.active_tab = MainTab::Animation
                                }
                                "Switch to Physics" => {
                                    self.active_tab = MainTab::Physics
                                }
                                "Toggle Wireframe" => {
                                    self.render_style =
                                        if self.render_style == RenderStyle::Wireframe {
                                            RenderStyle::Shaded
                                        } else {
                                            RenderStyle::Wireframe
                                        };
                                }
                                "Select Tool" => self.tool_mode = ToolMode::Select,
                                "Move Tool" => self.tool_mode = ToolMode::Move,
                                "Rotate Tool" => self.tool_mode = ToolMode::Rotate,
                                "Scale Tool" => self.tool_mode = ToolMode::Scale,
                                "Clear Console" => self.console_log.clear(),
                                "Toggle Theme" => {
                                    self.theme_manager.toggle_theme();
                                }
                                "New Entity" => {
                                    self.add_entity("New Entity", "\u{25CB}");
                                }
                                "Delete Entity" => {
                                    self.delete_selected();
                                }
                                "Select All" => {
                                    self.select_all();
                                }
                                "Deselect All" => {
                                    self.deselect_all();
                                }
                                _ => {}
                            }
                            self.console_log.push(LogEntry {
                                level: LogLevel::Info,
                                message: format!("Command: {}", cmd),
                            });
                            self.show_command_palette = false;
                            self.command_query.clear();
                        }

                        // Escape to close
                        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.show_command_palette = false;
                        }
                    });
            });
    }
}
