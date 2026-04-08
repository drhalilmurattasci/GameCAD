//! Icosphere primitive generator.
//!
//! Generates a sphere by recursively subdividing an icosahedron. This produces
//! a much more uniform triangle distribution than a UV sphere, avoiding the
//! pole singularity artifacts.

use std::collections::HashMap;
use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use crate::half_edge::EditMesh;

/// Generate an icosphere by subdividing an icosahedron.
///
/// * `radius` - sphere radius
/// * `subdivisions` - number of recursive subdivisions (0 = icosahedron, 1 = 80 faces,
///   2 = 320 faces, 3 = 1280 faces)
pub fn generate_icosphere(radius: f32, subdivisions: u32) -> EditMesh {
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;

    // Initial icosahedron vertices (normalized to unit sphere)
    let mut positions: Vec<Vec3> = vec![
        Vec3::new(-1.0, t, 0.0).normalize(),
        Vec3::new(1.0, t, 0.0).normalize(),
        Vec3::new(-1.0, -t, 0.0).normalize(),
        Vec3::new(1.0, -t, 0.0).normalize(),
        Vec3::new(0.0, -1.0, t).normalize(),
        Vec3::new(0.0, 1.0, t).normalize(),
        Vec3::new(0.0, -1.0, -t).normalize(),
        Vec3::new(0.0, 1.0, -t).normalize(),
        Vec3::new(t, 0.0, -1.0).normalize(),
        Vec3::new(t, 0.0, 1.0).normalize(),
        Vec3::new(-t, 0.0, -1.0).normalize(),
        Vec3::new(-t, 0.0, 1.0).normalize(),
    ];

    // Initial icosahedron faces (20 triangles)
    let mut indices: Vec<[u32; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    // Subdivision: split each triangle into 4 by inserting midpoints
    for _ in 0..subdivisions {
        let mut midpoint_cache: HashMap<(u32, u32), u32> = HashMap::new();
        let mut new_indices = Vec::with_capacity(indices.len() * 4);

        for tri in &indices {
            let a = tri[0];
            let b = tri[1];
            let c = tri[2];

            let ab = get_midpoint(a, b, &mut positions, &mut midpoint_cache);
            let bc = get_midpoint(b, c, &mut positions, &mut midpoint_cache);
            let ca = get_midpoint(c, a, &mut positions, &mut midpoint_cache);

            new_indices.push([a, ab, ca]);
            new_indices.push([b, bc, ab]);
            new_indices.push([c, ca, bc]);
            new_indices.push([ab, bc, ca]);
        }

        indices = new_indices;
    }

    // Scale to radius and compute normals + UVs
    let normals: Vec<Vec3> = positions.iter().map(|p| p.normalize()).collect();
    let scaled_positions: Vec<Vec3> = positions.iter().map(|p| p.normalize() * radius).collect();
    let uvs: Vec<Vec2> = normals
        .iter()
        .map(|n| {
            let u = 0.5 + n.z.atan2(n.x) / (2.0 * PI);
            let v = 0.5 - n.y.asin() / PI;
            Vec2::new(u, v)
        })
        .collect();

    // Flatten to indexed triangle list
    let flat_indices: Vec<u32> = indices.iter().flat_map(|t| t.iter().copied()).collect();

    EditMesh::from_triangles(&scaled_positions, &normals, &uvs, &flat_indices)
}

/// Get or create the midpoint vertex between two vertices.
/// The midpoint is normalized to lie on the unit sphere.
fn get_midpoint(
    a: u32,
    b: u32,
    positions: &mut Vec<Vec3>,
    cache: &mut HashMap<(u32, u32), u32>,
) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }

    let mid = (positions[a as usize] + positions[b as usize]).normalize();
    let idx = positions.len() as u32;
    positions.push(mid);
    cache.insert(key, idx);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icosphere_base_is_icosahedron() {
        let mesh = generate_icosphere(1.0, 0);
        assert_eq!(mesh.vertex_count(), 12);
        assert_eq!(mesh.face_count(), 20);
    }

    #[test]
    fn icosphere_subdivision_1() {
        let mesh = generate_icosphere(1.0, 1);
        assert_eq!(mesh.face_count(), 80);
    }

    #[test]
    fn icosphere_subdivision_2() {
        let mesh = generate_icosphere(1.0, 2);
        assert_eq!(mesh.face_count(), 320);
    }

    #[test]
    fn icosphere_subdivision_3() {
        let mesh = generate_icosphere(1.0, 3);
        assert_eq!(mesh.face_count(), 1280);
    }

    #[test]
    fn icosphere_vertices_on_sphere_surface() {
        let radius = 2.5;
        let mesh = generate_icosphere(radius, 2);
        for v in &mesh.vertices {
            let dist = v.position.length();
            assert!(
                (dist - radius).abs() < 0.01,
                "Vertex at distance {dist}, expected {radius}"
            );
        }
    }

    #[test]
    fn icosphere_normals_are_unit_length() {
        let mesh = generate_icosphere(1.0, 2);
        for v in &mesh.vertices {
            let len = v.normal.length();
            assert!(
                (len - 1.0).abs() < 0.02,
                "Normal length {len}, expected 1.0"
            );
        }
    }

    #[test]
    fn icosphere_to_triangles_roundtrip() {
        let mesh = generate_icosphere(1.0, 2);
        let (positions, normals, uvs, indices) = mesh.to_triangles();
        assert!(!positions.is_empty());
        assert!(!indices.is_empty());
        assert_eq!(indices.len() % 3, 0);
        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
    }
}
