//! Crystalline theme engine for the Forge Editor.
//!
//! Provides dark and light themes inspired by Baby Audio's Crystalline plugin,
//! a [`ThemeManager`] for runtime switching, and viewport gradient bands.

use egui::{Color32, Context, FontFamily, FontId, Stroke, Visuals};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// ThemeMode
// ─────────────────────────────────────────────────────────────────────

/// Dark or light theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Dark color scheme.
    Dark,
    /// Light color scheme.
    Light,
}

// ─────────────────────────────────────────────────────────────────────
// ThemeColors
// ─────────────────────────────────────────────────────────────────────

/// All theme colors in one struct.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    /// Main background color.
    pub background: Color32,
    /// Panel / card surface color.
    pub surface: Color32,
    /// Slightly elevated surface (tooltips, menus).
    pub surface_raised: Color32,
    /// Inset / sunken surface (input fields, wells).
    pub surface_sunken: Color32,
    /// Primary accent color.
    pub accent: Color32,
    /// Secondary accent color.
    pub secondary: Color32,
    /// Primary text color.
    pub text: Color32,
    /// Dimmed / secondary text color.
    pub text_dim: Color32,
    /// Disabled text color.
    pub text_disabled: Color32,
    /// Border color.
    pub border: Color32,
    /// Focused-element border color.
    pub border_focused: Color32,
    /// Error / danger color.
    pub error: Color32,
    /// Warning color.
    pub warning: Color32,
    /// Success color.
    pub success: Color32,
    /// Selection highlight color.
    pub selection: Color32,
}

impl ThemeColors {
    /// Crystalline dark palette.
    pub fn dark_default() -> Self {
        Self {
            background: hex(0x1f1f21),
            surface: hex(0x262629),
            surface_raised: hex(0x2b2b2f),
            surface_sunken: hex(0x0f1623),
            accent: hex(0x4eff93),
            secondary: hex(0x3e55ff),
            text: hex(0xe7e7ea),
            text_dim: hex(0x9b9ba1),
            text_disabled: hex(0x555555),
            border: hex(0x2a2a4a),
            border_focused: hex(0x4eff93),
            error: hex(0xe74c3c),
            warning: hex(0xf39c12),
            success: hex(0x2ecc71),
            selection: Color32::from_rgba_premultiplied(0x4e, 0xff, 0x93, 0x33),
        }
    }

