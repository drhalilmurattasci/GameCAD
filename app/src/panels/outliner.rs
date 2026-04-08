//! Outliner (scene-tree) panel on the left side.
//!
//! Displays the hierarchical entity tree with icons, supports single click,
//! Ctrl+click (toggle), and Shift+click (range) selection, and provides a
//! right-click context menu for add / duplicate / delete.

use eframe::egui;
use egui::{FontId, RichText};

use crate::panels::tree_row::{self, TreeRowStyle};
use crate::state::{EditorLayer, ForgeEditorApp};
use crate::state::types::*;

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
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() * 0.6)
                    .show(ui, |ui| {
                        let tree = self.outliner.clone();
                        let mut flat_idx = 0usize;
                        let mut toggle_ids = Vec::new();
                        for root in &tree {
                            self.draw_outliner_node(ui, root, 0, &mut flat_idx, &mut toggle_ids);
                        }
                        // Apply expand/collapse toggles back to the real outliner
                        if !toggle_ids.is_empty() {
                            fn apply_toggles(node: &mut OutlinerNode, ids: &[forge_core::id::NodeId]) {
                                if ids.contains(&node.id) {
                                    node.expanded = !node.expanded;
                                }
                                for child in &mut node.children {
                                    apply_toggles(child, ids);
                                }
                            }
                            for root in &mut self.outliner {
                                apply_toggles(root, &toggle_ids);
                            }
                        }
                    });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // ---- Layers panel ----
                ui.label(
                    RichText::new("Layers")
                        .font(FontId::proportional(13.0))
                        .color(tc!(self, text))
                        .strong(),
                );
                ui.separator();

                let active_path = self.active_layer.clone();
                let mut new_active: Option<Vec<usize>> = None;
                let style = TreeRowStyle {
                    accent: tc!(self, accent),
                    text: tc!(self, text),
                    text_dim: tc!(self, text_dim),
                    indent_px: 18.0,
                };

                let mut delete_path: Option<Vec<usize>> = None;
                egui::ScrollArea::vertical()
                    .id_salt("layers_scroll")
                    .show(ui, |ui| {
                        for (i, layer) in self.layers.iter_mut().enumerate() {
                            let path = vec![i];
                            let deleted = Self::draw_layer_entry(
                                ui, layer, &path, &active_path, 0,
                                &style, &mut new_active,
                            );
                            if deleted {
                                delete_path = Some(path);
                            }
                        }
                    });
                if let Some(new_path) = new_active {
                    self.active_layer = new_path;
                }

                // Handle layer deletion
                if let Some(del_path) = delete_path {
                    if del_path.len() == 1 && self.layers.len() > 1 {
                        let del = del_path[0];
                        let removed = self.layers.remove(del);
                        self.console_log.push(LogEntry {
                            level: LogLevel::Info,
                            message: format!("Deleted layer '{}'", removed.name),
                        });
                        if self.active_layer.first().is_some_and(|&a| a >= self.layers.len()) {
                            self.active_layer = vec![self.layers.len() - 1];
                        }
                    }
                }
            });
    }

    /// Recursively draw a single outliner node and its children.
    ///
    /// `toggle_ids` collects NodeIds whose `expanded` flag should be flipped
    /// after drawing (since we draw from an immutable clone).
    pub(crate) fn draw_outliner_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &OutlinerNode,
        depth: usize,
        flat_idx: &mut usize,
        toggle_ids: &mut Vec<forge_core::id::NodeId>,
    ) {
        let idx = *flat_idx;
        *flat_idx += 1;
        let selected = self.selected_entities.contains(&idx);
        let is_primary = idx == self.selected_entity;
        let node_name = node.name.clone();
        let has_children = !node.children.is_empty();

        let style = TreeRowStyle {
            accent: tc!(self, accent),
            text: tc!(self, text),
            text_dim: tc!(self, text_dim),
            indent_px: 16.0,
        };

        let row_resp = ui.horizontal(|ui| {
            tree_row::draw_row_background(ui, selected, is_primary, &style);
            tree_row::draw_indent(ui, depth, &style);

            if tree_row::draw_expand_toggle(ui, has_children, node.expanded, &style) {
                toggle_ids.push(node.id);
            }

            tree_row::draw_icon_label(ui, node.icon, selected, &style);
            let resp = tree_row::draw_name_button(ui, &node.name, selected, &style);

            let modifiers = ui.input(|i| i.modifiers);
            if resp.clicked() {
                if modifiers.ctrl {
                    if let Some(pos_in_vec) =
                        self.selected_entities.iter().position(|&e| e == idx)
                    {
                        self.selected_entities.remove(pos_in_vec);
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
                    self.selected_entity = idx;
                    self.selected_entities = vec![idx];
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: format!("Selected entity: {}", node.name),
                    });
                }
            }
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

        // Only draw children when expanded
        if node.expanded {
            for child in &node.children {
                self.draw_outliner_node(ui, child, depth + 1, flat_idx, toggle_ids);
            }
        } else {
            // Still need to advance flat_idx past collapsed children
            fn count_descendants(n: &OutlinerNode) -> usize {
                let mut c = n.children.len();
                for child in &n.children { c += count_descendants(child); }
                c
            }
            *flat_idx += count_descendants(node);
        }
    }

    /// Draw a single layer row in outliner tree style.
    /// Returns `true` if the delete button was clicked for this layer.
    fn draw_layer_entry(
        ui: &mut egui::Ui,
        layer: &mut EditorLayer,
        path: &[usize],
        active_path: &[usize],
        depth: usize,
        style: &TreeRowStyle,
        new_active: &mut Option<Vec<usize>>,
    ) -> bool {
        let is_active = path == active_path;
        let has_children = !layer.children.is_empty();
        let mut deleted = false;

        let row_resp = ui.horizontal(|ui| {
            tree_row::draw_row_background(ui, is_active, true, style);
            tree_row::draw_indent(ui, depth, style);

            if tree_row::draw_expand_toggle(ui, has_children, layer.expanded, style) {
                layer.expanded = !layer.expanded;
            }

            tree_row::draw_color_swatch(ui, layer.color, is_active, style);

            let name_resp = tree_row::draw_name_button(ui, &layer.name, is_active, style);
            if name_resp.clicked() {
                *new_active = Some(path.to_vec());
            }

            tree_row::draw_badge(ui, layer.total_entity_count(), style);
            tree_row::draw_toggle_button(ui, "\u{1F441}", "\u{2014}", &mut layer.visible, "Toggle visibility");
            tree_row::draw_toggle_button(ui, "\u{1F512}", "\u{1F513}", &mut layer.locked, "Toggle lock");

            // Delete button
            let del_text = RichText::new("\u{2716}").font(FontId::proportional(10.0)).color(style.text_dim);
            if ui.add(egui::Button::new(del_text).frame(false))
                .on_hover_text("Delete layer")
                .clicked()
            {
                deleted = true;
            }
        });

        // Right-click context menu
        row_resp.response.context_menu(|ui| {
            if ui.button("Delete Layer").clicked() {
                deleted = true;
                ui.close_menu();
            }
            if ui.button("Add Sublayer").clicked() {
                let child_color = layer.color.linear_multiply(0.7);
                let n = layer.children.len();
                layer.add_sublayer(EditorLayer::new(
                    format!("{} Sub-{}", layer.name, n + 1),
                    child_color,
                ));
                layer.expanded = true;
                ui.close_menu();
            }
        });

        // Draw sublayers only when expanded
        let mut child_delete_idx: Option<usize> = None;
        if layer.expanded {
            for (ci, sublayer) in layer.children.iter_mut().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(ci);
                if Self::draw_layer_entry(
                    ui, sublayer, &child_path, active_path, depth + 1,
                    style, new_active,
                ) {
                    child_delete_idx = Some(ci);
                }
            }
        }
        if let Some(ci) = child_delete_idx {
            layer.children.remove(ci);
        }
        deleted
    }
}
