//! Node-based material graph with typed pins, connections, and validation.

pub mod connection;
#[allow(clippy::module_inception)]
pub mod graph;
pub mod node;
pub mod types;
pub mod validate;

// Re-export all public types at the module level for backwards compatibility.
pub use connection::Connection;
pub use graph::MaterialGraph;
pub use node::{MaterialNode, NodeKind};
pub use types::{NodeId, Pin, PinDirection, PinId, PinType, PinValue};
pub use validate::GraphError;

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn make_simple_graph() -> MaterialGraph {
        let mut graph = MaterialGraph::new();
        let color_node = MaterialNode::new(NodeKind::ConstantColor, Vec2::ZERO);
        let output_node = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let color_out_pin = color_node
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let albedo_in_pin = output_node
            .pins
            .iter()
            .find(|p| p.name == "Albedo")
            .unwrap()
            .id;

        let color_id = color_node.id;
        let output_id = output_node.id;

        graph.add_node(color_node);
        graph.add_node(output_node);
        graph.connect(color_id, color_out_pin, output_id, albedo_in_pin);

        graph
    }

    #[test]
    fn add_and_get_node() {
        let mut graph = MaterialGraph::new();
        let node = MaterialNode::new(NodeKind::ConstantFloat, Vec2::ZERO);
        let id = node.id;
        graph.add_node(node);
        assert!(graph.get_node(&id).is_some());
    }

    #[test]
    fn remove_node_removes_connections() {
        let mut graph = make_simple_graph();
        let ids: Vec<NodeId> = graph.nodes.keys().copied().collect();
        graph.remove_node(&ids[0]);
        assert!(graph.connections.is_empty());
    }

    #[test]
    fn validate_missing_output() {
        let graph = MaterialGraph::new();
        let errors = graph.validate();
        assert!(errors.iter().any(|e| matches!(e, GraphError::MissingOutput)));
    }

    #[test]
    fn validate_disconnected_output() {
        let mut graph = MaterialGraph::new();
        graph.add_node(MaterialNode::new(NodeKind::PbrOutput, Vec2::ZERO));
        let errors = graph.validate();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GraphError::DisconnectedOutput { .. })));
    }

    #[test]
    fn validate_type_mismatch() {
        let mut graph = MaterialGraph::new();
        let float_node = MaterialNode::new(NodeKind::ConstantFloat, Vec2::ZERO);
        let output_node = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let float_out = float_node
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        // Try to connect Float -> Color (Albedo) -- type mismatch
        let albedo_pin = output_node
            .pins
            .iter()
            .find(|p| p.name == "Albedo")
            .unwrap()
            .id;

        let fid = float_node.id;
        let oid = output_node.id;
        graph.add_node(float_node);
        graph.add_node(output_node);
        graph.connect(fid, float_out, oid, albedo_pin);

        let errors = graph.validate();
        assert!(errors
            .iter()
            .any(|e| matches!(e, GraphError::TypeMismatch { .. })));
    }

    #[test]
    fn topological_sort_simple() {
        let graph = make_simple_graph();
        let sorted = graph.topological_sort().expect("no cycle");
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn cycle_detection() {
        let mut graph = MaterialGraph::new();
        let a = MaterialNode::new(NodeKind::MathAdd, Vec2::ZERO);
        let b = MaterialNode::new(NodeKind::MathAdd, Vec2::new(200.0, 0.0));

        let a_out = a
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let a_in = a
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Input && p.name == "A")
            .unwrap()
            .id;
        let b_out = b
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let b_in = b
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Input && p.name == "A")
            .unwrap()
            .id;

        let aid = a.id;
        let bid = b.id;
        graph.add_node(a);
        graph.add_node(b);

        graph.connect(aid, a_out, bid, b_in);
        graph.connect(bid, b_out, aid, a_in);

        assert!(graph.topological_sort().is_none());
    }

    #[test]
    fn disconnect() {
        let mut graph = make_simple_graph();
        assert_eq!(graph.connections.len(), 1);

        let conn = graph.connections[0].clone();
        graph.disconnect(&conn.to_node, &conn.to_pin);
        assert!(graph.connections.is_empty());
    }

    #[test]
    fn connect_replaces_existing_input() {
        let mut graph = make_simple_graph();
        let output_node_id = graph
            .nodes
            .values()
            .find(|n| n.kind == NodeKind::PbrOutput)
            .unwrap()
            .id;
        let albedo_pin = graph
            .nodes
            .get(&output_node_id)
            .unwrap()
            .pins
            .iter()
            .find(|p| p.name == "Albedo")
            .unwrap()
            .id;

        // Add a second color node and connect to the same input.
        let color2 = MaterialNode::new(NodeKind::ConstantColor, Vec2::new(0.0, 100.0));
        let color2_out = color2
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let color2_id = color2.id;
        graph.add_node(color2);
        graph.connect(color2_id, color2_out, output_node_id, albedo_pin);

        // Should still have only 1 connection to Albedo (the old one replaced).
        let albedo_conns: Vec<_> = graph
            .connections
            .iter()
            .filter(|c| c.to_node == output_node_id && c.to_pin == albedo_pin)
            .collect();
        assert_eq!(albedo_conns.len(), 1);
        assert_eq!(albedo_conns[0].from_node, color2_id);
    }

    #[test]
    fn topological_sort_three_nodes() {
        let mut graph = MaterialGraph::new();
        let f = MaterialNode::new(NodeKind::ConstantFloat, Vec2::ZERO);
        let add = MaterialNode::new(NodeKind::MathAdd, Vec2::new(200.0, 0.0));
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(400.0, 0.0));

        let f_out = f
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let add_a = add
            .pins
            .iter()
            .find(|p| p.name == "A")
            .unwrap()
            .id;
        let add_out = add
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let metallic_in = output
            .pins
            .iter()
            .find(|p| p.name == "Metallic")
            .unwrap()
            .id;

        let fid = f.id;
        let addid = add.id;
        let oid = output.id;

        graph.add_node(f);
        graph.add_node(add);
        graph.add_node(output);
        graph.connect(fid, f_out, addid, add_a);
        graph.connect(addid, add_out, oid, metallic_in);

        let sorted = graph.topological_sort().expect("no cycle");
        assert_eq!(sorted.len(), 3);
        // f must come before add, add before output.
        let f_idx = sorted.iter().position(|id| *id == fid).unwrap();
        let add_idx = sorted.iter().position(|id| *id == addid).unwrap();
        let out_idx = sorted.iter().position(|id| *id == oid).unwrap();
        assert!(f_idx < add_idx);
        assert!(add_idx < out_idx);
    }

    #[test]
    fn validate_no_errors_for_valid_graph() {
        let graph = make_simple_graph();
        let errors = graph.validate();
        // Only disconnected output pins expected (Normal, Metallic, etc.).
        // No type mismatch, no cycle, no missing output.
        assert!(!errors.iter().any(|e| matches!(e, GraphError::MissingOutput)));
        assert!(!errors.iter().any(|e| matches!(e, GraphError::Cycle(_))));
        assert!(!errors.iter().any(|e| matches!(e, GraphError::TypeMismatch { .. })));
    }

    #[test]
    fn remove_nonexistent_node() {
        let mut graph = make_simple_graph();
        let fake_id = NodeId::new();
        let result = graph.remove_node(&fake_id);
        assert!(result.is_none());
    }

    #[test]
    fn get_nonexistent_node() {
        let graph = MaterialGraph::new();
        assert!(graph.get_node(&NodeId::new()).is_none());
    }

    #[test]
    fn empty_graph_topological_sort() {
        let graph = MaterialGraph::new();
        let sorted = graph.topological_sort();
        assert_eq!(sorted, Some(vec![]));
    }

    #[test]
    fn all_node_kinds_have_labels() {
        for kind in NodeKind::ALL {
            let label = kind.label();
            assert!(!label.is_empty(), "{:?} has empty label", kind);
        }
    }

    #[test]
    fn all_node_kinds_have_pins() {
        for &kind in NodeKind::ALL {
            let pins = kind.default_pins();
            // Every node kind should have at least one pin.
            assert!(!pins.is_empty(), "{:?} has no pins", kind);
        }
    }

    #[test]
    fn node_find_pin_by_id() {
        let node = MaterialNode::new(NodeKind::MathAdd, Vec2::ZERO);
        let pin_id = node.pins[0].id;
        assert!(node.find_pin(&pin_id).is_some());
        assert!(node.find_pin(&PinId::new()).is_none());
    }
}
