//! Entity management helpers.
//!
//! Selection, deletion, duplication, and outliner-tree queries. These methods
//! keep `selected_entity` (primary) and `selected_entities` (multi-select
//! list) in sync.

use crate::app::ForgeEditorApp;
use crate::types::*;

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
        for &idx in &self.selected_entities {
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
        if idx < names.len() && idx > 0 {
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Deleted: {}", names[idx]),
            });
            let child_idx = idx - 1;
            if let Some(root) = self.outliner.first_mut() {
                if child_idx < root.children.len() {
                    root.children.remove(child_idx);
                }
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
        if idx > 0 && idx < names.len() {
            let new_name = format!("{}_copy", names[idx]);
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!("Duplicated: {} -> {}", names[idx], new_name),
            });
            let child_idx = idx - 1;
            if let Some(root) = self.outliner.first_mut() {
                if child_idx < root.children.len() {
                    let icon = root.children[child_idx].icon;
                    root.children.push(OutlinerNode {
                        name: new_name,
                        icon,
                        children: vec![],
                    });
                }
            }
            if idx < self.transforms.len() {
                let t = self.transforms[idx];
                self.transforms.push(t);
            }
        }
    }

    /// Add a new entity to the scene root with default transforms.
    pub(crate) fn add_entity(&mut self, name: &str, icon: &'static str) {
        self.console_log.push(LogEntry {
            level: LogLevel::Info,
            message: format!("Added: {}", name),
        });
        if let Some(root) = self.outliner.first_mut() {
            root.children.push(OutlinerNode {
                name: name.to_string(),
                icon,
                children: vec![],
            });
        }
        self.transforms
            .push([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        // Select the newly added entity
        let new_idx = self.entity_count() - 1;
        self.selected_entity = new_idx;
        self.selected_entities = vec![new_idx];
    }
}
