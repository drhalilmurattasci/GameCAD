//! Mesh element selection: vertices, edges, faces, loops, rings, grow/shrink.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::half_edge::{EditMesh, FaceId, HalfEdgeId, VertexId, INVALID_ID};

/// Selection mode for mesh editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Select individual vertices.
    Vertex,
    /// Select edges (half-edge pairs).
    Edge,
    /// Select faces (polygons).
    Face,
    /// Select the entire object.
    Object,
}

/// Tracks selected mesh elements (vertices, edges, and faces).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshSelection {
    /// Currently selected vertex indices.
    pub vertices: HashSet<VertexId>,
    /// Currently selected half-edge indices.
    pub edges: HashSet<HalfEdgeId>,
    /// Currently selected face indices.
    pub faces: HashSet<FaceId>,
}

impl MeshSelection {
    /// Creates an empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex to the selection.
    #[inline]
    pub fn select_vertex(&mut self, vertex_id: VertexId) {
        self.vertices.insert(vertex_id);
    }

    /// Add an edge (half-edge) to the selection.
    #[inline]
    pub fn select_edge(&mut self, edge_id: HalfEdgeId) {
        self.edges.insert(edge_id);
    }

    /// Add a face to the selection.
    #[inline]
    pub fn select_face(&mut self, face_id: FaceId) {
        self.faces.insert(face_id);
    }

