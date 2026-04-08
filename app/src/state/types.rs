//! Shared data types for the Forge Editor.
//!
//! Enums for tabs, tool modes, render styles, and structs for the outliner
//! tree, agent tasks, and console log entries live here so they can be
//! imported across the crate with a single `use crate::state::types::*`.

use egui::Color32;

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

/// Top-level editor workspace tabs (Map Editor, Gameplay, etc.).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MainTab {
    MapEditor,
    Gameplay,
    ObjectEditor,
    ScriptEditor,
    MaterialEditor,
    Animation,
    Physics,
}

impl MainTab {
    /// All tab variants in display order.
    pub(crate) const ALL: [MainTab; 7] = [
        MainTab::MapEditor,
        MainTab::Gameplay,
        MainTab::ObjectEditor,
        MainTab::ScriptEditor,
        MainTab::MaterialEditor,
        MainTab::Animation,
        MainTab::Physics,
    ];

    /// Human-readable display label.
    pub(crate) fn label(self) -> &'static str {
        match self {
            MainTab::MapEditor => "Map Editor",
            MainTab::Gameplay => "Gameplay",
            MainTab::ObjectEditor => "Object Editor",
            MainTab::ScriptEditor => "Script Editor",
            MainTab::MaterialEditor => "Material Editor",
            MainTab::Animation => "Animation",
            MainTab::Physics => "Physics",
        }
    }
}

// ---------------------------------------------------------------------------
// Tool mode
// ---------------------------------------------------------------------------

/// Active transform / interaction tool in the viewport.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ToolMode {
    Select,
    Move,
    Rotate,
    Scale,
}

impl ToolMode {
    /// Human-readable display label.
    pub(crate) fn label(self) -> &'static str {
        match self {
            ToolMode::Select => "Select",
            ToolMode::Move => "Move",
            ToolMode::Rotate => "Rotate",
            ToolMode::Scale => "Scale",
        }
    }

    /// Keyboard shortcut letter for this tool.
    pub(crate) fn shortcut(self) -> &'static str {
        match self {
            ToolMode::Select => "Q",
            ToolMode::Move => "M",
            ToolMode::Rotate => "E",
            ToolMode::Scale => "R",
        }
    }
}

// ---------------------------------------------------------------------------
// Render style
// ---------------------------------------------------------------------------

/// Viewport shading / render style.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RenderStyle {
    Shaded,
    Wireframe,
    ShadedWireframe,
    Unlit,
    Ghost,
    Normals,
    Depth,
    Clay,
}

impl RenderStyle {
    pub(crate) const ALL: [RenderStyle; 8] = [
        RenderStyle::Shaded,
        RenderStyle::Wireframe,
        RenderStyle::ShadedWireframe,
        RenderStyle::Unlit,
        RenderStyle::Ghost,
        RenderStyle::Normals,
        RenderStyle::Depth,
        RenderStyle::Clay,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            RenderStyle::Shaded => "Shaded",
            RenderStyle::Wireframe => "Wireframe",
            RenderStyle::ShadedWireframe => "Shaded + Wire",
            RenderStyle::Unlit => "Unlit",
            RenderStyle::Ghost => "Ghost / X-Ray",
            RenderStyle::Normals => "Normals",
            RenderStyle::Depth => "Depth",
            RenderStyle::Clay => "Clay",
        }
    }

    /// Cycle to the next render style (wraps around).
    pub(crate) fn next(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|&s| s == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

// ---------------------------------------------------------------------------
// Bottom panel tab
// ---------------------------------------------------------------------------

/// Tabs in the bottom panel (Content Browser, Console, etc.).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BottomTab {
    ContentBrowser,
    Console,
    AgentProgress,
    Timeline,
}

impl BottomTab {
    pub(crate) const ALL: [BottomTab; 4] = [
        BottomTab::ContentBrowser,
        BottomTab::Console,
        BottomTab::AgentProgress,
        BottomTab::Timeline,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            BottomTab::ContentBrowser => "Content Browser",
            BottomTab::Console => "Console",
            BottomTab::AgentProgress => "Agent Progress",
            BottomTab::Timeline => "Timeline",
        }
    }
}

// ---------------------------------------------------------------------------
// Outliner node
// ---------------------------------------------------------------------------

/// A node in the outliner scene-graph tree.
#[derive(Clone)]
pub(crate) struct OutlinerNode {
    pub(crate) id: forge_core::id::NodeId,
    pub(crate) name: String,
    pub(crate) icon: &'static str,
    pub(crate) expanded: bool,
    pub(crate) children: Vec<OutlinerNode>,
}

// ---------------------------------------------------------------------------
// Agent task
// ---------------------------------------------------------------------------

/// A background agent task shown in the Agent Progress panel.
#[derive(Clone)]
pub(crate) struct AgentTask {
    pub(crate) name: String,
    pub(crate) progress: f32, // 0.0 .. 1.0
    pub(crate) status: TaskStatus,
}

/// Lifecycle state of an agent task.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TaskStatus {
    Queued,
    Running,
    Complete,
    Failed,
}

