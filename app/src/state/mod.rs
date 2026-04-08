//! Core application state and the eframe update loop.
//!
//! [`ForgeEditorApp`] owns all editor state: outliner tree, entity transforms,
//! selection, camera, tasks, console log, and theme.  The `eframe::App::update`
//! implementation drives task animation, keyboard shortcuts, layout, and the
//! command-palette overlay each frame.

use eframe::egui;
use egui::Pos2;
use forge_core::commands::CommandHistory;
use forge_core::ecs::World;
use forge_core::events::EventBus;
use forge_ui::theme::ThemeManager;
use forge_viewport::camera::OrbitCamera;
use glam::Vec3;

use crate::state::settings::EditorSettings;
use crate::state::types::*;

pub(crate) mod entity;
pub(crate) mod settings;
pub(crate) mod types;

// ─────────────────────────────────────────────────────────────────────
// GPU Viewport State
// ─────────────────────────────────────────────────────────────────────

/// GPU rendering state for the 3D viewport.
pub(crate) struct GpuViewportState {
    pub renderer: forge_render::Renderer,
    pub gpu_meshes: Vec<forge_render::GpuMesh>,
    pub offscreen_texture: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
    pub viewport_size: (u32, u32),
    pub device: std::sync::Arc<wgpu::Device>,
    pub queue: std::sync::Arc<wgpu::Queue>,
}

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
    pub(crate) active_layer: Vec<usize>, // path into layer tree, e.g. [2] or [2, 0]
    // Per-frame caches for layer visibility and lock
    pub(crate) cached_hidden_ids: std::collections::HashSet<forge_core::id::NodeId>,
    pub(crate) cached_locked_ids: std::collections::HashSet<forge_core::id::NodeId>,
    pub(crate) cached_flat_ids: Vec<forge_core::id::NodeId>,
    pub(crate) cached_hidden_frame: u64,
    // Core systems
    pub(crate) world: World,
    pub(crate) event_bus: EventBus,
    pub(crate) command_history: CommandHistory,
    // Mesh storage for modeling integration
    pub(crate) meshes: std::collections::HashMap<forge_core::id::NodeId, forge_modeling::half_edge::EditMesh>,
    // Material library
    pub(crate) material_library: forge_materials::library::MaterialLibrary,
    // Asset database
    pub(crate) asset_db: forge_assets::database::AssetDatabase,
    // Scene graph
    pub(crate) scene: forge_scene::graph::SceneGraph,
    // GPU rendering state (None in tests / fallback to painter)
    pub(crate) gpu: Option<GpuViewportState>,
}

/// A named, color-coded editor layer with optional sublayers.
#[derive(Clone, Debug)]
pub(crate) struct EditorLayer {
    pub(crate) name: String,
    pub(crate) color: egui::Color32,
    pub(crate) visible: bool,
    pub(crate) locked: bool,
    pub(crate) expanded: bool,
    /// Sublayers nested under this layer.
    pub(crate) children: Vec<EditorLayer>,
    /// Stable entity IDs assigned to this layer (immune to index shifts).
    pub(crate) entity_ids: std::collections::HashSet<forge_core::id::NodeId>,
}

impl EditorLayer {
    /// Create a new layer with the given name and color.
    pub(crate) fn new(name: impl Into<String>, color: egui::Color32) -> Self {
        Self {
            name: name.into(),
            color,
            visible: true,
            locked: false,
            expanded: true,
            children: Vec::new(),
            entity_ids: std::collections::HashSet::new(),
        }
    }

    /// Add a sublayer under this layer.
    pub(crate) fn add_sublayer(&mut self, sublayer: EditorLayer) {
        self.children.push(sublayer);
    }

    /// Total entity count including sublayers.
    pub(crate) fn total_entity_count(&self) -> usize {
        let mut count = self.entity_ids.len();
        for child in &self.children {
            count += child.total_entity_count();
        }
        count
    }
}

