//! Per-face render style computation and icosphere geometry generation.
//!
//! `face_style` maps each `RenderStyle` variant into a fill colour and
//! stroke for a single polygon face.  `generate_icosphere_faces` produces
//! 80 triangles (one subdivision of a base icosahedron).

use egui::{Color32, Stroke};
use glam::Vec3;

use crate::state::ForgeEditorApp;
use crate::state::types::*;

impl ForgeEditorApp {
    /// Compute the fill colour and stroke for a single polygon face,
    /// taking the current render style into account.
    pub(crate) fn face_style(
        &self,
        normal: Vec3,
        base_color: Color32,
        wire_color: Color32,
        cam_pos: Vec3,
        face_center: Vec3,
        max_depth: f32,
    ) -> (Color32, Stroke) {
        let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
        match self.render_style {
            RenderStyle::Shaded => {
                let ndl = normal.dot(light_dir).max(0.15);
                let r = (base_color.r() as f32 * ndl).min(255.0) as u8;
                let g = (base_color.g() as f32 * ndl).min(255.0) as u8;
                let b = (base_color.b() as f32 * ndl).min(255.0) as u8;
                (Color32::from_rgb(r, g, b), Stroke::NONE)
            }
            RenderStyle::Wireframe => (
                Color32::TRANSPARENT,
                Stroke::new(1.2, wire_color),
            ),
            RenderStyle::ShadedWireframe => {
                let ndl = normal.dot(light_dir).max(0.15);
                let r = (base_color.r() as f32 * ndl).min(255.0) as u8;
                let g = (base_color.g() as f32 * ndl).min(255.0) as u8;
                let b = (base_color.b() as f32 * ndl).min(255.0) as u8;
                (
                    Color32::from_rgb(r, g, b),
                    Stroke::new(1.0, wire_color),
                )
            }
            RenderStyle::Unlit => (base_color, Stroke::NONE),
            RenderStyle::Ghost => {
                let r = base_color.r();
                let g = base_color.g();
                let b = base_color.b();
                (
                    Color32::from_rgba_premultiplied(
                        (r as f32 * 0.3) as u8,
                        (g as f32 * 0.3) as u8,
                        (b as f32 * 0.3) as u8,
                        76,
                    ),
                    Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(
                            (wire_color.r() as f32 * 0.5) as u8,
                            (wire_color.g() as f32 * 0.5) as u8,
                            (wire_color.b() as f32 * 0.5) as u8,
                            127,
                        ),
                    ),
                )
            }
            RenderStyle::Normals => {
                let nr = (normal.x.abs() * 255.0).min(255.0) as u8;
                let ng = (normal.y.abs() * 255.0).min(255.0) as u8;
                let nb = (normal.z.abs() * 255.0).min(255.0) as u8;
                (
                    Color32::from_rgb(nr, ng, nb),
                    Stroke::new(0.5, Color32::from_rgb(30, 30, 30)),
                )
            }
            RenderStyle::Depth => {
                let dist = (face_center - cam_pos).length();
                let t = (dist / max_depth).clamp(0.0, 1.0);
                let v = ((1.0 - t) * 255.0) as u8;
                (Color32::from_rgb(v, v, v), Stroke::NONE)
            }
            RenderStyle::Clay => {
                let ndl = normal.dot(light_dir).max(0.0);
                // Soft falloff: blend between shadow and highlight
                let ambient = 0.35_f32;
                let intensity = ambient + (1.0 - ambient) * ndl;
                let base_gray = 180.0_f32;
                let v = (base_gray * intensity).min(255.0) as u8;
                (Color32::from_rgb(v, v, v), Stroke::NONE)
            }
        }
    }

    /// Generate 80 icosphere triangle faces (one subdivision of an icosahedron).
    pub(crate) fn generate_icosphere_faces(center: Vec3, radius: f32) -> Vec<[Vec3; 3]> {
        // Base icosahedron vertices
        let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let raw = [
            Vec3::new(-1.0, t, 0.0),
            Vec3::new(1.0, t, 0.0),
            Vec3::new(-1.0, -t, 0.0),
            Vec3::new(1.0, -t, 0.0),
            Vec3::new(0.0, -1.0, t),
            Vec3::new(0.0, 1.0, t),
            Vec3::new(0.0, -1.0, -t),
            Vec3::new(0.0, 1.0, -t),
            Vec3::new(t, 0.0, -1.0),
            Vec3::new(t, 0.0, 1.0),
            Vec3::new(-t, 0.0, -1.0),
            Vec3::new(-t, 0.0, 1.0),
        ];
        let verts: Vec<Vec3> = raw.iter().map(|v| v.normalize()).collect();

        let base_tris: [(usize, usize, usize); 20] = [
            (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
            (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
            (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
            (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
        ];

        // One level of subdivision for ~80 triangles
        let mut subdivided: Vec<[Vec3; 3]> = Vec::with_capacity(80);
        for &(a, b, c) in &base_tris {
            let v0 = verts[a];
            let v1 = verts[b];
            let v2 = verts[c];
            let m01 = ((v0 + v1) * 0.5).normalize();
            let m12 = ((v1 + v2) * 0.5).normalize();
            let m20 = ((v2 + v0) * 0.5).normalize();
            subdivided.push([v0, m01, m20]);
            subdivided.push([m01, v1, m12]);
            subdivided.push([m20, m12, v2]);
            subdivided.push([m01, m12, m20]);
        }

        // Scale and translate
        subdivided
            .iter()
            .map(|tri| {
                [
                    center + tri[0] * radius,
                    center + tri[1] * radius,
                    center + tri[2] * radius,
                ]
            })
            .collect()
    }
}
