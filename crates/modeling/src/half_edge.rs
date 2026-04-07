//! Half-edge mesh data structure for efficient topological queries and editing.

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Index into the half-edge array.
pub type HalfEdgeId = usize;
/// Index into the vertex array.
pub type VertexId = usize;
/// Index into the face array.
pub type FaceId = usize;

/// Sentinel value indicating "no element".
pub const INVALID_ID: usize = usize::MAX;

// ─────────────────────────────────────────────────────────────────────
// Core data types
// ─────────────────────────────────────────────────────────────────────

/// A single half-edge in the mesh.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HalfEdge {
    /// Next half-edge around the same face.
    pub next: HalfEdgeId,
    /// Previous half-edge around the same face.
    pub prev: HalfEdgeId,
    /// Twin (opposite) half-edge sharing the same geometric edge.
    pub twin: HalfEdgeId,
    /// Vertex this half-edge originates from.
    pub vertex: VertexId,
    /// Face this half-edge borders (INVALID_ID for boundary edges).
    pub face: FaceId,
}

impl Default for HalfEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl HalfEdge {
    /// Creates a half-edge with all fields set to [`INVALID_ID`].
    pub fn new() -> Self {
        Self {
            next: INVALID_ID,
            prev: INVALID_ID,
            twin: INVALID_ID,
            vertex: INVALID_ID,
            face: INVALID_ID,
        }
    }
}

/// A vertex in the mesh.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    /// One of the outgoing half-edges from this vertex.
    pub edge: HalfEdgeId,
}

/// A face (polygon) in the mesh.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Face {
    /// One of the half-edges bounding this face.
    pub edge: HalfEdgeId,
    /// Face normal.
    pub normal: Vec3,
}

// ─────────────────────────────────────────────────────────────────────
// EditMesh
// ─────────────────────────────────────────────────────────────────────

/// The primary editable mesh structure using a half-edge representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMesh {
    pub vertices: Vec<Vertex>,
    pub half_edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl EditMesh {
    /// Creates an empty mesh.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            half_edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of faces.
    #[inline]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Number of half-edges (each geometric edge has two half-edges).
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.half_edges.len()
    }

    /// Build an `EditMesh` from indexed triangle data.
    ///
    /// * `positions` - vertex positions
    /// * `normals`   - per-vertex normals (same length as positions)
    /// * `uvs`       - per-vertex texture coordinates (same length as positions)
    /// * `indices`   - triangle indices (length must be a multiple of 3)
    pub fn from_triangles(
        positions: &[Vec3],
        normals: &[Vec3],
        uvs: &[Vec2],
        indices: &[u32],
    ) -> Self {
        assert_eq!(positions.len(), normals.len());
        assert_eq!(positions.len(), uvs.len());
        assert!(indices.len() % 3 == 0);

        let vertex_count = positions.len();
        let tri_count = indices.len() / 3;

        // Build vertices (edge will be filled in later).
        let mut vertices: Vec<Vertex> = (0..vertex_count)
            .map(|i| Vertex {
                position: positions[i],
                normal: normals[i],
                uv: uvs[i],
                edge: INVALID_ID,
            })
            .collect();

        let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(tri_count * 3);
        let mut faces: Vec<Face> = Vec::with_capacity(tri_count);

        // Map from directed edge (v0, v1) -> half-edge index for twin linking.
        let mut edge_map: HashMap<(usize, usize), HalfEdgeId> = HashMap::new();

        for tri in 0..tri_count {
            let base = half_edges.len();
            let face_id = faces.len();

            let i0 = indices[tri * 3] as usize;
            let i1 = indices[tri * 3 + 1] as usize;
            let i2 = indices[tri * 3 + 2] as usize;
            let tri_verts = [i0, i1, i2];

            // Create 3 half-edges for this triangle.
            for k in 0..3 {
                let he = HalfEdge {
                    next: base + (k + 1) % 3,
                    prev: base + (k + 2) % 3,
                    twin: INVALID_ID,
                    vertex: tri_verts[k],
                    face: face_id,
                };
                half_edges.push(he);
            }

            // Compute face normal.
            let p0 = positions[i0];
            let p1 = positions[i1];
            let p2 = positions[i2];
            let normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();

            faces.push(Face {
                edge: base,
                normal,
            });

            // Set vertex -> edge references.
            for k in 0..3 {
                if vertices[tri_verts[k]].edge == INVALID_ID {
                    vertices[tri_verts[k]].edge = base + k;
                }
            }

            // Build twin links.
            for k in 0..3 {
                let v0 = tri_verts[k];
                let v1 = tri_verts[(k + 1) % 3];
                let he_id = base + k;

                if let Some(&twin_id) = edge_map.get(&(v1, v0)) {
                    half_edges[he_id].twin = twin_id;
                    half_edges[twin_id].twin = he_id;
                }

                edge_map.insert((v0, v1), he_id);
            }
        }

        EditMesh {
            vertices,
            half_edges,
            faces,
        }
    }

    /// Convert back to indexed triangle data.
    ///
    /// Returns `(positions, normals, uvs, indices)`.
    pub fn to_triangles(&self) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec2>, Vec<u32>) {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for face_id in 0..self.faces.len() {
            let verts: Vec<VertexId> = self.iter_face_vertices(face_id).collect();

            // Triangulate the face as a fan from the first vertex.
            if verts.len() < 3 {
                continue;
            }

            let base = positions.len() as u32;

            for &vid in &verts {
                let v = &self.vertices[vid];
                positions.push(v.position);
                normals.push(v.normal);
                uvs.push(v.uv);
            }

            for i in 1..(verts.len() - 1) {
                indices.push(base);
                indices.push(base + i as u32);
                indices.push(base + i as u32 + 1);
            }
        }

        (positions, normals, uvs, indices)
    }

    /// Iterates over vertex IDs of a face, walking the half-edge loop.
    pub fn iter_face_vertices(&self, face_id: FaceId) -> FaceVertexIter<'_> {
        let start = self.faces[face_id].edge;
        FaceVertexIter {
            mesh: self,
            start,
            current: start,
            done: false,
        }
    }

    /// Add a vertex and return its id.
    pub fn add_vertex(&mut self, position: Vec3, normal: Vec3, uv: Vec2) -> VertexId {
        let id = self.vertices.len();
        self.vertices.push(Vertex {
            position,
            normal,
            uv,
            edge: INVALID_ID,
        });
        id
    }

    /// Add a half-edge and return its id.
    pub fn add_half_edge(&mut self, he: HalfEdge) -> HalfEdgeId {
        let id = self.half_edges.len();
        self.half_edges.push(he);
        id
    }

    /// Add a face from a sequence of vertex ids, creating the half-edge loop.
    /// Returns the face id.
    pub fn add_face(&mut self, vertex_ids: &[VertexId]) -> FaceId {
        assert!(vertex_ids.len() >= 3, "A face must have at least 3 vertices");

        let face_id = self.faces.len();
        let n = vertex_ids.len();
        let base = self.half_edges.len();

        // Create half-edges.
        for i in 0..n {
            let he = HalfEdge {
                next: base + (i + 1) % n,
                prev: base + (i + n - 1) % n,
                twin: INVALID_ID,
                vertex: vertex_ids[i],
                face: face_id,
            };
            self.half_edges.push(he);
        }

        // Set vertex edge references.
        for i in 0..n {
            if self.vertices[vertex_ids[i]].edge == INVALID_ID {
                self.vertices[vertex_ids[i]].edge = base + i;
            }
        }

        // Compute face normal from the first three vertices.
        let p0 = self.vertices[vertex_ids[0]].position;
        let p1 = self.vertices[vertex_ids[1]].position;
        let p2 = self.vertices[vertex_ids[2]].position;
        let normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();

        self.faces.push(Face {
            edge: base,
            normal,
        });

        face_id
    }

    /// Link twin half-edges by scanning all edges.
    /// Call after building faces to establish twin connectivity.
    pub fn link_twins(&mut self) {
        let mut edge_map: HashMap<(VertexId, VertexId), HalfEdgeId> = HashMap::new();

        for he_id in 0..self.half_edges.len() {
            let he = self.half_edges[he_id];
            if he.face == INVALID_ID {
                continue;
            }
            let v0 = he.vertex;
            let v1 = self.half_edges[he.next].vertex;

            if let Some(&twin_id) = edge_map.get(&(v1, v0)) {
                self.half_edges[he_id].twin = twin_id;
                self.half_edges[twin_id].twin = he_id;
            }
            edge_map.insert((v0, v1), he_id);
        }
    }
}

