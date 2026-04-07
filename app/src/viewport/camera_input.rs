//! Mouse-driven camera controls and viewport interaction.
//!
//! Implements Unreal Engine / Maya style navigation:
//! - Right-drag: orbit, Shift+right-drag: fast orbit
//! - Middle-drag: pan
//! - Scroll: zoom (Alt for fast zoom)
//! - Alt+left-drag: orbit (Maya), Alt+middle-drag: pan, Alt+right-drag: dolly
//! - Left-click: entity picking, left-drag: tool transform or box select
//! - Ctrl+Shift+left-drag: zoom extents

use eframe::egui;
use egui::{PointerButton, Pos2, Response};
use glam::Vec3;

use crate::app::ForgeEditorApp;
use crate::types::*;

impl ForgeEditorApp {
    /// Handle all mouse-based camera controls: orbit, pan, zoom, Alt+drag,
    /// Shift+drag, tool transforms, and box select.
    pub(crate) fn handle_camera_input(
        &mut self,
        ctx: &egui::Context,
        response: &Response,
        pointer_pos: Option<Pos2>,
    ) {
        let modifiers = ctx.input(|i| i.modifiers);
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);

        // Track right-click start position for context menu vs drag detection
        if response.drag_started_by(PointerButton::Secondary) {
            self.right_click_start_pos = pointer_pos;
        }

        // Scroll wheel: Zoom
        if response.hovered() && scroll_delta.abs() > 0.0 {
            let zoom_factor = if modifiers.alt { 3.0 } else { 1.0 };
            self.orbit_camera
                .zoom(-scroll_delta * zoom_factor * 0.1);
        }

        // Left + Right buttons held: Pan scene (like Unreal middle-click pan)
        let both_buttons = ctx.input(|i| {
            i.pointer.button_down(PointerButton::Primary)
                && i.pointer.button_down(PointerButton::Secondary)
        });
        if both_buttons && response.dragged() {
            let delta = response.drag_delta();
            self.orbit_camera.pan(delta.x, delta.y);
            self.is_panning = true;
        }

        // Right drag: Orbit camera (or Shift+Right drag: fast orbit)
        // Skip if both buttons are held (that's pan above)
        if response.dragged_by(PointerButton::Secondary) && !modifiers.alt && !both_buttons {
            let delta = response.drag_delta();
            let speed = if modifiers.shift { 0.01 } else { 0.005 };
            self.orbit_camera.orbit(delta.x * speed, -delta.y * speed);
            self.is_orbiting = true;
        }

        // Middle click+drag: Pan camera
        if response.dragged_by(PointerButton::Middle) && !modifiers.alt {
            let delta = response.drag_delta();
            self.orbit_camera.pan(delta.x, delta.y);
            self.is_panning = true;
        }

        // Alt + Left drag: Orbit camera (Maya style)
        if modifiers.alt
            && response.dragged_by(PointerButton::Primary)
            && !modifiers.ctrl
            && !modifiers.shift
        {
            let delta = response.drag_delta();
            self.orbit_camera
                .orbit(delta.x * 0.005, -delta.y * 0.005);
            self.is_orbiting = true;
        }

        // Alt + Middle drag: Pan camera (Maya style)
        if modifiers.alt && response.dragged_by(PointerButton::Middle) {
            let delta = response.drag_delta();
            self.orbit_camera.pan(delta.x, delta.y);
            self.is_panning = true;
        }

        // Alt + Right drag: Dolly zoom
        if modifiers.alt && response.dragged_by(PointerButton::Secondary) {
            let delta = response.drag_delta();
            self.orbit_camera.zoom(delta.y * 0.5);
        }

        // Ctrl + Shift + Left drag: Zoom extents (fit all in view)
        if modifiers.ctrl
            && modifiers.shift
            && response.dragged_by(PointerButton::Primary)
        {
            self.orbit_camera.focus_on(Vec3::ZERO, 10.0);
            self.orbit_camera.yaw = 0.5;
            self.orbit_camera.pitch = 0.35;
            self.console_log.push(LogEntry {
                level: LogLevel::Info,
                message: "Zoom extents: fit all in view".into(),
            });
        }

        // Left click: Select entity (simple - pick closest placeholder object)
        if response.clicked_by(PointerButton::Primary) && !modifiers.alt
            && let Some(pos) = pointer_pos
        {
            self.handle_viewport_click(pos, &response.rect, modifiers);
        }

        // Left drag: Tool transform OR box select
        let tool_active = self.tool_mode != ToolMode::Select
            && self.selected_entity > 0
            && self.selected_entity < self.transforms.len()
            && !modifiers.alt;

