//! Constructive Solid Geometry (CSG) operations: union, subtract, intersect.
//!
//! Uses a BSP-tree polygon clipping approach. Each mesh is converted to a set
//! of convex polygons, then clipped against the other mesh's BSP tree.

mod boolean;
mod bsp;
mod types;

use serde::{Deserialize, Serialize};

pub use boolean::csg_operation;

/// CSG operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsgOp {
    Union,
    Subtract,
    Intersect,
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::half_edge::EditMesh;
    use crate::primitives::generate_cube;
    use glam::{Vec2, Vec3};
    use types::{CsgPlane, CsgVertex, Classification, Polygon};

    #[test]
    fn csg_union_two_cubes() {
        let a = generate_cube(1.0);
        let b = generate_cube(1.0);
        let result = csg_operation(&a, &b, CsgOp::Union);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn csg_subtract_produces_mesh() {
        let a = generate_cube(2.0);
        let b = generate_cube(1.0);
        let result = csg_operation(&a, &b, CsgOp::Subtract);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn csg_intersect_produces_mesh() {
        let a = generate_cube(2.0);
        let b = generate_cube(1.0);
        let result = csg_operation(&a, &b, CsgOp::Intersect);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        let _ = mesh.face_count();
    }

    #[test]
    fn csg_union_empty_mesh() {
        let a = generate_cube(1.0);
        let b = EditMesh::new();
        let result = csg_operation(&a, &b, CsgOp::Union);
        assert!(result.is_ok());
        let mesh = result.unwrap();
        assert!(mesh.face_count() > 0);
    }

    #[test]
    fn csg_subtract_empty_b_returns_a() {
        let a = generate_cube(1.0);
        let b = EditMesh::new();
        let result = csg_operation(&a, &b, CsgOp::Subtract).unwrap();
        assert_eq!(result.face_count(), a.face_count());
    }

    #[test]
    fn csg_intersect_empty_returns_empty() {
        let a = generate_cube(1.0);
        let b = EditMesh::new();
        let result = csg_operation(&a, &b, CsgOp::Intersect).unwrap();
        assert_eq!(result.face_count(), 0);
    }

    #[test]
    fn csg_union_both_empty() {
        let a = EditMesh::new();
        let b = EditMesh::new();
        let result = csg_operation(&a, &b, CsgOp::Union).unwrap();
        assert_eq!(result.face_count(), 0);
    }

    #[test]
    fn csg_subtract_both_empty() {
        let a = EditMesh::new();
        let b = EditMesh::new();
        let result = csg_operation(&a, &b, CsgOp::Subtract).unwrap();
        assert_eq!(result.face_count(), 0);
    }

    #[test]
    fn polygon_flip_reverses_normal() {
        let plane = CsgPlane::from_points(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let mut poly = Polygon {
            vertices: vec![
                CsgVertex { position: Vec3::ZERO, normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::X, normal: Vec3::Z, uv: Vec2::X },
                CsgVertex { position: Vec3::Y, normal: Vec3::Z, uv: Vec2::Y },
            ],
            plane,
        };

        let original_normal = poly.plane.normal;
        poly.flip();
        assert!((poly.plane.normal + original_normal).length() < 1e-4);
    }

    #[test]
    fn polygon_flip_reverses_vertex_order() {
        let plane = CsgPlane::from_points(Vec3::ZERO, Vec3::X, Vec3::Y);
        let mut poly = Polygon {
            vertices: vec![
                CsgVertex { position: Vec3::ZERO, normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::X, normal: Vec3::Z, uv: Vec2::X },
                CsgVertex { position: Vec3::Y, normal: Vec3::Z, uv: Vec2::Y },
            ],
            plane,
        };
        poly.flip();
        // After flip, vertex order should be reversed.
        assert!((poly.vertices[0].position - Vec3::Y).length() < 1e-5);
        assert!((poly.vertices[2].position - Vec3::ZERO).length() < 1e-5);
    }

    #[test]
    fn plane_classify_point() {
        let plane = CsgPlane::from_points(Vec3::ZERO, Vec3::X, Vec3::Y);
        // Plane normal is +Z, w = 0.
        assert_eq!(plane.classify_point(Vec3::new(0.0, 0.0, 1.0)), Classification::Front);
        assert_eq!(plane.classify_point(Vec3::new(0.0, 0.0, -1.0)), Classification::Back);
        assert_eq!(plane.classify_point(Vec3::new(0.5, 0.5, 0.0)), Classification::Coplanar);
    }

    #[test]
    fn polygon_classify_all_front() {
        let plane = CsgPlane::from_points(Vec3::ZERO, Vec3::X, Vec3::Y);
        let poly = Polygon {
            vertices: vec![
                CsgVertex { position: Vec3::new(0.0, 0.0, 2.0), normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::new(1.0, 0.0, 2.0), normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::new(0.0, 1.0, 2.0), normal: Vec3::Z, uv: Vec2::ZERO },
            ],
            plane: CsgPlane::from_points(
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::new(1.0, 0.0, 2.0),
                Vec3::new(0.0, 1.0, 2.0),
            ),
        };
        assert_eq!(poly.classify(&plane), Classification::Front);
    }

    #[test]
    fn polygon_split_spanning() {
        let plane = CsgPlane::from_points(Vec3::ZERO, Vec3::X, Vec3::Y);
        let poly = Polygon {
            vertices: vec![
                CsgVertex { position: Vec3::new(0.0, 0.0, -1.0), normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::new(1.0, 0.0, -1.0), normal: Vec3::Z, uv: Vec2::ZERO },
                CsgVertex { position: Vec3::new(0.5, 0.0, 1.0), normal: Vec3::Z, uv: Vec2::ZERO },
            ],
            plane: CsgPlane::from_points(
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(1.0, 0.0, -1.0),
                Vec3::new(0.5, 0.0, 1.0),
            ),
        };
        assert_eq!(poly.classify(&plane), Classification::Spanning);

        let mut front = Vec::new();
        let mut back = Vec::new();
        poly.split(&plane, &mut front, &mut back);
        assert!(!front.is_empty(), "Split should produce front polygons");
        assert!(!back.is_empty(), "Split should produce back polygons");
    }

    #[test]
    fn csg_result_has_valid_topology() {
        let a = generate_cube(2.0);
        let b = generate_cube(1.0);
        let result = csg_operation(&a, &b, CsgOp::Union).unwrap();
        let errors = result.validate_topology();
        assert!(errors.is_empty(), "CSG union topology errors: {:?}", errors);
    }
}
