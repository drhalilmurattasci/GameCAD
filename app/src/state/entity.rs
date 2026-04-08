//! Entity management helpers.
//!
//! Selection, deletion, duplication, and outliner-tree queries. These methods
//! keep `selected_entity` (primary) and `selected_entities` (multi-select
//! list) in sync.

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    // -- Entity helper methods --

    /// Flatten the outliner tree into an ordered list of entity names.
    pub(crate) fn flatten_outliner_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        fn collect(node: &OutlinerNode, names: &mut Vec<String>) {
            names.push(node.name.clone());
            for child in &node.children {
                collect(child, names);
            }
        }
        for root in &self.outliner {
            collect(root, &mut names);
        }
        names
    }

    /// Flatten the outliner tree into an ordered list of NodeIds.
    pub(crate) fn flatten_outliner_ids(&self) -> Vec<forge_core::id::NodeId> {
        let mut ids = Vec::new();
        fn collect(node: &OutlinerNode, ids: &mut Vec<forge_core::id::NodeId>) {
            ids.push(node.id);
            for child in &node.children {
                collect(child, ids);
            }
        }
        for root in &self.outliner {
            collect(root, &mut ids);
        }
        ids
    }

    /// Total number of entities in the flattened outliner tree.
    pub(crate) fn entity_count(&self) -> usize {
        self.flatten_outliner_names().len()
    }

    /// Returns the icon for a given flat entity index.
    pub(crate) fn entity_icon(&self, idx: usize) -> Option<&'static str> {
        fn find_at(node: &OutlinerNode, target: usize, current: &mut usize) -> Option<&'static str> {
            if *current == target {
                return Some(node.icon);
            }
            *current += 1;
            for child in &node.children {
                if let Some(icon) = find_at(child, target, current) {
                    return Some(icon);
                }
            }
            None
        }
        let mut current = 0;
        for root in &self.outliner {
            if let Some(icon) = find_at(root, idx, &mut current) {
                return Some(icon);
            }
        }
        None
    }

    /// Returns true if entity at idx looks like a light (by icon).
    pub(crate) fn is_light_entity(&self, idx: usize) -> bool {
        self.entity_icon(idx) == Some("\u{2600}")
    }

    /// Returns true if entity at idx looks like a camera (by icon).
    pub(crate) fn is_camera_entity(&self, idx: usize) -> bool {
        self.entity_icon(idx) == Some("\u{1F3A5}")
    }

    /// Returns true if entity at idx looks like a mesh (by icon).
    pub(crate) fn is_mesh_entity(&self, idx: usize) -> bool {
        self.entity_icon(idx) == Some("\u{25A6}")
    }

    /// Keeps `selected_entity` (primary) consistent with `selected_entities`.
    ///
    /// If the multi-select list is empty, the primary falls back to 0 (root).
    /// If the current primary is not in the list, it snaps to the first entry.
    pub(crate) fn sync_selection(&mut self) {
        if self.selected_entities.is_empty() {
            self.selected_entity = 0;
        } else if !self.selected_entities.contains(&self.selected_entity) {
            self.selected_entity = self.selected_entities[0];
        }
    }

    /// Select every entity in the scene.
    pub(crate) fn select_all(&mut self) {
        let count = self.entity_count();
        self.selected_entities = (0..count).collect();
        if !self.selected_entities.is_empty() {
            self.selected_entity = self.selected_entities[0];
        }
        self.console_log.push(LogEntry {
            level: LogLevel::Info,
            message: format!("Selected all ({} entities)", count),
        });
    }

    /// Clear all selection state.
    pub(crate) fn deselect_all(&mut self) {
        self.selected_entities.clear();
        self.selected_entity = 0;
        self.console_log.push(LogEntry {
            level: LogLevel::Info,
            message: "Deselected all".into(),
        });
    }

    /// Invert the current selection (toggle every entity).
    pub(crate) fn invert_selection(&mut self) {
        let count = self.entity_count();
        let current: std::collections::HashSet<usize> =
            self.selected_entities.iter().copied().collect();
        self.selected_entities = (0..count).filter(|i| !current.contains(i)).collect();
        self.sync_selection();
        self.console_log.push(LogEntry {
            level: LogLevel::Info,
            message: "Inverted selection".into(),
        });
    }

    /// Delete all currently selected entities from the scene.
    pub(crate) fn delete_selected(&mut self) {
        if self.selected_entities.is_empty() {
            self.console_log.push(LogEntry {
                level: LogLevel::Warn,
                message: "Nothing selected to delete".into(),
            });
            return;
        }
        let names = self.flatten_outliner_names();
        let ids = self.flatten_outliner_ids();
        // Remove entity IDs from all layers before deleting
        for &idx in &self.selected_entities {
            if let Some(&entity_id) = ids.get(idx) {
                Self::remove_id_from_all_layers(&mut self.layers, entity_id);
            }
            if idx < names.len() {
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Deleted: {}", names[idx]),
                });
            }
        }
        let mut indices_to_remove: Vec<usize> = self
            .selected_entities
            .iter()
            .filter_map(|&idx| if idx > 0 { Some(idx - 1) } else { None })
            .collect();
        indices_to_remove.sort_unstable();
        indices_to_remove.dedup();
        if let Some(root) = self.outliner.first_mut() {
            for &ri in indices_to_remove.iter().rev() {
                if ri < root.children.len() {
                    root.children.remove(ri);
                }
            }
        }
        for &ri in indices_to_remove.iter().rev() {
            let ti = ri + 1;
            if ti < self.transforms.len() {
                self.transforms.remove(ti);
            }
        }
        self.selected_entities.clear();
        self.selected_entity = 0;
    }

    /// Delete a single entity by its flat index.
    pub(crate) fn delete_entity(&mut self, idx: usize) {
        let names = self.flatten_outliner_names();
        let ids = self.flatten_outliner_ids();
        if idx < names.len() && idx > 0 {
            // Remove from all layers before deleting
            if let Some(&entity_id) = ids.get(idx) {
                Self::remove_id_from_all_layers(&mut self.layers, entity_id);
            }
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Deleted: {}", names[idx]),
            });
            let child_idx = idx - 1;
            if let Some(root) = self.outliner.first_mut()
                && child_idx < root.children.len()
            {
                root.children.remove(child_idx);
            }
            if idx < self.transforms.len() {
                self.transforms.remove(idx);
            }
            self.selected_entities.retain(|&i| i != idx);
            if self.selected_entity == idx {
                self.selected_entity = 0;
            }
            self.sync_selection();
        }
    }

    /// Duplicate all currently selected entities.
    pub(crate) fn duplicate_selected(&mut self) {
        if self.selected_entities.is_empty() {
            self.console_log.push(LogEntry {
                level: LogLevel::Warn,
                message: "Nothing selected to duplicate".into(),
            });
            return;
        }
        let indices: Vec<usize> = self.selected_entities.clone();
        for &idx in &indices {
            if idx > 0 {
                self.duplicate_entity(idx);
            }
        }
    }

    /// Duplicate a single entity by flat index, appending `_copy` to its name.
    pub(crate) fn duplicate_entity(&mut self, idx: usize) {
        let names = self.flatten_outliner_names();
        let ids = self.flatten_outliner_ids();
        if idx > 0 && idx < names.len() {
            let new_name = format!("{}_copy", names[idx]);
            let new_id = forge_core::id::NodeId::new();
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Duplicated: {} -> {}", names[idx], new_name),
            });
            let child_idx = idx - 1;
            if let Some(root) = self.outliner.first_mut()
                && child_idx < root.children.len()
            {
                let icon = root.children[child_idx].icon;
                root.children.push(OutlinerNode {
                    id: new_id,
                    name: new_name,
                    icon,
                    expanded: true,
                    children: vec![],
                });
            }
            if idx < self.transforms.len() {
                let t = self.transforms[idx];
                self.transforms.push(t);
            }
            // Assign duplicate to same layer as original
            if let Some(&original_id) = ids.get(idx) {
                Self::add_to_same_layer(&mut self.layers, original_id, new_id);
            }
        }
    }

    /// Add a new entity to the scene root with default transforms.
    pub(crate) fn add_entity(&mut self, name: &str, icon: &'static str) {
        let new_id = forge_core::id::NodeId::new();
        self.console_log.push(LogEntry {
            level: LogLevel::Info,
            message: format!("Added: {}", name),
        });
        if let Some(root) = self.outliner.first_mut() {
            root.children.push(OutlinerNode {
                id: new_id,
                name: name.to_string(),
                icon,
                expanded: true,
                children: vec![],
            });
        }
        self.transforms
            .push([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        // Also add to scene graph
        let scene_node = forge_scene::node::SceneNode::new(name);
        self.scene.add_node(scene_node, self.scene.root());

        // Select the newly added entity
        let new_idx = self.entity_count() - 1;
        self.selected_entity = new_idx;
        self.selected_entities = vec![new_idx];

        // Auto-assign to active layer
        if let Some(layer) = self.active_layer_mut() {
            layer.entity_ids.insert(new_id);
        }
    }

    /// Add a mesh entity with generated geometry.
    pub(crate) fn add_mesh_entity(&mut self, name: &str, mesh: forge_modeling::half_edge::EditMesh) {
        self.add_entity(name, "\u{25A6}");
        // Store mesh data keyed by a new NodeId
        let node_id = forge_core::id::NodeId::new();
        self.meshes.insert(node_id, mesh);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> ForgeEditorApp {
        ForgeEditorApp::default()
    }

    #[test]
    fn flatten_outliner_names_default() {
        let app = make_app();
        let names = app.flatten_outliner_names();
        // Default scene: Scene Root + 4 children
        assert_eq!(names.len(), 5);
        assert_eq!(names[0], "Scene Root");
        assert_eq!(names[1], "Cube");
        assert_eq!(names[2], "Sphere");
        assert_eq!(names[3], "Directional Light");
        assert_eq!(names[4], "Main Camera");
    }

    #[test]
    fn entity_count_matches_flatten() {
        let app = make_app();
        assert_eq!(app.entity_count(), 5);
    }

    #[test]
    fn entity_icon_valid() {
        let app = make_app();
        // Scene Root = folder icon
        assert_eq!(app.entity_icon(0), Some("\u{1F5C2}"));
        // Cube = mesh icon
        assert_eq!(app.entity_icon(1), Some("\u{25A6}"));
        // Light = sun icon
        assert_eq!(app.entity_icon(3), Some("\u{2600}"));
        // Camera = camera icon
        assert_eq!(app.entity_icon(4), Some("\u{1F3A5}"));
        // Out of bounds
        assert_eq!(app.entity_icon(99), None);
    }

    #[test]
    fn is_type_checks() {
        let app = make_app();
        assert!(app.is_mesh_entity(1));  // Cube
        assert!(app.is_mesh_entity(2));  // Sphere
        assert!(app.is_light_entity(3)); // Directional Light
        assert!(app.is_camera_entity(4)); // Main Camera
        assert!(!app.is_light_entity(1)); // Cube is not light
        assert!(!app.is_camera_entity(1)); // Cube is not camera
    }

    #[test]
    fn sync_selection_empty() {
        let mut app = make_app();
        app.selected_entities.clear();
        app.selected_entity = 5;
        app.sync_selection();
        assert_eq!(app.selected_entity, 0);
    }

    #[test]
    fn sync_selection_primary_not_in_list() {
        let mut app = make_app();
        app.selected_entities = vec![2, 3];
        app.selected_entity = 1;
        app.sync_selection();
        assert_eq!(app.selected_entity, 2);
    }

    #[test]
    fn select_all() {
        let mut app = make_app();
        app.select_all();
        assert_eq!(app.selected_entities.len(), 5);
        assert_eq!(app.selected_entities, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn deselect_all() {
        let mut app = make_app();
        app.deselect_all();
        assert!(app.selected_entities.is_empty());
        assert_eq!(app.selected_entity, 0);
    }

    #[test]
    fn invert_selection() {
        let mut app = make_app();
        app.selected_entities = vec![1, 3];
        app.selected_entity = 1;
        app.invert_selection();
        assert_eq!(app.selected_entities, vec![0, 2, 4]);
    }

    #[test]
    fn add_entity_increments_count() {
        let mut app = make_app();
        let before = app.entity_count();
        app.add_entity("New Entity", "\u{25CB}");
        assert_eq!(app.entity_count(), before + 1);
        assert_eq!(app.transforms.len(), before + 1);
    }

    #[test]
    fn add_entity_selects_new() {
        let mut app = make_app();
        app.add_entity("New Test", "\u{25CB}");
        let count = app.entity_count();
        assert_eq!(app.selected_entity, count - 1);
        assert_eq!(app.selected_entities, vec![count - 1]);
    }

    #[test]
    fn delete_entity_removes_from_outliner() {
        let mut app = make_app();
        let before = app.entity_count();
        app.delete_entity(1); // Delete Cube
        assert_eq!(app.entity_count(), before - 1);
        let names = app.flatten_outliner_names();
        assert!(!names.contains(&"Cube".to_string()));
    }

    #[test]
    fn delete_entity_zero_is_noop() {
        let mut app = make_app();
        let before = app.entity_count();
        app.delete_entity(0); // root cannot be deleted
        assert_eq!(app.entity_count(), before);
    }

    #[test]
    fn delete_selected_empty_warns() {
        let mut app = make_app();
        app.selected_entities.clear();
        let log_before = app.console_log.len();
        app.delete_selected();
        assert!(app.console_log.len() > log_before);
        assert_eq!(app.console_log.last().unwrap().message, "Nothing selected to delete");
    }

    #[test]
    fn duplicate_entity_appends_copy() {
        let mut app = make_app();
        let before = app.entity_count();
        app.duplicate_entity(1); // duplicate Cube
        assert_eq!(app.entity_count(), before + 1);
        let names = app.flatten_outliner_names();
        assert!(names.contains(&"Cube_copy".to_string()));
    }

    #[test]
    fn duplicate_selected_empty_warns() {
        let mut app = make_app();
        app.selected_entities.clear();
        let log_before = app.console_log.len();
        app.duplicate_selected();
        assert!(app.console_log.len() > log_before);
    }

    #[test]
    fn add_mesh_entity_stores_mesh() {
        let mut app = make_app();
        let mesh = forge_modeling::primitives::generate_cube(1.0);
        let before_meshes = app.meshes.len();
        app.add_mesh_entity("Test Cube", mesh);
        assert_eq!(app.meshes.len(), before_meshes + 1);
    }

    #[test]
    fn default_transforms_match_outliner() {
        let app = make_app();
        assert_eq!(app.transforms.len(), app.entity_count());
    }

    #[test]
    fn default_active_layer_is_objects() {
        let app = make_app();
        assert_eq!(app.active_layer, vec![2]);
        assert_eq!(app.layers[2].name, "Objects");
    }

    #[test]
    fn default_layers_have_7_entries() {
        let app = make_app();
        assert_eq!(app.layers.len(), 7);
    }

    #[test]
    fn add_entity_assigns_to_active_layer() {
        let mut app = make_app();
        assert_eq!(app.active_layer, vec![2]); // Objects
        let before = app.layers[2].entity_ids.len();
        app.add_entity("Test", "\u{25CB}");
        assert_eq!(app.layers[2].entity_ids.len(), before + 1);
    }

    #[test]
    fn add_mesh_entity_assigns_to_active_layer() {
        let mut app = make_app();
        let mesh = forge_modeling::primitives::generate_cube(1.0);
        let before = app.layers[2].entity_ids.len();
        app.add_mesh_entity("Test Cube", mesh);
        assert_eq!(app.layers[2].entity_ids.len(), before + 1);
    }

    #[test]
    fn add_entity_to_different_layer() {
        let mut app = make_app();
        let objects_before = app.layers[2].entity_ids.len();
        let lights_before = app.layers[4].entity_ids.len();
        app.active_layer = vec![4]; // Lights
        app.add_entity("New Light", "\u{2600}");
        assert_eq!(app.layers[4].entity_ids.len(), lights_before + 1);
        assert_eq!(app.layers[2].entity_ids.len(), objects_before); // Objects not affected
    }

    #[test]
    fn sublayer_add_and_count() {
        let mut app = make_app();
        let sublayer = crate::state::EditorLayer::new("Sub-1", egui::Color32::GRAY);
        app.layers[2].add_sublayer(sublayer);
        assert_eq!(app.layers[2].children.len(), 1);
        assert_eq!(app.layers[2].children[0].name, "Sub-1");
    }

    #[test]
    fn total_entity_count_includes_sublayers() {
        let mut app = make_app();
        let base = app.layers[2].entity_ids.len(); // default entities already assigned
        app.layers[2].entity_ids.insert(forge_core::id::NodeId::new());
        let mut sub = crate::state::EditorLayer::new("Sub", egui::Color32::GRAY);
        sub.entity_ids.insert(forge_core::id::NodeId::new());
        sub.entity_ids.insert(forge_core::id::NodeId::new());
        app.layers[2].add_sublayer(sub);
        // base + 1 (parent) + 2 (sublayer)
        assert_eq!(app.layers[2].total_entity_count(), base + 3);
    }

    #[test]
    fn delete_selected_multiple() {
        let mut app = make_app();
        app.selected_entities = vec![1, 2]; // Cube and Sphere
        app.delete_selected();
        let names = app.flatten_outliner_names();
        assert!(!names.contains(&"Cube".to_string()));
        assert!(!names.contains(&"Sphere".to_string()));
        assert!(app.selected_entities.is_empty());
    }
}
