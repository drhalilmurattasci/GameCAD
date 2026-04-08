//! 3D scene object rendering via painter-based projection.
//!
//! Draws cubes, spheres (icosphere), lights (sun icon + direction arrow),
//! cameras (rectangle + lens), and generic entities (diamond).  Supports
//! per-face backface culling, depth sorting (back-to-front), render-style
//! shading, and yellow wireframe highlights for selected entities.

use eframe::egui;
use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2,
};
use glam::{Mat4, Vec3};

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    /// Project and draw all scene entities (meshes, lights, cameras, generics).
    pub(crate) fn draw_projected_objects(
        &self,
        painter: &egui::Painter,
        vp: &Mat4,
        rect: &Rect,
        wire_color: Color32,
        cam_pos: Vec3,
    ) {
        let names = self.flatten_outliner_names();
        let entity_count = names.len().min(self.transforms.len());
        let style = self.render_style;
        let is_ghost = style == RenderStyle::Ghost;

        // Max depth for depth mode (use camera distance * 2 as a reasonable max)
        let max_depth = self.orbit_camera.distance * 3.0;

        #[allow(clippy::needless_range_loop)]
        for idx in 1..entity_count {
            if self.is_entity_hidden(idx) {
                continue;
            }
            let pos = Vec3::new(
                self.transforms[idx][0],
                self.transforms[idx][1],
                self.transforms[idx][2],
            );
            let scale = self.transforms[idx][6];
            let rot_x = self.transforms[idx][3].to_radians();
            let rot_y = self.transforms[idx][4].to_radians();
            let rot_z = self.transforms[idx][5].to_radians();
            let rot_mat = Mat4::from_euler(glam::EulerRot::XYZ, rot_x, rot_y, rot_z);
            let is_selected = self.selected_entities.contains(&idx);

            if self.is_mesh_entity(idx) {
                let is_sphere = names[idx].to_lowercase().contains("sphere");

                // Base color for the object
                let base_color = if is_sphere {
                    let sec = self.theme_manager.current_theme().secondary;
                    if is_selected {
                        sec
                    } else {
                        sec.linear_multiply(0.85)
                    }
                } else if is_selected {
                    wire_color
                } else {
                    wire_color.linear_multiply(0.85)
                };

                if is_sphere {
                    // --- Sphere: icosphere solid faces ---
                    // Generate at origin, then rotate + translate
                    let raw_faces = Self::generate_icosphere_faces(Vec3::ZERO, scale);
                    let faces: Vec<[Vec3; 3]> = raw_faces
                        .iter()
                        .map(|tri| [
                            pos + rot_mat.transform_vector3(tri[0]),
                            pos + rot_mat.transform_vector3(tri[1]),
                            pos + rot_mat.transform_vector3(tri[2]),
                        ])
                        .collect();
                    // Collect faces with depth for sorting (back-to-front)
                    let mut face_data: Vec<(f32, [Pos2; 3], Vec3)> = Vec::new();
                    for tri in &faces {
                        let edge1 = tri[1] - tri[0];
                        let edge2 = tri[2] - tri[0];
                        let normal = edge1.cross(edge2).normalize();
                        let center = (tri[0] + tri[1] + tri[2]) / 3.0;
                        let to_cam = (cam_pos - center).normalize();

                        // Backface culling (skip for Ghost mode)
                        if !is_ghost && normal.dot(to_cam) < 0.0 {
                            continue;
                        }

                        if let (Some(p0), Some(p1), Some(p2)) = (
                            Self::project_3d(vp, rect, tri[0]),
                            Self::project_3d(vp, rect, tri[1]),
                            Self::project_3d(vp, rect, tri[2]),
                        ) {
                            let depth = (center - cam_pos).length();
                            face_data.push((depth, [p0, p1, p2], normal));
                        }
                    }
                    // Sort back-to-front
                    face_data.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

                    for (_depth, pts, normal) in &face_data {
                        let (fill, stroke) = self.face_style(
                            *normal,
                            base_color,
                            wire_color,
                            cam_pos,
                            pos, // approximate face center for depth shading
                            max_depth,
                        );
                        painter.add(egui::Shape::convex_polygon(
                            pts.to_vec(),
                            fill,
                            stroke,
                        ));
                    }

                    // Label
                    if let Some(lp) = Self::project_3d(
                        vp,
                        rect,
                        pos + Vec3::new(0.0, scale + 0.3, 0.0),
                    ) {
                        let label_color = if is_selected {
                            self.theme_manager.current_theme().secondary
                        } else {
                            self.theme_manager.current_theme().secondary.linear_multiply(0.7)
                        };
                        painter.text(
                            lp,
                            Align2::CENTER_BOTTOM,
                            &names[idx],
                            FontId::proportional(10.0),
                            label_color,
                        );
                    }
                } else {
                    // --- Cube: 6 filled quad faces ---
                    let s = scale;
                    let offsets = [
                        Vec3::new(-s, -s, -s), // 0
                        Vec3::new(s, -s, -s),  // 1
                        Vec3::new(s, s, -s),   // 2
                        Vec3::new(-s, s, -s),  // 3
                        Vec3::new(-s, -s, s),  // 4
                        Vec3::new(s, -s, s),   // 5
                        Vec3::new(s, s, s),    // 6
                        Vec3::new(-s, s, s),   // 7
                    ];
                    let cube_verts = offsets.map(|o| {
                        pos + rot_mat.transform_vector3(o)
                    });
                    // 6 faces: each is 4 vertex indices + outward normal (rotated)
                    let cube_faces: [([usize; 4], Vec3); 6] = [
                        ([0, 1, 2, 3], rot_mat.transform_vector3(Vec3::new(0.0, 0.0, -1.0))),
                        ([5, 4, 7, 6], rot_mat.transform_vector3(Vec3::new(0.0, 0.0, 1.0))),
                        ([4, 0, 3, 7], rot_mat.transform_vector3(Vec3::new(-1.0, 0.0, 0.0))),
                        ([1, 5, 6, 2], rot_mat.transform_vector3(Vec3::new(1.0, 0.0, 0.0))),
                        ([3, 2, 6, 7], rot_mat.transform_vector3(Vec3::new(0.0, 1.0, 0.0))),
                        ([4, 5, 1, 0], rot_mat.transform_vector3(Vec3::new(0.0, -1.0, 0.0))),
                    ];

                    // Collect faces with depth for sorting
                    let mut face_data: Vec<(f32, Vec<Pos2>, Vec3)> = Vec::new();
                    for (indices, normal) in &cube_faces {
                        let to_cam = (cam_pos - pos).normalize();
                        // Backface culling (skip for Ghost mode)
                        if !is_ghost && normal.dot(to_cam) < 0.0 {
                            continue;
                        }
                        let projected: Vec<Pos2> = indices
                            .iter()
                            .filter_map(|&i| Self::project_3d(vp, rect, cube_verts[i]))
                            .collect();
                        if projected.len() == 4 {
                            let center_3d = indices
                                .iter()
                                .fold(Vec3::ZERO, |acc, &i| acc + cube_verts[i])
                                / 4.0;
                            let depth = (center_3d - cam_pos).length();
                            face_data.push((depth, projected, *normal));
                        }
                    }
                    // Sort back-to-front
                    face_data.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

                    for (_depth, pts, normal) in &face_data {
                        let (fill, stroke) =
                            self.face_style(*normal, base_color, wire_color, cam_pos, pos, max_depth);
                        painter.add(egui::Shape::convex_polygon(
                            pts.clone(),
                            fill,
                            stroke,
                        ));
                    }

                    // Label (above the rotated top)
                    let label_offset = rot_mat.transform_vector3(Vec3::new(0.0, s + 0.3, 0.0));
                    if let Some(lp) =
                        Self::project_3d(vp, rect, pos + label_offset)
                    {
                        let cube_color = if is_selected {
                            wire_color
                        } else {
                            wire_color.linear_multiply(0.7)
                        };
                        painter.text(
                            lp,
                            Align2::CENTER_BOTTOM,
                            &names[idx],
                            FontId::proportional(10.0),
                            cube_color,
                        );
                    }
                }

                // Yellow wireframe highlight for selected entities
                if is_selected {
                    let yellow = Color32::from_rgb(255, 255, 0);
                    let yellow_stroke = Stroke::new(2.0, yellow);
                    if is_sphere {
                        // Draw 3 great circles (equator + 2 meridians) in yellow, rotated
                        let segments = 32;
                        for circle in 0..3 {
                            for s_idx in 0..segments {
                                let t0 = s_idx as f32 / segments as f32 * std::f32::consts::TAU;
                                let t1 = (s_idx + 1) as f32 / segments as f32 * std::f32::consts::TAU;
                                let (off_a, off_b) = match circle {
                                    0 => (
                                        Vec3::new(t0.cos() * scale, 0.0, t0.sin() * scale),
                                        Vec3::new(t1.cos() * scale, 0.0, t1.sin() * scale),
                                    ),
                                    1 => (
                                        Vec3::new(t0.cos() * scale, t0.sin() * scale, 0.0),
                                        Vec3::new(t1.cos() * scale, t1.sin() * scale, 0.0),
                                    ),
                                    _ => (
                                        Vec3::new(0.0, t0.cos() * scale, t0.sin() * scale),
                                        Vec3::new(0.0, t1.cos() * scale, t1.sin() * scale),
                                    ),
                                };
                                let a = pos + rot_mat.transform_vector3(off_a);
                                let b = pos + rot_mat.transform_vector3(off_b);
                                Self::draw_line_3d(painter, vp, rect, a, b, yellow_stroke);
                            }
                        }
                    } else {
                        // Cube: draw all 12 edges in yellow (rotated)
                        let s = scale;
                        let sel_offsets = [
                            Vec3::new(-s, -s, -s), Vec3::new(s, -s, -s),
                            Vec3::new(s, s, -s),   Vec3::new(-s, s, -s),
                            Vec3::new(-s, -s, s),  Vec3::new(s, -s, s),
                            Vec3::new(s, s, s),    Vec3::new(-s, s, s),
                        ];
                        let cv = sel_offsets.map(|o| pos + rot_mat.transform_vector3(o));
                        let edges = [
                            (0,1),(1,2),(2,3),(3,0),
                            (4,5),(5,6),(6,7),(7,4),
                            (0,4),(1,5),(2,6),(3,7),
                        ];
                        for (a, b) in edges {
                            Self::draw_line_3d(painter, vp, rect, cv[a], cv[b], yellow_stroke);
                        }
                    }
                }
            } else if self.is_light_entity(idx) {
                // Directional Light icon
                if let Some(lp) = Self::project_3d(vp, rect, pos) {
                    let sun_color = Color32::from_rgb(0xff, 0xf4, 0xd6);
                    painter.circle_filled(lp, 8.0, sun_color);
                    for i in 0..8 {
                        let angle = i as f32 * std::f32::consts::TAU / 8.0;
                        painter.line_segment(
                            [
                                Pos2::new(
                                    lp.x + 11.0 * angle.cos(),
                                    lp.y + 11.0 * angle.sin(),
                                ),
                                Pos2::new(
                                    lp.x + 17.0 * angle.cos(),
                                    lp.y + 17.0 * angle.sin(),
                                ),
                            ],
                            Stroke::new(1.5, sun_color),
                        );
                    }
                    if let Some(tp) = Self::project_3d(vp, rect, Vec3::ZERO) {
                        let dx = tp.x - lp.x;
                        let dy = tp.y - lp.y;
                        let len = (dx * dx + dy * dy).sqrt();
                        if len > 20.0 {
                            let arrow_end = Pos2::new(
                                lp.x + dx / len * 30.0,
                                lp.y + dy / len * 30.0,
                            );
                            painter.line_segment(
                                [lp, arrow_end],
                                Stroke::new(
                                    1.5,
                                    Color32::from_rgba_premultiplied(0xff, 0xf4, 0xd6, 100),
                                ),
                            );
                        }
                    }
                    painter.text(
                        Pos2::new(lp.x, lp.y - 22.0),
                        Align2::CENTER_BOTTOM,
                        &names[idx],
                        FontId::proportional(10.0),
                        tc!(self, text_dim),
                    );
                    // Yellow wireframe highlight for selected light
                    if is_selected {
                        let yellow = Color32::from_rgb(255, 255, 0);
                        painter.circle_stroke(
                            lp,
                            22.0,
                            Stroke::new(2.0, yellow),
                        );
                    }
                }
            } else if self.is_camera_entity(idx) {
                // Camera icon
                if let Some(cp) = Self::project_3d(vp, rect, pos) {
                    let cam_color = if is_selected {
                        tc!(self, text)
                    } else {
                        tc!(self, text_dim)
                    };
                    let cr = Rect::from_center_size(cp, Vec2::new(24.0, 16.0));
                    painter.rect_stroke(
                        cr,
                        CornerRadius::same(2),
                        Stroke::new(1.5, cam_color),
                        StrokeKind::Outside,
                    );
                    painter.circle_stroke(
                        Pos2::new(cr.right() + 6.0, cp.y),
                        5.0,
                        Stroke::new(1.5, cam_color),
                    );
                    painter.text(
                        Pos2::new(cp.x, cp.y + 16.0),
                        Align2::CENTER_TOP,
                        &names[idx],
                        FontId::proportional(10.0),
                        cam_color,
                    );
                    // Yellow wireframe highlight for selected camera
                    if is_selected {
                        let yellow = Color32::from_rgb(255, 255, 0);
                        painter.circle_stroke(
                            cp,
                            20.0,
                            Stroke::new(2.0, yellow),
                        );
                    }
                }
            } else {
                // Generic entity - draw a small diamond
                if let Some(sp) = Self::project_3d(vp, rect, pos) {
                    let size = 6.0;
                    let color = if is_selected {
                        tc!(self, accent)
                    } else {
                        tc!(self, text_dim)
                    };
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
}
