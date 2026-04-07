//! Core type definitions: NodeId, PinId, PinType, PinDirection, PinValue, Pin.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────
// Identifiers (local to the material graph, not the scene NodeId)
// ─────────────────────────────────────────────────────────────────────

/// Unique identifier for a node within a material graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Generate a new random node identifier.
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// Unique identifier for a pin on a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinId(pub Uuid);

impl PinId {
    /// Generate a new random pin identifier.
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PinId {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Pin types
// ─────────────────────────────────────────────────────────────────────

/// The data type carried by a pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    /// Scalar `f32`.
    Float,
    /// 2-component vector.
    Vec2,
    /// 3-component vector.
    Vec3,
    /// 4-component vector.
    Vec4,
    /// RGBA color (logically equivalent to Vec4 but semantically distinct).
    Color,
    /// A 2D texture handle.
    Texture,
    /// A shader snippet / sub-graph output.
    Shader,
}

/// Whether a pin receives or provides data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinDirection {
    /// The pin receives data from an upstream node.
    Input,
    /// The pin provides data to downstream nodes.
    Output,
}

/// A concrete value stored in a pin for use as a default or override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinValue {
    /// Scalar float.
    Float(f32),
    /// 2-component vector.
    Vec2(glam::Vec2),
    /// 3-component vector.
    Vec3(glam::Vec3),
    /// 4-component vector.
    Vec4(glam::Vec4),
    /// RGBA color as `[r, g, b, a]`.
    Color([f32; 4]),
    /// Texture asset path or identifier.
    Texture(String),
    /// Inline shader snippet.
    Shader(String),
}

/// A single input or output port on a material node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    /// Unique identifier for this pin.
    pub id: PinId,
    /// Human-readable label (e.g. "Albedo", "UV").
    pub name: String,
    /// The data type this pin carries.
    pub pin_type: PinType,
    /// Whether this pin receives or provides data.
    pub direction: PinDirection,
    /// Optional default value used when the pin is unconnected.
    pub default_value: Option<PinValue>,
}
