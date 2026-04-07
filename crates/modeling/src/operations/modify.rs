//! Mesh modification operations: merge vertices, flip normals, recalculate normals.

use glam::Vec3;

use crate::half_edge::{EditMesh, VertexId, INVALID_ID};

/// Merge vertices closer than `threshold` distance.
///
/// Returns the number of vertices that were merged.
pub fn merge_vertices(mesh: &mut EditMesh, threshold: f32) -> usize {
    let threshold_sq = threshold * threshold;
    let vertex_count = mesh.vertices.len();

    // Build merge map: for each vertex, find the canonical (lowest index) vertex
    // within threshold distance.
    let mut canonical: Vec<VertexId> = (0..vertex_count).collect();

    for i in 0..vertex_count {
        for j in (i + 1)..vertex_count {
            if canonical[j] != j {
                continue;
            }
            let dist_sq = mesh.vertices[i]
                .position
                .distance_squared(mesh.vertices[j].position);
            if dist_sq <= threshold_sq {
                canonical[j] = canonical[i];
            }
        }
    }

    let merged = canonical.iter().enumerate().filter(|(i, c)| **c != *i).count();

    if merged == 0 {
        return 0;
    }

    // Update all half-edges to reference canonical vertices.
    for he in &mut mesh.half_edges {
        he.vertex = canonical[he.vertex];
    }

    // Update vertex edge references.
    for (vid, &canon) in canonical.iter().enumerate() {
        if canon != vid {
            // This vertex is being merged away; if the canonical vertex has no edge,
            // take this one's edge.
            if mesh.vertices[canon].edge == INVALID_ID {
                mesh.vertices[canon].edge = mesh.vertices[vid].edge;
            }
        }
    }

    merged
}

/// Flip all face normals and reverse winding order.
pub fn flip_normals(mesh: &mut EditMesh) {
    // Flip face normals.
    for face in &mut mesh.faces {
        face.normal = -face.normal;
    }

    // Flip vertex normals.
    for vertex in &mut mesh.vertices {
        vertex.normal = -vertex.normal;
    }

    // Reverse winding of each face by swapping next/prev pointers.
    for fid in 0..mesh.faces.len() {
        let start = mesh.faces[fid].edge;
        let mut he_id = start;

        // Collect half-edge ids for this face.
        let mut face_hes = Vec::new();
        loop {
            face_hes.push(he_id);
            he_id = mesh.half_edges[he_id].next;
            if he_id == start {
                break;
            }
        }

        // Reverse the loop: swap next and prev, and shift vertex references.
        let n = face_hes.len();
        // Save the original vertices in order.
        let orig_verts: Vec<VertexId> = face_hes.iter().map(|&h| mesh.half_edges[h].vertex).collect();

        for i in 0..n {
            let he = &mut mesh.half_edges[face_hes[i]];
            std::mem::swap(&mut he.next, &mut he.prev);
            // Shift vertex: each half-edge should now point to what was the next vertex.
            he.vertex = orig_verts[(i + 1) % n];
        }
    }

    mesh.link_twins();
}

/// Recalculate all face and vertex normals from geometry.
pub fn recalculate_normals(mesh: &mut EditMesh) {
    // Zero out vertex normals.
    for v in &mut mesh.vertices {
        v.normal = Vec3::ZERO;
    }

    // Compute face normals and accumulate to vertices.
    for fid in 0..mesh.faces.len() {
        let verts: Vec<VertexId> = mesh.iter_face_vertices(fid).collect();
        if verts.len() < 3 {
            continue;
        }

        let p0 = mesh.vertices[verts[0]].position;
        let p1 = mesh.vertices[verts[1]].position;
        let p2 = mesh.vertices[verts[2]].position;
        let normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();

        mesh.faces[fid].normal = normal;

        for &vid in &verts {
            mesh.vertices[vid].normal += normal;
        }
    }

    // Normalize vertex normals.
    for v in &mut mesh.vertices {
        v.normal = v.normal.normalize_or_zero();
    }
}
