//! NodeKind and MaterialNode definitions with default pins per kind.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::types::{NodeId, Pin, PinDirection, PinId, PinType};

// ─────────────────────────────────────────────────────────────────────
// Node kinds
// ─────────────────────────────────────────────────────────────────────

/// The functional type of a material graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    PbrOutput,
    TextureSample,
    ConstantColor,
    ConstantFloat,
    ConstantVec3,
    MathAdd,
    MathMultiply,
    MathMix,
    MathLerp,
    Fresnel,
    NormalMap,
    NoisePerlin,
    NoiseVoronoi,
}

impl NodeKind {
    /// All available node kinds (for menus).
    pub const ALL: &'static [NodeKind] = &[
        NodeKind::PbrOutput,
        NodeKind::TextureSample,
        NodeKind::ConstantColor,
        NodeKind::ConstantFloat,
        NodeKind::ConstantVec3,
        NodeKind::MathAdd,
        NodeKind::MathMultiply,
        NodeKind::MathMix,
        NodeKind::MathLerp,
        NodeKind::Fresnel,
        NodeKind::NormalMap,
        NodeKind::NoisePerlin,
        NodeKind::NoiseVoronoi,
    ];

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::PbrOutput => "PBR Output",
            NodeKind::TextureSample => "Texture Sample",
            NodeKind::ConstantColor => "Constant Color",
            NodeKind::ConstantFloat => "Constant Float",
            NodeKind::ConstantVec3 => "Constant Vec3",
            NodeKind::MathAdd => "Math Add",
            NodeKind::MathMultiply => "Math Multiply",
            NodeKind::MathMix => "Math Mix",
            NodeKind::MathLerp => "Math Lerp",
            NodeKind::Fresnel => "Fresnel",
            NodeKind::NormalMap => "Normal Map",
            NodeKind::NoisePerlin => "Perlin Noise",
            NodeKind::NoiseVoronoi => "Voronoi Noise",
        }
    }

    /// Build the default set of pins for this node kind.
    pub fn default_pins(self) -> Vec<Pin> {
        match self {
            NodeKind::PbrOutput => vec![
                pin_in("Albedo", PinType::Color),
                pin_in("Normal", PinType::Vec3),
                pin_in("Metallic", PinType::Float),
                pin_in("Roughness", PinType::Float),
                pin_in("AO", PinType::Float),
                pin_in("Emissive", PinType::Color),
            ],
            NodeKind::TextureSample => vec![
                pin_in("UV", PinType::Vec2),
                pin_in("Texture", PinType::Texture),
                pin_out("Color", PinType::Color),
                pin_out("R", PinType::Float),
                pin_out("G", PinType::Float),
                pin_out("B", PinType::Float),
                pin_out("A", PinType::Float),
            ],
            NodeKind::ConstantColor => vec![pin_out("Color", PinType::Color)],
            NodeKind::ConstantFloat => vec![pin_out("Value", PinType::Float)],
            NodeKind::ConstantVec3 => vec![pin_out("Vec3", PinType::Vec3)],
            NodeKind::MathAdd => vec![
                pin_in("A", PinType::Float),
                pin_in("B", PinType::Float),
                pin_out("Result", PinType::Float),
            ],
            NodeKind::MathMultiply => vec![
                pin_in("A", PinType::Float),
                pin_in("B", PinType::Float),
                pin_out("Result", PinType::Float),
            ],
            NodeKind::MathMix => vec![
                pin_in("A", PinType::Color),
                pin_in("B", PinType::Color),
                pin_in("Factor", PinType::Float),
                pin_out("Result", PinType::Color),
            ],
            NodeKind::MathLerp => vec![
                pin_in("A", PinType::Float),
                pin_in("B", PinType::Float),
                pin_in("T", PinType::Float),
                pin_out("Result", PinType::Float),
            ],
            NodeKind::Fresnel => vec![
                pin_in("IOR", PinType::Float),
                pin_in("Normal", PinType::Vec3),
                pin_out("Factor", PinType::Float),
            ],
            NodeKind::NormalMap => vec![
                pin_in("Texture", PinType::Texture),
                pin_in("Strength", PinType::Float),
                pin_out("Normal", PinType::Vec3),
            ],
            NodeKind::NoisePerlin => vec![
                pin_in("UV", PinType::Vec2),
                pin_in("Scale", PinType::Float),
                pin_out("Value", PinType::Float),
            ],
            NodeKind::NoiseVoronoi => vec![
                pin_in("UV", PinType::Vec2),
                pin_in("Scale", PinType::Float),
                pin_out("Value", PinType::Float),
                pin_out("Distance", PinType::Float),
            ],
        }
    }
}

/// Shorthand to create an input pin with the given name and type.
fn pin_in(name: &str, pin_type: PinType) -> Pin {
    Pin {
        id: PinId::new(),
        name: name.into(),
        pin_type,
        direction: PinDirection::Input,
        default_value: None,
    }
}

/// Shorthand to create an output pin with the given name and type.
fn pin_out(name: &str, pin_type: PinType) -> Pin {
    Pin {
        id: PinId::new(),
        name: name.into(),
        pin_type,
        direction: PinDirection::Output,
        default_value: None,
    }
}

// ─────────────────────────────────────────────────────────────────────
// MaterialNode
// ─────────────────────────────────────────────────────────────────────

/// A single node in the material graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Functional type of the node (e.g. constant, math op, PBR output).
    pub kind: NodeKind,
    /// Position on the editor canvas (for layout).
    pub position: Vec2,
    /// The input and output pins belonging to this node.
    pub pins: Vec<Pin>,
}

impl MaterialNode {
    /// Create a new node of the given kind at the given position.
    pub fn new(kind: NodeKind, position: Vec2) -> Self {
        Self {
            id: NodeId::new(),
            kind,
            position,
            pins: kind.default_pins(),
        }
    }

    /// Find a pin by its id.
    pub fn find_pin(&self, pin_id: &PinId) -> Option<&Pin> {
        self.pins.iter().find(|p| p.id == *pin_id)
    }
}
