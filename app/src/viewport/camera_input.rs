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

use crate::state::ForgeEditorApp;
use crate::state::types::*;

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

        // Scroll wheel: Zoom or Shift+Scroll: change working height
        if response.hovered() && scroll_delta.abs() > 0.0 {
            if modifiers.shift {
                // Shift+scroll: change working height layer by snap size increments
                let step = self.settings.snap.size;
                if scroll_delta > 0.0 {
                    self.settings.height.level += step;
                } else {
                    self.settings.height.level -= step;
                }
                // Snap to grid
                self.settings.height.level = (self.settings.height.level / step).round() * step;
                self.console_log.push(LogEntry {
                    level: LogLevel::Info,
                    message: format!("Working height: {:.2}m", self.settings.height.level),
                });
            } else {
                let zoom_factor = if modifiers.alt { self.settings.camera.alt_zoom_multiplier } else { 1.0 };
                self.orbit_camera
                    .zoom(-scroll_delta * zoom_factor * self.settings.camera.zoom_speed);
            }
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
            let base = self.settings.camera.orbit_speed;
            let speed = if modifiers.shift { base * self.settings.camera.fast_orbit_multiplier } else { base };
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
            let orbit_spd = self.settings.camera.orbit_speed;
            self.orbit_camera
                .orbit(delta.x * orbit_spd, -delta.y * orbit_spd);
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
            && !modifiers.alt
            && !self.is_entity_locked(self.selected_entity);

        if tool_active && response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();
            let cam_dist = self.orbit_camera.distance;
            let sel = self.selected_entity;
            match self.tool_mode {
                ToolMode::Move => {
                    // Camera-relative movement on the XZ ground plane.
                    // Object stays at its current Y unless Shift or Ctrl is held.
                    let speed = self.settings.tools.move_speed * cam_dist;
                    let cam_pos = self.orbit_camera.position();
                    let cam_target = self.orbit_camera.target;
                    let fwd_full = cam_target - cam_pos;
                    let cam_fwd = glam::Vec3::new(fwd_full.x, 0.0, fwd_full.z).normalize_or_zero();
                    let cam_right = cam_fwd.cross(glam::Vec3::Y).normalize_or_zero();

                    if modifiers.shift && !modifiers.ctrl {
                        // Shift: Y axis only (up/down)
                        self.transforms[sel][1] -= delta.y * speed;
                    } else if modifiers.ctrl {
                        // Ctrl: camera-right (X) + world up (Y)
                        let dx = cam_right * delta.x * speed;
                        self.transforms[sel][0] += dx.x;
                        self.transforms[sel][2] += dx.z;
                        self.transforms[sel][1] -= delta.y * speed;
                    } else {
                        // Default: camera-right + camera-forward, XZ plane ONLY
                        // Y is never modified in default mode
                        let move_xz = cam_right * delta.x * speed + cam_fwd * (-delta.y) * speed;
                        self.transforms[sel][0] += move_xz.x;
                        // skip Y — stay on same height
                        self.transforms[sel][2] += move_xz.z;
                    }
                }
                ToolMode::Rotate => {
                    let speed = self.settings.tools.rotate_speed;
                    if modifiers.shift {
                        self.transforms[sel][5] += delta.x * speed;
                    } else {
                        self.transforms[sel][4] += delta.x * speed;
                        self.transforms[sel][3] += delta.y * speed;
                    }
                }
                ToolMode::Scale => {
                    let factor = 1.0 + delta.x * self.settings.tools.scale_speed;
                    let min_s = self.settings.tools.min_scale;
                    self.transforms[sel][6] = (self.transforms[sel][6] * factor).max(min_s);
                    self.transforms[sel][7] = (self.transforms[sel][7] * factor).max(min_s);
                    self.transforms[sel][8] = (self.transforms[sel][8] * factor).max(min_s);
                }
                ToolMode::Select => {} // handled below
            }
        }

        // S key: Box select mode
        // Hold S + left-drag to draw selection box, release either to apply
        let s_held = ctx.input(|i| i.key_down(egui::Key::S));
        self.box_select_key_held = s_held;

        if s_held && response.dragged_by(PointerButton::Primary) {
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
            // Snap to grid on release — only snap axes that were moved
            if self.settings.snap.enabled {
                if self.tool_mode == ToolMode::Move {
                    let s = self.settings.snap.size;
                    self.transforms[sel][0] = (self.transforms[sel][0] / s).round() * s;
                    self.transforms[sel][2] = (self.transforms[sel][2] / s).round() * s;
                    if modifiers.shift || modifiers.ctrl {
                        self.transforms[sel][1] = (self.transforms[sel][1] / s).round() * s;
                    }
                } else if self.tool_mode == ToolMode::Rotate {
                    let r = self.settings.snap.rotation_degrees;
                    self.transforms[sel][3] = (self.transforms[sel][3] / r).round() * r;
                    self.transforms[sel][4] = (self.transforms[sel][4] / r).round() * r;
                    self.transforms[sel][5] = (self.transforms[sel][5] / r).round() * r;
                } else if self.tool_mode == ToolMode::Scale {
                    let si = self.settings.snap.scale_increment;
                    self.transforms[sel][6] = (self.transforms[sel][6] / si).round() * si;
                    self.transforms[sel][7] = (self.transforms[sel][7] / si).round() * si;
                    self.transforms[sel][8] = (self.transforms[sel][8] / si).round() * si;
                }
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

        // Box select completes when S or left mouse is released
        let box_finished = (response.drag_stopped_by(PointerButton::Primary) || !s_held)
            && self.box_select_start.is_some();
        if box_finished {
            if let (Some(start), Some(end)) =
                (self.box_select_start, self.box_select_end)
            {
                let dist = start.distance(end);
                if dist > self.settings.selection.box_select_threshold {
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