impl ForgeEditorApp {
    /// Rebuild the per-frame hidden entity cache. Call once at top of update().
    pub(crate) fn rebuild_hidden_cache(&mut self) {
        if self.cached_hidden_frame == self.frame_count {
            return;
        }
        // Rebuild flat ID list
        self.cached_flat_ids.clear();
        fn collect_ids(node: &OutlinerNode, ids: &mut Vec<forge_core::id::NodeId>) {
            ids.push(node.id);
            for child in &node.children {
                collect_ids(child, ids);
            }
        }
        for root in &self.outliner {
            collect_ids(root, &mut self.cached_flat_ids);
        }

        // Rebuild hidden ID set
        self.cached_hidden_ids.clear();
        fn collect_hidden(
            layer: &EditorLayer,
            hidden: &mut std::collections::HashSet<forge_core::id::NodeId>,
        ) {
            if !layer.visible {
                hidden.extend(&layer.entity_ids);
                for child in &layer.children {
                    collect_all(child, hidden);
                }
            } else {
                for child in &layer.children {
                    collect_hidden(child, hidden);
                }
            }
        }
        fn collect_all(
            layer: &EditorLayer,
            hidden: &mut std::collections::HashSet<forge_core::id::NodeId>,
        ) {
            hidden.extend(&layer.entity_ids);
            for child in &layer.children {
                collect_all(child, hidden);
            }
        }
        for layer in &self.layers {
            collect_hidden(layer, &mut self.cached_hidden_ids);
        }

        // Rebuild locked ID set
        self.cached_locked_ids.clear();
        fn collect_locked(
            layer: &EditorLayer,
            locked: &mut std::collections::HashSet<forge_core::id::NodeId>,
        ) {
            if layer.locked {
                locked.extend(&layer.entity_ids);
                for child in &layer.children {
                    collect_all_locked(child, locked);
                }
            } else {
                for child in &layer.children {
                    collect_locked(child, locked);
                }
            }
        }
        fn collect_all_locked(
            layer: &EditorLayer,
            locked: &mut std::collections::HashSet<forge_core::id::NodeId>,
        ) {
            locked.extend(&layer.entity_ids);
            for child in &layer.children {
                collect_all_locked(child, locked);
            }
        }
        for layer in &self.layers {
            collect_locked(layer, &mut self.cached_locked_ids);
        }

        self.cached_hidden_frame = self.frame_count;
    }

    /// Check if a flat entity index is hidden by layer visibility. O(1) per call.
    pub(crate) fn is_entity_hidden(&self, idx: usize) -> bool {
        self.cached_flat_ids
            .get(idx)
            .is_some_and(|id| self.cached_hidden_ids.contains(id))
    }

    /// Check if a flat entity index is locked by its layer. O(1) per call.
    pub(crate) fn is_entity_locked(&self, idx: usize) -> bool {
        self.cached_flat_ids
            .get(idx)
            .is_some_and(|id| self.cached_locked_ids.contains(id))
    }

    /// Resolve the active layer path to an immutable reference.
    pub(crate) fn active_layer_ref(&self) -> Option<&EditorLayer> {
        let mut current: &[EditorLayer] = &self.layers;
        let mut result: Option<&EditorLayer> = None;
        for &i in &self.active_layer {
            let layer = current.get(i)?;
            result = Some(layer);
            current = &layer.children;
        }
        result
    }

    /// Resolve the active layer path to a mutable reference.
    pub(crate) fn active_layer_mut(&mut self) -> Option<&mut EditorLayer> {
        let mut current: &mut [EditorLayer] = &mut self.layers;
        for &i in &self.active_layer.clone() {
            let layer = current.get_mut(i)?;
            current = std::slice::from_mut(layer);
        }
        current.first_mut()
    }

    /// Remove a NodeId from all layers (recursively including sublayers).
    pub(crate) fn remove_id_from_all_layers(layers: &mut [EditorLayer], id: forge_core::id::NodeId) {
        for layer in layers.iter_mut() {
            layer.entity_ids.remove(&id);
            Self::remove_id_from_all_layers(&mut layer.children, id);
        }
    }

    /// Find which layer contains `original` and insert `new_id` there too.
    pub(crate) fn add_to_same_layer(
        layers: &mut [EditorLayer],
        original: forge_core::id::NodeId,
        new_id: forge_core::id::NodeId,
    ) {
        for layer in layers.iter_mut() {
            if layer.entity_ids.contains(&original) {
                layer.entity_ids.insert(new_id);
                return;
            }
            Self::add_to_same_layer(&mut layer.children, original, new_id);
        }
    }
}

