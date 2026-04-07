//! Connection (wire) between two pins on material graph nodes.

use serde::{Deserialize, Serialize};

use super::types::{NodeId, PinId};

/// A wire between two pins on (possibly different) nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Source node (output side).
    pub from_node: NodeId,
    /// Source pin on `from_node`.
    pub from_pin: PinId,
    /// Destination node (input side).
    pub to_node: NodeId,
    /// Destination pin on `to_node`.
    pub to_pin: PinId,
}