    /// Crystalline light palette.
    pub fn light_default() -> Self {
        Self {
            background: hex(0xf0f0f4),
            surface: hex(0xe8e8ec),
            surface_raised: hex(0xffffff),
            surface_sunken: hex(0xdcdce0),
            accent: hex(0x00b860),
            secondary: hex(0x2d44cc),
            text: hex(0x1a1a2e),
            text_dim: hex(0x6b6b7b),
            text_disabled: hex(0xa0a0a8),
            border: hex(0xd0d0d8),
            border_focused: hex(0x00b860),
            error: hex(0xd43a2a),
            warning: hex(0xd68a0a),
            success: hex(0x1fa855),
            selection: Color32::from_rgba_premultiplied(0x00, 0xb8, 0x60, 0x33),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// ThemeManager
// ─────────────────────────────────────────────────────────────────────

/// 8-band viewport gradient (top to bottom).
pub type ViewportGradient = [[u8; 3]; 8];

const DARK_GRADIENT: ViewportGradient = [
    [0x18, 0x18, 0x20],
    [0x16, 0x16, 0x1e],
    [0x14, 0x14, 0x1c],
    [0x12, 0x12, 0x1a],
    [0x10, 0x10, 0x18],
    [0x0e, 0x0e, 0x16],
    [0x0d, 0x0d, 0x14],
    [0x0c, 0x0c, 0x12],
];

const LIGHT_GRADIENT: ViewportGradient = [
    [0xc8, 0xcc, 0xe0],
    [0xcf, 0xd3, 0xe4],
    [0xd6, 0xda, 0xe8],
    [0xdd, 0xe0, 0xec],
    [0xe4, 0xe6, 0xf0],
    [0xea, 0xec, 0xf4],
    [0xf0, 0xf1, 0xf6],
    [0xf5, 0xf5, 0xf8],
];

/// Manages dark/light/custom theme switching at runtime.
pub struct ThemeManager {
    mode: ThemeMode,
    dark: ThemeColors,
    light: ThemeColors,
    custom_themes: Vec<(String, ThemeColors)>,
}

impl ThemeManager {
    /// Creates a new theme manager defaulting to **light** mode.
    pub fn new() -> Self {
        Self {
            mode: ThemeMode::Light,
            dark: ThemeColors::dark_default(),
            light: ThemeColors::light_default(),
            custom_themes: Vec::new(),
        }
    }

    /// Returns the active theme mode.
    #[inline]
    pub fn current_mode(&self) -> ThemeMode {
        self.mode
    }

    /// Returns `true` if the current mode is [`ThemeMode::Dark`].
    #[inline]
    pub fn is_dark(&self) -> bool {
        self.mode == ThemeMode::Dark
    }

    /// Returns the color palette for the current mode.
    #[inline]
    pub fn current_theme(&self) -> &ThemeColors {
        match self.mode {
            ThemeMode::Dark => &self.dark,
            ThemeMode::Light => &self.light,
        }
    }

    /// Returns the 8-band viewport background gradient for the current mode.
    #[inline]
    pub fn viewport_gradient(&self) -> &ViewportGradient {
        match self.mode {
            ThemeMode::Dark => &DARK_GRADIENT,
            ThemeMode::Light => &LIGHT_GRADIENT,
        }
    }

    /// Grid line color adapts to background.
    #[inline]
    pub fn grid_color(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgba_premultiplied(255, 255, 255, 15)
        } else {
            Color32::from_rgba_premultiplied(0, 0, 0, 20)
        }
    }

    /// Major grid line color.
    #[inline]
    pub fn grid_major_color(&self) -> Color32 {
        if self.is_dark() {
            Color32::from_rgba_premultiplied(255, 255, 255, 30)
        } else {
            Color32::from_rgba_premultiplied(0, 0, 0, 40)
        }
    }

    /// Wireframe color for 3D objects.
    #[inline]
    pub fn wireframe_color(&self) -> Color32 {
        if self.is_dark() {
            hex(0x4eff93)
        } else {
            hex(0x00b860)
        }
    }

    /// Switch between dark and light mode.
    pub fn toggle_theme(&mut self) {
        self.mode = match self.mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    /// Set the theme mode explicitly.
    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.mode = mode;
    }

    /// Register a custom named theme.
    pub fn add_custom_theme(&mut self, name: String, colors: ThemeColors) {
        self.custom_themes.push((name, colors));
    }

    /// List all available theme names (built-in + custom).
    pub fn list_themes(&self) -> Vec<&str> {
        let mut names = vec!["Dark", "Light"];
        for (name, _) in &self.custom_themes {
            names.push(name.as_str());
        }
        names
    }

    /// Select a theme by name. Returns `true` if the name was recognized.
    pub fn select_theme(&mut self, name: &str) -> bool {
        match name {
            "Dark" => { self.mode = ThemeMode::Dark; true }
            "Light" => { self.mode = ThemeMode::Light; true }
            _ => {
                // Custom themes aren't a separate mode yet — future extension
                false
            }
        }
    }

    /// Apply the active theme to egui.
    pub fn apply_to_egui(&self, ctx: &Context) {
        let colors = self.current_theme();
        let mut style = (*ctx.style()).clone();

        let mut visuals = if self.is_dark() {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.panel_fill = colors.background;
        visuals.window_fill = colors.surface;
        visuals.extreme_bg_color = if self.is_dark() { colors.surface_sunken } else { colors.surface_raised };
        visuals.faint_bg_color = colors.surface;

        visuals.override_text_color = Some(colors.text);

        visuals.selection.bg_fill = colors.accent.linear_multiply(0.25);
        visuals.selection.stroke = Stroke::new(1.0, colors.accent);

        visuals.widgets.noninteractive.bg_fill = colors.surface;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_dim);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.border);

        visuals.widgets.inactive.bg_fill = colors.surface_raised;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.border);

        visuals.widgets.hovered.bg_fill = colors.surface_raised;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors.accent);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors.accent);

        visuals.widgets.active.bg_fill = colors.accent.linear_multiply(0.3);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors.accent);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, colors.accent);

        visuals.window_stroke = Stroke::new(1.0, colors.border);

        style.visuals = visuals;

        style.text_styles.insert(egui::TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Button, FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));

        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.window_margin = egui::Margin::same(8);

        ctx.set_style(style);
    }

    /// Render a theme selector widget: toggle button + color swatch preview.
    pub fn theme_selector_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Sun/Moon toggle
            let icon = if self.is_dark() { "🌙" } else { "☀️" };
            if ui.button(icon).on_hover_text("Toggle theme (Ctrl+T)").clicked() {
                self.toggle_theme();
            }

            ui.separator();

            // Theme name
            let label = if self.is_dark() { "Dark" } else { "Light" };
            ui.label(label);

            ui.separator();

            // Color swatches
            let tc = *self.current_theme();
            let swatch_size = egui::vec2(14.0, 14.0);
            let swatches = [
                ("BG", tc.background),
                ("Surface", tc.surface),
                ("Accent", tc.accent),
                ("2nd", tc.secondary),
                ("Text", tc.text),
            ];
            for (tip, color) in &swatches {
                let (rect, resp) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());
                ui.painter().rect_filled(rect, 2, *color);
                ui.painter().rect_stroke(rect, 2, Stroke::new(1.0, tc.border), egui::StrokeKind::Outside);
                resp.on_hover_text(*tip);
            }

            ui.separator();

            // Gradient preview strip
            let gradient = self.viewport_gradient();
            let strip_w = 80.0;
            let band_w = strip_w / 8.0;
            let strip_h = 14.0;
            let (strip_rect, _) = ui.allocate_exact_size(egui::vec2(strip_w, strip_h), egui::Sense::hover());
            for (i, rgb) in gradient.iter().enumerate() {
                let x0 = strip_rect.left() + i as f32 * band_w;
                let band_rect = egui::Rect::from_min_size(
                    egui::pos2(x0, strip_rect.top()),
                    egui::vec2(band_w + 1.0, strip_h),
                );
                ui.painter().rect_filled(band_rect, 0, Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
            }
        });
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

