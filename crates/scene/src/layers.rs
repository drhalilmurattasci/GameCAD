//! Layer management for organizing scene nodes into visibility/lock groups.

use forge_core::math::Color;
use serde::{Deserialize, Serialize};

/// A named layer that nodes can belong to for organizational control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Human-readable name of the layer.
    pub name: String,
    /// Display color for the layer in the UI.
    pub color: Color,
    /// Whether objects on this layer are rendered.
    pub visible: bool,
    /// Whether objects on this layer can be edited.
    pub locked: bool,
}

impl Layer {
    /// Creates a new visible, unlocked layer.
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            name: name.into(),
            color,
            visible: true,
            locked: false,
        }
    }
}

/// Manages a collection of layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerManager {
    layers: Vec<Layer>,
}

impl LayerManager {
    /// Creates a new manager with a single "Default" layer.
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::new("Default", Color::WHITE)],
        }
    }

    /// Adds a new layer. Returns `false` if a layer with that name already exists.
    pub fn add(&mut self, layer: Layer) -> bool {
        if self.layers.iter().any(|l| l.name == layer.name) {
            return false;
        }
        self.layers.push(layer);
        true
    }

    /// Removes a layer by name. The "Default" layer cannot be removed.
    ///
    /// Returns `true` if the layer was found and removed.
    pub fn remove(&mut self, name: &str) -> bool {
        if name == "Default" {
            return false;
        }
        let len_before = self.layers.len();
        self.layers.retain(|l| l.name != name);
        self.layers.len() < len_before
    }

    /// Toggles visibility for the layer with the given name.
    pub fn toggle_visibility(&mut self, name: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == name) {
            layer.visible = !layer.visible;
        }
    }

    /// Toggles the locked state for the layer with the given name.
    pub fn toggle_locked(&mut self, name: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == name) {
            layer.locked = !layer.locked;
        }
    }

    /// Returns a reference to the layer with the given name.
    pub fn get(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Returns a mutable reference to the layer with the given name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Returns all layers.
    pub fn all(&self) -> &[Layer] {
        &self.layers
    }

    /// Returns the number of layers.
    pub fn count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_one_layer() {
        let mgr = LayerManager::new();
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get("Default").is_some());
    }

    #[test]
    fn add_layer() {
        let mut mgr = LayerManager::new();
        assert!(mgr.add(Layer::new("Foreground", Color::new(1.0, 0.0, 0.0, 1.0))));
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut mgr = LayerManager::new();
        assert!(!mgr.add(Layer::new("Default", Color::WHITE)));
    }

    #[test]
    fn remove_layer() {
        let mut mgr = LayerManager::new();
        mgr.add(Layer::new("Temp", Color::BLACK));
        assert!(mgr.remove("Temp"));
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn cannot_remove_default() {
        let mut mgr = LayerManager::new();
        assert!(!mgr.remove("Default"));
    }

    #[test]
    fn toggle_visibility() {
        let mut mgr = LayerManager::new();
        assert!(mgr.get("Default").unwrap().visible);
        mgr.toggle_visibility("Default");
        assert!(!mgr.get("Default").unwrap().visible);
        mgr.toggle_visibility("Default");
        assert!(mgr.get("Default").unwrap().visible);
    }

    #[test]
    fn toggle_locked() {
        let mut mgr = LayerManager::new();
        assert!(!mgr.get("Default").unwrap().locked);
        mgr.toggle_locked("Default");
        assert!(mgr.get("Default").unwrap().locked);
    }
}
