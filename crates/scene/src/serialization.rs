//! Scene serialization and deserialization to/from RON files.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::graph::SceneGraph;
use crate::node::SceneNode;

/// On-disk representation of a scene file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFile {
    /// Format version for forward-compatibility.
    pub version: u32,
    /// All nodes in the scene.
    pub nodes: Vec<SceneNode>,
}

impl SceneFile {
    /// Current file format version.
    pub const CURRENT_VERSION: u32 = 1;
}

/// Serializes a scene graph to a RON file at `path`.
pub fn save_scene(graph: &SceneGraph, path: &Path) -> Result<()> {
    let root = graph.root();
    let node_ids = graph.iter_depth_first(root);
    let nodes: Vec<SceneNode> = node_ids
        .into_iter()
        .filter_map(|id| graph.get_node(id).cloned())
        .collect();

    let scene_file = SceneFile {
        version: SceneFile::CURRENT_VERSION,
        nodes,
    };

    let pretty = ron::ser::PrettyConfig::default();
    let data = ron::ser::to_string_pretty(&scene_file, pretty)
        .context("Failed to serialize scene to RON")?;

    std::fs::write(path, data).context("Failed to write scene file")?;

    Ok(())
}

/// Deserializes a scene graph from a RON file at `path`.
pub fn load_scene(path: &Path) -> Result<SceneGraph> {
    let data = std::fs::read_to_string(path).context("Failed to read scene file")?;

    let scene_file: SceneFile =
        ron::from_str(&data).context("Failed to parse scene file as RON")?;

    let mut graph = SceneGraph::new();
    let root = graph.root();

    // Copy properties from the saved root onto the new graph root so that
    // root name, transform, visibility, etc. survive the roundtrip.
    let mut nodes_iter = scene_file.nodes.into_iter();
    if let Some(saved_root) = nodes_iter.next()
        && let Some(root_node) = graph.get_node_mut(root)
    {
        root_node.name = saved_root.name;
        root_node.transform = saved_root.transform;
        root_node.visible = saved_root.visible;
        root_node.locked = saved_root.locked;
        root_node.layer = saved_root.layer;
        root_node.node_type = saved_root.node_type;
    }

    for node in nodes_iter {
        let parent = node.parent.unwrap_or(root);
        // If the parent is not in the graph yet, place under root.
        let actual_parent = if graph.get_node(parent).is_some() {
            parent
        } else {
            root
        };
        graph.add_node(node, actual_parent);
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeType, SceneNode};
    use forge_core::math::Color;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("forge_scene_test");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn roundtrip_save_load() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        graph.add_node(SceneNode::with_type("Cube", NodeType::Empty), root);
        graph.add_node(
            SceneNode::with_type(
                "Light",
                NodeType::PointLight {
                    position: glam::Vec3::new(0.0, 5.0, 0.0),
                    color: Color::WHITE,
                    intensity: 10.0,
                    radius: 50.0,
                },
            ),
            root,
        );

        let path = temp_path("test_roundtrip.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        assert_eq!(loaded.node_count(), 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_preserves_root_name() {
        let graph = SceneGraph::new();
        // The root node is named "Root" by default.
        let root = graph.root();
        assert_eq!(graph.get_node(root).unwrap().name, "Root");

        let path = temp_path("test_root_name.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();
        assert_eq!(loaded.get_node(loaded.root()).unwrap().name, "Root");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_preserves_root_locked_flag() {
        let graph = SceneGraph::new();
        assert!(graph.get_node(graph.root()).unwrap().locked);

        let path = temp_path("test_root_locked.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();
        assert!(loaded.get_node(loaded.root()).unwrap().locked);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_preserves_node_names() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("Alpha"), root);
        let _b = graph.add_node(SceneNode::new("Beta"), a);

        let path = temp_path("test_names.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        // Check that both node names survived the roundtrip.
        let all_ids = loaded.iter_depth_first(loaded.root());
        let names: Vec<&str> = all_ids
            .iter()
            .filter_map(|id| loaded.get_node(*id))
            .map(|n| n.name.as_str())
            .collect();
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_preserves_hierarchy() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let _b = graph.add_node(SceneNode::new("B"), a);
        let _c = graph.add_node(SceneNode::new("C"), a);

        let path = temp_path("test_hierarchy.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        // Root has 1 child (A), A has 2 children (B, C).
        let loaded_root = loaded.root();
        assert_eq!(loaded.children(loaded_root).len(), 1);
        let loaded_a = loaded.children(loaded_root)[0];
        assert_eq!(loaded.children(loaded_a).len(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_preserves_transform() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let mut node = SceneNode::new("Moved");
        node.transform.position = glam::Vec3::new(1.0, 2.0, 3.0);
        node.transform.scale = glam::Vec3::new(2.0, 2.0, 2.0);
        let _id = graph.add_node(node, root);

        let path = temp_path("test_transform.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        let loaded_ids = loaded.iter_depth_first(loaded.root());
        let moved = loaded_ids
            .iter()
            .filter_map(|id| loaded.get_node(*id))
            .find(|n| n.name == "Moved")
            .unwrap();
        assert!((moved.transform.position.x - 1.0).abs() < 1e-5);
        assert!((moved.transform.position.y - 2.0).abs() < 1e-5);
        assert!((moved.transform.position.z - 3.0).abs() < 1e-5);
        assert!((moved.transform.scale.x - 2.0).abs() < 1e-5);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn roundtrip_empty_scene() {
        let graph = SceneGraph::new();
        let path = temp_path("test_empty.ron");
        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();
        assert_eq!(loaded.node_count(), 1); // Only root.
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_nonexistent_file_errors() {
        let path = temp_path("does_not_exist_12345.ron");
        assert!(load_scene(&path).is_err());
    }

    #[test]
    fn scene_file_version() {
        assert_eq!(SceneFile::CURRENT_VERSION, 1);
    }
}
