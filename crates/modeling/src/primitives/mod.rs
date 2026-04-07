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

    #[test]
    fn cube_has_correct_counts() {
        let mesh = generate_cube(1.0);
        assert_eq!(mesh.vertex_count(), 24);
        // 6 faces * 2 triangles = 12 triangle faces
        assert_eq!(mesh.face_count(), 12);
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
    fn cylinder_has_faces() {
        let mesh = generate_cylinder(1.0, 2.0, 8);
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
    fn plane_has_correct_counts() {
        let mesh = generate_plane(1.0, 1.0);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.face_count(), 2);
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
    fn cube_roundtrip_triangle_data() {
        let mesh = generate_cube(2.0);
        let (positions, normals, uvs, indices) = mesh.to_triangles();
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
        assert!(indices.len() % 3 == 0);
    }
}
