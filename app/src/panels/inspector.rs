//! Inspector panel on the right side.
//!
//! Shows the properties of the currently selected entity: Transform fields
//! (position, rotation, scale) and component-specific sections for meshes,
//! lights, and cameras.

use eframe::egui;
use egui::{Color32, CornerRadius, FontId, RichText, Sense, Vec2};

use crate::state::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the right-side inspector panel with entity properties.
    pub(crate) fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(true)
            .min_width(220.0)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Inspector")
                        .font(FontId::proportional(13.0))
                        .color(tc!(self, text))
                        .strong(),
                );
                ui.separator();

                let entity_names = self.flatten_outliner_names();
                let sel = self.selected_entity;
                if sel >= entity_names.len() || sel >= self.transforms.len() {
                    ui.label(RichText::new("No selection").color(tc!(self, text_dim)));
                    return;
                }

                let locked = self.is_entity_locked(sel);
                ui.label(
                    RichText::new(&entity_names[sel])
                        .font(FontId::proportional(14.0))
                        .color(tc!(self, accent))
                        .strong(),
                );
                if locked {
                    ui.label(
                        RichText::new("\u{1F512} Locked")
                            .font(FontId::proportional(10.0))
                            .color(tc!(self, text_dim)),
                    );
                }
                ui.add_space(8.0);
                ui.set_enabled(!locked);

                // Transform section
                egui::CollapsingHeader::new(
                    RichText::new("Transform")
                        .font(FontId::proportional(12.0))
                        .color(tc!(self, text)),
                )
                .default_open(true)
                .show(ui, |ui| {
                    let t = &mut self.transforms[sel];
                    Self::draw_vec3_field(ui, "Position", &mut t[0..3]);
                    Self::draw_vec3_field(ui, "Rotation", &mut t[3..6]);
                    Self::draw_vec3_field(ui, "Scale", &mut t[6..9]);
                });

                ui.add_space(8.0);

                // Component section - dynamically based on entity icon type
                if self.is_mesh_entity(sel) {
                    egui::CollapsingHeader::new(
                        RichText::new("Mesh Renderer")
                            .font(FontId::proportional(12.0))
                            .color(tc!(self, text)),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Mesh:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.label(
                                RichText::new(&entity_names[sel])
                                    .color(tc!(self, text))
                                    .font(FontId::proportional(11.0)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Material:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.label(
                                RichText::new("Default PBR")
                                    .color(tc!(self, secondary))
                                    .font(FontId::proportional(11.0)),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Cast Shadows:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.label(
                                RichText::new("Yes")
                                    .color(tc!(self, text))
                                    .font(FontId::proportional(11.0)),
                            );
                        });
                    });
                } else if self.is_light_entity(sel) {
                    egui::CollapsingHeader::new(
                        RichText::new("Light")
                            .font(FontId::proportional(12.0))
                            .color(tc!(self, text)),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Intensity:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.light_intensity)
                                    .speed(0.01)
                                    .range(0.0..=10.0)
                                    .max_decimals(2),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Color:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(16.0, 16.0), Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::same(3),
                                Color32::from_rgb(255, 244, 214),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Shadows:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.label(
                                RichText::new("Soft")
                                    .color(tc!(self, text))
                                    .font(FontId::proportional(11.0)),
                            );
                        });
                    });
                } else if self.is_camera_entity(sel) {
                    egui::CollapsingHeader::new(
                        RichText::new("Camera")
                            .font(FontId::proportional(12.0))
                            .color(tc!(self, text)),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("FOV:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.camera_fov)
                                    .speed(0.5)
                                    .range(1.0..=179.0)
                                    .max_decimals(1),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Near:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.camera_near)
                                    .speed(0.01)
                                    .range(0.001..=100.0)
                                    .max_decimals(3),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Far:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.add(
                                egui::DragValue::new(&mut self.camera_far)
                                    .speed(1.0)
                                    .range(1.0..=100000.0)
                                    .max_decimals(1),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Projection:")
                                    .color(tc!(self, text_dim))
                                    .font(FontId::proportional(11.0)),
                            );
                            ui.label(
                                RichText::new("Perspective")
                                    .color(tc!(self, text))
                                    .font(FontId::proportional(11.0)),
                            );
                        });
                    });
                }
            });
    }

    /// Draw a labeled XYZ drag-value row (used for position, rotation, scale).
    pub(crate) fn draw_vec3_field(ui: &mut egui::Ui, label: &str, vals: &mut [f32]) {
        let dim = Color32::from_rgb(0x9b, 0x9b, 0xa1);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(label)
                    .font(FontId::proportional(11.0))
                    .color(dim),
            );
        });
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("X")
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(0xe7, 0x4c, 0x3c)),
            );
            ui.add(
                egui::DragValue::new(&mut vals[0])
                    .speed(0.1)
                    .max_decimals(2),
            );
            ui.label(
                RichText::new("Y")
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(0x2e, 0xcc, 0x71)),
            );
            ui.add(
                egui::DragValue::new(&mut vals[1])
                    .speed(0.1)
                    .max_decimals(2),
            );
            ui.label(
                RichText::new("Z")
                    .font(FontId::proportional(10.0))
                    .color(Color32::from_rgb(0x3e, 0x55, 0xff)),
            );
            ui.add(
                egui::DragValue::new(&mut vals[2])
                    .speed(0.1)
                    .max_decimals(2),
            );
        });
    }
}
