//! Cube primitive generator.

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate a cube centered at the origin.
pub fn generate_cube(size: f32) -> EditMesh {
    let h = size * 0.5;

    // 24 vertices (4 per face for independent normals/UVs).
    #[rustfmt::skip]
    let positions = vec![
        // Front face (+Z)
        Vec3::new(-h, -h,  h), Vec3::new( h, -h,  h), Vec3::new( h,  h,  h), Vec3::new(-h,  h,  h),
        // Back face (-Z)
        Vec3::new( h, -h, -h), Vec3::new(-h, -h, -h), Vec3::new(-h,  h, -h), Vec3::new( h,  h, -h),
        // Top face (+Y)
        Vec3::new(-h,  h,  h), Vec3::new( h,  h,  h), Vec3::new( h,  h, -h), Vec3::new(-h,  h, -h),
        // Bottom face (-Y)
        Vec3::new(-h, -h, -h), Vec3::new( h, -h, -h), Vec3::new( h, -h,  h), Vec3::new(-h, -h,  h),
        // Right face (+X)
        Vec3::new( h, -h,  h), Vec3::new( h, -h, -h), Vec3::new( h,  h, -h), Vec3::new( h,  h,  h),
        // Left face (-X)
        Vec3::new(-h, -h, -h), Vec3::new(-h, -h,  h), Vec3::new(-h,  h,  h), Vec3::new(-h,  h, -h),
    ];

    #[rustfmt::skip]
    let normals = vec![
        Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z,
        Vec3::NEG_Z, Vec3::NEG_Z, Vec3::NEG_Z, Vec3::NEG_Z,
        Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y,
        Vec3::NEG_Y, Vec3::NEG_Y, Vec3::NEG_Y, Vec3::NEG_Y,
        Vec3::X, Vec3::X, Vec3::X, Vec3::X,
        Vec3::NEG_X, Vec3::NEG_X, Vec3::NEG_X, Vec3::NEG_X,
    ];

    #[rustfmt::skip]
    let uvs = vec![
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
    ];

    // Two triangles per face, 6 faces.
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
         0,  1,  2,   0,  2,  3,  // front
         4,  5,  6,   4,  6,  7,  // back
         8,  9, 10,   8, 10, 11,  // top
        12, 13, 14,  12, 14, 15,  // bottom
        16, 17, 18,  16, 18, 19,  // right
        20, 21, 22,  20, 22, 23,  // left
    ];

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