    /// Clear all selections.
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
        self.faces.clear();
    }

    /// Select all elements in the mesh.
    pub fn select_all(&mut self, mesh: &EditMesh) {
        self.vertices = (0..mesh.vertices.len()).collect();
        self.edges = (0..mesh.half_edges.len()).collect();
        self.faces = (0..mesh.faces.len()).collect();
    }

    /// Select an edge loop starting from the given half-edge.
    ///
    /// An edge loop follows edges that continue "straight through" quad vertices.
    /// For each edge, we cross the face and pick the opposite edge.
    pub fn select_loop(&mut self, mesh: &EditMesh, start_edge: HalfEdgeId) {
        if start_edge >= mesh.half_edges.len() {
            return;
        }

        self.edges.insert(start_edge);
        // Also insert the twin if it exists.
        let twin = mesh.half_edges[start_edge].twin;
        if twin != INVALID_ID {
            self.edges.insert(twin);
        }

        // Walk forward.
        self.walk_loop(mesh, start_edge, true);
        // Walk backward.
        self.walk_loop(mesh, start_edge, false);
    }

    /// Walk an edge loop in one direction from `start_edge`, inserting edges
    /// until the loop completes, hits a boundary, or encounters a non-quad face.
    fn walk_loop(&mut self, mesh: &EditMesh, start_edge: HalfEdgeId, forward: bool) {
        let mut current = start_edge;

        loop {
            // Cross to the other side of the edge.
            let cross_edge = if forward {
                mesh.half_edges[current].twin
            } else {
                let twin = mesh.half_edges[current].twin;
                if twin == INVALID_ID {
                    break;
                }
                twin
            };

            if cross_edge == INVALID_ID {
                break;
            }

            // Walk across the face to find the "opposite" edge.
            // In a quad face, the opposite edge is 2 steps away.
            let face_id = mesh.half_edges[cross_edge].face;
            if face_id == INVALID_ID {
                break;
            }

            // Count edges in the face.
            let face_edge_count = mesh.iter_face_vertices(face_id).count();

            // For quads, the opposite edge is 2 half-edges away.
            // For non-quads, we stop the loop.
            if face_edge_count != 4 {
                break;
            }

            // Walk 2 edges from the crossing edge to get the opposite.
            let next1 = mesh.half_edges[cross_edge].next;
            let next2 = mesh.half_edges[next1].next;

            if next2 == start_edge || next2 == current {
                break;
            }

            current = next2;
            if !self.edges.insert(current) {
                break; // Already selected, loop complete.
            }

            // Also insert twin.
            let twin = mesh.half_edges[current].twin;
            if twin != INVALID_ID {
                self.edges.insert(twin);
            }
        }
    }

    /// Select an edge ring starting from the given half-edge.
    ///
    /// An edge ring selects parallel edges around a loop of faces.
    /// From the start edge, it crosses to the adjacent face and picks the
    /// "next" parallel edge (one step in next, then one step in next).
    pub fn select_ring(&mut self, mesh: &EditMesh, start_edge: HalfEdgeId) {
        if start_edge >= mesh.half_edges.len() {
            return;
        }

        self.edges.insert(start_edge);
        let twin = mesh.half_edges[start_edge].twin;
        if twin != INVALID_ID {
            self.edges.insert(twin);
        }

        // Walk in both directions around the ring.
        self.walk_ring(mesh, start_edge, true);
        self.walk_ring(mesh, start_edge, false);
    }

    /// Walk an edge ring in one direction, inserting parallel edges until the
    /// ring completes, hits a boundary, or encounters a non-quad face.
    fn walk_ring(&mut self, mesh: &EditMesh, start_edge: HalfEdgeId, use_next: bool) {
        let mut current = start_edge;

        loop {
            let he = &mesh.half_edges[current];
            let face_id = he.face;
            if face_id == INVALID_ID {
                break;
            }

            // Count face edges - only works cleanly for quads.
            let face_edge_count = mesh.iter_face_vertices(face_id).count();
            if face_edge_count != 4 {
                break;
            }

            // For a ring: move one step in the face to get the "parallel" edge.
            let parallel = if use_next {
                mesh.half_edges[he.next].next
            } else {
                mesh.half_edges[he.prev].prev
            };

            // Cross to the twin to continue the ring.
            let twin = mesh.half_edges[parallel].twin;
            if twin == INVALID_ID {
                // Still select the parallel edge.
                self.edges.insert(parallel);
                break;
            }

            if !self.edges.insert(parallel) {
                break; // Already visited.
            }
            self.edges.insert(twin);

            if twin == start_edge || parallel == start_edge {
                break;
            }

            current = twin;
        }
    }

    /// Grow the face selection by adding all faces adjacent to currently selected faces.
    pub fn grow_selection(&mut self, mesh: &EditMesh) {
        let mut new_faces: HashSet<FaceId> = self.faces.clone();
        let mut new_vertices: HashSet<VertexId> = self.vertices.clone();
        let mut new_edges: HashSet<HalfEdgeId> = self.edges.clone();

        // Grow faces: for each selected face, find adjacent faces via twin edges.
        for &fid in &self.faces {
            if fid >= mesh.faces.len() {
                continue;
            }
            let start = mesh.faces[fid].edge;
            let mut he_id = start;
            loop {
                let he = &mesh.half_edges[he_id];
                new_edges.insert(he_id);

                // Add vertex.
                new_vertices.insert(he.vertex);

                // Add adjacent face via twin.
                if he.twin != INVALID_ID {
                    let adj_face = mesh.half_edges[he.twin].face;
                    if adj_face != INVALID_ID {
                        new_faces.insert(adj_face);
                    }
                }

                he_id = he.next;
                if he_id == start {
                    break;
                }
            }
        }

        // Grow vertices: for each selected vertex, find connected vertices.
        for &vid in &self.vertices {
            if vid >= mesh.vertices.len() {
                continue;
            }
            let start_he = mesh.vertices[vid].edge;
            if start_he == INVALID_ID {
                continue;
            }
            // Walk around the vertex fan.
            let next_vid = mesh.half_edges[mesh.half_edges[start_he].next].vertex;
            new_vertices.insert(next_vid);
        }

        self.faces = new_faces;
        self.vertices = new_vertices;
        self.edges = new_edges;
    }

    /// Shrink the face selection by removing faces on the boundary of the selection.
    pub fn shrink_selection(&mut self, mesh: &EditMesh) {
        let mut boundary_faces: HashSet<FaceId> = HashSet::new();

        for &fid in &self.faces {
            if fid >= mesh.faces.len() {
                continue;
            }
            let start = mesh.faces[fid].edge;
            let mut he_id = start;
            let mut is_interior = true;
            loop {
                let he = &mesh.half_edges[he_id];
                if he.twin == INVALID_ID {
                    is_interior = false;
                    break;
                }
                let adj_face = mesh.half_edges[he.twin].face;
                if adj_face == INVALID_ID || !self.faces.contains(&adj_face) {
                    is_interior = false;
                    break;
                }
                he_id = he.next;
                if he_id == start {
                    break;
                }
            }
            if !is_interior {
                boundary_faces.insert(fid);
            }
        }

        for fid in boundary_faces {
            self.faces.remove(&fid);
        }
    }

    /// Returns true if anything is selected.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() && self.edges.is_empty() && self.faces.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::generate_cube;

    #[test]
    fn select_vertex_and_clear() {
        let mut sel = MeshSelection::new();
        sel.select_vertex(0);
        sel.select_vertex(1);
        assert_eq!(sel.vertices.len(), 2);
        sel.clear();
        assert!(sel.is_empty());
    }

    #[test]
    fn select_all() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_all(&mesh);
        assert_eq!(sel.vertices.len(), mesh.vertex_count());
        assert_eq!(sel.faces.len(), mesh.face_count());
        assert_eq!(sel.edges.len(), mesh.edge_count());
    }

    #[test]
    fn grow_selection_adds_adjacent() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_face(0);
        let before = sel.faces.len();
        sel.grow_selection(&mesh);
        assert!(sel.faces.len() > before);
    }

    #[test]
    fn shrink_selection_removes_boundary() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_all(&mesh);
        let before = sel.faces.len();
        sel.shrink_selection(&mesh);
        // For a cube, all faces are on the boundary, so shrinking should remove them all.
        assert!(sel.faces.len() < before);
    }

    #[test]
    fn select_edge_and_face() {
        let mut sel = MeshSelection::new();
        sel.select_edge(5);
        sel.select_face(2);
        assert_eq!(sel.edges.len(), 1);
        assert_eq!(sel.faces.len(), 1);
        assert!(!sel.is_empty());
    }

    #[test]
    fn select_duplicate_vertex() {
        let mut sel = MeshSelection::new();
        sel.select_vertex(0);
        sel.select_vertex(0);
        // HashSet should deduplicate.
        assert_eq!(sel.vertices.len(), 1);
    }

    #[test]
    fn grow_empty_selection_is_noop() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.grow_selection(&mesh);
        assert!(sel.is_empty());
    }

    #[test]
    fn shrink_empty_selection_is_noop() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.shrink_selection(&mesh);
        assert!(sel.is_empty());
    }

    #[test]
    fn shrink_single_face_removes_it() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_face(0);
        sel.shrink_selection(&mesh);
        // A single face is always on the boundary, so it should be removed.
        assert!(sel.faces.is_empty());
    }

    #[test]
    fn select_loop_on_out_of_bounds_edge() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_loop(&mesh, 99999);
        // Should be a no-op, not panic.
        assert!(sel.edges.is_empty());
    }

    #[test]
    fn select_ring_on_out_of_bounds_edge() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_ring(&mesh, 99999);
        assert!(sel.edges.is_empty());
    }

    #[test]
    fn grow_with_out_of_bounds_face() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.faces.insert(99999);
        // Should not panic.
        sel.grow_selection(&mesh);
    }

    #[test]
    fn shrink_with_out_of_bounds_face() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.faces.insert(99999);
        sel.shrink_selection(&mesh);
    }

    #[test]
    fn select_loop_inserts_start_edge() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_loop(&mesh, 0);
        assert!(sel.edges.contains(&0), "Loop should contain the start edge");
    }

    #[test]
    fn select_ring_inserts_start_edge() {
        let mesh = generate_cube(1.0);
        let mut sel = MeshSelection::new();
        sel.select_ring(&mesh, 0);
        assert!(sel.edges.contains(&0), "Ring should contain the start edge");
    }
}
