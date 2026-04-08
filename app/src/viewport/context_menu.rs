//! Viewport right-click context menu.
//!
//! Provides Add (Mesh, Light, Camera, Audio, Particle), selection actions,
//! duplicate/delete, grouping, transform resets, snap settings, properties,
//! and view presets (Front/Back/Left/Right/Top/Bottom).

use eframe::egui;
use egui::Response;

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    /// Draw the right-click context menu for the viewport.
    pub(crate) fn draw_context_menu(&mut self, response: &Response) {
        response.context_menu(|ui| {
            // Add submenu
            ui.menu_button("Add", |ui| {
                if ui.button("Empty").clicked() {
                    self.add_entity("New Empty", "\u{25CB}");
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Mesh", |ui| {
                    use forge_modeling::primitives;
                    if ui.button("Cube").clicked() {
                        let mesh = primitives::generate_cube(1.0);
                        self.add_mesh_entity("New Cube", mesh);
                        ui.close_menu();
                    }
                    if ui.button("Sphere").clicked() {
                        let mesh = primitives::generate_icosphere(0.5, 3);
                        self.add_mesh_entity("New Sphere", mesh);
                        ui.close_menu();
                    }
                    if ui.button("Cylinder").clicked() {
                        let mesh = primitives::generate_cylinder(0.5, 1.0, 32);
                        self.add_mesh_entity("New Cylinder", mesh);
                        ui.close_menu();
                    }
                    if ui.button("Cone").clicked() {
                        let mesh = primitives::generate_cone(0.5, 1.0, 32);
                        self.add_mesh_entity("New Cone", mesh);
                        ui.close_menu();
                    }
                    if ui.button("Plane").clicked() {
                        let mesh = primitives::generate_plane(1.0, 1.0);
                        self.add_mesh_entity("New Plane", mesh);
                        ui.close_menu();
                    }
                    if ui.button("Torus").clicked() {
                        let mesh = primitives::generate_torus(0.5, 0.15, 32, 16);
                        self.add_mesh_entity("New Torus", mesh);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Light", |ui| {
                    for (label, name) in [
                        ("Directional Light", "New Directional Light"),
                        ("Point Light", "New Point Light"),
                        ("Spot Light", "New Spot Light"),
                        ("Area Light", "New Area Light"),
                    ] {
                        if ui.button(label).clicked() {
                            self.add_entity(name, "\u{2600}");
                            ui.close_menu();
                        }
                    }
                });
                if ui.button("Camera").clicked() {
                    self.add_entity("New Camera", "\u{1F3A5}");
                    ui.close_menu();
                }
                if ui.button("Audio Source").clicked() {
                    self.add_entity("New Audio Source", "\u{266B}");
                    ui.close_menu();
                }
                if ui.button("Particle System").clicked() {
                    self.add_entity("New Particle System", "\u{2728}");
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui
                .add(egui::Button::new("Select All        Ctrl+A"))
                .clicked()
            {
                self.select_all();
                ui.close_menu();
            }
            if ui.button("Deselect All").clicked() {
                self.deselect_all();
                ui.close_menu();
            }
            if ui.button("Invert Selection").clicked() {
                self.invert_selection();
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add(egui::Button::new("Duplicate         Ctrl+D"))
                .clicked()
            {
                self.duplicate_selected();
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Delete            Del"))
                .clicked()
            {
                self.delete_selected();
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add(egui::Button::new("Group             Ctrl+G"))
                .clicked()
            {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Group selected entities".into(),
                });
                ui.close_menu();
            }
            if ui.button("Ungroup").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Ungroup".into(),
                });
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("Transform", |ui| {
                if ui.button("Reset Position").clicked() {
                    if self.selected_entity < self.transforms.len() {
                        self.transforms[self.selected_entity][0] = 0.0;
                        self.transforms[self.selected_entity][1] = 0.0;
                        self.transforms[self.selected_entity][2] = 0.0;
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Reset position".into(),
                    });
                    ui.close_menu();
                }
                if ui.button("Reset Rotation").clicked() {
                    if self.selected_entity < self.transforms.len() {
                        self.transforms[self.selected_entity][3] = 0.0;
                        self.transforms[self.selected_entity][4] = 0.0;
                        self.transforms[self.selected_entity][5] = 0.0;
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Reset rotation".into(),
                    });
                    ui.close_menu();
                }
                if ui.button("Reset Scale").clicked() {
                    if self.selected_entity < self.transforms.len() {
                        self.transforms[self.selected_entity][6] = 1.0;
                        self.transforms[self.selected_entity][7] = 1.0;
                        self.transforms[self.selected_entity][8] = 1.0;
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Reset scale".into(),
                    });
                    ui.close_menu();
                }
                if ui.button("Reset All").clicked() {
                    if self.selected_entity < self.transforms.len() {
                        self.transforms[self.selected_entity] =
                            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
                    }
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Reset all transforms".into(),
                    });
                    ui.close_menu();
                }
            });
            ui.menu_button("Snap Settings", |ui| {
                if ui.button("Snap to Grid").clicked() {
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Snap to grid".into(),
                    });
                    ui.close_menu();
                }
                if ui.button("Snap to Surface").clicked() {
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Snap to surface".into(),
                    });
                    ui.close_menu();
                }
                if ui.button("Snap to Vertex").clicked() {
                    self.console_log.push(LogEntry {
                        level: LogLevel::Info,
                        message: "Snap to vertex".into(),
                    });
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button("Properties").clicked() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: "Open properties".into(),
                });
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("View", |ui| {
                use forge_viewport::camera::AxisView;
                for (label, view) in [
                    ("Front", AxisView::Front),
                    ("Back", AxisView::Back),
                    ("Left", AxisView::Left),
                    ("Right", AxisView::Right),
                    ("Top", AxisView::Top),
                    ("Bottom", AxisView::Bottom),
                ] {
                    if ui.button(label).clicked() {
                        self.orbit_camera.set_axis_view(view);
                        self.console_log.push(LogEntry {
                            level: LogLevel::Info,
                            message: format!("View: {}", label),
                        });
                        ui.close_menu();
                    }
                }
            });
        });
    }
}
