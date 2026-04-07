//! egui widget functions for editing individual property values.

use egui::{Color32, RichText, Ui};
use forge_core::math::{Color, Vec3};

// ── Crystalline accent colors for axis labels ───────────────────────
const X_COLOR: Color32 = Color32::from_rgb(0xFF, 0x55, 0x55); // red
const Y_COLOR: Color32 = Color32::from_rgb(0x4E, 0xFF, 0x93); // green (accent)
const Z_COLOR: Color32 = Color32::from_rgb(0x3E, 0x55, 0xFF); // blue (secondary)

/// Draws a labeled float drag-value widget. Returns `true` if the value changed.
pub fn draw_float(ui: &mut Ui, label: &str, value: &mut f32, speed: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.add(egui::DragValue::new(value).speed(speed)).changed() {
            changed = true;
        }
    });
    changed
}

/// Draws a labeled Vec3 editor with colored X / Y / Z sub-labels.
/// Returns `true` if any component changed.
pub fn draw_vec3(ui: &mut Ui, label: &str, value: &mut Vec3) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.label(RichText::new("X").color(X_COLOR).strong());
        if ui
            .add(egui::DragValue::new(&mut value.x).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label(RichText::new("Y").color(Y_COLOR).strong());
        if ui
            .add(egui::DragValue::new(&mut value.y).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label(RichText::new("Z").color(Z_COLOR).strong());
        if ui
            .add(egui::DragValue::new(&mut value.z).speed(0.1))
            .changed()
        {
            changed = true;
        }
    });
    changed
}

/// Draws a color picker for a linear RGBA [`Color`].
/// Returns `true` if the color changed.
pub fn draw_color(ui: &mut Ui, label: &str, value: &mut Color) -> bool {
    let mut rgba = [value.r, value.g, value.b, value.a];
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.color_edit_button_rgba_unmultiplied(&mut rgba).changed() {
            value.r = rgba[0];
            value.g = rgba[1];
            value.b = rgba[2];
            value.a = rgba[3];
            changed = true;
        }
    });
    changed
}

/// Draws a labeled boolean checkbox. Returns `true` if the value changed.
pub fn draw_bool(ui: &mut Ui, label: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui.checkbox(value, label).changed() {
            changed = true;
        }
    });
    changed
}

/// Draws a labeled single-line text editor. Returns `true` if the text changed.
pub fn draw_string(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.text_edit_singleline(value).changed() {
            changed = true;
        }
    });
    changed
}

/// Draws a labeled enum dropdown (combo box). Returns `true` if the selection changed.
pub fn draw_enum(ui: &mut Ui, label: &str, options: &[String], selected: &mut usize) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let current = options
            .get(*selected)
            .map(|s| s.as_str())
            .unwrap_or("<invalid>");
        egui::ComboBox::from_id_salt(label)
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (i, option) in options.iter().enumerate() {
                    if ui.selectable_value(selected, i, option).changed() {
                        changed = true;
                    }
                }
            });
    });
    changed
}

/// Draws a labeled float slider clamped to `[min, max]`. Returns `true` if the value changed.
pub fn draw_slider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.add(egui::Slider::new(value, min..=max)).changed() {
            changed = true;
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    // Widget functions require an egui context to test. Compilation alone
    // validates the API surface; runtime testing is done via integration tests.

    #[test]
    fn module_compiles() {
        // Existence test -- if this module compiles, the widget signatures are valid.
    }
}
