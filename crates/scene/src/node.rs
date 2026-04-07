//! Scene node definitions including node types (meshes, lights, cameras, groups).

use forge_core::id::{AssetId, MaterialId, NodeId};
use forge_core::math::{Color, Transform, Vec3};
use serde::{Deserialize, Serialize};

/// The type-specific payload for a scene node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// An empty node used as a transform placeholder or group parent.
    Empty,

    /// A renderable mesh.
    Mesh {
        asset_id: AssetId,
        material_ids: Vec<MaterialId>,
    },

    /// A directional light (like the sun).
    DirectionalLight {
        direction: Vec3,
        color: Color,
        intensity: f32,
    },

    /// A point light that emits in all directions.
    PointLight {
        position: Vec3,
        color: Color,
        intensity: f32,
        radius: f32,
    },

    /// A spot light with a cone of influence.
    SpotLight {
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
        range: f32,
    },

    /// A camera node.
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },

    /// A grouping node for organizational purposes.
    Group,
}

/// A single node in the scene graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Human-readable name.
    pub name: String,
    /// Parent node, if any.
    pub parent: Option<NodeId>,
    /// Child node identifiers (order is significant).
    pub children: Vec<NodeId>,
    /// Local transform relative to the parent.
    pub transform: Transform,
    /// The type-specific data for this node.
    pub node_type: NodeType,
    /// Whether this node (and its children) are visible.
    pub visible: bool,
    /// Whether this node is locked against editing.
    pub locked: bool,
    /// The layer this node belongs to.
    pub layer: String,
}

impl SceneNode {
    /// Creates a new empty node with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            parent: None,
            children: Vec::new(),
            transform: Transform::IDENTITY,
            node_type: NodeType::Empty,
            visible: true,
            locked: false,
            layer: "Default".to_string(),
        }
    }

    /// Creates a new node with a specific type.
    pub fn with_type(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            node_type,
            ..Self::new(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_node_defaults() {
        let node = SceneNode::new("TestNode");
        assert_eq!(node.name, "TestNode");
        assert!(node.visible);
        assert!(!node.locked);
        assert!(node.children.is_empty());
        assert!(node.parent.is_none());
        assert_eq!(node.layer, "Default");
    }

    #[test]
    fn with_type_sets_type() {
        let node = SceneNode::with_type(
            "MyCamera",
            NodeType::Camera {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
        );
        assert_eq!(node.name, "MyCamera");
        assert!(matches!(node.node_type, NodeType::Camera { .. }));
    }
}
