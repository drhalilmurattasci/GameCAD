//! Editor settings and constants.
//!
//! All configurable values (grid, snap, camera, tools, viewport, layers)
//! live here as a single [`EditorSettings`] struct. Defaults are defined
//! once and used throughout the app. Settings can be saved/loaded from
//! a TOML file.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// Constants (non-configurable)
// ─────────────────────────────────────────────────────────────────────

/// Application name shown in title bar.
pub const APP_NAME: &str = "GameCAD";

/// Application version.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default window width on first launch.
pub const DEFAULT_WINDOW_WIDTH: f32 = 1600.0;

/// Default window height on first launch.
pub const DEFAULT_WINDOW_HEIGHT: f32 = 900.0;

/// Maximum undo history depth.
pub const MAX_UNDO_DEPTH: usize = 200;

/// Minimum distance (pixels) for a drag to count as box select.
pub const BOX_SELECT_MIN_DISTANCE: f32 = 5.0;

/// Minimum distance (pixels) for right-click to count as drag vs context menu.
pub const CONTEXT_MENU_DRAG_THRESHOLD: f32 = 5.0;

// ─────────────────────────────────────────────────────────────────────
// Grid settings
// ─────────────────────────────────────────────────────────────────────

/// Grid display and behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GridSettings {
    /// Whether the grid is visible.
    pub visible: bool,
    /// Grid cell spacing in meters.
    pub size: f32,
    /// Number of minor lines between major lines.
    pub major_every: u32,
    /// Maximum grid extent (in grid cells from origin).
    pub extent: i32,
    /// Grid fade-out distance from camera.
    pub fade_distance: f32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            visible: true,
            size: 1.0,
            major_every: 5,
            extent: 20,
            fade_distance: 50.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Snap settings
// ─────────────────────────────────────────────────────────────────────

/// Snap behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapSettings {
    /// Whether grid snapping is enabled.
    pub enabled: bool,
    /// Snap increment in meters.
    pub size: f32,
    /// Snap rotation increment in degrees.
    pub rotation_degrees: f32,
    /// Snap scale increment.
    pub scale_increment: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 0.5,
            rotation_degrees: 15.0,
            scale_increment: 0.1,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Camera settings
// ─────────────────────────────────────────────────────────────────────

/// Camera control configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraSettings {
    /// Orbit speed multiplier (degrees per pixel).
    pub orbit_speed: f32,
    /// Fast orbit multiplier (when Shift is held).
    pub fast_orbit_multiplier: f32,
    /// Pan speed multiplier.
    pub pan_speed: f32,
    /// Zoom speed multiplier.
    pub zoom_speed: f32,
    /// Alt+scroll zoom multiplier.
    pub alt_zoom_multiplier: f32,
    /// Default field of view in degrees.
    pub default_fov: f32,
    /// Near clipping plane.
    pub near_clip: f32,
    /// Far clipping plane.
    pub far_clip: f32,
    /// Default camera distance from target.
    pub default_distance: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            orbit_speed: 0.005,
            fast_orbit_multiplier: 2.0,
            pan_speed: 1.0,
            zoom_speed: 0.1,
            alt_zoom_multiplier: 3.0,
            default_fov: 45.0,
            default_distance: 8.0,
            near_clip: 0.1,
            far_clip: 1000.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tool settings
// ─────────────────────────────────────────────────────────────────────

/// Transform tool speed configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolSettings {
    /// Move speed multiplier (scaled by camera distance).
    pub move_speed: f32,
    /// Rotation speed in degrees per pixel.
    pub rotate_speed: f32,
    /// Scale speed multiplier per pixel.
    pub scale_speed: f32,
    /// Minimum allowed scale value.
    pub min_scale: f32,
}

