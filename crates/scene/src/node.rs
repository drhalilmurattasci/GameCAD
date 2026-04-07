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

impl NodeType {
    /// Returns a default directional light.
    #[inline]
    pub fn default_directional_light() -> Self {
        Self::DirectionalLight {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
        }
    }

    /// Returns a default point light.
    #[inline]
    pub fn default_point_light() -> Self {
        Self::PointLight {
            position: Vec3::ZERO,
            color: Color::WHITE,
            intensity: 1.0,
            radius: 10.0,
        }
    }

    /// Returns a default spot light.
    #[inline]
    pub fn default_spot_light() -> Self {
        Self::SpotLight {
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
            inner_angle: 30.0,
            outer_angle: 45.0,
            range: 10.0,
        }
    }

    /// Returns a default camera.
    #[inline]
    pub fn default_camera() -> Self {
        Self::Camera {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Returns a default mesh (nil asset, no materials).
    #[inline]
    pub fn default_mesh() -> Self {
        Self::Mesh {
            asset_id: AssetId::NIL,
            material_ids: Vec::new(),
        }
    }

    /// Returns a human-readable label for the node type.
    #[inline]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Mesh { .. } => "Mesh",
            Self::DirectionalLight { .. } => "Directional Light",
            Self::PointLight { .. } => "Point Light",
            Self::SpotLight { .. } => "Spot Light",
            Self::Camera { .. } => "Camera",
            Self::Group => "Group",
        }
    }
}

impl Default for NodeType {
    #[inline]
    fn default() -> Self {
        Self::Empty
    }
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
    #[inline]
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
    #[inline]
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
        assert!(matches!(node.node_type, NodeType::Empty));
        assert_eq!(node.transform, Transform::IDENTITY);
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

    #[test]
    fn node_type_default_is_empty() {
        assert!(matches!(NodeType::default(), NodeType::Empty));
    }

    #[test]
    fn default_directional_light() {
        let nt = NodeType::default_directional_light();
        match nt {
            NodeType::DirectionalLight {
                direction,
                color,
                intensity,
            } => {
                assert!((direction.y - (-1.0)).abs() < 1e-5);
                assert_eq!(color, Color::WHITE);
                assert!((intensity - 1.0).abs() < 1e-5);
            }
            _ => panic!("Expected DirectionalLight"),
        }
    }

    #[test]
    fn default_point_light() {
        let nt = NodeType::default_point_light();
        match nt {
            NodeType::PointLight {
                position,
                color,
                intensity,
                radius,
            } => {
                assert_eq!(position, Vec3::ZERO);
                assert_eq!(color, Color::WHITE);
                assert!((intensity - 1.0).abs() < 1e-5);
                assert!((radius - 10.0).abs() < 1e-5);
            }
            _ => panic!("Expected PointLight"),
        }
    }

    #[test]
    fn default_spot_light() {
        let nt = NodeType::default_spot_light();
        match nt {
            NodeType::SpotLight {
                inner_angle,
                outer_angle,
                range,
                ..
            } => {
                assert!((inner_angle - 30.0).abs() < 1e-5);
                assert!((outer_angle - 45.0).abs() < 1e-5);
                assert!((range - 10.0).abs() < 1e-5);
            }
            _ => panic!("Expected SpotLight"),
        }
    }

    #[test]
    fn default_camera() {
        let nt = NodeType::default_camera();
        match nt {
            NodeType::Camera { fov, near, far } => {
                assert!((fov - 60.0).abs() < 1e-5);
                assert!((near - 0.1).abs() < 1e-5);
                assert!((far - 1000.0).abs() < 1e-5);
            }
            _ => panic!("Expected Camera"),
        }
    }

    #[test]
    fn default_mesh() {
        let nt = NodeType::default_mesh();
        match nt {
            NodeType::Mesh {
                asset_id,
                material_ids,
            } => {
                assert_eq!(asset_id, AssetId::NIL);
                assert!(material_ids.is_empty());
            }
            _ => panic!("Expected Mesh"),
        }
    }

    #[test]
    fn node_type_labels() {
        assert_eq!(NodeType::Empty.label(), "Empty");
        assert_eq!(NodeType::Group.label(), "Group");
        assert_eq!(NodeType::default_camera().label(), "Camera");
        assert_eq!(NodeType::default_mesh().label(), "Mesh");
        assert_eq!(
            NodeType::default_directional_light().label(),
            "Directional Light"
        );
        assert_eq!(NodeType::default_point_light().label(), "Point Light");
        assert_eq!(NodeType::default_spot_light().label(), "Spot Light");
    }

    #[test]
    fn with_type_preserves_defaults() {
        let node = SceneNode::with_type("G", NodeType::Group);
        assert!(matches!(node.node_type, NodeType::Group));
        assert!(node.visible);
        assert!(!node.locked);
        assert_eq!(node.layer, "Default");
    }

    #[test]
    fn each_node_gets_unique_id() {
        let a = SceneNode::new("A");
        let b = SceneNode::new("B");
        assert_ne!(a.id, b.id);
    }
}
