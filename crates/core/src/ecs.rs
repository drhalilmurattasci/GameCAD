//! Thin wrapper around [`hecs`] providing a friendlier API for the editor.

use std::fmt;

use hecs;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// EntityId
// ─────────────────────────────────────────────────────────────────────

/// A lightweight identifier for an entity, wrapping [`hecs::Entity`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(hecs::Entity);

impl EntityId {
    /// Wraps an existing [`hecs::Entity`].
    #[inline]
    pub fn from_hecs(entity: hecs::Entity) -> Self {
        Self(entity)
    }

    /// Returns the inner [`hecs::Entity`].
    #[inline]
    pub fn to_hecs(self) -> hecs::Entity {
        self.0
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({})", self.0.id())
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.id())
    }
}

// hecs::Entity is not Serialize/Deserialize, so we go through `to_bits`/`from_bits`.
impl Serialize for EntityId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bits = u64::deserialize(deserializer)?;
        let entity = hecs::Entity::from_bits(bits)
            .ok_or_else(|| serde::de::Error::custom("invalid entity bits"))?;
        Ok(Self(entity))
    }
}

// ─────────────────────────────────────────────────────────────────────
// Component marker
// ─────────────────────────────────────────────────────────────────────

/// Marker trait for types that can be stored as ECS components.
pub trait Component: Send + Sync + 'static {}

// Blanket implementation: anything that is Send + Sync + 'static is a Component.
impl<T: Send + Sync + 'static> Component for T {}

// ─────────────────────────────────────────────────────────────────────
// World
// ─────────────────────────────────────────────────────────────────────

/// Editor world wrapping [`hecs::World`].
pub struct World {
    inner: hecs::World,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Creates a new, empty world.
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }

    /// Returns a reference to the underlying [`hecs::World`].
    #[inline]
    pub fn inner(&self) -> &hecs::World {
        &self.inner
    }

    /// Returns a mutable reference to the underlying [`hecs::World`].
    #[inline]
    pub fn inner_mut(&mut self) -> &mut hecs::World {
        &mut self.inner
    }

    /// Spawns a new entity with the given component bundle and returns its [`EntityId`].
    pub fn spawn_entity(&mut self, components: impl hecs::DynamicBundle) -> EntityId {
        let entity = self.inner.spawn(components);
        EntityId(entity)
    }

    /// Despawns the entity, removing it and all its components from the world.
    ///
    /// Returns `Err` if the entity does not exist.
    pub fn despawn_entity(&mut self, id: EntityId) -> Result<(), hecs::NoSuchEntity> {
        self.inner.despawn(id.0)
    }

    /// Adds a single component to an existing entity.
    ///
    /// If the entity already has a component of this type it is replaced.
    /// Returns `Err` if the entity does not exist.
    pub fn add_component<C: Component>(
        &mut self,
        id: EntityId,
        component: C,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.inner.insert_one(id.0, component)
    }

    /// Removes a single component from an entity, returning it.
    pub fn remove_component<C: Component>(
        &mut self,
        id: EntityId,
    ) -> Result<C, hecs::ComponentError> {
        self.inner.remove_one::<C>(id.0)
    }

    /// Provides query access identical to [`hecs::World::query`].
    #[inline]
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }

    /// Returns `true` if the entity exists in the world.
    #[inline]
    pub fn contains(&self, id: EntityId) -> bool {
        self.inner.contains(id.0)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(f32, f32, f32);

    #[derive(Debug, PartialEq)]
    struct Velocity(f32, f32, f32);

    #[test]
    fn spawn_and_despawn() {
        let mut world = World::new();
        let id = world.spawn_entity((Position(1.0, 2.0, 3.0),));
        assert!(world.contains(id));
        world.despawn_entity(id).unwrap();
        assert!(!world.contains(id));
    }

    #[test]
    fn add_and_remove_component() {
        let mut world = World::new();
        let id = world.spawn_entity((Position(0.0, 0.0, 0.0),));
        world.add_component(id, Velocity(1.0, 0.0, 0.0)).unwrap();

        let vel = world.remove_component::<Velocity>(id).unwrap();
        assert_eq!(vel, Velocity(1.0, 0.0, 0.0));
    }

    #[test]
    fn query_entities() {
        let mut world = World::new();
        world.spawn_entity((Position(1.0, 0.0, 0.0), Velocity(0.0, 1.0, 0.0)));
        world.spawn_entity((Position(2.0, 0.0, 0.0),));

        let mut count = 0;
        for (_id, (_pos, _vel)) in world.query::<(&Position, &Velocity)>().iter() {
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn entity_id_display() {
        let mut world = World::new();
        let id = world.spawn_entity((Position(0.0, 0.0, 0.0),));
        let display = format!("{id}");
        assert!(!display.is_empty());
    }

    #[test]
    fn entity_id_serde_roundtrip() {
        let mut world = World::new();
        let id = world.spawn_entity((Position(0.0, 0.0, 0.0),));
        let json = serde_json::to_string(&id).unwrap();
        let _deserialized: EntityId = serde_json::from_str(&json).unwrap();
    }
}
