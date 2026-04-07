//! # Forge Editor -- UI
//!
//! Crystalline-themed egui widgets for the Forge Editor. Includes:
//!
//! - [`theme`] -- dark/light theme engine with viewport gradients
//! - [`toolbar`] -- horizontal toolbar with icon buttons
//! - [`tabs`] -- workspace tab bar
//! - [`panels`] -- dockable editor panel system
//! - [`command_palette`] -- fuzzy-search command palette overlay
//! - [`shortcuts`] -- keyboard shortcut mapping
//! - [`status_bar`] -- bottom status bar
//! - [`agent_progress`] -- AI agent task progress display

pub mod agent_progress;
pub mod command_palette;
pub mod panels;
pub mod shortcuts;
pub mod status_bar;
pub mod tabs;
pub mod theme;
pub mod toolbar;

/// Convenience re-exports.
pub mod prelude {
    pub use crate::agent_progress::{AgentTask, TaskStatus, show_task_list};
    pub use crate::command_palette::{CommandPalette, PaletteCommand};
    pub use crate::panels::{EditorPanel, PanelId, PanelManager};
    pub use crate::shortcuts::{Shortcut, ShortcutMap};
    pub use crate::status_bar::{StatusBar, StatusBarState};
    pub use crate::tabs::{TabBar, WorkspaceTab};
    pub use crate::theme::{ThemeColors, ThemeManager, ThemeMode, apply_to_egui, hex_to_color32};
    pub use crate::toolbar::{Toolbar, ToolbarButton};
}
