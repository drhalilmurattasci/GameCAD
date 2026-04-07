//! Shared data types for the Forge Editor.
//!
//! Enums for tabs, tool modes, render styles, and structs for the outliner
//! tree, agent tasks, and console log entries live here so they can be
//! imported across the crate with a single `use crate::types::*`.

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
            ToolMode::Move => "W",
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
    pub(crate) name: String,
    pub(crate) icon: &'static str,
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
