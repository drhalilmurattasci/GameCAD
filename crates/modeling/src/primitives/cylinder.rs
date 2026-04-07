//! Cylinder primitive generator.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a cylinder centered at the origin with its axis along Y.
pub fn generate_cylinder(radius: f32, height: f32, segments: u32) -> EditMesh {
    let segments = segments.max(3);
    let half_h = height * 0.5;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // --- Side ---
    let side_base = positions.len() as u32;
    for i in 0..=segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let normal = Vec3::new(cos_t, 0.0, sin_t);
        let u = i as f32 / segments as f32;

        // Bottom ring.
        positions.push(Vec3::new(radius * cos_t, -half_h, radius * sin_t));
        normals.push(normal);
        uvs.push(Vec2::new(u, 0.0));

        // Top ring.
        positions.push(Vec3::new(radius * cos_t, half_h, radius * sin_t));
        normals.push(normal);
        uvs.push(Vec2::new(u, 1.0));
    }

    for i in 0..segments {
        let bl = side_base + i * 2;
        let br = side_base + (i + 1) * 2;
        let tl = bl + 1;
        let tr = br + 1;

        indices.push(bl);
        indices.push(br);
        indices.push(tr);

        indices.push(bl);
        indices.push(tr);
        indices.push(tl);
    }

    // --- Top cap ---
    let top_center_idx = positions.len() as u32;
    positions.push(Vec3::new(0.0, half_h, 0.0));
    normals.push(Vec3::Y);
    uvs.push(Vec2::new(0.5, 0.5));

    let top_ring_base = positions.len() as u32;
    for i in 0..segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        positions.push(Vec3::new(radius * cos_t, half_h, radius * sin_t));
        normals.push(Vec3::Y);
        uvs.push(Vec2::new(0.5 + 0.5 * cos_t, 0.5 + 0.5 * sin_t));
    }

    for i in 0..segments {
        indices.push(top_center_idx);
        indices.push(top_ring_base + i);
        indices.push(top_ring_base + (i + 1) % segments);
    }

    // --- Bottom cap ---
    let bot_center_idx = positions.len() as u32;
    positions.push(Vec3::new(0.0, -half_h, 0.0));
    normals.push(Vec3::NEG_Y);
    uvs.push(Vec2::new(0.5, 0.5));

    let bot_ring_base = positions.len() as u32;
    for i in 0..segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        positions.push(Vec3::new(radius * cos_t, -half_h, radius * sin_t));
        normals.push(Vec3::NEG_Y);
        uvs.push(Vec2::new(0.5 + 0.5 * cos_t, 0.5 + 0.5 * sin_t));
    }

    for i in 0..segments {
        indices.push(bot_center_idx);
        indices.push(bot_ring_base + (i + 1) % segments);
        indices.push(bot_ring_base + i);
    }

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
