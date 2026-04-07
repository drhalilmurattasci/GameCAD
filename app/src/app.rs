//! Core application state and the eframe update loop.
//!
//! [`ForgeEditorApp`] owns all editor state: outliner tree, entity transforms,
//! selection, camera, tasks, console log, and theme.  The `eframe::App::update`
//! implementation drives task animation, keyboard shortcuts, layout, and the
//! command-palette overlay each frame.

use eframe::egui;
use egui::Pos2;
use forge_ui::theme::ThemeManager;
use forge_viewport::camera::OrbitCamera;
use glam::Vec3;

use crate::settings::EditorSettings;
use crate::types::*;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

/// Top-level application struct holding all editor state.
pub(crate) struct ForgeEditorApp {
    pub(crate) active_tab: MainTab,
    pub(crate) tool_mode: ToolMode,
    pub(crate) render_style: RenderStyle,
    pub(crate) bottom_tab: BottomTab,
    pub(crate) selected_entity: usize, // index into flattened outliner
    pub(crate) outliner: Vec<OutlinerNode>,
    pub(crate) tasks: Vec<AgentTask>,
    pub(crate) console_log: Vec<LogEntry>,
    pub(crate) show_command_palette: bool,
    pub(crate) command_query: String,
    // Inspector drag-value state
    pub(crate) transforms: Vec<[f32; 9]>, // pos xyz, rot xyz, scale xyz per entity
    // Editable component properties
    pub(crate) light_intensity: f32,
    pub(crate) camera_fov: f32,
    pub(crate) camera_near: f32,
    pub(crate) camera_far: f32,
    pub(crate) frame_count: u64,
    // 3D orbit camera for the viewport
    pub(crate) orbit_camera: OrbitCamera,
    pub(crate) is_orbiting: bool,
    pub(crate) is_panning: bool,
    // Right-click context menu tracking
    pub(crate) right_click_start_pos: Option<Pos2>,
    // Selection state
    pub(crate) selected_entities: Vec<usize>,
    // Box selection (S key held + left-drag)
    pub(crate) box_select_start: Option<Pos2>,
    pub(crate) box_select_end: Option<Pos2>,
    pub(crate) box_select_key_held: bool,
    // Theme manager (dark/light toggle)
    pub(crate) theme_manager: ThemeManager,
    // Centralized editor settings (grid, snap, camera, tools, viewport, height, selection)
    pub(crate) settings: EditorSettings,
    // Layers (color-coded, layer 0 = black base)
    pub(crate) layers: Vec<EditorLayer>,
    pub(crate) active_layer: usize,
}

/// A named, color-coded editor layer.
#[derive(Clone, Debug)]
pub(crate) struct EditorLayer {
    pub(crate) name: String,
    pub(crate) color: egui::Color32,
    pub(crate) visible: bool,
    pub(crate) locked: bool,
}

