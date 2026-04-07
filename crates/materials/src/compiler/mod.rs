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

    #[test]
    fn compile_multiply_node() {
        let mut graph = MaterialGraph::new();
        let f1 = MaterialNode::new(NodeKind::ConstantFloat, Vec2::ZERO);
        let mul = MaterialNode::new(NodeKind::MathMultiply, Vec2::new(200.0, 0.0));
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(400.0, 0.0));

        let f1_out = codegen::output_pin(&f1);
        let mul_a = mul.pins.iter().find(|p| p.name == "A").unwrap().id;
        let mul_out = codegen::output_pin(&mul);
        let roughness_in = output.pins.iter().find(|p| p.name == "Roughness").unwrap().id;

        let f1id = f1.id;
        let mulid = mul.id;
        let oid = output.id;
        graph.add_node(f1);
        graph.add_node(mul);
        graph.add_node(output);
        graph.connect(f1id, f1_out, mulid, mul_a);
        graph.connect(mulid, mul_out, oid, roughness_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("*"));
        assert!(shader.contains("result.roughness"));
    }

    #[test]
    fn compile_lerp_node() {
        let mut graph = MaterialGraph::new();
        let lerp = MaterialNode::new(NodeKind::MathLerp, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let lerp_out = codegen::output_pin(&lerp);
        let metallic_in = output.pins.iter().find(|p| p.name == "Metallic").unwrap().id;

        let lid = lerp.id;
        let oid = output.id;
        graph.add_node(lerp);
        graph.add_node(output);
        graph.connect(lid, lerp_out, oid, metallic_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("mix("));
    }

    #[test]
    fn compile_mix_node() {
        let mut graph = MaterialGraph::new();
        let mix_node = MaterialNode::new(NodeKind::MathMix, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let mix_out = codegen::output_pin(&mix_node);
        let albedo_in = output.pins.iter().find(|p| p.name == "Albedo").unwrap().id;

        let mid = mix_node.id;
        let oid = output.id;
        graph.add_node(mix_node);
        graph.add_node(output);
        graph.connect(mid, mix_out, oid, albedo_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("mix("));
        // The fix: should NOT contain vec4<f32>(factor) wrapping.
        assert!(!shader.contains("vec4<f32>(0.5)"), "MathMix should not wrap factor in vec4");
    }

    #[test]
    fn compile_fresnel_node() {
        let mut graph = MaterialGraph::new();
        let fresnel = MaterialNode::new(NodeKind::Fresnel, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let fresnel_out = codegen::output_pin(&fresnel);
        let metallic_in = output.pins.iter().find(|p| p.name == "Metallic").unwrap().id;

        let fid = fresnel.id;
        let oid = output.id;
        graph.add_node(fresnel);
        graph.add_node(output);
        graph.connect(fid, fresnel_out, oid, metallic_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("pow("));
        assert!(shader.contains("in.view_dir"));
    }

    #[test]
    fn compile_vec3_node() {
        let mut graph = MaterialGraph::new();
        let vec3 = MaterialNode::new(NodeKind::ConstantVec3, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let vec3_out = codegen::output_pin(&vec3);
        let normal_in = output.pins.iter().find(|p| p.name == "Normal").unwrap().id;

        let vid = vec3.id;
        let oid = output.id;
        graph.add_node(vec3);
        graph.add_node(output);
        graph.connect(vid, vec3_out, oid, normal_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("vec3<f32>"));
        assert!(shader.contains("result.normal"));
    }

    #[test]
    fn compile_noise_perlin() {
        let mut graph = MaterialGraph::new();
        let noise = MaterialNode::new(NodeKind::NoisePerlin, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let noise_out = codegen::output_pin(&noise);
        let roughness_in = output.pins.iter().find(|p| p.name == "Roughness").unwrap().id;

        let nid = noise.id;
        let oid = output.id;
        graph.add_node(noise);
        graph.add_node(output);
        graph.connect(nid, noise_out, oid, roughness_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("fract(sin("));
    }

    #[test]
    fn compile_noise_voronoi() {
        let mut graph = MaterialGraph::new();
        let noise = MaterialNode::new(NodeKind::NoiseVoronoi, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let value_pin = noise
            .pins
            .iter()
            .find(|p| p.name == "Value" && p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let roughness_in = output.pins.iter().find(|p| p.name == "Roughness").unwrap().id;

        let nid = noise.id;
        let oid = output.id;
        graph.add_node(noise);
        graph.add_node(output);
        graph.connect(nid, value_pin, oid, roughness_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("result.roughness"));
    }

    #[test]
    fn compile_normal_map() {
        let mut graph = MaterialGraph::new();
        let nmap = MaterialNode::new(NodeKind::NormalMap, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let nmap_out = codegen::output_pin(&nmap);
        let normal_in = output.pins.iter().find(|p| p.name == "Normal").unwrap().id;

        let nid = nmap.id;
        let oid = output.id;
        graph.add_node(nmap);
        graph.add_node(output);
        graph.connect(nid, nmap_out, oid, normal_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("t_normal"));
        assert!(shader.contains("result.normal"));
    }

    #[test]
    fn compile_texture_sample() {
        let mut graph = MaterialGraph::new();
        let tex = MaterialNode::new(NodeKind::TextureSample, Vec2::ZERO);
        let output = MaterialNode::new(NodeKind::PbrOutput, Vec2::new(300.0, 0.0));

        let color_pin = tex
            .pins
            .iter()
            .find(|p| p.name == "Color" && p.direction == PinDirection::Output)
            .unwrap()
            .id;
        let albedo_in = output.pins.iter().find(|p| p.name == "Albedo").unwrap().id;

        let tid = tex.id;
        let oid = output.id;
        graph.add_node(tex);
        graph.add_node(output);
        graph.connect(tid, color_pin, oid, albedo_in);

        let shader = compile_graph(&graph).unwrap();
        assert!(shader.contains("textureSample(t_diffuse"));
        assert!(shader.contains("result.albedo"));
    }

    #[test]
    fn compiled_shader_has_valid_wgsl_structure() {
        let mut graph = MaterialGraph::new();
        graph.add_node(MaterialNode::new(NodeKind::PbrOutput, Vec2::ZERO));
        let shader = compile_graph(&graph).unwrap();

        // Check for all required WGSL structures.
        assert!(shader.contains("struct VertexOutput"));
        assert!(shader.contains("struct PbrResult"));
        assert!(shader.contains("@fragment"));
        assert!(shader.contains("fn fs_main"));
        assert!(shader.contains("-> @location(0) vec4<f32>"));
        // Check proper closing brace.
        assert!(shader.contains("return vec4<f32>(final_color, result.albedo.a);"));
    }
}
