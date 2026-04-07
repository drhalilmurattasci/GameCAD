//! Graph validation, cycle detection, and topological sorting.

use std::collections::{HashMap, VecDeque};
use std::fmt;

use super::graph::MaterialGraph;
use super::node::NodeKind;
use super::types::{NodeId, PinDirection, PinType};

// ─────────────────────────────────────────────────────────────────────
// GraphError
// ─────────────────────────────────────────────────────────────────────

/// Validation errors found in a material graph.
#[derive(Debug, Clone)]
pub enum GraphError {
    /// A cycle was detected involving the listed nodes.
    Cycle(Vec<NodeId>),
    /// A connection links pins of incompatible types.
    TypeMismatch {
        connection_index: usize,
        from_type: PinType,
        to_type: PinType,
    },
    /// The graph has no PbrOutput node.
    MissingOutput,
    /// A PbrOutput input pin has no incoming connection and no default value.
    DisconnectedOutput { pin_name: String },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::Cycle(ids) => write!(f, "Cycle detected involving {} nodes", ids.len()),
            GraphError::TypeMismatch {
                connection_index,
                from_type,
                to_type,
            } => write!(
                f,
                "Type mismatch on connection {connection_index}: {from_type:?} -> {to_type:?}"
            ),
            GraphError::MissingOutput => write!(f, "Graph has no PBR Output node"),
            GraphError::DisconnectedOutput { pin_name } => {
                write!(f, "PBR Output pin '{pin_name}' is disconnected")
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Validation implementation on MaterialGraph
// ─────────────────────────────────────────────────────────────────────

impl MaterialGraph {
    /// Validate the graph, returning any errors found.
    pub fn validate(&self) -> Vec<GraphError> {
        let mut errors = Vec::new();

        // Check for PbrOutput
        let output_nodes: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::PbrOutput)
            .collect();
        if output_nodes.is_empty() {
            errors.push(GraphError::MissingOutput);
        }

        // Check type mismatches
        for (i, conn) in self.connections.iter().enumerate() {
            let from_type = self
                .nodes
                .get(&conn.from_node)
                .and_then(|n| n.find_pin(&conn.from_pin))
                .map(|p| p.pin_type);
            let to_type = self
                .nodes
                .get(&conn.to_node)
                .and_then(|n| n.find_pin(&conn.to_pin))
                .map(|p| p.pin_type);

            if let (Some(ft), Some(tt)) = (from_type, to_type)
                && !types_compatible(ft, tt)
            {
                errors.push(GraphError::TypeMismatch {
                    connection_index: i,
                    from_type: ft,
                    to_type: tt,
                });
            }
        }

        // Check for disconnected output pins
        for output_node in &output_nodes {
            for pin in &output_node.pins {
                if pin.direction == PinDirection::Input {
                    let connected = self
                        .connections
                        .iter()
                        .any(|c| c.to_node == output_node.id && c.to_pin == pin.id);
                    if !connected && pin.default_value.is_none() {
                        errors.push(GraphError::DisconnectedOutput {
                            pin_name: pin.name.clone(),
                        });
                    }
                }
            }
        }

        // Check for cycles using Kahn's algorithm
        if self.has_cycle() {
            let node_ids: Vec<NodeId> = self.nodes.keys().copied().collect();
            errors.push(GraphError::Cycle(node_ids));
        }

        errors
    }

    /// Returns `true` if the graph contains a cycle (Kahn's algorithm).
    fn has_cycle(&self) -> bool {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.insert(*id, 0);
        }
        for conn in &self.connections {
            *in_degree.entry(conn.to_node).or_insert(0) += 1;
        }

        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut visited = 0usize;
        while let Some(node_id) = queue.pop_front() {
            visited += 1;
            for conn in &self.connections {
                if conn.from_node == node_id
                    && let Some(deg) = in_degree.get_mut(&conn.to_node)
                {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(conn.to_node);
                    }
                }
            }
        }

        visited != self.nodes.len()
    }

    /// Topological sort of nodes (Kahn's algorithm). Returns `None` if there is a cycle.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.insert(*id, 0);
        }
        for conn in &self.connections {
            *in_degree.entry(conn.to_node).or_insert(0) += 1;
        }

        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::with_capacity(self.nodes.len());
        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id);
            for conn in &self.connections {
                if conn.from_node == node_id
                    && let Some(deg) = in_degree.get_mut(&conn.to_node)
                {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(conn.to_node);
                    }
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            Some(sorted)
        } else {
            None
        }
    }
}

/// Check if two pin types are compatible for connection.
///
/// Exact matches always pass. Additionally, `Color` is interchangeable with
/// `Vec4` and `Vec3` to support common PBR workflows.
fn types_compatible(from: PinType, to: PinType) -> bool {
    if from == to {
        return true;
    }
    // Allow Color <-> Vec4 and Vec3 <-> Color implicit conversions.
    matches!(
        (from, to),
        (PinType::Color, PinType::Vec4)
            | (PinType::Vec4, PinType::Color)
            | (PinType::Vec3, PinType::Color)
            | (PinType::Color, PinType::Vec3)
    )
}
