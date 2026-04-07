//! Plane primitive generator.

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a plane on the XZ plane centered at the origin with its normal pointing up (+Y).
pub fn generate_plane(width: f32, depth: f32) -> EditMesh {
    let hw = width * 0.5;
    let hd = depth * 0.5;

    let positions = vec![
        Vec3::new(-hw, 0.0, -hd),
        Vec3::new(hw, 0.0, -hd),
        Vec3::new(hw, 0.0, hd),
        Vec3::new(-hw, 0.0, hd),
    ];

    let normals = vec![Vec3::Y; 4];

    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
