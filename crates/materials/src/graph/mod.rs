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
}
