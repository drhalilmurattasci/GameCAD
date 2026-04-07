//! Compile a [`MaterialGraph`] into a WGSL fragment shader string.

pub mod assemble;
pub mod codegen;

// Re-export public API for backwards compatibility.
pub use assemble::compile_graph;

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{MaterialGraph, MaterialNode, NodeKind, PinDirection};
    use glam::Vec2;

    #[test]
    fn compile_empty_graph_fails() {
        let graph = MaterialGraph::new();
        assert!(compile_graph(&graph).is_err());
    }

    #[test]
    fn compile_output_only() {
        let mut graph = MaterialGraph::new();
        graph.add_node(MaterialNode::new(NodeKind::PbrOutput, Vec2::ZERO));
        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("fs_main"));
        assert!(shader.contains("PbrResult"));
    }

    #[test]
    fn compile_color_to_output() {
        let mut graph = MaterialGraph::new();
        let color = MaterialNode::new(NodeKind::ConstantColor, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let color_out = color
            .pins
            .iter()
            .find(|p| p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let albedo_in = output
            .pins
            .iter()
            .find(|p| p.name == "Albedo")
            .unwrap()
            .id;

        let cid = color.id;
        let oid = output.id;
        graph.add_node(color);
        graph.add_node(output);
        graph.connect(cid, color_out, oid, albedo_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("result.albedo"));
        assert!(shader.contains("vec4<f32>(0.5, 0.5, 0.5, 1.0)"));
    }

    #[test]
    fn compile_math_chain() {
        let mut graph = MaterialGraph::new();
        let f1 = MaterialNode::new(NodeKind::ConstantFloat, Vec2::ZERO);
        let f2 = MaterialNode::new(NodeKind::ConstantFloat, Vec2::new(0.0, 100.0));
        let add = MaterialNode::new(NodeKind::MathAdd, Vec2::new(200.0, 50.0));
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(400.0, 0.0));

        let f1_out = codegen::output_pin(&f1);
        let f2_out = codegen::output_pin(&f2);
        let add_a = add.pins.iter().find(|p| p.name == "A").unwrap().id;
        let add_b = add.pins.iter().find(|p| p.name == "B").unwrap().id;
        let add_out = codegen::output_pin(&add);
        let metallic_in = output
            .pins
            .iter()
            .find(|p| p.name == "Metallic")
            .unwrap()
            .id;

        let f1id = f1.id;
        let f2id = f2.id;
        let addid = add.id;
        let oid = output.id;

        graph.add_node(f1);
        graph.add_node(f2);
        graph.add_node(add);
        graph.add_node(output);

        graph.connect(f1id, f1_out, addid, add_a);
        graph.connect(f2id, f2_out, addid, add_b);
        graph.connect(addid, add_out, oid, metallic_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("+"));
        assert!(shader.contains("result.metallic"));
    }
}
