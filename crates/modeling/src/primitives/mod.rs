//! Generate common primitive meshes (cube, sphere, cylinder, cone, plane, torus).

mod cone;
mod cube;
mod cylinder;
mod plane;
mod sphere;
mod torus;

pub use cone::generate_cone;
pub use cube::generate_cube;
pub use cylinder::generate_cylinder;
pub use plane::generate_plane;
pub use sphere::generate_sphere;
pub use torus::generate_torus;

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::half_edge::INVALID_ID;
    use glam::Vec3;

    /// Verify that all vertex normals are unit-length (or zero for degenerate geometry).
    fn assert_normals_valid(mesh: &crate::half_edge::EditMesh, name: &str) {
        for (i, v) in mesh.vertices.iter().enumerate() {
            let len = v.normal.length();
            assert!(
                (len - 1.0).abs() < 0.02 || len < 1e-6,
                "{name}: vertex {i} normal length = {len}"
            );
        }
    }

    /// Verify that all UVs are in the [0,1] range.
    fn assert_uvs_valid(mesh: &crate::half_edge::EditMesh, name: &str) {
        for (i, v) in mesh.vertices.iter().enumerate() {
            assert!(
                v.uv.x >= -1e-5 && v.uv.x <= 1.0 + 1e-5 && v.uv.y >= -1e-5 && v.uv.y <= 1.0 + 1e-5,
                "{name}: vertex {i} UV ({}, {}) out of range",
                v.uv.x,
                v.uv.y
            );
        }
    }

    /// Verify basic topology is valid.
    fn assert_topology_valid(mesh: &crate::half_edge::EditMesh, name: &str) {
        let errors = mesh.validate_topology();
        assert!(errors.is_empty(), "{name}: topology errors: {:?}", errors);
    }

    #[test]
    fn cube_has_correct_counts() {
        let mesh = generate_cube(1.0);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.face_count(), 12);
    }

    #[test]
    fn cube_valid_normals_and_uvs() {
        let mesh = generate_cube(1.0);
        assert_normals_valid(&mesh, "cube");
        assert_uvs_valid(&mesh, "cube");
        assert_topology_valid(&mesh, "cube");
    }

    #[test]
    fn cube_face_normals_point_outward() {
        let mesh = generate_cube(1.0);
        for face in &mesh.faces {
            // Each face normal should be axis-aligned for a cube.
            let n = face.normal;
            let max_comp = n.x.abs().max(n.y.abs()).max(n.z.abs());
            assert!(
                (max_comp - 1.0).abs() < 1e-4,
                "Cube face normal not axis-aligned: {:?}",
                n
            );
        }
    }

    #[test]
    fn sphere_has_correct_vertex_count() {
        let segments = 8u32;
        let rings = 4u32;
        let mesh = generate_sphere(1.0, segments, rings);
        let expected_verts = ((rings + 1) * (segments + 1)) as usize;
        assert_eq!(mesh.vertex_count(), expected_verts);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn sphere_valid_normals_and_uvs() {
        let mesh = generate_sphere(1.0, 16, 8);
        assert_normals_valid(&mesh, "sphere");
        assert_uvs_valid(&mesh, "sphere");
        assert_topology_valid(&mesh, "sphere");
    }

    #[test]
    fn sphere_vertices_on_surface() {
        let radius = 2.5;
        let mesh = generate_sphere(radius, 12, 6);
        for (i, v) in mesh.vertices.iter().enumerate() {
            let dist = v.position.length();
            assert!(
                (dist - radius).abs() < 1e-4,
                "Sphere vertex {i} distance from center = {dist}, expected {radius}"
            );
        }
    }

    #[test]
    fn sphere_minimum_segments() {
        // Test that minimum clamping works (segments=1 should clamp to 3).
        let mesh = generate_sphere(1.0, 1, 1);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn cylinder_has_faces() {
        let mesh = generate_cylinder(1.0, 2.0, 8);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn cylinder_valid_normals_and_uvs() {
        let mesh = generate_cylinder(1.0, 2.0, 16);
        assert_normals_valid(&mesh, "cylinder");
        assert_uvs_valid(&mesh, "cylinder");
        assert_topology_valid(&mesh, "cylinder");
    }

    #[test]
    fn cylinder_correct_face_count() {
        let segs = 8u32;
        let mesh = generate_cylinder(1.0, 2.0, segs);
        // Side: 2*segs, top cap: segs, bottom cap: segs => 4*segs total triangles.
        assert_eq!(mesh.face_count(), (4 * segs) as usize);
    }

    #[test]
    fn cylinder_minimum_segments() {
        let mesh = generate_cylinder(1.0, 2.0, 1);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn cone_has_faces() {
        let mesh = generate_cone(1.0, 2.0, 8);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn cone_valid_normals_and_uvs() {
        let mesh = generate_cone(1.0, 2.0, 16);
        assert_normals_valid(&mesh, "cone");
        assert_uvs_valid(&mesh, "cone");
        assert_topology_valid(&mesh, "cone");
    }

    #[test]
    fn cone_correct_face_count() {
        let segs = 8u32;
        let mesh = generate_cone(1.0, 2.0, segs);
        // Side: segs triangles, bottom cap: segs triangles => 2*segs.
        assert_eq!(mesh.face_count(), (2 * segs) as usize);
    }

    #[test]
    fn plane_has_correct_counts() {
        let mesh = generate_plane(1.0, 1.0);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
    }

    #[test]
    fn plane_valid_normals_and_uvs() {
        let mesh = generate_plane(3.0, 2.0);
        assert_normals_valid(&mesh, "plane");
        assert_uvs_valid(&mesh, "plane");
        assert_topology_valid(&mesh, "plane");
        // All normals should point up.
        for v in &mesh.vertices {
            assert!((v.normal - Vec3::Y).length() < 1e-4);
        }
    }

    #[test]
    fn torus_has_correct_vertex_count() {
        let major = 8u32;
        let minor = 6u32;
        let mesh = generate_torus(1.0, 0.3, major, minor);
        let expected_verts = ((major + 1) * (minor + 1)) as usize;
        assert_eq!(mesh.vertex_count(), expected_verts);
        assert_eq!(mesh.face_count(), (major * minor * 2) as usize);
    }

    #[test]
    fn torus_valid_normals_and_uvs() {
        let mesh = generate_torus(1.0, 0.3, 12, 8);
        assert_normals_valid(&mesh, "torus");
        assert_uvs_valid(&mesh, "torus");
        assert_topology_valid(&mesh, "torus");
    }

    #[test]
    fn cube_roundtrip_triangle_data() {
        let mesh = generate_cube(2.0);
        let (positions, normals, uvs, indices) = mesh.to_triangles();
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
        assert!(indices.len() % 3 == 0);
    }

    #[test]
    fn all_primitives_vertex_edge_refs_valid() {
        // These primitives should have all vertex edge references set.
        // Sphere and torus have UV-seam duplicate vertices at poles/wraps
        // that may be unused; they are tested separately.
        let meshes = [
            ("cube", generate_cube(1.0)),
            ("cylinder", generate_cylinder(1.0, 2.0, 8)),
            ("cone", generate_cone(1.0, 2.0, 8)),
            ("plane", generate_plane(1.0, 1.0)),
        ];
        for (name, mesh) in &meshes {
            for (i, v) in mesh.vertices.iter().enumerate() {
                assert!(
                    v.edge != INVALID_ID && v.edge < mesh.half_edges.len(),
                    "{name}: vertex {i} has invalid edge ref {}",
                    v.edge
                );
            }
        }
    }

    #[test]
    fn sphere_used_vertices_have_valid_edge_refs() {
        let mesh = generate_sphere(1.0, 8, 4);
        // Count how many vertices have valid edge refs.
        let valid_count = mesh
            .vertices
            .iter()
            .filter(|v| v.edge != INVALID_ID)
            .count();
        // The vast majority of vertices should have valid refs.
        assert!(
            valid_count > mesh.vertex_count() * 3 / 4,
            "Too few valid vertex edge refs: {valid_count}/{}",
            mesh.vertex_count()
        );
    }

    #[test]
    fn torus_used_vertices_have_valid_edge_refs() {
        let mesh = generate_torus(1.0, 0.3, 8, 6);
        let valid_count = mesh
            .vertices
            .iter()
            .filter(|v| v.edge != INVALID_ID)
            .count();
        assert!(
            valid_count > mesh.vertex_count() * 3 / 4,
            "Too few valid vertex edge refs: {valid_count}/{}",
            mesh.vertex_count()
        );
    }
}
