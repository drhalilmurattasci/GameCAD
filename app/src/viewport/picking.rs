//! Viewport entity picking and box selection.
//!
//! Click selection projects all entity positions to screen space and picks
//! the closest one within a 40-pixel threshold.  Box selection selects all
//! entities whose projected position falls inside the drag rectangle.
//! Both respect Ctrl (toggle) and Shift (additive) modifiers.

use egui::{Pos2, Rect};
use glam::Vec3;

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    /// Handle a left-click in the viewport by picking the closest entity.
    pub(crate) fn handle_viewport_click(&mut self, pos: Pos2, rect: &Rect, modifiers: egui::Modifiers) {
        let aspect = if rect.height() > 0.0 {
            rect.width() / rect.height()
        } else {
            16.0 / 9.0
        };
        let view = self.orbit_camera.view_matrix();
        let proj = self.orbit_camera.projection_matrix(aspect);
        let vp = proj * view;

        // Build list of entities with screen positions from transforms
        let names = self.flatten_outliner_names();
        let mut hits: Vec<(usize, f32)> = Vec::new(); // (entity_idx, distance)

        for idx in 1..names.len().min(self.transforms.len()) {
            if self.is_entity_hidden(idx) || self.is_entity_locked(idx) {
                continue;
            }
            let world_pos = Vec3::new(
                self.transforms[idx][0],
                self.transforms[idx][1],
                self.transforms[idx][2],
            );
            if let Some(screen_pos) = Self::project_3d(&vp, rect, world_pos) {
                let dist = pos.distance(screen_pos);
                if dist < 40.0 {
                    hits.push((idx, dist));
                }
            }
        }

        // Sort by distance, pick closest
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(&(entity_idx, _)) = hits.first() {
            if modifiers.ctrl {
                // Toggle selection
                if let Some(pos_in_vec) = self
                    .selected_entities
                    .iter()
                    .position(|&e| e == entity_idx)
                {
                    self.selected_entities.remove(pos_in_vec);
                } else {
                    self.selected_entities.push(entity_idx);
                }
                self.sync_selection();
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Toggle select: {}", names[entity_idx]),
                });
            } else if modifiers.shift {
                // Add to selection
                if !self.selected_entities.contains(&entity_idx) {
                    self.selected_entities.push(entity_idx);
                }
                self.selected_entity = entity_idx;
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Add to selection: {}", names[entity_idx]),
                });
            } else {
                // Single select
                self.selected_entity = entity_idx;
                self.selected_entities = vec![entity_idx];
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Selected: {}", names[entity_idx]),
                });
            }
        } else if !modifiers.ctrl && !modifiers.shift {
            // Clicked empty space -> deselect
            self.selected_entities.clear();
            self.selected_entity = 0;
        }
    }

    /// Handle a box-select drag in the viewport.
    pub(crate) fn handle_viewport_box_select(&mut self, start: Pos2, end: Pos2, rect: &Rect) {
        let aspect = if rect.height() > 0.0 {
            rect.width() / rect.height()
        } else {
            16.0 / 9.0
        };
        let view = self.orbit_camera.view_matrix();
        let proj = self.orbit_camera.projection_matrix(aspect);
        let vp = proj * view;

        let sel_rect = Rect::from_two_pos(start, end);
        let names = self.flatten_outliner_names();

        self.selected_entities.clear();
        for idx in 1..names.len().min(self.transforms.len()) {
            if self.is_entity_hidden(idx) || self.is_entity_locked(idx) {
                continue;
            }
            let world_pos = Vec3::new(
                self.transforms[idx][0],
                self.transforms[idx][1],
                self.transforms[idx][2],
            );
            if let Some(screen_pos) = Self::project_3d(&vp, rect, world_pos)
                && sel_rect.contains(screen_pos)
            {
                self.selected_entities.push(idx);
            }
        }
        if !self.selected_entities.is_empty() {
            self.selected_entity = self.selected_entities[0];
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: format!(
                    "Box selected {} entities",
                    self.selected_entities.len()
                ),
            });
        } else {
            self.selected_entity = 0;
        }
    }
}
