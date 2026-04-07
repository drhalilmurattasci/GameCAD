//! Face deletion operations.

use std::collections::{HashMap, HashSet};

use crate::half_edge::{EditMesh, Face, FaceId, HalfEdge, HalfEdgeId, INVALID_ID};

/// Delete faces from the mesh.
///
/// This marks faces as removed by clearing their half-edges. A compaction pass
/// removes orphaned elements.
pub fn delete_faces(mesh: &mut EditMesh, face_ids: &[FaceId]) {
    let face_set: HashSet<FaceId> = face_ids.iter().copied().collect();

    // Collect half-edges belonging to deleted faces.
    let mut deleted_hes: HashSet<HalfEdgeId> = HashSet::new();
    for &fid in face_ids {
        if fid >= mesh.faces.len() {
            continue;
        }
        let start = mesh.faces[fid].edge;
        let mut he_id = start;
        loop {
            deleted_hes.insert(he_id);
            he_id = mesh.half_edges[he_id].next;
            if he_id == start {
                break;
            }
        }
    }

    // Clear twin references from surviving half-edges pointing to deleted ones.
    for he in &mut mesh.half_edges {
        if he.twin != INVALID_ID && deleted_hes.contains(&he.twin) {
            he.twin = INVALID_ID;
        }
    }

    // Build index remaps by removing deleted elements.
    // We rebuild the mesh without deleted faces and their half-edges.
    let mut new_faces: Vec<Face> = Vec::new();
    let mut new_half_edges: Vec<HalfEdge> = Vec::new();
    let mut face_remap: HashMap<FaceId, FaceId> = HashMap::new();
    let mut he_remap: HashMap<HalfEdgeId, HalfEdgeId> = HashMap::new();

    // First pass: remap half-edges.
    for (old_id, he) in mesh.half_edges.iter().enumerate() {
        if !deleted_hes.contains(&old_id) {
            let new_id = new_half_edges.len();
            he_remap.insert(old_id, new_id);
            new_half_edges.push(*he);
        }
    }

    // Second pass: remap faces.
    for (old_id, face) in mesh.faces.iter().enumerate() {
        if !face_set.contains(&old_id) {
            let new_id = new_faces.len();
            face_remap.insert(old_id, new_id);
            let mut f = *face;
            if let Some(&new_he) = he_remap.get(&f.edge) {
                f.edge = new_he;
            }
            new_faces.push(f);
        }
    }

    // Update references in new half-edges.
    for he in &mut new_half_edges {
        if he.next != INVALID_ID {
            he.next = he_remap.get(&he.next).copied().unwrap_or(INVALID_ID);
        }
        if he.prev != INVALID_ID {
            he.prev = he_remap.get(&he.prev).copied().unwrap_or(INVALID_ID);
        }
        if he.twin != INVALID_ID {
            he.twin = he_remap.get(&he.twin).copied().unwrap_or(INVALID_ID);
        }
        if he.face != INVALID_ID {
            he.face = face_remap.get(&he.face).copied().unwrap_or(INVALID_ID);
        }
    }

    // Update vertex edge references.
    for v in &mut mesh.vertices {
        if v.edge != INVALID_ID {
            v.edge = he_remap.get(&v.edge).copied().unwrap_or(INVALID_ID);
        }
    }

    mesh.half_edges = new_half_edges;
    mesh.faces = new_faces;
}
