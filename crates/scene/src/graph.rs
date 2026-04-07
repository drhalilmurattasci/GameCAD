//! Scene graph: a flat-map tree of [`SceneNode`]s with parent-child relationships.

use forge_core::id::NodeId;
use forge_core::math::Transform;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::node::SceneNode;

/// The scene graph: a tree of nodes stored in a flat map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGraph {
    nodes: IndexMap<NodeId, SceneNode>,
    root: NodeId,
}

impl SceneGraph {
    /// Creates a new scene graph with a single root node.
    #[inline]
    pub fn new() -> Self {
        let mut root_node = SceneNode::new("Root");
        root_node.locked = true;
        let root_id = root_node.id;
        let mut nodes = IndexMap::new();
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
        }
    }

    /// Returns the root node ID.
    #[inline]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Adds a node to the graph as a child of `parent`.
    ///
    /// Returns the node's ID. If `parent` does not exist, the node is added
    /// under the root.
    pub fn add_node(&mut self, mut node: SceneNode, parent: NodeId) -> NodeId {
        let actual_parent = if self.nodes.contains_key(&parent) {
            parent
        } else {
            warn!("Parent {:?} not found, adding under root", parent);
            self.root
        };

        let id = node.id;
        node.parent = Some(actual_parent);
        // Clear stale children from deserialization; they will be re-established
        // as each child is added individually.
        node.children.clear();
        self.nodes.insert(id, node);

        if let Some(parent_node) = self.nodes.get_mut(&actual_parent) {
            parent_node.children.push(id);
        }

        id
    }

    /// Removes a node and all its descendants from the graph.
    ///
    /// Returns `true` if the node was found and removed. The root cannot be removed.
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        if id == self.root {
            warn!("Cannot remove the root node");
            return false;
        }

        // Collect IDs to remove (depth-first).
        let ids_to_remove = self.collect_subtree(id);

        // Unlink from parent.
        if let Some(node) = self.nodes.get(&id) {
            let parent_id = node.parent;
            if let Some(parent_id) = parent_id
                && let Some(parent) = self.nodes.get_mut(&parent_id)
            {
                parent.children.retain(|child| *child != id);
            }
        }

        let had_node = self.nodes.contains_key(&id);
        for remove_id in ids_to_remove {
            self.nodes.swap_remove(&remove_id);
        }

        had_node
    }

    /// Moves a node to be a child of a new parent.
    ///
    /// Returns `true` on success. Reparenting to self or to a descendant is
    /// rejected to prevent cycles.
    pub fn reparent(&mut self, node_id: NodeId, new_parent: NodeId) -> bool {
        if node_id == self.root {
            warn!("Cannot reparent the root node");
            return false;
        }
        if node_id == new_parent {
            warn!("Cannot reparent a node to itself");
            return false;
        }
        if !self.nodes.contains_key(&node_id) || !self.nodes.contains_key(&new_parent) {
            return false;
        }

        // Prevent reparenting to a descendant.
        if self.is_ancestor(node_id, new_parent) {
            warn!("Cannot reparent a node under its own descendant");
            return false;
        }

        // Remove from old parent.
        if let Some(node) = self.nodes.get(&node_id) {
            let old_parent = node.parent;
            if let Some(old_parent_id) = old_parent
                && let Some(parent) = self.nodes.get_mut(&old_parent_id)
            {
                parent.children.retain(|c| *c != node_id);
            }
        }

        // Add to new parent.
        if let Some(parent) = self.nodes.get_mut(&new_parent) {
            parent.children.push(node_id);
        }
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.parent = Some(new_parent);
        }

        true
    }

    /// Returns an immutable reference to a node.
    #[inline]
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    /// Returns a mutable reference to a node.
    #[inline]
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&id)
    }

    /// Returns the direct children of a node.
    #[inline]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.nodes
            .get(&id)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Computes the world transform for a node by composing all ancestor transforms.
    pub fn world_transform(&self, id: NodeId) -> Transform {
        let mut chain = Vec::new();
        let mut current = Some(id);
        while let Some(cid) = current {
            if let Some(node) = self.nodes.get(&cid) {
                chain.push(node.transform);
                current = node.parent;
            } else {
                break;
            }
        }

        // Compose from root to leaf.
        let mut result = Transform::IDENTITY;
        for t in chain.into_iter().rev() {
            let mat = result.matrix() * t.matrix();
            let (scale, rotation, position) = mat.to_scale_rotation_translation();
            result = Transform {
                position,
                rotation,
                scale,
            };
        }
        result
    }

    /// Iterates over nodes in depth-first order starting from `start`.
    pub fn iter_depth_first(&self, start: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut stack = vec![start];

        while let Some(id) = stack.pop() {
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                // Push children in reverse so they come out in order.
                for child in node.children.iter().rev() {
                    stack.push(*child);
                }
            }
        }

        result
    }

    /// Returns the total number of nodes (including root).
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // ── Private helpers ─────────────────────────────────────────────

    /// Collects all node IDs in the subtree rooted at `id` (inclusive).
    fn collect_subtree(&self, id: NodeId) -> Vec<NodeId> {
        self.iter_depth_first(id)
    }

    /// Returns `true` if `ancestor` is a (transitive) ancestor of `descendant`.
    fn is_ancestor(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut current = Some(descendant);
        while let Some(cid) = current {
            if cid == ancestor {
                return true;
            }
            current = self.nodes.get(&cid).and_then(|n| n.parent);
        }
        false
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph_has_root() {
        let graph = SceneGraph::new();
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(graph.root()).is_some());
    }

    #[test]
    fn add_and_get_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let node = SceneNode::new("Cube");
        let id = graph.add_node(node, root);
        assert!(graph.get_node(id).is_some());
        assert_eq!(graph.get_node(id).unwrap().name, "Cube");
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn remove_node_and_children() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        let parent = SceneNode::new("Parent");
        let parent_id = graph.add_node(parent, root);

        let child = SceneNode::new("Child");
        let child_id = graph.add_node(child, parent_id);

        assert_eq!(graph.node_count(), 3);
        assert!(graph.remove_node(parent_id));
        assert_eq!(graph.node_count(), 1); // only root remains
        assert!(graph.get_node(child_id).is_none());
    }

    #[test]
    fn cannot_remove_root() {
        let mut graph = SceneGraph::new();
        assert!(!graph.remove_node(graph.root()));
    }

    #[test]
    fn reparent_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), root);

        assert!(graph.reparent(b, a));
        assert_eq!(graph.children(a).len(), 1);
        assert_eq!(graph.children(a)[0], b);
        // Root no longer has B as direct child.
        assert!(!graph.children(root).contains(&b));
    }

    #[test]
    fn depth_first_iteration() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), a);
        let _c = graph.add_node(SceneNode::new("C"), root);

        let order = graph.iter_depth_first(root);
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], root);
        assert_eq!(order[1], a);
        assert_eq!(order[2], b);
    }

    // ── Edge case tests ────────────────────────────────────────────

    #[test]
    fn cannot_reparent_to_self() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        assert!(!graph.reparent(a, a));
        // Node should still be a child of root.
        assert!(graph.children(root).contains(&a));
    }

    #[test]
    fn cannot_reparent_to_descendant() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), a);
        let c = graph.add_node(SceneNode::new("C"), b);
        // Reparenting A under its grandchild C must fail.
        assert!(!graph.reparent(a, c));
    }

    #[test]
    fn cannot_reparent_root() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        assert!(!graph.reparent(root, a));
    }

    #[test]
    fn reparent_nonexistent_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let fake_id = NodeId::new();
        assert!(!graph.reparent(fake_id, root));
    }

    #[test]
    fn reparent_to_nonexistent_parent() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let fake_id = NodeId::new();
        assert!(!graph.reparent(a, fake_id));
    }

    #[test]
    fn add_node_with_invalid_parent_falls_back_to_root() {
        let mut graph = SceneGraph::new();
        let fake_parent = NodeId::new();
        let node = SceneNode::new("Orphan");
        let id = graph.add_node(node, fake_parent);
        // Should have been placed under root.
        assert!(graph.children(graph.root()).contains(&id));
        assert_eq!(graph.get_node(id).unwrap().parent, Some(graph.root()));
    }

    #[test]
    fn remove_nonexistent_node() {
        let mut graph = SceneGraph::new();
        let fake_id = NodeId::new();
        assert!(!graph.remove_node(fake_id));
    }

    #[test]
    fn remove_leaf_node() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        assert!(graph.remove_node(a));
        assert!(graph.children(root).is_empty());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn remove_deep_subtree() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), a);
        let _c = graph.add_node(SceneNode::new("C"), b);
        let _d = graph.add_node(SceneNode::new("D"), b);
        // Remove A, which should remove A, B, C, D.
        assert!(graph.remove_node(a));
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn children_of_nonexistent_returns_empty() {
        let graph = SceneGraph::new();
        let fake_id = NodeId::new();
        assert!(graph.children(fake_id).is_empty());
    }

    #[test]
    fn world_transform_single_node() {
        let graph = SceneGraph::new();
        let root = graph.root();
        let wt = graph.world_transform(root);
        assert_eq!(wt.position, glam::Vec3::ZERO);
        assert_eq!(wt.scale, glam::Vec3::ONE);
    }

    #[test]
    fn world_transform_propagates() {
        let mut graph = SceneGraph::new();
        let root = graph.root();

        let mut parent_node = SceneNode::new("Parent");
        parent_node.transform.position = glam::Vec3::new(10.0, 0.0, 0.0);
        let parent_id = graph.add_node(parent_node, root);

        let mut child_node = SceneNode::new("Child");
        child_node.transform.position = glam::Vec3::new(5.0, 0.0, 0.0);
        let child_id = graph.add_node(child_node, parent_id);

        let wt = graph.world_transform(child_id);
        // Child is at local (5,0,0) under parent at (10,0,0), so world = (15,0,0).
        assert!((wt.position.x - 15.0).abs() < 1e-5);
    }

    #[test]
    fn world_transform_nonexistent_returns_identity() {
        let graph = SceneGraph::new();
        let fake_id = NodeId::new();
        let wt = graph.world_transform(fake_id);
        assert_eq!(wt, forge_core::math::Transform::IDENTITY);
    }

    #[test]
    fn default_is_same_as_new() {
        let a = SceneGraph::new();
        let b = SceneGraph::default();
        assert_eq!(a.node_count(), b.node_count());
    }

    #[test]
    fn reparent_updates_parent_field() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), root);
        graph.reparent(b, a);
        assert_eq!(graph.get_node(b).unwrap().parent, Some(a));
    }

    #[test]
    fn iter_depth_first_single_root() {
        let graph = SceneGraph::new();
        let order = graph.iter_depth_first(graph.root());
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], graph.root());
    }

    #[test]
    fn add_multiple_children_preserves_order() {
        let mut graph = SceneGraph::new();
        let root = graph.root();
        let a = graph.add_node(SceneNode::new("A"), root);
        let b = graph.add_node(SceneNode::new("B"), root);
        let c = graph.add_node(SceneNode::new("C"), root);
        let children = graph.children(root);
        assert_eq!(children, &[a, b, c]);
    }
}
