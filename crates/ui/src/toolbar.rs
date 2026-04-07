//! Top toolbar with icon buttons and separators.
//!
//! Build a [`Toolbar`] by chaining [`Toolbar::button`] and [`Toolbar::separator`]
//! calls, then call [`Toolbar::show`] to render it and get a list of clicked
//! button indices.

use egui::{Response, Ui, Vec2};

use crate::theme::ThemeColors;

/// A single toolbar button with a text label, tooltip, and enabled state.
pub struct ToolbarButton {
    /// Display text for the button.
    pub label: String,
    /// Hover tooltip text.
    pub tooltip: String,
    /// Whether the button is clickable.
    pub enabled: bool,
    /// Whether the button is in an active/pressed state.
    pub active: bool,
}

impl ToolbarButton {
    /// Creates a new enabled, non-active toolbar button.
    pub fn new(label: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tooltip: tooltip.into(),
            enabled: true,
            active: false,
        }
    }

    /// Builder method to set the enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder method to mark the button as currently active/pressed.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Show the button. Returns the [`Response`].
    ///
    /// `colors` should come from `ThemeManager::current_theme()` so buttons
    /// render correctly in both dark and light modes.
    pub fn show(&self, ui: &mut Ui, colors: &ThemeColors) -> Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            let text_color = if !self.enabled {
                colors.text_disabled
            } else if self.active {
                colors.accent
            } else {
                colors.text
            };

            let btn = egui::Button::new(
                egui::RichText::new(&self.label)
                    .size(13.0)
                    .color(text_color),
            )
            .min_size(Vec2::new(28.0, 28.0));

            let response = ui.add(btn);

            // Draw active indicator underline
            if self.active && self.enabled {
                let rect = response.rect;
                let underline = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), rect.bottom() - 2.0),
                    Vec2::new(rect.width(), 2.0),
                );
                ui.painter().rect_filled(underline, 0.0, colors.accent);
            }

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
    ///
    /// `colors` should come from `ThemeManager::current_theme()`.
    pub fn show(&self, ui: &mut Ui, colors: &ThemeColors) -> Vec<usize> {
        let mut clicked = Vec::new();
        let mut button_index = 0usize;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

            for item in &self.items {
                match item {
                    ToolbarItem::Button(btn) => {
                        if btn.show(ui, colors).clicked() {
                            clicked.push(button_index);
                        }
                        button_index += 1;
                    }
                    ToolbarItem::Separator => {
                        draw_separator(ui, colors);
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
pub fn draw_separator(ui: &mut Ui, colors: &ThemeColors) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_button_builder() {
        let btn = ToolbarButton::new("Test", "Tooltip")
            .enabled(false)
            .active(true);
        assert!(!btn.enabled);
        assert!(btn.active);
        assert_eq!(btn.label, "Test");
        assert_eq!(btn.tooltip, "Tooltip");
    }

    #[test]
    fn toolbar_button_defaults() {
        let btn = ToolbarButton::new("X", "");
        assert!(btn.enabled);
        assert!(!btn.active);
    }

    #[test]
    fn toolbar_builder_chain() {
        let tb = Toolbar::new()
            .button(ToolbarButton::new("A", ""))
            .separator()
            .button(ToolbarButton::new("B", ""));
        assert_eq!(tb.items.len(), 3);
    }

    #[test]
    fn toolbar_default() {
        let tb = Toolbar::default();
        assert!(tb.items.is_empty());
    }
}
