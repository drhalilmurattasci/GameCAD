//! Extrude and inset face operations.

use anyhow::{bail, Result};
use glam::Vec3;
use std::collections::{HashMap, HashSet};

use crate::half_edge::{EditMesh, FaceId, VertexId, INVALID_ID};

use super::helpers::{add_triangle, find_half_edge};

/// Extrude the given faces along their averaged normal by `distance`.
///
/// Returns the newly created side faces and the moved top faces.
pub fn extrude_faces(
    mesh: &mut EditMesh,
    face_ids: &[FaceId],
    distance: f32,
) -> Result<Vec<FaceId>> {
    if face_ids.is_empty() {
        bail!("No faces to extrude");
    }

    let face_set: HashSet<FaceId> = face_ids.iter().copied().collect();

    // Compute averaged normal of selected faces.
    let mut avg_normal = Vec3::ZERO;
    for &fid in face_ids {
        if fid >= mesh.faces.len() {
            bail!("Face id {} out of range", fid);
        }
        avg_normal += mesh.faces[fid].normal;
    }
    avg_normal = avg_normal.normalize_or_zero();
    let offset = avg_normal * distance;

    // Collect all unique vertices on the boundary of the selected faces.
    let mut selected_verts: HashSet<VertexId> = HashSet::new();
    for &fid in face_ids {
        for vid in mesh.iter_face_vertices(fid) {
            selected_verts.insert(vid);
        }
    }

    // Duplicate vertices: old vertex -> new vertex (moved position).
    let mut vert_map: HashMap<VertexId, VertexId> = HashMap::new();
    for &vid in &selected_verts {
        let v = mesh.vertices[vid];
        let new_vid = mesh.add_vertex(v.position + offset, v.normal, v.uv);
        vert_map.insert(vid, new_vid);
    }

    // Collect edges of selected faces for side face creation.
    let mut boundary_edges: Vec<(VertexId, VertexId)> = Vec::new();
    for &fid in face_ids {
        let verts: Vec<VertexId> = mesh.iter_face_vertices(fid).collect();
        let n = verts.len();
        for i in 0..n {
            let v0 = verts[i];
            let v1 = verts[(i + 1) % n];

            // Check if the neighboring face across this edge is also selected.
            // If not, this is a boundary edge that needs a side face.
            let he_id = find_half_edge(mesh, v0, v1, fid);
            let is_boundary = if let Some(heid) = he_id {
                let twin = mesh.half_edges[heid].twin;
                if twin == INVALID_ID {
                    true
                } else {
                    !face_set.contains(&mesh.half_edges[twin].face)
                }
            } else {
                true
            };

            if is_boundary {
                boundary_edges.push((v0, v1));
            }
        }
    }

    let mut new_face_ids = Vec::new();

    // Create side faces (quads as two triangles) for each boundary edge.
    for (v0, v1) in &boundary_edges {
        let nv0 = vert_map[v0];
        let nv1 = vert_map[v1];

        // Quad: v0, v1, nv1, nv0 (winding order for outward-facing side).
        let p0 = mesh.vertices[*v0].position;
        let p1 = mesh.vertices[*v1].position;
        let p3 = mesh.vertices[nv0].position;
        let side_normal = (p1 - p0).cross(p3 - p0).normalize_or_zero();

        // Create as two triangles via add_face on the quad.
        // Use two triangles: (v0, v1, nv1) and (v0, nv1, nv0).
        let fid1 = add_triangle(mesh, *v0, *v1, nv1, side_normal);
        let fid2 = add_triangle(mesh, *v0, nv1, nv0, side_normal);
        new_face_ids.push(fid1);
        new_face_ids.push(fid2);
    }

    // Update the original selected faces to reference the new (moved) vertices.
    for &fid in face_ids {
        let start = mesh.faces[fid].edge;
        let mut he_id = start;
        loop {
            let old_vid = mesh.half_edges[he_id].vertex;
            if let Some(&new_vid) = vert_map.get(&old_vid) {
                mesh.half_edges[he_id].vertex = new_vid;
            }
            he_id = mesh.half_edges[he_id].next;
            if he_id == start {
                break;
            }
        }

        // Recompute face normal.
        let verts: Vec<VertexId> = mesh.iter_face_vertices(fid).collect();
        if verts.len() >= 3 {
            let p0 = mesh.vertices[verts[0]].position;
            let p1 = mesh.vertices[verts[1]].position;
            let p2 = mesh.vertices[verts[2]].position;
            mesh.faces[fid].normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();
        }

        new_face_ids.push(fid);
    }

    mesh.link_twins();
    Ok(new_face_ids)
}

/// Inset faces by moving their vertices toward the face center by `inset` amount.
///
/// Creates a ring of new faces between the original boundary and the inset face.
/// Returns the newly created inset (inner) face IDs.
pub fn inset_faces(
    mesh: &mut EditMesh,
    face_ids: &[FaceId],
    inset: f32,
) -> Result<Vec<FaceId>> {
    if face_ids.is_empty() {
        bail!("No faces to inset");
    }

    let mut new_face_ids = Vec::new();

    for &fid in face_ids {
        if fid >= mesh.faces.len() {
            bail!("Face id {} out of range", fid);
        }

        let verts: Vec<VertexId> = mesh.iter_face_vertices(fid).collect();
        let n = verts.len();
        if n < 3 {
            continue;
        }

        // Compute face center.
        let center: Vec3 = verts
            .iter()
            .map(|&vid| mesh.vertices[vid].position)
            .sum::<Vec3>()
            / n as f32;

        // Create inset vertices (moved toward center).
        let mut inner_verts = Vec::new();
        for &vid in &verts {
            let v = mesh.vertices[vid];
            let dir = center - v.position;
            let new_pos = v.position + dir.normalize_or_zero() * inset.min(dir.length());
            let new_vid = mesh.add_vertex(new_pos, v.normal, v.uv);
            inner_verts.push(new_vid);
        }

        // Create ring quads between outer and inner vertices.
        let face_normal = mesh.faces[fid].normal;
        for i in 0..n {
            let j = (i + 1) % n;
            let outer0 = verts[i];
            let outer1 = verts[j];
            let inner0 = inner_verts[i];
            let inner1 = inner_verts[j];

            let f1 = add_triangle(mesh, outer0, outer1, inner1, face_normal);
            let f2 = add_triangle(mesh, outer0, inner1, inner0, face_normal);
            new_face_ids.push(f1);
            new_face_ids.push(f2);
        }

        // Update the original face to use the inner vertices.
        let start = mesh.faces[fid].edge;
        let mut he_id = start;
        let mut idx = 0;
        loop {
            mesh.half_edges[he_id].vertex = inner_verts[idx];
            idx += 1;
            he_id = mesh.half_edges[he_id].next;
            if he_id == start {
                break;
            }
        }

        new_face_ids.push(fid);
    }

    mesh.link_twins();
    Ok(new_face_ids)
}
