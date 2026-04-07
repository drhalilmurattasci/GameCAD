//! Vertex transform operations.

use glam::Vec3;

use crate::half_edge::{EditMesh, VertexId};

/// Translate the given vertices by an offset.
pub fn translate_vertices(mesh: &mut EditMesh, vertex_ids: &[VertexId], offset: Vec3) {
    for &vid in vertex_ids {
        if vid < mesh.vertices.len() {
            mesh.vertices[vid].position += offset;
        }
    }
}

/// Scale the given vertices relative to a center point.
pub fn scale_vertices(
    mesh: &mut EditMesh,
    vertex_ids: &[VertexId],
    center: Vec3,
    scale: Vec3,
) {
    for &vid in vertex_ids {
        if vid < mesh.vertices.len() {
            let p = mesh.vertices[vid].position;
            mesh.vertices[vid].position = center + (p - center) * scale;
        }
    }
}
