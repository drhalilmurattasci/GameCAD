//! Bottom panel: Content Browser, Console, Agent Progress, and Timeline.
//!
//! Each sub-tab is drawn by a dedicated method. The console supports a
//! clear button and auto-scroll. The timeline shows an animated playhead
//! and keyframe dots.

use eframe::egui;
use egui::{
    Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    /// Draw the bottom panel with tab strip and active tab content.
    pub(crate) fn draw_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .min_height(120.0)
            .default_height(180.0)
            .show(ctx, |ui| {
                // Tab strip
                ui.horizontal(|ui| {
                    for tab in BottomTab::ALL {
                        let active = tab == self.bottom_tab;
                        let resp = ui.add(
                            egui::Button::new(
                                RichText::new(tab.label())
                                    .font(FontId::proportional(12.0))
                                    .color(if active {
                                        tc!(self, accent)
                                    } else {
                                        tc!(self, text_dim)
                                    }),
                            )
                            .frame(false),
                        );
                        if active {
                            let r = resp.rect;
                            ui.painter().line_segment(
                                [
                                    Pos2::new(r.left(), r.bottom()),
                                    Pos2::new(r.right(), r.bottom()),
                                ],
                                Stroke::new(2.0, tc!(self, accent)),
                            );
                        }
                        if resp.clicked() {
                            self.bottom_tab = tab;
                        }
                    }
                });
                ui.separator();

                match self.bottom_tab {
                    BottomTab::ContentBrowser => self.draw_content_browser(ui),
                    BottomTab::Console => self.draw_console(ui),
                    BottomTab::AgentProgress => self.draw_agent_progress(ui),
                    BottomTab::Timeline => self.draw_timeline(ui),
                }
            });
    }

    /// Draw the content browser grid with material swatches.
    pub(crate) fn draw_content_browser(&mut self, ui: &mut egui::Ui) {
        use forge_materials::material::ColorOrTexture;

        // Collect materials into a Vec to avoid borrow conflict
        let mat_entries: Vec<(String, Color32)> = self
            .material_library
            .list()
            .map(|(_id, mat)| {
                let color = match &mat.albedo {
                    ColorOrTexture::Color(c) => Color32::from_rgba_premultiplied(
                        (c.r * 255.0) as u8,
                        (c.g * 255.0) as u8,
                        (c.b * 255.0) as u8,
                        (c.a * 255.0) as u8,
                    ),
                    ColorOrTexture::Texture(_) => Color32::GRAY,
                };
                (mat.name.clone(), color)
            })
            .collect();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (name, color) in &mat_entries {
                    ui.vertical(|ui| {
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(64.0, 64.0), Sense::click());
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(4), *color);
                        let border_color = if resp.hovered() {
                            tc!(self, accent)
                        } else {
                            tc!(self, border)
                        };
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(4),
                            Stroke::new(1.0, border_color),
                            StrokeKind::Outside,
                        );
                        if resp.clicked() {
                            self.console_log.push(LogEntry {
                                level: LogLevel::Info,
                                message: format!("Selected material: {}", name),
                            });
                        }
                        ui.label(
                            RichText::new(name)
                                .font(FontId::proportional(10.0))
                                .color(tc!(self, text_dim)),
                        );
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }

    /// Draw the console log with clear button and scrollable entries.
    pub(crate) fn draw_console(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Clear")
                            .font(FontId::proportional(11.0))
                            .color(tc!(self, text_dim)),
                    )
                    .frame(false),
                )
                .clicked()
            {
                self.console_log.clear();
            }
            ui.label(
                RichText::new(format!("{} entries", self.console_log.len()))
                    .font(FontId::proportional(10.0))
                    .color(tc!(self, text_dim)),
            );
        });
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for entry in &self.console_log {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(entry.level.prefix())
                                .font(FontId::monospace(11.0))
                                .color(entry.level.color()),
                        );
                        ui.label(
                            RichText::new(&entry.message)
                                .font(FontId::monospace(11.0))
                                .color(tc!(self, text)),
                        );
                    });
                }
            });
    }

    /// Draw the agent progress panel with task bars.
    pub(crate) fn draw_agent_progress(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);
                for task in &self.tasks {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        // Task name (fixed width)
                        let name_text = RichText::new(&task.name)
                            .font(FontId::proportional(12.0))
                            .color(tc!(self, text));
                        ui.allocate_ui(Vec2::new(160.0, 18.0), |ui| {
                            ui.label(name_text);
                        });

                        // Progress bar
                        let bar_width = 300.0;
                        let bar_height = 14.0;
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(bar_width, bar_height), Sense::hover());
                        // Background
                        ui.painter().rect_filled(
                            rect,
                            CornerRadius::same(3),
                            Color32::from_rgb(0x18, 0x18, 0x1a),
                        );
                        // Fill
                        if task.progress > 0.0 {
                            let fill_rect = Rect::from_min_size(
                                rect.min,
                                Vec2::new(bar_width * task.progress, bar_height),
                            );
                            ui.painter().rect_filled(
                                fill_rect,
                                CornerRadius::same(3),
                                task.status.color(),
                            );
                        }
                        // Border
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(3),
                            Stroke::new(1.0, tc!(self, border)),
                            StrokeKind::Outside,
                        );

                        // Percentage
                        ui.label(
                            RichText::new(format!("{:>3.0}%", task.progress * 100.0))
                                .font(FontId::monospace(11.0))
                                .color(tc!(self, text)),
                        );

                        // Status label
                        ui.label(
                            RichText::new(task.status.label())
                                .font(FontId::proportional(11.0))
                                .color(task.status.color()),
                        );
                    });
                    ui.add_space(2.0);
                }
            });
    }

    /// Draw the timeline track with keyframe dots and animated playhead.
    pub(crate) fn draw_timeline(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Timeline")
                    .font(FontId::proportional(13.0))
                    .color(tc!(self, text))
                    .strong(),
            );
            ui.separator();
            ui.label(
                RichText::new("0:00 / 10:00")
                    .font(FontId::monospace(11.0))
                    .color(tc!(self, text_dim)),
            );
        });
        ui.add_space(4.0);

        // Timeline track bar
        let desired = Vec2::new(ui.available_width() - 16.0, 24.0);
        let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
        let painter = ui.painter();

        // Track background
        painter.rect_filled(
            rect,
            CornerRadius::same(3),
            Color32::from_rgb(0x18, 0x18, 0x1a),
        );
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            Stroke::new(1.0, tc!(self, border)),
            StrokeKind::Outside,
        );

        // Keyframe dots
        for frac in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let kx = rect.min.x + rect.width() * frac;
            let ky = rect.center().y;
            painter.circle_filled(
                Pos2::new(kx, ky),
                4.0,
                tc!(self, accent).linear_multiply(0.5),
            );
        }

        // Animated playhead
        let t = (self.frame_count as f32 * 0.002) % 1.0;
        let x = rect.min.x + rect.width() * t;
        painter.line_segment(
            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
            Stroke::new(2.0, tc!(self, accent)),
        );

        // Playhead triangle marker
        let tri_size = 5.0;
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(x - tri_size, rect.min.y),
                Pos2::new(x + tri_size, rect.min.y),
                Pos2::new(x, rect.min.y + tri_size),
            ],
            tc!(self, accent),
            Stroke::NONE,
        ));
    }
}
