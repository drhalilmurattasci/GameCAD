//! Tab bar and toolbar panels.
//!
//! The tab bar shows the workspace tabs (Map Editor, Gameplay, etc.) with
//! underline indicators. The toolbar row below it provides tool-mode buttons,
//! a render-style dropdown, the theme toggle, theme selector, and the
//! command-palette trigger.

use eframe::egui;
use egui::{Color32, FontId, RichText, Stroke};

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Draw the top tab bar with workspace tabs and accent underlines.
    pub(crate) fn draw_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                // Logo / title
                ui.label(
                    RichText::new("FORGE")
                        .font(FontId::proportional(16.0))
                        .color(tc!(self, accent))
                        .strong(),
                );
                ui.add_space(16.0);
                for (i, tab) in MainTab::ALL.iter().enumerate() {
                    let active = *tab == self.active_tab;
                    let text_color = if active {
                        tc!(self, text)
                    } else {
                        tc!(self, text_dim)
                    };
                    let resp = ui.add(
                        egui::Button::new(
                            RichText::new(tab.label())
                                .font(FontId::proportional(13.0))
                                .color(text_color),
                        )
                        .frame(false),
                    );
                    if active {
                        let r = resp.rect;
                        ui.painter().line_segment(
                            [
                                egui::Pos2::new(r.left() + 2.0, r.bottom()),
                                egui::Pos2::new(r.right() - 2.0, r.bottom()),
                            ],
                            Stroke::new(2.5, tc!(self, accent)),
                        );
                    }
                    if resp.clicked() {
                        self.active_tab = *tab;
                        self.console_log.push(LogEntry {
                            level: LogLevel::Info,
                            message: format!("Switched to tab: {}", tab.label()),
                        });
                    }
                    if resp.hovered() && !active {
                        let r = resp.rect;
                        ui.painter().line_segment(
                            [
                                egui::Pos2::new(r.left() + 2.0, r.bottom()),
                                egui::Pos2::new(r.right() - 2.0, r.bottom()),
                            ],
                            Stroke::new(1.5, tc!(self, text_dim)),
                        );
                    }
                    // Tooltip with shortcut number
                    resp.on_hover_text(format!("Shortcut: {}", i + 1));
                }
            });
            ui.add_space(1.0);
        });
    }

    /// Draw the toolbar row: tool buttons, render-style combo, theme toggle, command palette.
    pub(crate) fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let tools = [
                    ToolMode::Select,
                    ToolMode::Move,
                    ToolMode::Rotate,
                    ToolMode::Scale,
                ];
                for tool in &tools {
                    let active = *tool == self.tool_mode;
                    let btn = egui::Button::new(
                        RichText::new(tool.label())
                            .font(FontId::proportional(12.0))
                            .color(if active {
                                tc!(self, accent)
                            } else {
                                tc!(self, text)
                            }),
                    )
                    .fill(if active {
                        tc!(self, accent).linear_multiply(0.15)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(if active {
                        Stroke::new(1.0, tc!(self, accent))
                    } else {
                        Stroke::NONE
                    });
                    let resp = ui.add(btn);
                    if resp.clicked() {
                        self.tool_mode = *tool;
                        self.console_log.push(LogEntry {
                            level: LogLevel::Info,
                            message: format!("Tool changed: {}", tool.label()),
                        });
                    }
                    resp.on_hover_text(format!("{} ({})", tool.label(), tool.shortcut()));
                }

                // separator
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Grid toggle + size
                let grid_label = if self.show_grid { "\u{25A6}" } else { "\u{25A1}" };
                let grid_btn = egui::Button::new(
                    RichText::new(grid_label)
                        .font(FontId::proportional(14.0))
                        .color(if self.show_grid { tc!(self, accent) } else { tc!(self, text_dim) }),
                );
                if ui.add(grid_btn).on_hover_text("Toggle grid (G)").clicked() {
                    self.show_grid = !self.show_grid;
                    self.console_log.push(crate::types::LogEntry {
                        level: crate::types::LogLevel::Info,
                        message: format!("Grid {}", if self.show_grid { "ON" } else { "OFF" }),
                    });
                }
                ui.label(RichText::new("Grid:").font(FontId::proportional(11.0)).color(tc!(self, text_dim)));
                ui.add(
                    egui::DragValue::new(&mut self.grid_size)
                        .speed(0.1)
                        .range(0.1..=100.0)
                        .max_decimals(1)
                        .suffix("m"),
                ).on_hover_text("Grid spacing");

                ui.add_space(4.0);

                // Snap toggle + size
                let snap_label = if self.snap_enabled { "\u{1F9F2}" } else { "\u{25CB}" };
                let snap_btn = egui::Button::new(
                    RichText::new(snap_label)
                        .font(FontId::proportional(14.0))
                        .color(if self.snap_enabled { tc!(self, accent) } else { tc!(self, text_dim) }),
                );
                if ui.add(snap_btn).on_hover_text("Toggle snap").clicked() {
                    self.snap_enabled = !self.snap_enabled;
                    self.console_log.push(crate::types::LogEntry {
                        level: crate::types::LogLevel::Info,
                        message: format!("Snap {}", if self.snap_enabled { "ON" } else { "OFF" }),
                    });
                }
                ui.label(RichText::new("Snap:").font(FontId::proportional(11.0)).color(tc!(self, text_dim)));
                ui.add(
                    egui::DragValue::new(&mut self.snap_size)
                        .speed(0.05)
                        .range(0.01..=50.0)
                        .max_decimals(2)
                        .suffix("m"),
                ).on_hover_text("Snap increment");

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Render style dropdown
                ui.label(
                    RichText::new("Render:")
                        .color(tc!(self, text_dim))
                        .font(FontId::proportional(12.0)),
                );
                egui::ComboBox::from_id_salt("render_style")
                    .selected_text(
                        RichText::new(self.render_style.label())
                            .font(FontId::proportional(12.0))
                            .color(tc!(self, text)),
                    )
                    .show_ui(ui, |ui| {
                        for rs in &RenderStyle::ALL {
                            ui.selectable_value(&mut self.render_style, *rs, rs.label());
                        }
                    });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Theme toggle button (sun/moon)
                let theme_icon = if self.theme_manager.is_dark() {
                    "\u{1F319}"
                } else {
                    "\u{2600}"
                };
                let theme_btn = egui::Button::new(
                    RichText::new(theme_icon).font(FontId::proportional(14.0)),
                );
                if ui
                    .add(theme_btn)
                    .on_hover_text("Toggle theme (Ctrl+T)")
                    .clicked()
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

                // Theme selector with swatches
                self.theme_manager.theme_selector_ui(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Extra context buttons
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("\u{2318} Cmd")
                                .font(FontId::proportional(11.0))
                                .color(tc!(self, text_dim)),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Command Palette (Ctrl+Shift+P)")
                    .clicked()
                {
                    self.show_command_palette = !self.show_command_palette;
                    self.command_query.clear();
                }
            });
            ui.add_space(2.0);
        });
    }
}
