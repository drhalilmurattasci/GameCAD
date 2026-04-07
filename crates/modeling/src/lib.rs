//! # Modeling
//!
//! Half-edge mesh data structure, primitive generation, mesh editing operations,
//! element selection, and constructive solid geometry for the Forge Editor.

pub mod csg;
pub mod half_edge;
pub mod operations;
pub mod primitives;
pub mod selection;

/// Convenience re-exports of the most commonly used types and functions.
pub mod prelude {
    pub use crate::csg::{csg_operation, CsgOp};
    pub use crate::half_edge::{EditMesh, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};
    pub use crate::operations::*;
    pub use crate::primitives::*;
    pub use crate::selection::{MeshSelection, SelectionMode};
}