impl Default for ForgeEditorApp {
    fn default() -> Self {
        let outliner = vec![OutlinerNode {
            name: "Scene Root".into(),
            icon: "\u{1F5C2}",
            children: vec![
                OutlinerNode {
                    name: "Cube".into(),
                    icon: "\u{25A6}",
                    children: vec![],
                },
                OutlinerNode {
                    name: "Sphere".into(),
                    icon: "\u{25A6}",
                    children: vec![],
                },
                OutlinerNode {
                    name: "Directional Light".into(),
                    icon: "\u{2600}",
                    children: vec![],
                },
                OutlinerNode {
                    name: "Main Camera".into(),
                    icon: "\u{1F3A5}",
                    children: vec![],
                },
            ],
        }];

        let transforms = vec![
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],   // Scene Root
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],    // Cube
            [3.0, 1.5, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],   // Sphere
            [5.0, 8.0, 2.0, -45.0, 30.0, 0.0, 1.0, 1.0, 1.0], // Light
            [0.0, 5.0, -10.0, 25.0, 0.0, 0.0, 1.0, 1.0, 1.0], // Camera
        ];

        let tasks = vec![
            AgentTask {
                name: "Loading Assets".into(),
                progress: 1.0,
                status: TaskStatus::Complete,
            },
            AgentTask {
                name: "Building Lighting".into(),
                progress: 0.67,
                status: TaskStatus::Running,
            },
            AgentTask {
                name: "Compiling Shaders".into(),
                progress: 0.15,
                status: TaskStatus::Running,
            },
            AgentTask {
                name: "Generating Thumbnails".into(),
                progress: 0.0,
                status: TaskStatus::Queued,
            },
            AgentTask {
                name: "Lightmap Bake".into(),
                progress: 0.45,
                status: TaskStatus::Failed,
            },
        ];

        let console_log = vec![
            LogEntry {
                level: LogLevel::Info,
                message: "Forge Editor v0.1.0 initialized".into(),
            },
            LogEntry {
                level: LogLevel::Info,
                message: "Loaded project: UntitledProject".into(),
            },
            LogEntry {
                level: LogLevel::Warn,
                message: "Texture 'ground_albedo.png' not found, using fallback".into(),
            },
            LogEntry {
                level: LogLevel::Info,
                message: "Scene 'Main' loaded with 4 entities".into(),
            },
            LogEntry {
                level: LogLevel::Error,
                message: "Shader compilation warning in 'pbr_frag.glsl' line 42".into(),
            },
            LogEntry {
                level: LogLevel::Info,
                message: "Agent pipeline started: 4 tasks queued".into(),
            },
            LogEntry {
                level: LogLevel::Info,
                message: "Asset loading complete (12 assets)".into(),
            },
        ];

        Self {
            active_tab: MainTab::MapEditor,
            tool_mode: ToolMode::Select,
            render_style: RenderStyle::Shaded,
            bottom_tab: BottomTab::AgentProgress,
            selected_entity: 1, // Cube
            outliner,
            tasks,
            console_log,
            show_command_palette: false,
            command_query: String::new(),
            transforms,
            light_intensity: 1.0,
            camera_fov: 60.0,
            camera_near: 0.1,
            camera_far: 1000.0,
            frame_count: 0,
            orbit_camera: {
                let mut cam = OrbitCamera::new(Vec3::ZERO, 15.0);
                cam.yaw = 0.5; // ~30 degrees
                cam.pitch = 0.35; // ~20 degrees
                cam
            },
            is_orbiting: false,
            is_panning: false,
            right_click_start_pos: None,
            selected_entities: vec![1],
            box_select_start: None,
            box_select_end: None,
            box_select_key_held: false,
            theme_manager: ThemeManager::new(),
            settings: EditorSettings::default(),
            layers: vec![
                EditorLayer { name: "Base".into(), color: egui::Color32::from_rgb(0, 0, 0), visible: true, locked: false },
                EditorLayer { name: "Environment".into(), color: egui::Color32::from_rgb(0x2e, 0xcc, 0x71), visible: true, locked: false },
                EditorLayer { name: "Characters".into(), color: egui::Color32::from_rgb(0x3e, 0x55, 0xff), visible: true, locked: false },
                EditorLayer { name: "Lights".into(), color: egui::Color32::from_rgb(0xff, 0xd7, 0x00), visible: true, locked: false },
                EditorLayer { name: "Effects".into(), color: egui::Color32::from_rgb(0xe9, 0x45, 0x60), visible: true, locked: false },
                EditorLayer { name: "UI".into(), color: egui::Color32::from_rgb(0x9b, 0x59, 0xb6), visible: true, locked: false },
            ],
            active_layer: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// eframe impl
// ---------------------------------------------------------------------------

impl eframe::App for ForgeEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme (dark or light) from ThemeManager
        self.theme_manager.apply_to_egui(ctx);
        self.frame_count += 1;

        // Animate agent tasks
        if let Some(t) = self.tasks.get_mut(1)
            && t.status == TaskStatus::Running
        {
            t.progress = (t.progress + 0.001).min(1.0);
            if t.progress >= 1.0 {
                t.status = TaskStatus::Complete;
            }
        }
        if let Some(t) = self.tasks.get_mut(2)
            && t.status == TaskStatus::Running
        {
            t.progress = (t.progress + 0.0005).min(1.0);
            if t.progress >= 1.0 {
                t.status = TaskStatus::Complete;
            }
        }
        // Start thumbnails once shaders > 50%
        if self.tasks.get(2).is_some_and(|t| t.progress > 0.5)
            && let Some(t) = self.tasks.get_mut(3)
        {
            if t.status == TaskStatus::Queued {
                t.status = TaskStatus::Running;
            }
            if t.status == TaskStatus::Running {
                t.progress = (t.progress + 0.0008).min(1.0);
                if t.progress >= 1.0 {
                    t.status = TaskStatus::Complete;
                }
            }
        }

        // Keyboard shortcuts (only when palette is closed)
        if !self.show_command_palette {
            self.handle_shortcuts(ctx);
        }

        // Request continuous repaint for animations
        ctx.request_repaint();

        // -- Layout --
        self.draw_tab_bar(ctx);
        self.draw_toolbar(ctx);
        self.draw_status_bar(ctx);
        self.draw_bottom_panel(ctx);
        self.draw_left_panel(ctx);
        self.draw_right_panel(ctx);
        self.draw_viewport(ctx);

        // Command palette overlay
        if self.show_command_palette {
            self.draw_command_palette(ctx);
        }
    }
}