impl Default for EditMesh {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────
// FaceVertexIter
// ─────────────────────────────────────────────────────────────────────

/// Iterator over vertex IDs around a face.
pub struct FaceVertexIter<'a> {
    mesh: &'a EditMesh,
    start: HalfEdgeId,
    current: HalfEdgeId,
    done: bool,
}

impl<'a> Iterator for FaceVertexIter<'a> {
    type Item = VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let he = &self.mesh.half_edges[self.current];
        let vertex = he.vertex;

        self.current = he.next;
        if self.current == self.start {
            self.done = true;
        }

        Some(vertex)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_triangles_single_tri() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let normals = vec![Vec3::Z; 3];
        let uvs = vec![Vec2::ZERO; 3];
        let indices = vec![0, 1, 2];

        let mesh = EditMesh::from_triangles(&positions, &normals, &uvs, &indices);

        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.edge_count(), 3);
    }

    #[test]
    fn face_vertex_iteration() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let normals = vec![Vec3::Z; 3];
        let uvs = vec![Vec2::ZERO; 3];
        let indices = vec![0, 1, 2];

        let mesh = EditMesh::from_triangles(&positions, &normals, &uvs, &indices);
        let verts: Vec<VertexId> = mesh.iter_face_vertices(0).collect();
        assert_eq!(verts, vec![0, 1, 2]);
    }

    #[test]
    fn roundtrip_single_triangle() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let normals = vec![Vec3::Z; 3];
        let uvs = vec![Vec2::ZERO; 3];
        let indices = vec![0, 1, 2];

        let mesh = EditMesh::from_triangles(&positions, &normals, &uvs, &indices);
        let (out_pos, out_norm, out_uv, out_idx) = mesh.to_triangles();

        assert_eq!(out_pos.len(), 3);
        assert_eq!(out_idx.len(), 3);
        assert_eq!(out_norm.len(), 3);
        assert_eq!(out_uv.len(), 3);
    }

    #[test]
    fn twin_linking() {
        // Two triangles sharing an edge: (0,1,2) and (1,3,2).
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 1.0, 0.0),
            Vec3::new(1.5, 1.0, 0.0),
        ];
        let normals = vec![Vec3::Z; 4];
        let uvs = vec![Vec2::ZERO; 4];
        let indices = vec![0, 1, 2, 1, 3, 2];

        let mesh = EditMesh::from_triangles(&positions, &normals, &uvs, &indices);

        // The edge 1->2 in tri0 should be twin of edge 2->1 in tri1.
        let mut found_twin = false;
        for he in &mesh.half_edges {
            if he.twin != INVALID_ID {
                found_twin = true;
                break;
            }
        }
        assert!(found_twin, "Expected at least one twin-linked edge");
    }
}
