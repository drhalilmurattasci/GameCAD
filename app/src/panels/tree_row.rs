//! Reusable tree-row rendering primitives.
//!
//! Shared building blocks for any tree UI: outliner scene tree, layer tree,
//! asset browser tree, etc.  Each function draws one small piece of a row so
//! callers can compose them freely inside a `ui.horizontal()` block.

use eframe::egui;
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};

/// Theme colors and layout constants for tree rows.
#[derive(Clone, Copy)]
pub(crate) struct TreeRowStyle {
    pub accent: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    /// Pixels of indentation per depth level.
    pub indent_px: f32,
}

// ---------------------------------------------------------------------------
// Indentation
// ---------------------------------------------------------------------------

/// Apply tree indentation for the given depth level.
pub(crate) fn draw_indent(ui: &mut egui::Ui, depth: usize, style: &TreeRowStyle) {
    ui.add_space(depth as f32 * style.indent_px + 2.0);
}

// ---------------------------------------------------------------------------
// Row background
// ---------------------------------------------------------------------------

/// Draw a highlighted background rect for the current row.
///
/// Call this at the very start of a `ui.horizontal()`, before any widgets.
/// `primary` uses full selection color, `false` uses a dimmer secondary.
pub(crate) fn draw_row_background(ui: &egui::Ui, active: bool, primary: bool, style: &TreeRowStyle) {
    if !active {
        return;
    }
    let row_rect = Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ui.available_width(), 20.0),
    );
    let color = if primary {
        style.accent.linear_multiply(0.15)
    } else {
        style.accent.linear_multiply(0.08)
    };
    ui.painter().rect_filled(row_rect, CornerRadius::same(2), color);
}

// ---------------------------------------------------------------------------
// Expand / collapse toggle
// ---------------------------------------------------------------------------

/// Draw a +/- toggle if the node has children, or a leaf branch line otherwise.
///
/// Returns `true` if the expand/collapse button was clicked.
pub(crate) fn draw_expand_toggle(
    ui: &mut egui::Ui,
    has_children: bool,
    expanded: bool,
    style: &TreeRowStyle,
) -> bool {
    if has_children {
        let icon = if expanded { "\u{229F}" } else { "\u{229E}" }; // ⊟ / ⊞
        let text = RichText::new(icon)
            .font(FontId::monospace(13.0))
            .color(style.text_dim);
        ui.add(egui::Button::new(text).frame(false))
            .on_hover_text(if expanded { "Collapse" } else { "Expand" })
            .clicked()
    } else {
        // Leaf node: horizontal branch line
        let tree_line_color = style.text_dim.linear_multiply(0.5);
        let (line_rect, _) = ui.allocate_exact_size(Vec2::new(13.0, 13.0), egui::Sense::hover());
        let mid_y = line_rect.center().y;
        ui.painter().line_segment(
            [Pos2::new(line_rect.left() + 2.0, mid_y), Pos2::new(line_rect.right() - 2.0, mid_y)],
            Stroke::new(1.0, tree_line_color),
        );
        false
    }
}

// ---------------------------------------------------------------------------
// Name button
// ---------------------------------------------------------------------------

/// Draw a frameless name button with accent color when active.
///
/// If `active`, also draws an accent stroke border around the name.
/// Returns the button's `Response` so the caller can handle clicks.
pub(crate) fn draw_name_button(
    ui: &mut egui::Ui,
    name: &str,
    active: bool,
    style: &TreeRowStyle,
) -> egui::Response {
    let color = if active { style.accent } else { style.text };
    let text = RichText::new(name)
        .font(FontId::proportional(12.0))
        .color(color);
    let resp = ui.add(egui::Button::new(text).frame(false));
    if active {
        ui.painter().rect_stroke(
            resp.rect.expand(1.0),
            CornerRadius::same(3),
            Stroke::new(1.0, style.accent.linear_multiply(0.4)),
            StrokeKind::Outside,
        );
    }
    resp
}

// ---------------------------------------------------------------------------
// Icon label
// ---------------------------------------------------------------------------

/// Draw an icon glyph colored by selection state.
pub(crate) fn draw_icon_label(ui: &mut egui::Ui, icon: &str, active: bool, style: &TreeRowStyle) {
    let color = if active { style.accent } else { style.text_dim };
    ui.label(RichText::new(icon).font(FontId::proportional(13.0)).color(color));
}

// ---------------------------------------------------------------------------
// Color swatch
// ---------------------------------------------------------------------------

/// Draw a small colored square, with accent border when active.
pub(crate) fn draw_color_swatch(ui: &mut egui::Ui, color: Color32, active: bool, style: &TreeRowStyle) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2, color);
    if active {
        ui.painter().rect_stroke(
            rect, 2,
            Stroke::new(1.5, style.accent),
            StrokeKind::Outside,
        );
    }
}

// ---------------------------------------------------------------------------
// Toggle button
// ---------------------------------------------------------------------------

/// Generic frameless toggle button (visibility eye, lock, etc.).
///
/// Flips `*state` on click and shows `tooltip` on hover.
pub(crate) fn draw_toggle_button(
    ui: &mut egui::Ui,
    on_icon: &str,
    off_icon: &str,
    state: &mut bool,
    tooltip: &str,
) {
    let icon = if *state { on_icon } else { off_icon };
    let text = RichText::new(icon).font(FontId::proportional(11.0));
    if ui.add(egui::Button::new(text).frame(false))
        .on_hover_text(tooltip)
        .clicked()
    {
        *state = !*state;
    }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

/// Draw a small `(N)` count badge in dim text. No-op when `count == 0`.
pub(crate) fn draw_badge(ui: &mut egui::Ui, count: usize, style: &TreeRowStyle) {
    if count > 0 {
        ui.label(
            RichText::new(format!("({})", count))
                .font(FontId::proportional(10.0))
                .color(style.text_dim),
        );
    }
}
