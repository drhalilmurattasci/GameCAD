//! Central viewport panel and its sub-modules.
//!
//! The viewport renders a 3D scene using either GPU-accelerated wgpu rendering
//! (with PBR shading, depth buffer, and grid shader) or a painter-based
//! fallback. HUD overlays (tab info, camera, shortcuts) are always painter-based.

pub(crate) mod axis_indicator;
pub(crate) mod camera_input;
pub(crate) mod context_menu;
pub(crate) mod gizmo;
pub(crate) mod gpu_mesh;
pub(crate) mod grid;
pub(crate) mod objects;
pub(crate) mod picking;
pub(crate) mod projection;
pub(crate) mod styles;

use eframe::egui;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect,
    Sense, Stroke, StrokeKind,
};

use crate::state::ForgeEditorApp;

impl ForgeEditorApp {
    /// Draw the central viewport panel with the 3D scene.
    pub(crate) fn draw_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(tc!(self, bg))
                    .inner_margin(Margin::same(0)),
            )
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();

                // Allocate interactive area for mouse input
                let response = ui.allocate_rect(rect, Sense::click_and_drag());
                let painter = ui.painter_at(rect);

                // ---- Read input state ----
                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

                // ---- Mouse controls (Unreal Engine style) ----
                self.handle_camera_input(ctx, &response, pointer_pos);

                // ---- Right-click context menu ----
                self.draw_context_menu(&response);

                // ---- GPU Rendering ----
                self.gpu_render_viewport(&painter, &rect);

                // ---- View-projection for overlays ----
                let aspect = if rect.height() > 0.0 {
                    rect.width() / rect.height()
                } else {
                    16.0 / 9.0
                };
                let view = self.orbit_camera.view_matrix();
                let proj = self.orbit_camera.projection_matrix(aspect);
                let vp = proj * view;

                // ---- Tool gizmo ----
                self.draw_tool_gizmo(&painter, &vp, &rect);

                // ---- Axis gizmo ----
                Self::draw_axis_gizmo(&painter, &view, &rect);

                // ---- Box selection ----
                if let (Some(start), Some(end)) = (self.box_select_start, self.box_select_end) {
                    let sel_rect = Rect::from_two_pos(start, end);
                    painter.rect_filled(
                        sel_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_premultiplied(0x4e, 0xff, 0x93, 20),
                    );
                    painter.rect_stroke(
                        sel_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, tc!(self, accent).linear_multiply(0.6)),
                        StrokeKind::Outside,
                    );
                }

                // ---- HUD Overlays ----
                self.draw_hud_overlays(&painter, &rect);
            });
    }

    /// Render the 3D viewport using GPU (wgpu) or fallback painter.
    fn gpu_render_viewport(&mut self, painter: &egui::Painter, rect: &Rect) {
        if self.gpu.is_none() {
            // Fallback: painter-based rendering
            self.draw_painter_viewport(painter, rect);
            return;
        }

        let w = (rect.width() as u32).max(1);
        let h = (rect.height() as u32).max(1);

        // Collect camera/light/style data while self is immutably borrowed
        let cam_eye = self.orbit_camera.position();
        let cam_target = self.orbit_camera.target;
        let cam_fov = self.orbit_camera.fov;
        let cam_near = self.orbit_camera.near;
        let cam_far = self.orbit_camera.far;
        let render_style_mapped = match self.render_style {
            crate::state::types::RenderStyle::Shaded => forge_render::RenderStyle::Pbr,
            crate::state::types::RenderStyle::Wireframe => forge_render::RenderStyle::Wireframe,
            crate::state::types::RenderStyle::Unlit
            | crate::state::types::RenderStyle::Clay => forge_render::RenderStyle::Unlit,
            crate::state::types::RenderStyle::Normals => forge_render::RenderStyle::Normals,
            crate::state::types::RenderStyle::Depth => forge_render::RenderStyle::Depth,
            _ => forge_render::RenderStyle::Pbr,
        };
        let grid_visible = self.settings.grid.visible;

        let gpu = self.gpu.as_mut().unwrap();

        // Resize if viewport changed
        if gpu.viewport_size != (w, h) {
            gpu.renderer.resize(&gpu.device, w, h);
            gpu.offscreen_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("viewport_offscreen"),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: gpu.renderer.surface_format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            gpu.offscreen_view = gpu.offscreen_texture.create_view(
                &wgpu::TextureViewDescriptor::default(),
            );
            gpu.viewport_size = (w, h);
        }

        // Sync camera
        let aspect = w as f32 / h as f32;
        let render_camera = forge_render::Camera {
            eye: cam_eye,
            target: cam_target,
            up: glam::Vec3::Y,
            projection: forge_render::Projection::Perspective {
                fov_y_radians: cam_fov,
                near: cam_near,
                far: cam_far,
            },
            aspect,
        };
        gpu.renderer.update_camera(&gpu.queue, &render_camera);

        // Sync lights
        let lights = forge_render::LightSet::default();
        gpu.renderer.update_lights(&gpu.queue, &lights);

        // Sync render style
        gpu.renderer.render_style = render_style_mapped;
        gpu.renderer.show_grid = grid_visible;

        // Collect mesh data before mutable GPU borrow
        // (flatten_outliner_names and is_mesh_entity borrow self immutably)
        drop(gpu); // release mutable borrow temporarily

        let names = self.flatten_outliner_names();
        let entity_count = names.len().min(self.transforms.len());
        let mesh_entries: Vec<_> = self.meshes.values().cloned().collect();

        // Collect per-entity rendering info
        struct MeshRenderInfo {
            pos: glam::Vec3,
            rot: glam::Vec3,
            scale: glam::Vec3,
            color: [f32; 4],
            mesh_idx: usize,
        }
        let mut render_infos = Vec::new();
        let mut m_idx = 0;
        for idx in 1..entity_count {
            if !self.is_mesh_entity(idx) {
                continue;
            }
            if self.is_entity_hidden(idx) {
                m_idx += 1;
                continue;
            }
            if m_idx >= mesh_entries.len() {
                break;
            }
            let is_selected = self.selected_entities.contains(&idx);
            let color = if is_selected {
                [0.9, 0.9, 0.3, 1.0]
            } else if names[idx].to_lowercase().contains("sphere") {
                [0.3, 0.6, 0.9, 1.0]
            } else {
                [0.7, 0.7, 0.7, 1.0]
            };
            render_infos.push(MeshRenderInfo {
                pos: glam::Vec3::new(self.transforms[idx][0], self.transforms[idx][1], self.transforms[idx][2]),
                rot: glam::Vec3::new(self.transforms[idx][3], self.transforms[idx][4], self.transforms[idx][5]),
                scale: glam::Vec3::new(self.transforms[idx][6], self.transforms[idx][7], self.transforms[idx][8]),
                color,
                mesh_idx: m_idx,
            });
            m_idx += 1;
        }

        // Re-acquire mutable borrow and build GPU meshes
        let gpu = self.gpu.as_mut().unwrap();
        gpu.gpu_meshes.clear();
        for info in &render_infos {
            let edit_mesh = &mesh_entries[info.mesh_idx];
            let gpu_mesh = gpu_mesh::editmesh_to_gpu(
                &gpu.device, edit_mesh, info.pos, info.rot, info.scale, info.color,
            );
            gpu.gpu_meshes.push(gpu_mesh);
        }

        // Render to offscreen texture
        let mesh_refs: Vec<&forge_render::GpuMesh> = gpu.gpu_meshes.iter().collect();
        gpu.renderer.render_frame(
            &gpu.device,
            &gpu.queue,
            &gpu.offscreen_view,
            &mesh_refs,
        );

        // Read back pixels from GPU texture for egui display
        // (For production, use PaintCallback - this is a working initial approach)
        let row_bytes = w * 4;
        let padded_row = ((row_bytes + 255) / 256) * 256;
        let buf_size = (padded_row * h) as u64;

        let staging_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: buf_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("readback_encoder") },
        );
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu.offscreen_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        gpu.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer synchronously
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        gpu.device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = rx.recv() {
            let data = buffer_slice.get_mapped_range();
            let mut pixels = Vec::with_capacity((w * h) as usize);
            for row in 0..h {
                let offset = (row * padded_row) as usize;
                for col in 0..w {
                    let px = offset + (col * 4) as usize;
                    // BGRA → RGBA (wgpu surface format is typically Bgra8UnormSrgb)
                    let b = data[px];
                    let g = data[px + 1];
                    let r = data[px + 2];
                    let a = data[px + 3];
                    pixels.push(Color32::from_rgba_unmultiplied(r, g, b, a));
                }
            }
            drop(data);
            staging_buffer.unmap();

            let image = egui::ColorImage {
                size: [w as usize, h as usize],
                pixels,
            };
            let texture = painter.ctx().load_texture(
                "gpu_viewport",
                image,
                egui::TextureOptions::LINEAR,
            );
            painter.image(
                texture.id(),
                *rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Draw non-mesh entities (lights, cameras) as painter overlays
        let vp = {
            let view = self.orbit_camera.view_matrix();
            let proj = self.orbit_camera.projection_matrix(aspect);
            proj * view
        };
        let cam_pos = self.orbit_camera.position();
        self.draw_non_mesh_entities(painter, &vp, rect, cam_pos);
    }

    /// Fallback painter-based viewport rendering (no GPU).
    fn draw_painter_viewport(&self, painter: &egui::Painter, rect: &Rect) {
        let gradient = self.theme_manager.viewport_gradient();
        let bands = gradient.len();
        for (i, rgb) in gradient.iter().enumerate() {
            let t0 = i as f32 / bands as f32;
            let t1 = (i + 1) as f32 / bands as f32;
            let c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            let band_rect = Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + t0 * rect.height()),
                Pos2::new(rect.right(), rect.top() + t1 * rect.height()),
            );
            painter.rect_filled(band_rect, CornerRadius::ZERO, c);
        }

        let aspect = if rect.height() > 0.0 { rect.width() / rect.height() } else { 16.0 / 9.0 };
        let view = self.orbit_camera.view_matrix();
        let proj = self.orbit_camera.projection_matrix(aspect);
        let vp = proj * view;
        let wire_c = self.theme_manager.wireframe_color();

        if self.settings.grid.visible {
            let grid_c = self.theme_manager.grid_color();
            let grid_mc = self.theme_manager.grid_major_color();
            Self::draw_perspective_grid(painter, &vp, rect, grid_c, grid_mc, self.settings.grid.size);
        }

        let cam_pos = self.orbit_camera.position();
        self.draw_projected_objects(painter, &vp, rect, wire_c, cam_pos);
    }

    /// Draw non-mesh entities (lights, cameras, generics) as painter overlays.
    fn draw_non_mesh_entities(
        &self,
        painter: &egui::Painter,
        vp: &glam::Mat4,
        rect: &Rect,
        _cam_pos: glam::Vec3,
    ) {
        let names = self.flatten_outliner_names();
        let entity_count = names.len().min(self.transforms.len());

        for idx in 1..entity_count {
            if self.is_entity_hidden(idx) {
                continue;
            }
            let pos = glam::Vec3::new(
                self.transforms[idx][0],
                self.transforms[idx][1],
                self.transforms[idx][2],
            );
            let is_selected = self.selected_entities.contains(&idx);

            if self.is_light_entity(idx) {
                if let Some(lp) = Self::project_3d(vp, rect, pos) {
                    let sun_color = Color32::from_rgb(0xff, 0xf4, 0xd6);
                    painter.circle_filled(lp, 8.0, sun_color);
                    for i in 0..8 {
                        let angle = i as f32 * std::f32::consts::TAU / 8.0;
                        painter.line_segment(
                            [
                                Pos2::new(lp.x + 11.0 * angle.cos(), lp.y + 11.0 * angle.sin()),
                                Pos2::new(lp.x + 17.0 * angle.cos(), lp.y + 17.0 * angle.sin()),
                            ],
                            Stroke::new(1.5, sun_color),
                        );
                    }
                    painter.text(
                        Pos2::new(lp.x, lp.y - 22.0),
                        Align2::CENTER_BOTTOM,
                        &names[idx],
                        FontId::proportional(10.0),
                        tc!(self, text_dim),
                    );
                    if is_selected {
                        painter.circle_stroke(lp, 22.0, Stroke::new(2.0, Color32::from_rgb(255, 255, 0)));
                    }
                }
            } else if self.is_camera_entity(idx) {
                if let Some(cp) = Self::project_3d(vp, rect, pos) {
                    let cam_color = if is_selected { tc!(self, text) } else { tc!(self, text_dim) };
                    let cr = Rect::from_center_size(cp, egui::Vec2::new(24.0, 16.0));
                    painter.rect_stroke(cr, CornerRadius::same(2), Stroke::new(1.5, cam_color), StrokeKind::Outside);
                    painter.circle_stroke(Pos2::new(cr.right() + 6.0, cp.y), 5.0, Stroke::new(1.5, cam_color));
                    painter.text(
                        Pos2::new(cp.x, cp.y + 16.0),
                        Align2::CENTER_TOP,
                        &names[idx],
                        FontId::proportional(10.0),
                        cam_color,
                    );
                    if is_selected {
                        painter.circle_stroke(cp, 20.0, Stroke::new(2.0, Color32::from_rgb(255, 255, 0)));
                    }
                }
            } else if !self.is_mesh_entity(idx) {
                if let Some(sp) = Self::project_3d(vp, rect, pos) {
                    let color = if is_selected { tc!(self, accent) } else { tc!(self, text_dim) };
                    let size = 6.0;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            Pos2::new(sp.x, sp.y - size),
                            Pos2::new(sp.x + size, sp.y),
                            Pos2::new(sp.x, sp.y + size),
                            Pos2::new(sp.x - size, sp.y),
                        ],
                        color,
                        Stroke::NONE,
                    ));
                    painter.text(
                        Pos2::new(sp.x, sp.y + size + 4.0),
                        Align2::CENTER_TOP,
                        &names[idx],
                        FontId::proportional(10.0),
                        color,
                    );
                }
            }
        }
    }

    /// Draw HUD text overlays on top of the viewport.
    fn draw_hud_overlays(&self, painter: &egui::Painter, rect: &Rect) {
        let grid_status = if self.settings.grid.visible { "Grid:ON" } else { "Grid:OFF" };
        let snap_status = if self.settings.snap.enabled {
            format!("Snap:{:.2}", self.settings.snap.size)
        } else {
            "Snap:OFF".to_string()
        };
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 12.0),
            Align2::LEFT_TOP,
            format!(
                "{} | {} | {} | {} | H:{:.1}m",
                self.active_tab.label(),
                self.render_style.label(),
                grid_status,
                snap_status,
                self.settings.height.level,
            ),
            FontId::proportional(13.0),
            tc!(self, accent),
        );

        if self.box_select_key_held {
            painter.text(
                Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
                Align2::LEFT_TOP,
                "S + Left-Drag: Box Select (release to apply)",
                FontId::proportional(11.0),
                tc!(self, accent),
            );
        } else if self.tool_mode == crate::state::types::ToolMode::Move && self.selected_entity > 0 {
            painter.text(
                Pos2::new(rect.left() + 12.0, rect.top() + 30.0),
                Align2::LEFT_TOP,
                "Drag: X+Z | Shift+Drag: Y only | Ctrl+Drag: X+Y",
                FontId::proportional(11.0),
                tc!(self, text_dim),
            );
        }

        // Camera info
        let cam = &self.orbit_camera;
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 46.0),
            Align2::LEFT_TOP,
            format!(
                "Yaw: {:.1}  Pitch: {:.1}  Dist: {:.1}  Target: ({:.1}, {:.1}, {:.1})",
                cam.yaw.to_degrees(), cam.pitch.to_degrees(), cam.distance,
                cam.target.x, cam.target.y, cam.target.z,
            ),
            FontId::proportional(10.0),
            Color32::from_rgba_premultiplied(0x9b, 0x9b, 0xa1, 160),
        );

        // Orbiting/Panning state
        let state_text = if self.is_orbiting { "Orbiting" }
            else if self.is_panning { "Panning" }
            else { "" };
        if !state_text.is_empty() {
            painter.text(
                Pos2::new(rect.left() + 12.0, rect.top() + 60.0),
                Align2::LEFT_TOP,
                state_text,
                FontId::proportional(11.0),
                Color32::from_rgb(0xff, 0xd7, 0x00),
            );
        }

        // GPU indicator
        if self.gpu.is_some() {
            painter.text(
                Pos2::new(rect.right() - 12.0, rect.bottom() - 12.0),
                Align2::RIGHT_BOTTOM,
                "GPU: wgpu PBR",
                FontId::proportional(10.0),
                Color32::from_rgb(0x4e, 0xff, 0x93),
            );
        }
    }
}
