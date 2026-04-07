//! Cone primitive generator.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a cone centered at the origin with its apex along +Y.
pub fn generate_cone(radius: f32, height: f32, segments: u32) -> EditMesh {
    let segments = segments.max(3);
    let half_h = height * 0.5;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // --- Side ---
    // The slope angle for computing normals.
    let slope = radius / height;
    let ny = 1.0 / (1.0 + slope * slope).sqrt();
    let nxz = slope * ny;

    let apex_base = positions.len() as u32;
    // One apex vertex per segment for correct UVs.
    for i in 0..segments {
        let theta = 2.0 * PI * (i as f32 + 0.5) / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let normal = Vec3::new(nxz * cos_t, ny, nxz * sin_t);

        positions.push(Vec3::new(0.0, half_h, 0.0));
        normals.push(normal);
        uvs.push(Vec2::new((i as f32 + 0.5) / segments as f32, 1.0));
    }

    let ring_base = positions.len() as u32;
    for i in 0..=segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let normal = Vec3::new(nxz * cos_t, ny, nxz * sin_t);

        positions.push(Vec3::new(radius * cos_t, -half_h, radius * sin_t));
        normals.push(normal);
        uvs.push(Vec2::new(i as f32 / segments as f32, 0.0));
    }

    for i in 0..segments {
        indices.push(apex_base + i);
        indices.push(ring_base + i);
        indices.push(ring_base + i + 1);
    }

    // --- Bottom cap ---
    let bot_center = positions.len() as u32;
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
        indices.push(bot_center);
        indices.push(bot_ring_base + (i + 1) % segments);
        indices.push(bot_ring_base + i);
    }

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
