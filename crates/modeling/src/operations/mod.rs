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
    fn flip_normals_double_flip_restores() {
        let mut mesh = generate_plane(1.0, 1.0);
        let original_normals: Vec<Vec3> = mesh.faces.iter().map(|f| f.normal).collect();
        flip_normals(&mut mesh);
        flip_normals(&mut mesh);
        for (i, face) in mesh.faces.iter().enumerate() {
            assert!(
                (face.normal - original_normals[i]).length() < 1e-4,
                "Double flip did not restore face {i} normal"
            );
        }
    }

    #[test]
    fn recalculate_normals_works() {
        let mut mesh = generate_cube(1.0);
        for v in &mut mesh.vertices {
            v.normal = Vec3::ZERO;
        }
        recalculate_normals(&mut mesh);
        for v in &mesh.vertices {
            assert!(v.normal.length() > 0.5);
        }
    }

    #[test]
    fn recalculate_normals_produces_unit_normals() {
        let mut mesh = generate_cube(1.0);
        recalculate_normals(&mut mesh);
        for v in &mesh.vertices {
            let len = v.normal.length();
            assert!(
                (len - 1.0).abs() < 0.02,
                "Vertex normal length = {len}, expected ~1.0"
            );
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
    fn translate_empty_ids_is_noop() {
        let mut mesh = generate_plane(1.0, 1.0);
        let orig = mesh.vertices[0].position;
        translate_vertices(&mut mesh, &[], Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(mesh.vertices[0].position, orig);
    }

    #[test]
    fn translate_out_of_bounds_id_is_safe() {
        let mut mesh = generate_plane(1.0, 1.0);
        translate_vertices(&mut mesh, &[9999], Vec3::ONE);
        // Should not panic.
    }

    #[test]
    fn scale_vertices_works() {
        let mut mesh = generate_cube(2.0);
        let ids: Vec<VertexId> = (0..mesh.vertex_count()).collect();
        scale_vertices(&mut mesh, &ids, Vec3::ZERO, Vec3::splat(2.0));
        let max_x = mesh
            .vertices
            .iter()
            .map(|v| v.position.x)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_x - 2.0).abs() < 1e-4);
    }

    #[test]
    fn scale_by_one_is_identity() {
        let mut mesh = generate_cube(1.0);
        let original: Vec<Vec3> = mesh.vertices.iter().map(|v| v.position).collect();
        let ids: Vec<VertexId> = (0..mesh.vertex_count()).collect();
        scale_vertices(&mut mesh, &ids, Vec3::ZERO, Vec3::ONE);
        for (i, v) in mesh.vertices.iter().enumerate() {
            assert!(
                (v.position - original[i]).length() < 1e-6,
                "Scale by 1 changed vertex {i}"
            );
        }
    }

    #[test]
    fn delete_faces_reduces_count() {
        let mut mesh = generate_cube(1.0);
        let original = mesh.face_count();
        delete_faces(&mut mesh, &[0, 1]);
        assert_eq!(mesh.face_count(), original - 2);
    }

    #[test]
    fn delete_all_faces() {
        let mut mesh = generate_plane(1.0, 1.0);
        let all_faces: Vec<usize> = (0..mesh.face_count()).collect();
        delete_faces(&mut mesh, &all_faces);
        assert_eq!(mesh.face_count(), 0);
        assert_eq!(mesh.edge_count(), 0);
    }

    #[test]
    fn delete_no_faces_is_noop() {
        let mut mesh = generate_cube(1.0);
        let original = mesh.face_count();
        delete_faces(&mut mesh, &[]);
        assert_eq!(mesh.face_count(), original);
    }

    #[test]
    fn delete_out_of_bounds_face_is_safe() {
        let mut mesh = generate_cube(1.0);
        let original = mesh.face_count();
        delete_faces(&mut mesh, &[9999]);
        assert_eq!(mesh.face_count(), original);
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
    fn extrude_single_face() {
        let mut mesh = generate_plane(1.0, 1.0);
        let result = extrude_faces(&mut mesh, &[0], 0.5);
        assert!(result.is_ok());
        let new_faces = result.unwrap();
        assert!(!new_faces.is_empty());
    }

    #[test]
    fn extrude_zero_distance() {
        let mut mesh = generate_plane(1.0, 1.0);
        let result = extrude_faces(&mut mesh, &[0], 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn extrude_empty_list_fails() {
        let mut mesh = generate_plane(1.0, 1.0);
        let result = extrude_faces(&mut mesh, &[], 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn extrude_out_of_bounds_face_fails() {
        let mut mesh = generate_plane(1.0, 1.0);
        let result = extrude_faces(&mut mesh, &[9999], 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn inset_faces_creates_ring() {
        let mut mesh = generate_plane(1.0, 1.0);
        let original_faces = mesh.face_count();
        let result = inset_faces(&mut mesh, &[0], 0.1);
        assert!(result.is_ok());
        assert!(mesh.face_count() > original_faces);
    }

    #[test]
    fn inset_empty_list_fails() {
        let mut mesh = generate_plane(1.0, 1.0);
        let result = inset_faces(&mut mesh, &[], 0.1);
        assert!(result.is_err());
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
    fn subdivide_preserves_valid_topology() {
        let mut mesh = generate_plane(1.0, 1.0);
        subdivide(&mut mesh).unwrap();
        let errors = mesh.validate_topology();
        assert!(errors.is_empty(), "Post-subdivide topology errors: {:?}", errors);
    }

    #[test]
    fn merge_vertices_works() {
        let mut mesh = generate_cube(1.0);
        let merged = merge_vertices(&mut mesh, 0.01);
        assert!(merged > 0);
    }

    #[test]
    fn merge_vertices_zero_threshold() {
        let mut mesh = generate_cube(1.0);
        // With zero threshold, only exactly coincident vertices merge.
        let merged = merge_vertices(&mut mesh, 0.0);
        assert!(merged > 0, "Expected some merges for coincident vertices");
    }

    #[test]
    fn merge_vertices_no_merge_when_far() {
        let mut mesh = generate_plane(100.0, 100.0);
        // Plane vertices are far apart.
        let merged = merge_vertices(&mut mesh, 0.001);
        assert_eq!(merged, 0);
    }
}
