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

    // Skip the first node in the file (it was the old root); reparent all
    // top-level nodes under the new graph root.
    for node in scene_file.nodes.into_iter().skip(1) {
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
                    color: forge_core::math::Color::WHITE,
                    intensity: 10.0,
                    radius: 50.0,
                },
            ),
            root,
        );

        let dir = std::env::temp_dir().join("forge_scene_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_scene.ron");

        save_scene(&graph, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        // Original had root + 2 nodes = 3.
        // Loaded should have new root + 2 nodes = 3.
        assert_eq!(loaded.node_count(), 3);

        // Clean up.
        let _ = std::fs::remove_file(&path);
    }
}