impl TaskStatus {
    pub(crate) fn label(self) -> &'static str {
        match self {
            TaskStatus::Queued => "Queued",
            TaskStatus::Running => "Running",
            TaskStatus::Complete => "\u{2713}",
            TaskStatus::Failed => "\u{2717}",
        }
    }

    pub(crate) fn color(self) -> Color32 {
        match self {
            TaskStatus::Queued => Color32::from_rgb(0x9b, 0x9b, 0xa1),
            TaskStatus::Running => Color32::from_rgb(0x4e, 0xff, 0x93),
            TaskStatus::Complete => Color32::from_rgb(0x2e, 0xcc, 0x71),
            TaskStatus::Failed => Color32::from_rgb(0xe7, 0x4c, 0x3c),
        }
    }
}

// ---------------------------------------------------------------------------
// Console log entry
// ---------------------------------------------------------------------------

/// A single console log entry.
#[derive(Clone)]
pub(crate) struct LogEntry {
    pub(crate) level: LogLevel,
    pub(crate) message: String,
}

/// Severity level for console log entries.
#[derive(Clone, Copy)]
pub(crate) enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub(crate) fn color(self) -> Color32 {
        match self {
            LogLevel::Info => Color32::from_rgb(0x9b, 0x9b, 0xa1),
            LogLevel::Warn => Color32::from_rgb(0xf3, 0x9c, 0x12),
            LogLevel::Error => Color32::from_rgb(0xe7, 0x4c, 0x3c),
        }
    }

    pub(crate) fn prefix(self) -> &'static str {
        match self {
            LogLevel::Info => "[INFO]",
            LogLevel::Warn => "[WARN]",
            LogLevel::Error => "[ERR ]",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_tab_all_contains_all_variants() {
        assert_eq!(MainTab::ALL.len(), 7);
        assert_eq!(MainTab::ALL[0], MainTab::MapEditor);
        assert_eq!(MainTab::ALL[6], MainTab::Physics);
    }

    #[test]
    fn main_tab_labels_non_empty() {
        for tab in &MainTab::ALL {
            assert!(!tab.label().is_empty(), "Tab {:?} has empty label", tab);
        }
    }

    #[test]
    fn tool_mode_labels_and_shortcuts() {
        let tools = [ToolMode::Select, ToolMode::Move, ToolMode::Rotate, ToolMode::Scale];
        for tool in &tools {
            assert!(!tool.label().is_empty());
            assert!(!tool.shortcut().is_empty());
        }
        assert_eq!(ToolMode::Select.shortcut(), "Q");
        assert_eq!(ToolMode::Move.shortcut(), "M");
        assert_eq!(ToolMode::Rotate.shortcut(), "E");
        assert_eq!(ToolMode::Scale.shortcut(), "R");
    }

    #[test]
    fn render_style_all_contains_8_variants() {
        assert_eq!(RenderStyle::ALL.len(), 8);
    }

    #[test]
    fn render_style_next_cycles() {
        let first = RenderStyle::Shaded;
        assert_eq!(first.next(), RenderStyle::Wireframe);
        assert_eq!(RenderStyle::Clay.next(), RenderStyle::Shaded); // wraps
    }

    #[test]
    fn render_style_next_full_cycle() {
        let mut style = RenderStyle::Shaded;
        for _ in 0..8 {
            style = style.next();
        }
        assert_eq!(style, RenderStyle::Shaded); // full cycle
    }

    #[test]
    fn render_style_labels_non_empty() {
        for rs in &RenderStyle::ALL {
            assert!(!rs.label().is_empty());
        }
    }

    #[test]
    fn bottom_tab_all_contains_4() {
        assert_eq!(BottomTab::ALL.len(), 4);
    }

    #[test]
    fn bottom_tab_labels_non_empty() {
        for tab in &BottomTab::ALL {
            assert!(!tab.label().is_empty());
        }
    }

    #[test]
    fn task_status_label_and_color() {
        let statuses = [TaskStatus::Queued, TaskStatus::Running, TaskStatus::Complete, TaskStatus::Failed];
        for status in &statuses {
            assert!(!status.label().is_empty());
            // Color should not be transparent (alpha > 0)
            let c = status.color();
            assert_ne!(c, Color32::TRANSPARENT);
        }
    }

    #[test]
    fn log_level_prefix_format() {
        assert_eq!(LogLevel::Info.prefix(), "[INFO]");
        assert_eq!(LogLevel::Warn.prefix(), "[WARN]");
        assert_eq!(LogLevel::Error.prefix(), "[ERR ]");
    }

    #[test]
    fn log_level_colors_distinct() {
        let info_c = LogLevel::Info.color();
        let warn_c = LogLevel::Warn.color();
        let err_c = LogLevel::Error.color();
        assert_ne!(info_c, warn_c);
        assert_ne!(warn_c, err_c);
        assert_ne!(info_c, err_c);
    }

    #[test]
    fn outliner_node_clone() {
        let node = OutlinerNode {
            id: forge_core::id::NodeId::new(),
            name: "Test".into(),
            icon: "\u{25A6}",
            expanded: true,
            children: vec![OutlinerNode {
                id: forge_core::id::NodeId::new(),
                name: "Child".into(),
                icon: "\u{25CB}",
                expanded: true,
                children: vec![],
            }],
        };
        let cloned = node.clone();
        assert_eq!(cloned.name, "Test");
        assert_eq!(cloned.children.len(), 1);
        assert_eq!(cloned.children[0].name, "Child");
    }

    #[test]
    fn agent_task_progress_bounds() {
        let task = AgentTask {
            name: "Test".into(),
            progress: 0.5,
            status: TaskStatus::Running,
        };
        assert!(task.progress >= 0.0 && task.progress <= 1.0);
    }
}
