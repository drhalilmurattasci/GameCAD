//! Internal helper functions shared across operation modules.

use glam::Vec3;

use crate::half_edge::{EditMesh, Face, FaceId, HalfEdge, HalfEdgeId, VertexId, INVALID_ID};

/// Find the half-edge from v0 to v1 in the given face.
#[inline]
pub(crate) fn find_half_edge(
    mesh: &EditMesh,
    v0: VertexId,
    v1: VertexId,
    face_id: FaceId,
) -> Option<HalfEdgeId> {
    let start = mesh.faces[face_id].edge;
    let mut he_id = start;
    loop {
        let he = &mesh.half_edges[he_id];
        let next_v = mesh.half_edges[he.next].vertex;
        if he.vertex == v0 && next_v == v1 {
            return Some(he_id);
        }
        he_id = he.next;
        if he_id == start {
            break;
        }
    }
    None
}

/// Helper to add a triangle face to the mesh, creating three half-edges and
/// setting vertex-edge back-references when they are unset. Returns the new face ID.
pub(crate) fn add_triangle(mesh: &mut EditMesh, v0: VertexId, v1: VertexId, v2: VertexId, normal: Vec3) -> FaceId {
    let face_id = mesh.faces.len();
    let base = mesh.half_edges.len();

    let he0 = HalfEdge {
        next: base + 1,
        prev: base + 2,
        twin: INVALID_ID,
        vertex: v0,
        face: face_id,
    };
    let he1 = HalfEdge {
        next: base + 2,
        prev: base,
        twin: INVALID_ID,
        vertex: v1,
        face: face_id,
    };
    let he2 = HalfEdge {
        next: base,
        prev: base + 1,
        twin: INVALID_ID,
        vertex: v2,
        face: face_id,
    };

    mesh.half_edges.push(he0);
    mesh.half_edges.push(he1);
    mesh.half_edges.push(he2);

    // Set vertex edge references if not set.
    for (i, vid) in [v0, v1, v2].iter().enumerate() {
        if mesh.vertices[*vid].edge == INVALID_ID {
            mesh.vertices[*vid].edge = base + i;
        }
    }

    mesh.faces.push(Face {
        edge: base,
        normal,
    });

    face_id
}
