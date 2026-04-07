//! Top toolbar with icon buttons and separators.
//!
//! Build a [`Toolbar`] by chaining [`Toolbar::button`] and [`Toolbar::separator`]
//! calls, then call [`Toolbar::show`] to render it and get a list of clicked
//! button indices.

use egui::{Response, Ui, Vec2};

use crate::theme::ThemeColors;

/// A single toolbar button with a text label, tooltip, and enabled state.
pub struct ToolbarButton {
    pub label: String,
    pub tooltip: String,
    pub enabled: bool,
}

impl ToolbarButton {
    /// Creates a new enabled toolbar button.
    pub fn new(label: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tooltip: tooltip.into(),
            enabled: true,
        }
    }

    /// Builder method to set the enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Show the button. Returns the [`Response`].
    pub fn show(&self, ui: &mut Ui) -> Response {
        let colors = ThemeColors::dark_default();

        ui.add_enabled_ui(self.enabled, |ui| {
            let btn = egui::Button::new(
                egui::RichText::new(&self.label)
                    .size(13.0)
                    .color(if self.enabled {
                        colors.text
                    } else {
                        colors.text_dim
                    }),
            )
            .min_size(Vec2::new(28.0, 28.0));

            let response = ui.add(btn);
            if !self.tooltip.is_empty() {
                response.on_hover_text(&self.tooltip)
            } else {
                response
            }
        })
        .inner
    }
}

/// A horizontal toolbar that can hold buttons and separators.
pub struct Toolbar {
    items: Vec<ToolbarItem>,
}

enum ToolbarItem {
    Button(ToolbarButton),
    Separator,
}

impl Toolbar {
    /// Creates a new empty toolbar.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append a button to the toolbar.
    pub fn button(mut self, button: ToolbarButton) -> Self {
        self.items.push(ToolbarItem::Button(button));
        self
    }

    /// Append a vertical separator to the toolbar.
    pub fn separator(mut self) -> Self {
        self.items.push(ToolbarItem::Separator);
        self
    }

    /// Show the toolbar. Returns indices of buttons that were clicked.
    pub fn show(&self, ui: &mut Ui) -> Vec<usize> {
        let colors = ThemeColors::dark_default();
        let mut clicked = Vec::new();
        let mut button_index = 0usize;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

            for item in &self.items {
                match item {
                    ToolbarItem::Button(btn) => {
                        if btn.show(ui).clicked() {
                            clicked.push(button_index);
                        }
                        button_index += 1;
                    }
                    ToolbarItem::Separator => {
                        separator(ui, &colors);
                    }
                }
            }
        });

        clicked
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a vertical separator line in the toolbar.
pub fn separator(ui: &mut Ui, colors: &ThemeColors) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 24.0), egui::Sense::hover());
    let center_x = rect.center().x;
    ui.painter().line_segment(
        [
            egui::pos2(center_x, rect.top() + 4.0),
            egui::pos2(center_x, rect.bottom() - 4.0),
        ],
        egui::Stroke::new(1.0, colors.border),
    );
}