/// Convert a 24-bit hex value (0xRRGGBB) to [`Color32`].
#[inline]
pub fn hex_to_color32(hex_val: u32) -> Color32 {
    hex(hex_val)
}

#[inline]
fn hex(v: u32) -> Color32 {
    let r = ((v >> 16) & 0xFF) as u8;
    let g = ((v >> 8) & 0xFF) as u8;
    let b = (v & 0xFF) as u8;
    Color32::from_rgb(r, g, b)
}

// ─────────────────────────────────────────────────────────────────────
// Legacy compat
// ─────────────────────────────────────────────────────────────────────

/// Apply the default theme (backward-compatible convenience function).
///
/// Equivalent to `ThemeManager::new().apply_to_egui(ctx)`.
/// Note: `ThemeManager::new()` defaults to **light** mode.
pub fn apply_to_egui(ctx: &Context) {
    ThemeManager::new().apply_to_egui(ctx);
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_color32_white() {
        assert_eq!(hex(0xFFFFFF), Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn hex_to_color32_black() {
        assert_eq!(hex(0x000000), Color32::from_rgb(0, 0, 0));
    }

    #[test]
    fn dark_default_colors_are_distinct() {
        let c = ThemeColors::dark_default();
        assert_ne!(c.background, c.accent);
        assert_ne!(c.text, c.text_dim);
    }

    #[test]
    fn light_default_colors_are_distinct() {
        let c = ThemeColors::light_default();
        assert_ne!(c.background, c.accent);
        assert_ne!(c.text, c.text_dim);
    }

    #[test]
    fn dark_and_light_are_different() {
        let d = ThemeColors::dark_default();
        let l = ThemeColors::light_default();
        assert_ne!(d.background, l.background);
        assert_ne!(d.accent, l.accent);
    }

    #[test]
    fn theme_manager_toggle() {
        let mut tm = ThemeManager::new();
        assert_eq!(tm.current_mode(), ThemeMode::Light);
        tm.toggle_theme();
        assert_eq!(tm.current_mode(), ThemeMode::Dark);
        tm.toggle_theme();
        assert_eq!(tm.current_mode(), ThemeMode::Light);
    }

    #[test]
    fn theme_manager_list() {
        let tm = ThemeManager::new();
        let list = tm.list_themes();
        assert_eq!(list, vec!["Dark", "Light"]);
    }

    #[test]
    fn gradient_bands_count() {
        assert_eq!(DARK_GRADIENT.len(), 8);
        assert_eq!(LIGHT_GRADIENT.len(), 8);
    }

    #[test]
    fn wireframe_color_differs_by_mode() {
        let mut tm = ThemeManager::new();
        // Default is Light
        let light_wire = tm.wireframe_color();
        tm.toggle_theme();
        // Now Dark
        let dark_wire = tm.wireframe_color();
        assert_ne!(dark_wire, light_wire);
    }

    #[test]
    fn select_theme_by_name() {
        let mut tm = ThemeManager::new();
        assert!(tm.select_theme("Dark"));
        assert_eq!(tm.current_mode(), ThemeMode::Dark);
        assert!(tm.select_theme("Light"));
        assert_eq!(tm.current_mode(), ThemeMode::Light);
        assert!(!tm.select_theme("NonExistent"));
    }

    #[test]
    fn set_mode_explicit() {
        let mut tm = ThemeManager::new();
        tm.set_mode(ThemeMode::Dark);
        assert!(tm.is_dark());
        tm.set_mode(ThemeMode::Light);
        assert!(!tm.is_dark());
    }

    #[test]
    fn custom_theme_listed() {
        let mut tm = ThemeManager::new();
        tm.add_custom_theme("Solarized".to_string(), ThemeColors::dark_default());
        let list = tm.list_themes();
        assert_eq!(list, vec!["Dark", "Light", "Solarized"]);
    }

    #[test]
    fn grid_colors_differ_by_mode() {
        let mut tm = ThemeManager::new();
        let light_grid = tm.grid_color();
        let light_major = tm.grid_major_color();
        tm.toggle_theme();
        let dark_grid = tm.grid_color();
        let dark_major = tm.grid_major_color();
        assert_ne!(light_grid, dark_grid);
        assert_ne!(light_major, dark_major);
    }

    #[test]
    fn hex_to_color32_accent_green() {
        let c = hex(0x4eff93);
        assert_eq!(c, Color32::from_rgb(0x4e, 0xff, 0x93));
    }

    #[test]
    fn selection_color_has_alpha() {
        let d = ThemeColors::dark_default();
        // selection color should be semi-transparent
        assert!(d.selection.a() < 255);

        let l = ThemeColors::light_default();
        assert!(l.selection.a() < 255);
    }

    #[test]
    fn viewport_gradient_bands_descend_in_dark() {
        // Dark gradient bands should generally get darker top to bottom
        let first = DARK_GRADIENT[0];
        let last = DARK_GRADIENT[7];
        let first_brightness: u16 = first.iter().map(|&c| c as u16).sum();
        let last_brightness: u16 = last.iter().map(|&c| c as u16).sum();
        assert!(first_brightness >= last_brightness);
    }

    #[test]
    fn viewport_gradient_bands_ascend_in_light() {
        // Light gradient bands should generally get lighter top to bottom
        let first = LIGHT_GRADIENT[0];
        let last = LIGHT_GRADIENT[7];
        let first_brightness: u16 = first.iter().map(|&c| c as u16).sum();
        let last_brightness: u16 = last.iter().map(|&c| c as u16).sum();
        assert!(first_brightness <= last_brightness);
    }

    #[test]
    fn light_text_readable_on_light_bg() {
        let l = ThemeColors::light_default();
        // Text should be substantially darker than background in light mode
        let text_brightness = l.text.r() as u16 + l.text.g() as u16 + l.text.b() as u16;
        let bg_brightness = l.background.r() as u16 + l.background.g() as u16 + l.background.b() as u16;
        assert!(bg_brightness > text_brightness + 200, "Light mode text must contrast with background");
    }

    #[test]
    fn dark_text_readable_on_dark_bg() {
        let d = ThemeColors::dark_default();
        // Text should be substantially brighter than background in dark mode
        let text_brightness = d.text.r() as u16 + d.text.g() as u16 + d.text.b() as u16;
        let bg_brightness = d.background.r() as u16 + d.background.g() as u16 + d.background.b() as u16;
        assert!(text_brightness > bg_brightness + 200, "Dark mode text must contrast with background");
    }
}
