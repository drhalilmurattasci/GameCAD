//! Per-node WGSL code generation helpers.

use crate::graph::{NodeKind, PinDirection, PinId};

/// Generate a unique variable name (`v0`, `v1`, ...) and advance the counter.
pub(crate) fn next_var(counter: &mut u32) -> String {
    let name = format!("v{counter}");
    *counter += 1;
    name
}

/// Return the first output pin ID of a node. Panics if none exists.
pub(crate) fn output_pin(node: &crate::graph::MaterialNode) -> PinId {
    node.pins
        .iter()
        .find(|p| p.direction == PinDirection::Output)
        .expect("Node must have an output pin")
        .id
}

/// Return a WGSL default literal for an unconnected input pin.
pub(crate) fn default_value_for_input(kind: NodeKind, pin_name: &str) -> String {
    match (kind, pin_name) {
        (_, "UV") => "in.uv".to_string(),
        (_, "Normal") => "in.world_normal".to_string(),
        (_, "Scale") | (_, "Strength") => "1.0".to_string(),
        (_, "IOR") => "0.04".to_string(),
        (_, "Factor") | (_, "T") => "0.5".to_string(),
        (_, "A") => "0.0".to_string(),
        (_, "B") => "0.0".to_string(),
        _ => "0.0".to_string(),
    }
}