        if tool_active && response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();
            let cam_dist = self.orbit_camera.distance;
            let sel = self.selected_entity;
            match self.tool_mode {
                ToolMode::Move => {
                    // Movement uses camera-relative directions so dragging
                    // right always moves the object screen-right regardless
                    // of camera orbit angle.
                    let speed = 0.003 * cam_dist;
                    let view = self.orbit_camera.view_matrix();
                    // Camera right vector (first row of view matrix)
                    let cam_right = glam::Vec3::new(view.col(0).x, view.col(0).y, view.col(0).z);
                    // Camera up is world Y
                    let cam_up = glam::Vec3::Y;
                    // Camera forward projected onto XZ plane (for depth movement)
                    let cam_fwd = glam::Vec3::new(-view.col(2).x, 0.0, -view.col(2).z).normalize_or_zero();

                    if modifiers.shift && !modifiers.ctrl {
                        // Shift only: vertical (Y axis) movement
                        self.transforms[sel][1] -= delta.y * speed;
                    } else if modifiers.ctrl {
                        // Ctrl: camera-right + vertical Y (up/down)
                        let move_vec = cam_right * delta.x * speed + cam_up * (-delta.y * speed);
                        self.transforms[sel][0] += move_vec.x;
                        self.transforms[sel][1] += move_vec.y;
                        self.transforms[sel][2] += move_vec.z;
                    } else {
                        // Default: camera-right + camera-forward (XZ plane)
                        let move_vec = cam_right * delta.x * speed + cam_fwd * delta.y * speed;
                        self.transforms[sel][0] += move_vec.x;
                        self.transforms[sel][1] += move_vec.y;
                        self.transforms[sel][2] += move_vec.z;
                    }
                }
                ToolMode::Rotate => {
                    let speed = 0.15;
                    if modifiers.shift {
                        self.transforms[sel][5] += delta.x * speed;
                    } else {
                        self.transforms[sel][4] += delta.x * speed;
                        self.transforms[sel][3] += delta.y * speed;
                    }
                }
                ToolMode::Scale => {
                    let factor = 1.0 + delta.x * 0.002;
                    self.transforms[sel][6] = (self.transforms[sel][6] * factor).max(0.01);
                    self.transforms[sel][7] = (self.transforms[sel][7] * factor).max(0.01);
                    self.transforms[sel][8] = (self.transforms[sel][8] * factor).max(0.01);
                }
                ToolMode::Select => {} // handled below
            }
        } else if !modifiers.alt
            && !modifiers.ctrl
            && !modifiers.shift
            && response.dragged_by(PointerButton::Primary)
        {
            // Box select (only when Select tool or no entity selected)
            if response.drag_started() {
                self.box_select_start = pointer_pos;
            }
            self.box_select_end = pointer_pos;
        }
        // Tool drag finished — snap + log once
        if tool_active && response.drag_stopped_by(PointerButton::Primary) {
            let sel = self.selected_entity;
            let names = self.flatten_outliner_names();
            let ent_name = names.get(sel).cloned().unwrap_or_default();
            // Snap to grid on release
            if self.snap_enabled && self.tool_mode == ToolMode::Move {
                let s = self.snap_size;
                self.transforms[sel][0] = (self.transforms[sel][0] / s).round() * s;
                self.transforms[sel][1] = (self.transforms[sel][1] / s).round() * s;
                self.transforms[sel][2] = (self.transforms[sel][2] / s).round() * s;
            }
            // Log final position/rotation/scale once
            let t = &self.transforms[sel];
            let msg = match self.tool_mode {
                ToolMode::Move => format!("Moved {} to ({:.2}, {:.2}, {:.2})", ent_name, t[0], t[1], t[2]),
                ToolMode::Rotate => format!("Rotated {} to ({:.1}, {:.1}, {:.1})°", ent_name, t[3], t[4], t[5]),
                ToolMode::Scale => format!("Scaled {} to ({:.2}, {:.2}, {:.2})", ent_name, t[6], t[7], t[8]),
                ToolMode::Select => String::new(),
            };
            if !msg.is_empty() {
                self.console_log.push(LogEntry { level: LogLevel::Info, message: msg });
            }
        }

        if response.drag_stopped_by(PointerButton::Primary) && !modifiers.alt && !tool_active {
            if let (Some(start), Some(end)) =
                (self.box_select_start, self.box_select_end)
            {
                let dist = start.distance(end);
                if dist > 10.0 {
                    self.handle_viewport_box_select(start, end, &response.rect);
                }
            }
            self.box_select_start = None;
            self.box_select_end = None;
        }

        // Reset orbit/pan flags on drag stop
        if response.drag_stopped() {
            self.is_orbiting = false;
            self.is_panning = false;
        }

        // Clear right-click tracking once the gesture ends
        if let Some(start) = self.right_click_start_pos
            && (response.drag_stopped_by(PointerButton::Secondary)
                || response.clicked_by(PointerButton::Secondary))
        {
            let _end = pointer_pos.unwrap_or(start);
            self.right_click_start_pos = None;
        }
    }
}
