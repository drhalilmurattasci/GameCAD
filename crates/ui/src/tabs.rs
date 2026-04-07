//! Workspace tab bar with Fusion-style horizontal tabs and active underline.
//!
//! [`TabBar`] renders a horizontal row of [`WorkspaceTab`]s and returns the
//! newly selected tab when the user clicks one.

use egui::{Rect, Stroke, Ui, Vec2};

use crate::theme::ThemeColors;

/// The different workspace tabs available in Forge Editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WorkspaceTab {
    MapEditor,
    Gameplay,
    ObjectEditor,
    ScriptEditor,
    MaterialEditor,
    Animation,
    Physics,
}

impl WorkspaceTab {
    /// All tabs in display order.
    pub const ALL: &'static [WorkspaceTab] = &[
        WorkspaceTab::MapEditor,
        WorkspaceTab::Gameplay,
        WorkspaceTab::ObjectEditor,
        WorkspaceTab::ScriptEditor,
        WorkspaceTab::MaterialEditor,
        WorkspaceTab::Animation,
        WorkspaceTab::Physics,
    ];

    /// Human-readable label.
    #[inline]
    pub fn label(self) -> &'static str {
        match self {
            Self::MapEditor => "Map Editor",
            Self::Gameplay => "Gameplay",
            Self::ObjectEditor => "Object Editor",
            Self::ScriptEditor => "Script Editor",
            Self::MaterialEditor => "Material Editor",
            Self::Animation => "Animation",
            Self::Physics => "Physics",
        }
    }

    /// Icon character for each tab (Unicode symbols).
    #[inline]
    pub fn icon(self) -> &'static str {
        match self {
            Self::MapEditor => "\u{1F5FA}",      // world map
            Self::Gameplay => "\u{1F3AE}",        // game controller
            Self::ObjectEditor => "\u{1F4E6}",    // package / object
            Self::ScriptEditor => "\u{1F4DC}",    // scroll / script
            Self::MaterialEditor => "\u{1F3A8}",  // palette
            Self::Animation => "\u{1F3AC}",       // clapper board
            Self::Physics => "\u{2699}",          // gear
        }
    }

    /// Keyboard shortcut hint (Ctrl+1 through Ctrl+7).
    #[inline]
    pub fn shortcut_hint(self) -> &'static str {
        match self {
            Self::MapEditor => "Ctrl+1",
            Self::Gameplay => "Ctrl+2",
            Self::ObjectEditor => "Ctrl+3",
            Self::ScriptEditor => "Ctrl+4",
            Self::MaterialEditor => "Ctrl+5",
            Self::Animation => "Ctrl+6",
            Self::Physics => "Ctrl+7",
        }
    }
}

/// Horizontal tab bar for switching workspaces.
pub struct TabBar {
    active: WorkspaceTab,
}

impl TabBar {
    /// Creates a new tab bar with the given tab initially active.
    pub fn new(active: WorkspaceTab) -> Self {
        Self { active }
    }

    /// Returns the currently active tab.
    #[inline]
    pub fn active(&self) -> WorkspaceTab {
        self.active
    }

    /// Draw the tab bar. Returns `Some(tab)` if the user clicked a different tab.
    ///
    /// `colors` should be obtained from `ThemeManager::current_theme()` so that
    /// the tab bar adapts to both dark and light themes.
    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) -> Option<WorkspaceTab> {
        let mut clicked = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

            for &tab in WorkspaceTab::ALL {
                let is_active = tab == self.active;
                let text_color = if is_active {
                    colors.accent
                } else {
                    colors.text_dim
                };

                let response = ui.allocate_ui(Vec2::new(130.0, 32.0), |ui| {
                    let rect = ui.max_rect();

                    // Background hover highlight
                    if ui.rect_contains_pointer(rect) && !is_active {
                        ui.painter()
                            .rect_filled(rect, 0.0, colors.surface_raised);
                    }

                    // Icon + Label
                    let display_text = format!("{} {}", tab.icon(), tab.label());
                    let text_pos = rect.center()
                        - Vec2::new(
                            ui.fonts(|f| {
                                f.layout_no_wrap(
                                    display_text.clone(),
                                    egui::FontId::proportional(13.0),
                                    text_color,
                                )
                                .rect
                                .width()
                            }) / 2.0,
                            7.0,
                        );
                    ui.painter().text(
                        text_pos,
                        egui::Align2::LEFT_TOP,
                        &display_text,
                        egui::FontId::proportional(13.0),
                        text_color,
                    );

                    // Active underline
                    if is_active {
                        let underline_rect = Rect::from_min_size(
                            rect.left_bottom() - Vec2::new(0.0, 3.0),
                            Vec2::new(rect.width(), 3.0),
                        );
                        ui.painter().rect_filled(underline_rect, 1.5, colors.accent);
                    }

                    // Click detection with tooltip showing shortcut
                    let response = ui.interact(rect, ui.id().with(tab.label()), egui::Sense::click());
                    response.on_hover_text(format!("{} ({})", tab.label(), tab.shortcut_hint()))
                });

                if response.inner.clicked() && !is_active {
                    clicked = Some(tab);
                }
            }
        });

        // Separator line under the tab bar
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [
                egui::pos2(rect.left(), ui.min_rect().bottom()),
                egui::pos2(rect.right(), ui.min_rect().bottom()),
            ],
            Stroke::new(1.0, colors.border),
        );

        if let Some(tab) = clicked {
            self.active = tab;
        }

        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_tab_all_has_correct_count() {
        assert_eq!(WorkspaceTab::ALL.len(), 7);
    }

    #[test]
    fn workspace_tab_labels_non_empty() {
        for tab in WorkspaceTab::ALL {
            assert!(!tab.label().is_empty());
        }
    }

    #[test]
    fn workspace_tab_icons_non_empty() {
        for tab in WorkspaceTab::ALL {
            assert!(!tab.icon().is_empty());
        }
    }

    #[test]
    fn workspace_tab_shortcut_hints_non_empty() {
        for tab in WorkspaceTab::ALL {
            let hint = tab.shortcut_hint();
            assert!(!hint.is_empty());
            assert!(hint.starts_with("Ctrl+"), "Shortcut should start with Ctrl+, got: {hint}");
        }
    }

    #[test]
    fn workspace_tab_unique_labels() {
        let labels: Vec<&str> = WorkspaceTab::ALL.iter().map(|t| t.label()).collect();
        for (i, a) in labels.iter().enumerate() {
            for (j, b) in labels.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Tabs {i} and {j} have duplicate labels");
                }
            }
        }
    }

    #[test]
    fn workspace_tab_unique_shortcuts() {
        let hints: Vec<&str> = WorkspaceTab::ALL.iter().map(|t| t.shortcut_hint()).collect();
        for (i, a) in hints.iter().enumerate() {
            for (j, b) in hints.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Tabs {i} and {j} have duplicate shortcuts");
                }
            }
        }
    }

    #[test]
    fn tab_bar_active_default() {
        let bar = TabBar::new(WorkspaceTab::MapEditor);
        assert_eq!(bar.active(), WorkspaceTab::MapEditor);
    }
}
