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

    /// Adds an entity to the selection without clearing existing selections.
    ///
    /// If the entity is already selected, this is a no-op.
    pub fn add(&mut self, entity: EntityId) {
        if !self.is_selected(entity) {
            self.entities.push(entity);
        }
    }

    /// Replaces the entire selection with the given set of entities.
    pub fn select_all(&mut self, entities: impl IntoIterator<Item = EntityId>) {
        self.entities.clear();
        self.entities.extend(entities);
        // Deduplicate
        self.entities.sort_by_key(|e| format!("{e:?}"));
        self.entities.dedup();
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
    #[inline]
    pub fn is_selected(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    /// Returns `true` if the selection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns the primary (first) selected entity, if any.
    #[inline]
    pub fn primary(&self) -> Option<EntityId> {
        self.entities.first().copied()
    }

    /// Returns the number of selected entities.
    #[inline]
    pub fn count(&self) -> usize {
        self.entities.len()
    }

    /// Returns a slice of all selected entities.
    #[inline]
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

    fn make_entities(n: usize) -> Vec<EntityId> {
        let mut world = World::new();
        (0..n)
            .map(|i| world.spawn_entity((i as u32,)))
            .collect()
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
    fn select_replaces_previous() {
        let entities = make_entities(2);
        let mut sel = Selection::new();
        sel.select(entities[0]);
        sel.select(entities[1]);
        assert_eq!(sel.count(), 1);
        assert!(!sel.is_selected(entities[0]));
        assert!(sel.is_selected(entities[1]));
    }

    #[test]
    fn add_does_not_replace() {
        let entities = make_entities(3);
        let mut sel = Selection::new();
        sel.select(entities[0]);
        sel.add(entities[1]);
        sel.add(entities[2]);
        assert_eq!(sel.count(), 3);
        assert!(sel.is_selected(entities[0]));
        assert!(sel.is_selected(entities[1]));
        assert!(sel.is_selected(entities[2]));
    }

    #[test]
    fn add_deduplicates() {
        let e = make_entity();
        let mut sel = Selection::new();
        sel.add(e);
        sel.add(e);
        assert_eq!(sel.count(), 1);
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
        assert!(sel.is_empty());
    }

    #[test]
    fn primary_returns_first() {
        let mut sel = Selection::new();
        assert!(sel.primary().is_none());
        let e = make_entity();
        sel.select(e);
        assert_eq!(sel.primary(), Some(e));
    }

    #[test]
    fn is_empty_works() {
        let mut sel = Selection::new();
        assert!(sel.is_empty());
        let e = make_entity();
        sel.select(e);
        assert!(!sel.is_empty());
    }

    #[test]
    fn deselect_specific_entity() {
        let entities = make_entities(3);
        let mut sel = Selection::new();
        sel.select(entities[0]);
        sel.add(entities[1]);
        sel.add(entities[2]);
        sel.deselect(entities[1]);
        assert_eq!(sel.count(), 2);
        assert!(!sel.is_selected(entities[1]));
    }

    #[test]
    fn deselect_nonexistent_is_noop() {
        let entities = make_entities(2);
        let mut sel = Selection::new();
        sel.select(entities[0]);
        sel.deselect(entities[1]); // not in selection
        assert_eq!(sel.count(), 1);
    }

    #[test]
    fn entities_returns_slice() {
        let entities = make_entities(2);
        let mut sel = Selection::new();
        sel.select(entities[0]);
        sel.add(entities[1]);
        assert_eq!(sel.entities().len(), 2);
    }

    #[test]
    fn default_is_empty() {
        let sel = Selection::default();
        assert!(sel.is_empty());
        assert_eq!(sel.count(), 0);
    }
}
