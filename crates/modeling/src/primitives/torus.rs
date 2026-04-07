//! Torus primitive generator.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a torus centered at the origin lying in the XZ plane.
pub fn generate_torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> EditMesh {
    let major_segments = major_segments.max(3);
    let minor_segments = minor_segments.max(3);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=major_segments {
        let theta = 2.0 * PI * i as f32 / major_segments as f32;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for j in 0..=minor_segments {
            let phi = 2.0 * PI * j as f32 / minor_segments as f32;
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            let x = (major_radius + minor_radius * cos_phi) * cos_theta;
            let y = minor_radius * sin_phi;
            let z = (major_radius + minor_radius * cos_phi) * sin_theta;

            let nx = cos_phi * cos_theta;
            let ny = sin_phi;
            let nz = cos_phi * sin_theta;

            positions.push(Vec3::new(x, y, z));
            normals.push(Vec3::new(nx, ny, nz).normalize_or_zero());
            uvs.push(Vec2::new(
                i as f32 / major_segments as f32,
                j as f32 / minor_segments as f32,
            ));
        }
    }

    let stride = minor_segments + 1;
    for i in 0..major_segments {
        for j in 0..minor_segments {
            let a = i * stride + j;
            let b = a + stride;

            indices.push(a);
            indices.push(b);
            indices.push(a + 1);

            indices.push(a + 1);
            indices.push(b);
            indices.push(b + 1);
        }
    }

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
