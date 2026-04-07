//! Entity selection tracking for the viewport.

use forge_core::ecs::EntityId;

/// Tracks the set of currently selected entities.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    entities: Vec<EntityId>,
}

impl Selection {
    /// Creates a new empty selection.
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Replaces the selection with a single entity.
    pub fn select(&mut self, entity: EntityId) {
        self.entities.clear();
        self.entities.push(entity);
    }

    /// Removes an entity from the selection.
    pub fn deselect(&mut self, entity: EntityId) {
        self.entities.retain(|e| *e != entity);
    }

    /// Toggles an entity in the selection: adds it if absent, removes it if present.
    pub fn toggle(&mut self, entity: EntityId) {
        if self.is_selected(entity) {
            self.deselect(entity);
        } else {
            self.entities.push(entity);
        }
    }

    /// Clears the entire selection.
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns `true` if the entity is currently selected.
    pub fn is_selected(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    /// Returns the primary (first) selected entity, if any.
    pub fn primary(&self) -> Option<EntityId> {
        self.entities.first().copied()
    }

    /// Returns the number of selected entities.
    pub fn count(&self) -> usize {
        self.entities.len()
    }

    /// Returns a slice of all selected entities.
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::ecs::World;

    fn make_entity() -> EntityId {
        let mut world = World::new();
        world.spawn_entity((42u32,))
    }

    #[test]
    fn select_and_count() {
        let e = make_entity();
        let mut sel = Selection::new();
        sel.select(e);
        assert_eq!(sel.count(), 1);
        assert!(sel.is_selected(e));
    }

    #[test]
    fn toggle_adds_and_removes() {
        let e = make_entity();
        let mut sel = Selection::new();
        sel.toggle(e);
        assert_eq!(sel.count(), 1);
        sel.toggle(e);
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn clear_empties() {
        let e = make_entity();
        let mut sel = Selection::new();
        sel.select(e);
        sel.clear();
        assert_eq!(sel.count(), 0);
    }

    #[test]
    fn primary_returns_first() {
        let mut sel = Selection::new();
        assert!(sel.primary().is_none());
        let e = make_entity();
        sel.select(e);
        assert_eq!(sel.primary(), Some(e));
    }
}
