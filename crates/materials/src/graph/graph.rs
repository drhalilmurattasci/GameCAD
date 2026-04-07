//! MaterialGraph -- add/remove/connect/disconnect nodes.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::connection::Connection;
use super::node::MaterialNode;
use super::types::{NodeId, PinId};

// ─────────────────────────────────────────────────────────────────────
// MaterialGraph
// ─────────────────────────────────────────────────────────────────────

/// The full node graph for a material.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialGraph {
    /// All nodes in the graph, keyed by their unique ID.
    pub nodes: HashMap<NodeId, MaterialNode>,
    /// All connections (wires) between node pins.
    pub connections: Vec<Connection>,
}

impl MaterialGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node, returning its id.
    pub fn add_node(&mut self, node: MaterialNode) -> NodeId {
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node and all connections involving it.
    pub fn remove_node(&mut self, id: &NodeId) -> Option<MaterialNode> {
        self.connections
            .retain(|c| c.from_node != *id && c.to_node != *id);
        self.nodes.remove(id)
    }

    /// Add a connection between two pins.
    pub fn connect(
        &mut self,
        from_node: NodeId,
        from_pin: PinId,
        to_node: NodeId,
        to_pin: PinId,
    ) {
        // Remove any existing connection to the same input pin.
        self.connections
            .retain(|c| !(c.to_node == to_node && c.to_pin == to_pin));
        self.connections.push(Connection {
            from_node,
            from_pin,
            to_node,
            to_pin,
        });
    }

    /// Disconnect a specific connection (by to-pin).
    pub fn disconnect(&mut self, to_node: &NodeId, to_pin: &PinId) {
        self.connections
            .retain(|c| !(c.to_node == *to_node && c.to_pin == *to_pin));
    }

    /// Get a reference to a node.
    pub fn get_node(&self, id: &NodeId) -> Option<&MaterialNode> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut MaterialNode> {
        self.nodes.get_mut(id)
    }
}
