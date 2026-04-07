//! compile_graph() -- topological sort + full WGSL shader assembly.

use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::graph::{MaterialGraph, NodeId, NodeKind, PinDirection, PinId};

use super::codegen::{default_value_for_input, next_var, output_pin};

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Compile the material graph into a complete WGSL fragment shader.
///
/// Returns an error if the graph contains cycles, has no PBR Output node, or
/// is otherwise malformed.
pub fn compile_graph(graph: &MaterialGraph) -> Result<String> {
    let sorted = graph
        .topological_sort()
        .context("Material graph contains a cycle")?;

    // Find the PBR output node.
    let output_id = sorted
        .iter()
        .find(|id| {
            graph
                .nodes
                .get(id)
                .map_or(false, |n| n.kind == NodeKind::PbrOutput)
        })
        .copied()
        .context("Graph has no PBR Output node")?;

    // Assign a variable name to every output pin.
    let mut pin_vars: HashMap<(NodeId, PinId), String> = HashMap::new();
    let mut var_counter: u32 = 0;

    // Build connection lookup: (to_node, to_pin) -> (from_node, from_pin).
    let mut conn_map: HashMap<(NodeId, PinId), (NodeId, PinId)> = HashMap::new();
    for conn in &graph.connections {
        conn_map.insert(
            (conn.to_node, conn.to_pin),
            (conn.from_node, conn.from_pin),
        );
    }

    let mut body_lines: Vec<String> = Vec::new();

    for &nid in &sorted {
        let node = match graph.nodes.get(&nid) {
            Some(n) => n,
            None => continue,
        };

        // Resolve an input pin to its variable name (from upstream output or a default).
        let resolve_input = |pin_name: &str| -> String {
            let pin = node.pins.iter().find(|p| {
                p.name == pin_name && p.direction == PinDirection::Input
            });
            if let Some(pin) = pin {
                if let Some(&(from_node, from_pin)) = conn_map.get(&(nid, pin.id)) {
                    if let Some(var) = pin_vars.get(&(from_node, from_pin)) {
                        return var.clone();
                    }
                }
            }
            // Return a default literal based on what this node kind expects.
            default_value_for_input(node.kind, pin_name)
        };

        match node.kind {
            NodeKind::PbrOutput => {
                // The output node gathers values into the final struct.
                // We handle this after the loop.
            }
            NodeKind::ConstantFloat => {
                let var = next_var(&mut var_counter);
                let out_pin = node
                    .pins
                    .iter()
                    .find(|p| p.direction == PinDirection::Output)
                    .unwrap();
                body_lines.push(format!("    let {var} = 0.5;"));
                pin_vars.insert((nid, out_pin.id), var);
            }
            NodeKind::ConstantColor => {
                let var = next_var(&mut var_counter);
                let out_pin = node
                    .pins
                    .iter()
                    .find(|p| p.direction == PinDirection::Output)
                    .unwrap();
                body_lines.push(format!(
                    "    let {var} = vec4<f32>(0.5, 0.5, 0.5, 1.0);"
                ));
                pin_vars.insert((nid, out_pin.id), var);
            }
            NodeKind::ConstantVec3 => {
                let var = next_var(&mut var_counter);
                let out_pin = node
                    .pins
                    .iter()
                    .find(|p| p.direction == PinDirection::Output)
                    .unwrap();
                body_lines.push(format!(
                    "    let {var} = vec3<f32>(0.0, 0.0, 0.0);"
                ));
                pin_vars.insert((nid, out_pin.id), var);
            }
            NodeKind::MathAdd => {
                let a = resolve_input("A");
                let b = resolve_input("B");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!("    let {var} = {a} + {b};"));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::MathMultiply => {
                let a = resolve_input("A");
                let b = resolve_input("B");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!("    let {var} = {a} * {b};"));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::MathMix => {
                let a = resolve_input("A");
                let b = resolve_input("B");
                let factor = resolve_input("Factor");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!("    let {var} = mix({a}, {b}, vec4<f32>({factor}));"));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::MathLerp => {
                let a = resolve_input("A");
                let b = resolve_input("B");
                let t = resolve_input("T");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!("    let {var} = mix({a}, {b}, {t});"));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::Fresnel => {
                let ior = resolve_input("IOR");
                let normal = resolve_input("Normal");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!(
                    "    let {var} = pow(1.0 - max(dot({normal}, in.view_dir), 0.0), 5.0) * (1.0 - {ior}) + {ior};"
                ));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::TextureSample => {
                let var_color = next_var(&mut var_counter);
                // Find output pins by name.
                let color_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "Color" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                let r_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "R" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                let g_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "G" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                let b_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "B" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                let a_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "A" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;

                let uv = resolve_input("UV");
                body_lines.push(format!(
                    "    let {var_color} = textureSample(t_diffuse, s_diffuse, {uv});"
                ));
                let vr = next_var(&mut var_counter);
                let vg = next_var(&mut var_counter);
                let vb = next_var(&mut var_counter);
                let va = next_var(&mut var_counter);
                body_lines.push(format!("    let {vr} = {var_color}.r;"));
                body_lines.push(format!("    let {vg} = {var_color}.g;"));
                body_lines.push(format!("    let {vb} = {var_color}.b;"));
                body_lines.push(format!("    let {va} = {var_color}.a;"));

                pin_vars.insert((nid, color_pin), var_color);
                pin_vars.insert((nid, r_pin), vr);
                pin_vars.insert((nid, g_pin), vg);
                pin_vars.insert((nid, b_pin), vb);
                pin_vars.insert((nid, a_pin), va);
            }
            NodeKind::NormalMap => {
                let strength = resolve_input("Strength");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!(
                    "    let {var} = normalize(in.world_normal + (textureSample(t_normal, s_normal, in.uv).rgb * 2.0 - 1.0) * {strength});"
                ));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::NoisePerlin => {
                let uv = resolve_input("UV");
                let scale = resolve_input("Scale");
                let var = next_var(&mut var_counter);
                let out_pin = output_pin(node);
                body_lines.push(format!(
                    "    let {var} = fract(sin(dot({uv} * {scale}, vec2<f32>(12.9898, 78.233))) * 43758.5453);"
                ));
                pin_vars.insert((nid, out_pin), var);
            }
            NodeKind::NoiseVoronoi => {
                let uv = resolve_input("UV");
                let scale = resolve_input("Scale");
                let var_val = next_var(&mut var_counter);
                let var_dist = next_var(&mut var_counter);
                let value_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "Value" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                let dist_pin = node
                    .pins
                    .iter()
                    .find(|p| p.name == "Distance" && p.direction == PinDirection::Output)
                    .unwrap()
                    .id;
                body_lines.push(format!(
                    "    let {var_val} = fract(sin(dot({uv} * {scale}, vec2<f32>(127.1, 311.7))) * 43758.5453);"
                ));
                body_lines.push(format!("    let {var_dist} = {var_val};"));
                pin_vars.insert((nid, value_pin), var_val);
                pin_vars.insert((nid, dist_pin), var_dist);
            }
        }
    }

    // Build the PBR Output assignment.
    let output_node = graph
        .nodes
        .get(&output_id)
        .context("PBR Output node missing")?;

    let resolve_output_input = |pin_name: &str, default: &str| -> String {
        let pin = output_node.pins.iter().find(|p| {
            p.name == pin_name && p.direction == PinDirection::Input
        });
        if let Some(pin) = pin {
            if let Some(&(from_node, from_pin)) = conn_map.get(&(output_id, pin.id)) {
                if let Some(var) = pin_vars.get(&(from_node, from_pin)) {
                    return var.clone();
                }
            }
        }
        default.to_string()
    };

    let albedo = resolve_output_input("Albedo", "vec4<f32>(0.5, 0.5, 0.5, 1.0)");
    let normal = resolve_output_input("Normal", "in.world_normal");
    let metallic = resolve_output_input("Metallic", "0.0");
    let roughness = resolve_output_input("Roughness", "0.5");
    let ao = resolve_output_input("AO", "1.0");
    let emissive = resolve_output_input("Emissive", "vec4<f32>(0.0, 0.0, 0.0, 0.0)");

    // Assemble the full shader.
    let mut shader = String::new();
    shader.push_str("// Auto-generated WGSL fragment shader\n\n");

    shader.push_str(
        "\
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_dir: vec3<f32>,
};

struct PbrResult {
    albedo: vec4<f32>,
    normal: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    emissive: vec4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var t_normal: texture_2d<f32>;
@group(0) @binding(3) var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
",
    );

    for line in &body_lines {
        shader.push_str(line);
        shader.push('\n');
    }

    shader.push_str(&format!(
        "\
    var result: PbrResult;
    result.albedo = {albedo};
    result.normal = {normal};
    result.metallic = {metallic};
    result.roughness = {roughness};
    result.ao = {ao};
    result.emissive = {emissive};

    // Simple PBR approximation output
    let diffuse = result.albedo.rgb * result.ao;
    let final_color = diffuse + result.emissive.rgb;
    return vec4<f32>(final_color, result.albedo.a);
}}
"
    ));

    Ok(shader)
}
