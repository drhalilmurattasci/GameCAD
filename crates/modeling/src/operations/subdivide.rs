//! Catmull-Clark subdivision.

use anyhow::Result;
use glam::{Vec2, Vec3};
use std::collections::{HashMap, HashSet};

use crate::half_edge::{EditMesh, FaceId, VertexId};

use super::helpers::add_triangle;
use super::modify::recalculate_normals;

/// Catmull-Clark subdivision.
///
/// Subdivides all faces in the mesh by splitting each face into quads.
pub fn subdivide(mesh: &mut EditMesh) -> Result<()> {
    let old_vertex_count = mesh.vertices.len();
    let old_face_count = mesh.faces.len();

    // Step 1: Compute face points (centroid of each face).
    let mut face_points: Vec<Vec3> = Vec::with_capacity(old_face_count);
    let mut face_vert_lists: Vec<Vec<VertexId>> = Vec::with_capacity(old_face_count);
    for fid in 0..old_face_count {
        let verts: Vec<VertexId> = mesh.iter_face_vertices(fid).collect();
        let centroid: Vec3 = verts
            .iter()
            .map(|&vid| mesh.vertices[vid].position)
            .sum::<Vec3>()
            / verts.len() as f32;
        face_points.push(centroid);
        face_vert_lists.push(verts);
    }

    // Step 2: Compute edge points.
    // Map (min_v, max_v) -> (edge midpoint, [adjacent face ids])
    let mut edge_data: HashMap<(VertexId, VertexId), (Vec3, Vec<FaceId>)> = HashMap::new();
    for (fid, verts) in face_vert_lists.iter().enumerate() {
        let n = verts.len();
        for i in 0..n {
            let v0 = verts[i];
            let v1 = verts[(i + 1) % n];
            let key = (v0.min(v1), v0.max(v1));
            let mid = (mesh.vertices[v0].position + mesh.vertices[v1].position) * 0.5;
            edge_data
                .entry(key)
                .and_modify(|e| e.1.push(fid))
                .or_insert((mid, vec![fid]));
        }
    }

    let mut edge_points: HashMap<(VertexId, VertexId), Vec3> = HashMap::new();
    for (&key, (mid, faces)) in &edge_data {
        if faces.len() == 2 {
            // Average of midpoint and adjacent face points.
            let fp_avg = (face_points[faces[0]] + face_points[faces[1]]) * 0.5;
            edge_points.insert(key, (*mid + fp_avg) * 0.5);
        } else {
            edge_points.insert(key, *mid);
        }
    }

    // Step 3: Compute new vertex positions for original vertices.
    // For each original vertex: collect adjacent faces and edges.
    let mut vert_faces: Vec<Vec<FaceId>> = vec![Vec::new(); old_vertex_count];
    let mut vert_edges: Vec<HashSet<(VertexId, VertexId)>> =
        vec![HashSet::new(); old_vertex_count];
    for (fid, verts) in face_vert_lists.iter().enumerate() {
        let n = verts.len();
        for i in 0..n {
            let v0 = verts[i];
            let v1 = verts[(i + 1) % n];
            vert_faces[v0].push(fid);
            let key = (v0.min(v1), v0.max(v1));
            vert_edges[v0].insert(key);
        }
    }

    let mut new_vertex_positions: Vec<Vec3> = Vec::with_capacity(old_vertex_count);
    for vid in 0..old_vertex_count {
        let n = vert_faces[vid].len() as f32;
        if n < 1.0 {
            new_vertex_positions.push(mesh.vertices[vid].position);
            continue;
        }

        let f_avg: Vec3 = vert_faces[vid]
            .iter()
            .map(|&fid| face_points[fid])
            .sum::<Vec3>()
            / n;

        let edge_mids: Vec3 = vert_edges[vid]
            .iter()
            .map(|key| edge_data[key].0)
            .sum::<Vec3>()
            / vert_edges[vid].len() as f32;

        let p = mesh.vertices[vid].position;
        let new_p = (f_avg + 2.0 * edge_mids + (n - 3.0) * p) / n;
        new_vertex_positions.push(new_p);
    }

    // Step 4: Build the new mesh.
    let mut new_mesh = EditMesh::new();

    // Add updated original vertices.
    for (vid, new_pos) in new_vertex_positions.iter().enumerate() {
        let v = &mesh.vertices[vid];
        new_mesh.add_vertex(*new_pos, v.normal, v.uv);
    }

    // Add face point vertices.
    let mut face_point_ids: Vec<VertexId> = Vec::with_capacity(old_face_count);
    for (fid, &fp) in face_points.iter().enumerate() {
        let normal = mesh.faces[fid].normal;
        let id = new_mesh.add_vertex(fp, normal, Vec2::new(0.5, 0.5));
        face_point_ids.push(id);
    }

    // Add edge point vertices.
    let mut edge_point_ids: HashMap<(VertexId, VertexId), VertexId> = HashMap::new();
    for (&key, &ep) in &edge_points {
        let normal = Vec3::Y; // will be recalculated
        let id = new_mesh.add_vertex(ep, normal, Vec2::new(0.5, 0.5));
        edge_point_ids.insert(key, id);
    }

    // Create new quads: for each original face, one quad per edge.
    for fid in 0..old_face_count {
        let verts = &face_vert_lists[fid];
        let n = verts.len();
        let fp_id = face_point_ids[fid];

        for i in 0..n {
            let v0 = verts[i];
            let v1 = verts[(i + 1) % n];
            let v_prev = verts[(i + n - 1) % n];

            let edge_key_next = (v0.min(v1), v0.max(v1));
            let edge_key_prev = (v_prev.min(v0), v_prev.max(v0));

            let ep_next = edge_point_ids[&edge_key_next];
            let ep_prev = edge_point_ids[&edge_key_prev];

            // Quad: ep_prev, v0, ep_next, fp_id
            // Split into two triangles.
            let p0 = new_mesh.vertices[ep_prev].position;
            let p1 = new_mesh.vertices[v0].position;
            let p2 = new_mesh.vertices[ep_next].position;
            let normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();

            add_triangle(&mut new_mesh, ep_prev, v0, ep_next, normal);
            add_triangle(&mut new_mesh, ep_prev, ep_next, fp_id, normal);
        }
    }

    new_mesh.link_twins();

    // Replace mesh contents.
    mesh.vertices = new_mesh.vertices;
    mesh.half_edges = new_mesh.half_edges;
    mesh.faces = new_mesh.faces;

    recalculate_normals(mesh);

    Ok(())
}
