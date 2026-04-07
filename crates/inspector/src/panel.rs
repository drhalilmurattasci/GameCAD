//! Inspector panel that renders editable properties for a selected scene node.

use egui::{CollapsingHeader, Color32, RichText, Stroke, Ui};
use forge_core::math::Vec3;
use forge_scene::node::{NodeType, SceneNode};

use crate::widgets;

// ── Crystalline theme constants ─────────────────────────────────────
const SECTION_TEXT_COLOR: Color32 = Color32::from_rgb(0x4E, 0xFF, 0x93); // accent
const SEPARATOR_COLOR: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3E); // border

/// The inspector panel. Holds transient editing state for rotation display.
///
/// The panel renders editable properties for a single [`SceneNode`], including
/// its transform, visibility flags, and node-type-specific fields (light
/// parameters, camera FOV, mesh asset references, etc.).
#[derive(Debug)]
pub struct InspectorPanel {
    /// Euler angle cache (degrees) so users can edit rotation as Euler angles
    /// without quaternion round-trip jitter.
    euler_cache: Vec3,
    /// Whether the Euler cache needs to be re-synced from the node's quaternion.
    /// Starts `true` so the very first render picks up the node's current rotation.
    euler_dirty: bool,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self {
            euler_cache: Vec3::ZERO,
            // Start dirty so the first call to `show` syncs from the node.
            euler_dirty: true,
        }
    }
}

impl InspectorPanel {
    /// Creates a new inspector panel.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Call this when the selection changes to force the Euler cache to
    /// re-sync from the new node's quaternion.
    #[inline]
    pub fn invalidate_euler_cache(&mut self) {
        self.euler_dirty = true;
    }

