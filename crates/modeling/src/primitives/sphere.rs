//! Sphere primitive generator.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a UV sphere centered at the origin.
pub fn generate_sphere(radius: f32, segments: u32, rings: u32) -> EditMesh {
    let segments = segments.max(3);
    let rings = rings.max(2);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices: rings+1 rows, segments+1 columns (for UV wrapping).
    for ring in 0..=rings {
        let phi = PI * ring as f32 / rings as f32;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for seg in 0..=segments {
            let theta = 2.0 * PI * seg as f32 / segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let normal = Vec3::new(sin_phi * cos_theta, cos_phi, sin_phi * sin_theta);
            let position = normal * radius;
            let uv = Vec2::new(seg as f32 / segments as f32, ring as f32 / rings as f32);

            positions.push(position);
            normals.push(normal);
            uvs.push(uv);
        }
    }

    // Generate indices.
    let stride = segments + 1;
    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * stride + seg;
            let b = a + stride;

            // Skip degenerate triangles at the poles.
            if ring != 0 {
                indices.push(a);
                indices.push(b);
                indices.push(a + 1);
            }
            if ring != rings - 1 {
                indices.push(a + 1);
                indices.push(b);
                indices.push(b + 1);
            }
        }
    }

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
