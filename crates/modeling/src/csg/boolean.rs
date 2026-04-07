//! CSG boolean operation and clipping functions.

use anyhow::Result;

use crate::half_edge::EditMesh;

use super::bsp::BspNode;
use super::types::{Classification, CsgPlane, CsgVertex, Polygon};

/// Perform a CSG operation on two meshes.
pub fn csg_operation(a: &EditMesh, b: &EditMesh, op: super::CsgOp) -> Result<EditMesh> {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    if polys_a.is_empty() || polys_b.is_empty() {
        // Return the non-empty mesh or empty.
        let result_polys = match op {
            super::CsgOp::Union => {
                let mut r = polys_a;
                r.extend(polys_b);
                r
            }
            super::CsgOp::Subtract => polys_a,
            super::CsgOp::Intersect => Vec::new(),
        };
        return Ok(polygons_to_mesh(&result_polys));
    }

    let bsp_a = BspNode::build(polys_a.clone());
    let bsp_b = BspNode::build(polys_b.clone());

    let result_polys = match op {
        super::CsgOp::Union => {
            // A.clip_to(B) + B.clip_to(A), then invert B's inside and clip again.
            let mut a_clipped = clip_polygons_to_tree(&polys_a, &bsp_b);
            let mut b_clipped = clip_polygons_to_tree(&polys_b, &bsp_a);

            // Remove coplanar faces from B that are inside A.
            b_clipped = invert_and_clip(&b_clipped, &bsp_a);

            a_clipped.extend(b_clipped);
            a_clipped
        }
        super::CsgOp::Subtract => {
            // Keep parts of A outside B, and inverted parts of B inside A.
            let a_clipped = clip_polygons_to_tree(&polys_a, &bsp_b);
            // Find portions of B that lie inside A, then flip their winding
            // so they form the inner walls of the subtraction cavity.
            let b_clipped = invert_and_clip_subtract(&polys_b, &bsp_a);

            let mut result = a_clipped;
            result.extend(b_clipped);
            result
        }
        super::CsgOp::Intersect => {
            // Keep parts of A inside B, and parts of B inside A.
            let mut a_inverted = polys_a.clone();
            for p in &mut a_inverted {
                p.flip();
            }
            let bsp_a_inv = BspNode::build(a_inverted);

            let mut b_clipped = clip_polygons_to_tree(&polys_b, &bsp_a_inv);
            for p in &mut b_clipped {
                p.flip();
            }

            let mut a_clipped = clip_polygons_to_tree(&polys_a, &bsp_b);
            // Only keep parts inside B.
            // Re-clip to ensure correctness.
            let bsp_b2 = BspNode::build(polys_b);
            a_clipped = keep_inside(&a_clipped, &bsp_b2);

            let mut result = a_clipped;
            result.extend(b_clipped);
            result
        }
    };

    Ok(polygons_to_mesh(&result_polys))
}

// ─────────────────────────────────────────────────────────────────────
// Clipping functions
// ─────────────────────────────────────────────────────────────────────

/// Clip polygons against a BSP tree, keeping front (outside) polygons.
fn clip_polygons_to_tree(polygons: &[Polygon], tree: &BspNode) -> Vec<Polygon> {
    let plane = match tree.plane {
        Some(p) => p,
        None => return polygons.to_vec(),
    };

    let mut front = Vec::new();
    let mut back = Vec::new();

    for poly in polygons {
        match poly.classify(&plane) {
            Classification::Coplanar => {
                // Keep coplanar if facing the same direction.
                if poly.plane.normal.dot(plane.normal) > 0.0 {
                    front.push(poly.clone());
                } else {
                    back.push(poly.clone());
                }
            }
            Classification::Front => front.push(poly.clone()),
            Classification::Back => back.push(poly.clone()),
            Classification::Spanning => {
                poly.split(&plane, &mut front, &mut back);
            }
        }
    }

    let mut result = if let Some(ref f) = tree.front {
        clip_polygons_to_tree(&front, f)
    } else {
        front
    };

    let back_result = if let Some(ref b) = tree.back {
        clip_polygons_to_tree(&back, b)
    } else {
        Vec::new() // Discard back polygons (they are inside the tree).
    };

    result.extend(back_result);
    result
}

/// Keep only polygons that are inside the BSP tree.
fn keep_inside(polygons: &[Polygon], tree: &BspNode) -> Vec<Polygon> {
    let plane = match tree.plane {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut front = Vec::new();
    let mut back = Vec::new();

    for poly in polygons {
        match poly.classify(&plane) {
            Classification::Coplanar => back.push(poly.clone()),
            Classification::Front => front.push(poly.clone()),
            Classification::Back => back.push(poly.clone()),
            Classification::Spanning => {
                poly.split(&plane, &mut front, &mut back);
            }
        }
    }

    let mut result = if let Some(ref f) = tree.front {
        keep_inside(&front, f)
    } else {
        Vec::new() // Front without subtree means outside.
    };

    let back_result = if let Some(ref b) = tree.back {
        keep_inside(&back, b)
    } else {
        back // Back without subtree means inside.
    };

    result.extend(back_result);
    result
}

/// Helper for union: invert then clip to remove coplanar duplicates.
fn invert_and_clip(polygons: &[Polygon], tree: &BspNode) -> Vec<Polygon> {
    let mut inverted: Vec<Polygon> = polygons.to_vec();
    for p in &mut inverted {
        p.flip();
    }
    let clipped = clip_polygons_to_tree(&inverted, tree);
    let mut result = clipped;
    for p in &mut result {
        p.flip();
    }
    result
}

/// Helper for subtraction: keep inverted B inside A.
fn invert_and_clip_subtract(b_polys: &[Polygon], tree_a: &BspNode) -> Vec<Polygon> {
    let inside_a = keep_inside(b_polys, tree_a);
    let mut result = inside_a;
    for p in &mut result {
        p.flip();
    }
    result
}

// ─────────────────────────────────────────────────────────────────────
// Conversion helpers
// ─────────────────────────────────────────────────────────────────────

/// Convert an EditMesh to a list of polygons.
fn mesh_to_polygons(mesh: &EditMesh) -> Vec<Polygon> {
    let mut polygons = Vec::new();

    for fid in 0..mesh.faces.len() {
        let vert_ids: Vec<_> = mesh.iter_face_vertices(fid).collect();
        if vert_ids.len() < 3 {
            continue;
        }

        let vertices: Vec<CsgVertex> = vert_ids
            .iter()
            .map(|&vid| {
                let v = &mesh.vertices[vid];
                CsgVertex {
                    position: v.position,
                    normal: v.normal,
                    uv: v.uv,
                }
            })
            .collect();

        let plane = CsgPlane::from_points(
            vertices[0].position,
            vertices[1].position,
            vertices[2].position,
        );

        polygons.push(Polygon { vertices, plane });
    }

    polygons
}

/// Convert a list of polygons back to an EditMesh.
fn polygons_to_mesh(polygons: &[Polygon]) -> EditMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for poly in polygons {
        if poly.vertices.len() < 3 {
            continue;
        }

        let base = positions.len() as u32;

        for v in &poly.vertices {
            positions.push(v.position);
            normals.push(v.normal);
            uvs.push(v.uv);
        }

        // Fan triangulation.
        for i in 1..(poly.vertices.len() - 1) {
            indices.push(base);
            indices.push(base + i as u32);
            indices.push(base + i as u32 + 1);
        }
    }

    if positions.is_empty() {
        return EditMesh::new();
    }

    EditMesh::from_triangles(&positions, &normals, &uvs, &indices)
}
