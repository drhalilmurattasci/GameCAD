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
    fn tab_bar_active_default() {
        let bar = TabBar::new(WorkspaceTab::MapEditor);
        assert_eq!(bar.active(), WorkspaceTab::MapEditor);
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
    pub fn active(&self) -> WorkspaceTab {
        self.active
    }

    /// Draw the tab bar. Returns `Some(tab)` if the user clicked a different tab.
    pub fn show(&mut self, ui: &mut Ui) -> Option<WorkspaceTab> {
        let colors = ThemeColors::dark_default();
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

                let response = ui.allocate_ui(Vec2::new(120.0, 32.0), |ui| {
                    let rect = ui.max_rect();

                    // Background hover highlight
                    if ui.rect_contains_pointer(rect) && !is_active {
                        ui.painter()
                            .rect_filled(rect, 0.0, colors.surface_raised);
                    }

                    // Label
                    let text_pos = rect.center()
                        - Vec2::new(
                            ui.fonts(|f| {
                                f.layout_no_wrap(
                                    tab.label().to_owned(),
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
                        tab.label(),
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

                    // Click detection
                    ui.interact(rect, ui.id().with(tab.label()), egui::Sense::click())
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