    /// Renders the full inspector UI for the given scene node.
    ///
    /// Returns `true` if any property was modified.
    pub fn show(&mut self, ui: &mut Ui, node: &mut SceneNode) -> bool {
        let mut changed = false;

        // ── Header ──────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new(&node.name)
                    .color(SECTION_TEXT_COLOR)
                    .strong(),
            );
        });
        ui.separator();

        // ── Name ────────────────────────────────────────────────────
        if widgets::draw_string(ui, "Name", &mut node.name) {
            changed = true;
        }

        // ── Visibility / Locked ─────────────────────────────────────
        ui.horizontal(|ui| {
            if widgets::draw_bool(ui, "Visible", &mut node.visible) {
                changed = true;
            }
            if widgets::draw_bool(ui, "Locked", &mut node.locked) {
                changed = true;
            }
        });

        ui.add_space(4.0);
        draw_themed_separator(ui);

        // ── Transform section ───────────────────────────────────────
        changed |= self.draw_transform_section(ui, node);

        draw_themed_separator(ui);

        // ── Node-type-specific section ──────────────────────────────
        changed |= self.draw_node_type_section(ui, node);

        changed
    }

    // ── Transform ───────────────────────────────────────────────────

    /// Draws the Transform collapsing section (position, rotation, scale).
    fn draw_transform_section(&mut self, ui: &mut Ui, node: &mut SceneNode) -> bool {
        let mut changed = false;

        CollapsingHeader::new(
            RichText::new("Transform")
                .color(SECTION_TEXT_COLOR)
                .strong(),
        )
        .default_open(true)
        .show(ui, |ui| {
            // Position
            if widgets::draw_vec3(ui, "Position", &mut node.transform.position) {
                changed = true;
            }

            // Rotation (displayed as Euler degrees)
            if self.euler_dirty {
                let (y, x, z) = node.transform.rotation.to_euler(glam::EulerRot::YXZ);
                self.euler_cache = Vec3::new(
                    x.to_degrees(),
                    y.to_degrees(),
                    z.to_degrees(),
                );
                self.euler_dirty = false;
            }

            if widgets::draw_vec3(ui, "Rotation", &mut self.euler_cache) {
                node.transform.rotation = glam::Quat::from_euler(
                    glam::EulerRot::YXZ,
                    self.euler_cache.y.to_radians(),
                    self.euler_cache.x.to_radians(),
                    self.euler_cache.z.to_radians(),
                );
                changed = true;
            }

            // Scale
            if widgets::draw_vec3(ui, "Scale", &mut node.transform.scale) {
                changed = true;
            }
        });

        changed
    }

    // ── Node-type specifics ─────────────────────────────────────────

    /// Draws the node-type-specific properties (light, camera, mesh, etc.).
    fn draw_node_type_section(&mut self, ui: &mut Ui, node: &mut SceneNode) -> bool {
        let mut changed = false;

        match &mut node.node_type {
            NodeType::Empty | NodeType::Group => {
                // No additional properties.
            }

            NodeType::Mesh {
                asset_id,
                material_ids,
            } => {
                CollapsingHeader::new(
                    RichText::new("Mesh").color(SECTION_TEXT_COLOR).strong(),
                )
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Asset ID");
                        let mut id_str = asset_id.to_string();
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            // Asset references are typically set via drag-and-drop;
                            // text editing is shown for inspection purposes.
                        }
                    });
                    ui.label(format!("Materials: {}", material_ids.len()));
                    for (i, mat_id) in material_ids.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("  [{}]", i));
                            ui.label(mat_id.to_string());
                        });
                    }
                });
            }

            NodeType::DirectionalLight {
                direction,
                color,
                intensity,
            } => {
                CollapsingHeader::new(
                    RichText::new("Directional Light")
                        .color(SECTION_TEXT_COLOR)
                        .strong(),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if widgets::draw_vec3(ui, "Direction", direction) {
                        changed = true;
                    }
                    if widgets::draw_color(ui, "Color", color) {
                        changed = true;
                    }
                    if widgets::draw_float(ui, "Intensity", intensity, 0.1) {
                        changed = true;
                    }
                });
            }

            NodeType::PointLight {
                position,
                color,
                intensity,
                radius,
            } => {
                CollapsingHeader::new(
                    RichText::new("Point Light")
                        .color(SECTION_TEXT_COLOR)
                        .strong(),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if widgets::draw_vec3(ui, "Position", position) {
                        changed = true;
                    }
                    if widgets::draw_color(ui, "Color", color) {
                        changed = true;
                    }
                    if widgets::draw_float(ui, "Intensity", intensity, 0.1) {
                        changed = true;
                    }
                    if widgets::draw_slider(ui, "Range", radius, 0.0, 1000.0) {
                        changed = true;
                    }
                });
            }

            NodeType::SpotLight {
                position,
                direction,
                color,
                intensity,
                inner_angle,
                outer_angle,
                range,
            } => {
                CollapsingHeader::new(
                    RichText::new("Spot Light")
                        .color(SECTION_TEXT_COLOR)
                        .strong(),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if widgets::draw_vec3(ui, "Position", position) {
                        changed = true;
                    }
                    if widgets::draw_vec3(ui, "Direction", direction) {
                        changed = true;
                    }
                    if widgets::draw_color(ui, "Color", color) {
                        changed = true;
                    }
                    if widgets::draw_float(ui, "Intensity", intensity, 0.1) {
                        changed = true;
                    }
                    if widgets::draw_slider(ui, "Inner Angle", inner_angle, 0.0, 180.0) {
                        changed = true;
                    }
                    if widgets::draw_slider(ui, "Outer Angle", outer_angle, 0.0, 180.0) {
                        changed = true;
                    }
                    if widgets::draw_slider(ui, "Range", range, 0.0, 1000.0) {
                        changed = true;
                    }
                });
            }

            NodeType::Camera { fov, near, far } => {
                CollapsingHeader::new(
                    RichText::new("Camera").color(SECTION_TEXT_COLOR).strong(),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if widgets::draw_slider(ui, "FOV", fov, 1.0, 179.0) {
                        changed = true;
                    }
                    if widgets::draw_float(ui, "Near", near, 0.01) {
                        changed = true;
                    }
                    if widgets::draw_float(ui, "Far", far, 1.0) {
                        changed = true;
                    }
                });
            }
        }

        changed
    }
}

/// Draws a thin horizontal line using the Crystalline border color.
fn draw_themed_separator(ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = rect.top();
    let stroke = Stroke::new(1.0, SEPARATOR_COLOR);
    ui.painter()
        .hline(rect.left()..=rect.right(), y, stroke);
    ui.add_space(6.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_default_creates() {
        let panel = InspectorPanel::new();
        // Euler cache starts dirty so the first render syncs from the node.
        assert!(panel.euler_dirty);
        assert_eq!(panel.euler_cache, Vec3::ZERO);
    }

    #[test]
    fn invalidate_euler_cache_sets_dirty() {
        let mut panel = InspectorPanel::new();
        panel.euler_dirty = false;
        panel.invalidate_euler_cache();
        assert!(panel.euler_dirty);
    }

    #[test]
    fn default_matches_new() {
        let a = InspectorPanel::new();
        let b = InspectorPanel::default();
        assert_eq!(a.euler_dirty, b.euler_dirty);
        assert_eq!(a.euler_cache, b.euler_cache);
    }
}
