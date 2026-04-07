//! Dockable panel system for the editor.
//!
//! [`PanelManager`] holds registered [`EditorPanel`] implementations, tracks
//! their visibility, and calls `update` + `ui` on each visible panel every frame.

use egui::{Context, Ui};
use indexmap::IndexMap;

/// Unique identifier for an editor panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PanelId {
    SceneHierarchy,
    Inspector,
    AssetBrowser,
    Console,
    Viewport,
    Properties,
    Timeline,
    AgentChat,
    CodeEditor,
}

impl PanelId {
    pub fn label(self) -> &'static str {
        match self {
            Self::SceneHierarchy => "Scene Hierarchy",
            Self::Inspector => "Inspector",
            Self::AssetBrowser => "Asset Browser",
            Self::Console => "Console",
            Self::Viewport => "Viewport",
            Self::Properties => "Properties",
            Self::Timeline => "Timeline",
            Self::AgentChat => "Agent Chat",
            Self::CodeEditor => "Code Editor",
        }
    }
}

/// Trait that every editor panel must implement.
pub trait EditorPanel {
    /// The panel's unique id.
    fn id(&self) -> PanelId;

    /// Human-readable title for the panel tab.
    fn title(&self) -> &str;

    /// Draw the panel contents.
    fn ui(&mut self, ui: &mut Ui);

    /// Whether the panel is currently visible.
    fn is_visible(&self) -> bool {
        true
    }

    /// Called once per frame before `ui()` to let the panel update internal state.
    fn update(&mut self, _ctx: &Context) {}
}

/// Manages the set of registered panels.
pub struct PanelManager {
    panels: IndexMap<PanelId, Box<dyn EditorPanel>>,
    visible: IndexMap<PanelId, bool>,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: IndexMap::new(),
            visible: IndexMap::new(),
        }
    }

    /// Register a panel. Replaces any existing panel with the same id.
    pub fn register(&mut self, panel: Box<dyn EditorPanel>) {
        let id = panel.id();
        self.visible.insert(id, true);
        self.panels.insert(id, panel);
    }

    /// Toggle visibility of a panel.
    pub fn set_visible(&mut self, id: PanelId, visible: bool) {
        self.visible.insert(id, visible);
    }

    /// Check if a panel is visible.
    pub fn is_visible(&self, id: PanelId) -> bool {
        self.visible.get(&id).copied().unwrap_or(false)
    }

    /// Iterate over all visible panels, calling `update` then `ui` on each.
    pub fn show_all(&mut self, ctx: &Context, ui: &mut Ui) {
        for (id, panel) in self.panels.iter_mut() {
            if self.visible.get(id).copied().unwrap_or(true) {
                panel.update(ctx);
                panel.ui(ui);
            }
        }
    }

    /// Get a mutable reference to a panel by id.
    pub fn get_mut(&mut self, id: PanelId) -> Option<&mut Box<dyn EditorPanel>> {
        self.panels.get_mut(&id)
    }

    /// List all registered panel ids.
    pub fn panel_ids(&self) -> Vec<PanelId> {
        self.panels.keys().copied().collect()
    }
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_id_labels_non_empty() {
        let ids = [
            PanelId::SceneHierarchy,
            PanelId::Inspector,
            PanelId::AssetBrowser,
            PanelId::Console,
            PanelId::Viewport,
            PanelId::Properties,
            PanelId::Timeline,
            PanelId::AgentChat,
            PanelId::CodeEditor,
        ];
        for id in ids {
            assert!(!id.label().is_empty());
        }
    }

    #[test]
    fn panel_manager_visibility_defaults_false_for_unregistered() {
        let mgr = PanelManager::new();
        assert!(!mgr.is_visible(PanelId::Console));
    }

    #[test]
    fn panel_manager_set_visible() {
        let mut mgr = PanelManager::new();
        mgr.set_visible(PanelId::Console, true);
        assert!(mgr.is_visible(PanelId::Console));
        mgr.set_visible(PanelId::Console, false);
        assert!(!mgr.is_visible(PanelId::Console));
    }
}