impl Default for ForgeEditorApp {
    fn default() -> Self {
        use forge_core::id::NodeId;

        let root_id = NodeId::new();
        let cube_id = NodeId::new();
        let sphere_id = NodeId::new();
        let light_id = NodeId::new();
        let camera_id = NodeId::new();

        let outliner = vec![OutlinerNode {
            id: root_id,
            name: "Scene Root".into(),
            icon: "\u{1F5C2}",
            expanded: true,
            children: vec![
                OutlinerNode {
                    id: cube_id,
                    name: "Cube".into(),
                    icon: "\u{25A6}",
                    expanded: true,
                    children: vec![],
                },
                OutlinerNode {
                    id: sphere_id,
                    name: "Sphere".into(),
                    icon: "\u{25A6}",
                    expanded: true,
                    children: vec![],
                },
                OutlinerNode {
                    id: light_id,
                    name: "Directional Light".into(),
                    icon: "\u{2600}",
                    expanded: true,
                    children: vec![],
                },
                OutlinerNode {
                    id: camera_id,
                    name: "Main Camera".into(),
                    icon: "\u{1F3A5}",
                    expanded: true,
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
            layers: {
                let mut objects_layer = EditorLayer::new("Objects", egui::Color32::from_rgb(0xff, 0x8c, 0x00));
                objects_layer.entity_ids.insert(cube_id);
                objects_layer.entity_ids.insert(sphere_id);
                let mut lights_layer = EditorLayer::new("Lights", egui::Color32::from_rgb(0xff, 0xd7, 0x00));
                lights_layer.entity_ids.insert(light_id);
                lights_layer.entity_ids.insert(camera_id);
                vec![
                    EditorLayer::new("Base", egui::Color32::from_rgb(0, 0, 0)),
                    EditorLayer::new("Environment", egui::Color32::from_rgb(0x2e, 0xcc, 0x71)),
                    objects_layer,
                    EditorLayer::new("Characters", egui::Color32::from_rgb(0x3e, 0x55, 0xff)),
                    lights_layer,
                    EditorLayer::new("Effects", egui::Color32::from_rgb(0xe9, 0x45, 0x60)),
                    EditorLayer::new("UI", egui::Color32::from_rgb(0x9b, 0x59, 0xb6)),
                ]
            },
            active_layer: vec![2], // Objects layer is default
            cached_hidden_ids: std::collections::HashSet::new(),
            cached_locked_ids: std::collections::HashSet::new(),
            cached_flat_ids: Vec::new(),
            cached_hidden_frame: 0,
            world: World::new(),
            event_bus: EventBus::new(),
            command_history: CommandHistory::with_max_depth(settings::MAX_UNDO_DEPTH),
            meshes: std::collections::HashMap::new(),
            material_library: forge_materials::library::MaterialLibrary::default(),
            asset_db: forge_assets::database::AssetDatabase::new(),
            scene: forge_scene::graph::SceneGraph::new(),
            gpu: None,
        }
    }
}

impl ForgeEditorApp {
    /// Create the app with GPU rendering if a wgpu render state is available.
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        // Initialize GPU rendering
        if let Some(wgpu_state) = cc.wgpu_render_state.as_ref() {
            let device = std::sync::Arc::new(wgpu_state.device.clone());
            let queue = std::sync::Arc::new(wgpu_state.queue.clone());
            let format = wgpu_state.target_format;

            let (w, h) = (1280u32, 720u32);
            let renderer = forge_render::Renderer::new(&device, format, w, h);

            // Create offscreen render target
            let offscreen_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("viewport_offscreen"),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Create default meshes on GPU
            let default_color = [0.7, 0.7, 0.7, 1.0];
            let cube_mesh = forge_modeling::primitives::generate_cube(1.0);
            let sphere_mesh = forge_modeling::primitives::generate_icosphere(0.5, 3);

            let gpu_cube = crate::viewport::gpu_mesh::editmesh_to_gpu(
                &device,
                &cube_mesh,
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::ZERO,
                Vec3::ONE,
                default_color,
            );
            let gpu_sphere = crate::viewport::gpu_mesh::editmesh_to_gpu(
                &device,
                &sphere_mesh,
                Vec3::new(3.0, 1.5, -1.0),
                Vec3::ZERO,
                Vec3::ONE,
                [0.3, 0.6, 0.9, 1.0],
            );

            // Store default EditMeshes in the app
            let cube_nid = forge_core::id::NodeId::new();
            let sphere_nid = forge_core::id::NodeId::new();
            app.meshes.insert(cube_nid, cube_mesh);
            app.meshes.insert(sphere_nid, sphere_mesh);

            app.gpu = Some(GpuViewportState {
                renderer,
                gpu_meshes: vec![gpu_cube, gpu_sphere],
                offscreen_texture,
                offscreen_view,
                viewport_size: (w, h),
                device,
                queue,
            });

            tracing::info!("GPU viewport initialized with {} default meshes", 2);
        }

        app
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
        self.rebuild_hidden_cache();

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
