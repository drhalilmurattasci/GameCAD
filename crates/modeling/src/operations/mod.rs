//! Mesh editing operations: extrude, inset, subdivide, merge, normals, transforms, delete.

mod delete;
mod extrude;
pub(crate) mod helpers;
mod modify;
mod subdivide;
mod transform;

pub use delete::delete_faces;
pub use extrude::{extrude_faces, inset_faces};
pub use modify::{flip_normals, merge_vertices, recalculate_normals};
pub use subdivide::subdivide;
pub use transform::{scale_vertices, translate_vertices};

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::half_edge::VertexId;
    use crate::primitives::{generate_cube, generate_plane};
    use glam::Vec3;

    #[test]
    fn flip_normals_reverses() {
        let mut mesh = generate_plane(1.0, 1.0);
        let original_normal = mesh.faces[0].normal;
        flip_normals(&mut mesh);
        let flipped = mesh.faces[0].normal;
        assert!((original_normal + flipped).length() < 1e-4);
    }

    #[test]
    fn recalculate_normals_works() {
        let mut mesh = generate_cube(1.0);
        // Zero out normals first.
        for v in &mut mesh.vertices {
            v.normal = Vec3::ZERO;
        }
        recalculate_normals(&mut mesh);
        // All vertex normals should be non-zero.
        for v in &mesh.vertices {
            assert!(v.normal.length() > 0.5);
        }
    }

    #[test]
    fn translate_vertices_works() {
        let mut mesh = generate_plane(1.0, 1.0);
        let ids: Vec<VertexId> = (0..mesh.vertex_count()).collect();
        let orig_y = mesh.vertices[0].position.y;
        translate_vertices(&mut mesh, &ids, Vec3::new(0.0, 5.0, 0.0));
        assert!((mesh.vertices[0].position.y - (orig_y + 5.0)).abs() < 1e-5);
    }

    #[test]
    fn scale_vertices_works() {
        let mut mesh = generate_cube(2.0);
        let ids: Vec<VertexId> = (0..mesh.vertex_count()).collect();
        scale_vertices(&mut mesh, &ids, Vec3::ZERO, Vec3::splat(2.0));
        // The cube was size 2, half-extent 1. After 2x scale, half-extent should be 2.
        let max_x = mesh
            .vertices
            .iter()
            .map(|v| v.position.x)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_x - 2.0).abs() < 1e-4);
    }

    #[test]
    fn delete_faces_reduces_count() {
        let mut mesh = generate_cube(1.0);
        let original = mesh.face_count();
        delete_faces(&mut mesh, &[0, 1]);
        assert_eq!(mesh.face_count(), original - 2);
    }

    #[test]
    fn extrude_faces_adds_geometry() {
        let mut mesh = generate_plane(1.0, 1.0);
        let original_verts = mesh.vertex_count();
        let result = extrude_faces(&mut mesh, &[0, 1], 1.0);
        assert!(result.is_ok());
        assert!(mesh.vertex_count() > original_verts);
    }

    #[test]
    fn subdivide_increases_face_count() {
        let mut mesh = generate_plane(1.0, 1.0);
        let original_faces = mesh.face_count();
        let result = subdivide(&mut mesh);
        assert!(result.is_ok());
        assert!(mesh.face_count() > original_faces);
    }

    #[test]
    fn merge_vertices_works() {
        let mut mesh = generate_cube(1.0);
        // Cube has 24 vertices (4 per face with duplicates for normals).
        // Many are at the same position. With a small threshold, several should merge.
        let merged = merge_vertices(&mut mesh, 0.01);
        assert!(merged > 0);
    }
}