impl Default for ToolSettings {
    fn default() -> Self {
        Self {
            move_speed: 0.003,
            rotate_speed: 0.15,
            scale_speed: 0.002,
            min_scale: 0.01,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Viewport settings
// ─────────────────────────────────────────────────────────────────────

/// Viewport display configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewportSettings {
    /// Show the shortcut reference overlay.
    pub show_shortcuts: bool,
    /// Show camera info overlay.
    pub show_camera_info: bool,
    /// Show status overlay (tab, render style, grid/snap).
    pub show_status_overlay: bool,
    /// Show FPS in status bar.
    pub show_fps: bool,
    /// Selection highlight line width.
    pub selection_line_width: f32,
    /// Gizmo arrow length in pixels.
    pub gizmo_size: f32,
}

impl Default for ViewportSettings {
    fn default() -> Self {
        Self {
            show_shortcuts: true,
            show_camera_info: true,
            show_status_overlay: true,
            show_fps: true,
            selection_line_width: 2.0,
            gizmo_size: 60.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Layer defaults
// ─────────────────────────────────────────────────────────────────────

/// Default layer color as [R, G, B].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDef {
    pub name: String,
    pub color: [u8; 3],
}

/// Default layers created for every new project.
pub const DEFAULT_LAYERS: &[(&str, [u8; 3])] = &[
    ("Base",        [0x00, 0x00, 0x00]),  // Black
    ("Environment", [0x2e, 0xcc, 0x71]),  // Green
    ("Characters",  [0x3e, 0x55, 0xff]),  // Blue
    ("Lights",      [0xff, 0xd7, 0x00]),  // Yellow
    ("Effects",     [0xe9, 0x45, 0x60]),  // Pink
    ("UI",          [0x9b, 0x59, 0xb6]),  // Purple
];

// ─────────────────────────────────────────────────────────────────────
// Working height
// ─────────────────────────────────────────────────────────────────────

/// Working height / construction plane configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HeightSettings {
    /// Current working height in meters.
    pub level: f32,
    /// Height plane extent for visualization.
    pub plane_extent: f32,
    /// Show height plane indicator when non-zero.
    pub show_plane: bool,
}

impl Default for HeightSettings {
    fn default() -> Self {
        Self {
            level: 0.0,
            plane_extent: 10.0,
            show_plane: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Selection settings
// ─────────────────────────────────────────────────────────────────────

/// Selection display and behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SelectionSettings {
    /// Default selection color (yellow highlight) [R, G, B]
    pub highlight_color: [u8; 3],
    /// Selection wireframe line width
    pub wireframe_width: f32,
    /// Selection hover highlight alpha (0-255)
    pub hover_alpha: u8,
    /// Whether to show selection bounding box
    pub show_bounds: bool,
    /// Box select minimum drag distance in pixels
    pub box_select_threshold: f32,
    /// Multi-select behavior: true = toggle, false = replace
    pub ctrl_click_toggles: bool,
}

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            highlight_color: [255, 255, 0], // Yellow
            wireframe_width: 2.0,
            hover_alpha: 40,
            show_bounds: true,
            box_select_threshold: 5.0,
            ctrl_click_toggles: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Master settings struct
// ─────────────────────────────────────────────────────────────────────

/// All editor settings in one struct. Serializable to/from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub grid: GridSettings,
    pub snap: SnapSettings,
    pub camera: CameraSettings,
    pub tools: ToolSettings,
    pub viewport: ViewportSettings,
    pub height: HeightSettings,
    pub selection: SelectionSettings,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            grid: GridSettings::default(),
            snap: SnapSettings::default(),
            camera: CameraSettings::default(),
            tools: ToolSettings::default(),
            viewport: ViewportSettings::default(),
            height: HeightSettings::default(),
            selection: SelectionSettings::default(),
        }
    }
}

impl EditorSettings {
    /// Load settings from a TOML file. Returns defaults if file doesn't exist.
    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to a TOML file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = EditorSettings::default();
        assert!(s.grid.size > 0.0);
        assert!(s.snap.size > 0.0);
        assert!(s.camera.default_fov > 0.0);
        assert!(s.tools.move_speed > 0.0);
        assert!(s.tools.min_scale > 0.0);
    }

    #[test]
    fn toml_roundtrip() {
        let s = EditorSettings::default();
        let toml_str = toml::to_string_pretty(&s).unwrap();
        let loaded: EditorSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(s.grid.size, loaded.grid.size);
        assert_eq!(s.snap.size, loaded.snap.size);
        assert_eq!(s.camera.default_fov, loaded.camera.default_fov);
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let s = EditorSettings::load(std::path::Path::new("/nonexistent/settings.toml"));
        assert_eq!(s.grid.size, 1.0);
        assert_eq!(s.snap.size, 0.5);
    }

    #[test]
    fn default_layers_count() {
        assert_eq!(DEFAULT_LAYERS.len(), 6);
        assert_eq!(DEFAULT_LAYERS[0].0, "Base");
        assert_eq!(DEFAULT_LAYERS[0].1, [0, 0, 0]);
    }

    #[test]
    fn selection_defaults_are_sane() {
        let s = SelectionSettings::default();
        assert_eq!(s.highlight_color, [255, 255, 0]);
        assert_eq!(s.wireframe_width, 2.0);
        assert_eq!(s.hover_alpha, 40);
        assert!(s.show_bounds);
        assert_eq!(s.box_select_threshold, 5.0);
        assert!(s.ctrl_click_toggles);

        // Roundtrip via EditorSettings
        let es = EditorSettings::default();
        let toml_str = toml::to_string_pretty(&es).unwrap();
        let loaded: EditorSettings = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.selection.highlight_color, [255, 255, 0]);
        assert_eq!(loaded.selection.box_select_threshold, 5.0);
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let partial = r#"
[grid]
size = 2.0
"#;
        let s: EditorSettings = toml::from_str(partial).unwrap();
        assert_eq!(s.grid.size, 2.0);
        // Other fields should be defaults
        assert_eq!(s.snap.size, 0.5);
        assert_eq!(s.camera.default_fov, 45.0);
    }
}
