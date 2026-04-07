//! NodeEditorState and DragWire types.

use crate::graph::{MaterialGraph, NodeId, PinId};

/// Persistent state for the node editor widget.
///
/// Owns the [`MaterialGraph`] being edited, the camera transform, and
/// transient interaction state (selected node, in-progress wire drag, etc.).
pub struct NodeEditorState {
    /// The material graph being edited.
    pub graph: MaterialGraph,
    /// Camera pan offset in graph-space units.
    pub camera_offset: glam::Vec2,
    /// Zoom level (1.0 = 100 %).
    pub zoom: f32,
    /// Currently selected node, if any.
    pub selected_node: Option<NodeId>,
    /// Node currently being dragged, if any.
    pub dragging_node: Option<NodeId>,
    /// Wire currently being dragged from an output pin, if any.
    pub dragging_wire: Option<DragWire>,
}

/// In-progress wire drag from an output pin to the mouse cursor.
pub struct DragWire {
    /// Node the wire originates from.
    pub from_node: NodeId,
    /// Output pin the wire originates from.
    pub from_pin: PinId,
    /// Current mouse position in graph-space coordinates.
    pub current_pos: glam::Vec2,
}

impl NodeEditorState {
    /// Create editor state wrapping the given graph at default zoom.
    pub fn new(graph: MaterialGraph) -> Self {
        Self {
            graph,
            camera_offset: glam::Vec2::ZERO,
            zoom: 1.0,
            selected_node: None,
            dragging_node: None,
            dragging_wire: None,
        }
    }
}
