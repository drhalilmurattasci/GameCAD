//! Outliner (scene-tree) panel on the left side.
//!
//! Displays the hierarchical entity tree with icons, supports single click,
//! Ctrl+click (toggle), and Shift+click (range) selection, and provides a
//! right-click context menu for add / duplicate / delete.

use eframe::egui;
use egui::{CornerRadius, FontId, Rect, RichText, Stroke, StrokeKind, Vec2};

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Draw the left-side outliner panel containing the scene tree.
    pub(crate) fn draw_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("outliner_panel")
            .resizable(true)
            .min_width(180.0)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Outliner")
                        .font(FontId::proportional(13.0))
                        .color(tc!(self, text))
                        .strong(),
                );
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Flatten the tree for display
                    let tree = self.outliner.clone();
                    let mut flat_idx = 0usize;
                    for root in &tree {
                        self.draw_outliner_node(ui, root, 0, &mut flat_idx);
                    }
                });
            });
    }

    /// Recursively draw a single outliner node and its children.
    pub(crate) fn draw_outliner_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &OutlinerNode,
        depth: usize,
        flat_idx: &mut usize,
    ) {
        let idx = *flat_idx;
        *flat_idx += 1;
        let selected = self.selected_entities.contains(&idx);
        let is_primary = idx == self.selected_entity;
        let indent = depth as f32 * 16.0;
        let node_name = node.name.clone();

        // Capture theme colors before closure
        let accent = tc!(self, accent);
        let text_color = tc!(self, text);
        let text_dim = tc!(self, text_dim);
        let surface = tc!(self, surface);
        let selection_bg = accent.linear_multiply(0.15);

        let row_resp = ui.horizontal(|ui| {
            // Draw highlighted background for selected rows
            if selected {
                let row_rect = Rect::from_min_size(
                    ui.cursor().min,
                    Vec2::new(ui.available_width(), 20.0),
                );
                ui.painter().rect_filled(
                    row_rect,
                    CornerRadius::same(2),
                    if is_primary {
                        selection_bg
                    } else {
                        accent.linear_multiply(0.08)
                    },
                );
            }

            ui.add_space(indent + 4.0);
            let icon_text = RichText::new(node.icon)
                .font(FontId::proportional(13.0))
                .color(if selected { accent } else { text_dim });
            ui.label(icon_text);
            let name_text = RichText::new(&node.name)
                .font(FontId::proportional(12.0))
                .color(if selected { accent } else { text_color });
            let resp = ui.add(egui::Button::new(name_text).frame(false));
            let modifiers = ui.input(|i| i.modifiers);
            if resp.clicked() {
                if modifiers.ctrl {
                    // Ctrl+click: toggle in multi-selection
                    if let Some(pos_in_vec) =
                        self.selected_entities.iter().position(|&e| e == idx)
                    {
                        self.selected_entities.remove(pos_in_vec);
                        // Update primary to first in selection or 0
                        self.selected_entity =
                            self.selected_entities.first().copied().unwrap_or(0);
                    } else {
                        self.selected_entities.push(idx);
                        self.selected_entity = idx;
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: format!("Toggle select: {}", node.name),
                    });
                } else if modifiers.shift {
                    // Shift+click: range select from primary to clicked
                    let start = self.selected_entity.min(idx);
                    let end = self.selected_entity.max(idx);
                    self.selected_entities.clear();
                    for i in start..=end {
                        self.selected_entities.push(i);
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: format!("Range select to: {}", node.name),
                    });
                } else {
                    // Single click: replace selection
                    self.selected_entity = idx;
                    self.selected_entities = vec![idx];
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: format!("Selected entity: {}", node.name),
                    });
                }
            }
            if selected {
                let r = resp.rect;
                ui.painter().rect_stroke(
                    r.expand(1.0),
                    CornerRadius::same(3),
                    Stroke::new(1.0, accent.linear_multiply(0.4)),
                    StrokeKind::Outside,
                );
            }

            // Suppress unused-variable warnings
            let _ = surface;
        });

        // Outliner context menu on right-click
        row_resp.response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Rename: {}", node_name),
                });
                ui.close_menu();
            }
            if ui.button("Duplicate").clicked() {
                self.duplicate_entity(idx);
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                self.delete_entity(idx);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Copy").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Copied: {}", node_name),
                });
                ui.close_menu();
            }
            if ui.button("Paste").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Paste".into(),
                });
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Add", |ui| {
                if ui.button("Empty").clicked() {
                    self.add_entity("New Empty", "\u{25CB}");
                    ui.close_menu();
                }
                if ui.button("Cube").clicked() {
                    self.add_entity("New Cube", "\u{25A6}");
                    ui.close_menu();
                }
                if ui.button("Sphere").clicked() {
                    self.add_entity("New Sphere", "\u{25A6}");
                    ui.close_menu();
                }
                if ui.button("Light").clicked() {
                    self.add_entity("New Light", "\u{2600}");
                    ui.close_menu();
                }
                if ui.button("Camera").clicked() {
                    self.add_entity("New Camera", "\u{1F3A5}");
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button("Select Children").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Select children of: {}", node_name),
                });
                ui.close_menu();
            }
        });

        for child in &node.children {
            self.draw_outliner_node(ui, child, depth + 1, flat_idx);
        }
    }
}
