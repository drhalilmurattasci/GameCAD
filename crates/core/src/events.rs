//! Typed event bus built on top of [`crossbeam_channel`].
//!
//! Any type implementing [`Event`] (i.e. `Clone + Send + Sync + 'static`) can
//! be published and subscribed to through the [`EventBus`].
//!
//! # Examples
//!
//! ```
//! use core::events::EventBus;
//!
//! let bus = EventBus::new();
//! let rx = bus.subscribe::<String>();
//! bus.publish("hello".to_string());
//! assert_eq!(rx.try_recv(), Some("hello".to_string()));
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;

use crate::ecs::EntityId;
use crate::id::AssetId;

// ─────────────────────────────────────────────────────────────────────
// Event trait
// ─────────────────────────────────────────────────────────────────────

/// Marker trait for event payloads.
pub trait Event: Clone + Send + Sync + 'static {}

// Blanket implementation.
impl<T: Clone + Send + Sync + 'static> Event for T {}

// ─────────────────────────────────────────────────────────────────────
// EventReceiver
// ─────────────────────────────────────────────────────────────────────

/// A typed handle returned by [`EventBus::subscribe`].
pub struct EventReceiver<T: Event> {
    rx: Receiver<T>,
}

impl<T: Event> EventReceiver<T> {
    /// Non-blocking attempt to receive the next event.
    pub fn try_recv(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }

    /// Drains all currently pending events into a [`Vec`].
    pub fn drain(&self) -> Vec<T> {
        self.rx.try_iter().collect()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Channel entry (type-erased)
// ─────────────────────────────────────────────────────────────────────

struct ChannelEntry {
    /// Type-erased `Vec<Sender<T>>` stored as `Box<dyn Any + Send + Sync>`.
    senders: Box<dyn Any + Send + Sync>,
}

// ─────────────────────────────────────────────────────────────────────
// EventBus
// ─────────────────────────────────────────────────────────────────────

/// A multi-producer, multi-consumer event bus keyed by [`TypeId`].
pub struct EventBus {
    channels: RwLock<HashMap<TypeId, ChannelEntry>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Creates a new, empty event bus.
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// Publishes an event to all current subscribers of type `T`.
    ///
    /// If there are no subscribers the event is silently dropped.
    pub fn publish<T: Event>(&self, event: T) {
        let channels = self.channels.read();
        if let Some(entry) = channels.get(&TypeId::of::<T>()) {
            let senders = entry
                .senders
                .downcast_ref::<Vec<Sender<T>>>()
                .expect("type mismatch in event bus");
            for sender in senders {
                // Ignore send errors (receiver dropped).
                let _ = sender.send(event.clone());
            }
        }
    }

    /// Subscribes to events of type `T`, returning an [`EventReceiver`].
    ///
    /// Multiple subscribers are supported; each receives its own copy of every
    /// published event.
    pub fn subscribe<T: Event>(&self) -> EventReceiver<T> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut channels = self.channels.write();
        let entry = channels
            .entry(TypeId::of::<T>())
            .or_insert_with(|| ChannelEntry {
                senders: Box::new(Vec::<Sender<T>>::new()),
            });
        entry
            .senders
            .downcast_mut::<Vec<Sender<T>>>()
            .expect("type mismatch in event bus")
            .push(tx);
        EventReceiver { rx }
    }

    /// Convenience: subscribes, publishes nothing, and immediately drains.
    ///
    /// Useful for one-shot polling when you already have a receiver.
    pub fn drain<T: Event>(&self, receiver: &EventReceiver<T>) -> Vec<T> {
        receiver.drain()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Common events
// ─────────────────────────────────────────────────────────────────────

/// Emitted when the current entity selection changes.
#[derive(Debug, Clone)]
pub struct SelectionChanged {
    /// The newly selected entities.
    pub selected: Vec<EntityId>,
}

/// Emitted when the scene graph has been structurally modified.
#[derive(Debug, Clone)]
pub struct SceneModified;

/// Emitted after an asset has been imported into the project.
#[derive(Debug, Clone)]
pub struct AssetImported {
    /// The identifier of the imported asset.
    pub asset_id: AssetId,
}

/// Emitted after an undo operation has been performed.
#[derive(Debug, Clone)]
pub struct UndoPerformed {
    /// Human-readable description of the undone command.
    pub description: String,
}

/// Emitted after a redo operation has been performed.
#[derive(Debug, Clone)]
pub struct RedoPerformed {
    /// Human-readable description of the redone command.
    pub description: String,
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEvent(u32);

    #[test]
    fn publish_and_receive() {
        let bus = EventBus::new();
        let rx = bus.subscribe::<TestEvent>();
        bus.publish(TestEvent(42));
        assert_eq!(rx.try_recv(), Some(TestEvent(42)));
    }

    #[test]
    fn multiple_subscribers() {
        let bus = EventBus::new();
        let rx1 = bus.subscribe::<TestEvent>();
        let rx2 = bus.subscribe::<TestEvent>();
        bus.publish(TestEvent(7));
        assert_eq!(rx1.try_recv(), Some(TestEvent(7)));
        assert_eq!(rx2.try_recv(), Some(TestEvent(7)));
    }

    #[test]
    fn drain_collects_all() {
        let bus = EventBus::new();
        let rx = bus.subscribe::<TestEvent>();
        bus.publish(TestEvent(1));
        bus.publish(TestEvent(2));
        bus.publish(TestEvent(3));
        let events = bus.drain(&rx);
        assert_eq!(events, vec![TestEvent(1), TestEvent(2), TestEvent(3)]);
    }

    #[test]
    fn no_subscriber_silent_drop() {
        let bus = EventBus::new();
        // Should not panic.
        bus.publish(TestEvent(99));
    }

    #[test]
    fn different_event_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct OtherEvent(String);

        let bus = EventBus::new();
        let rx_test = bus.subscribe::<TestEvent>();
        let rx_other = bus.subscribe::<OtherEvent>();

        bus.publish(TestEvent(1));
        bus.publish(OtherEvent("hello".into()));

        assert_eq!(rx_test.try_recv(), Some(TestEvent(1)));
        assert_eq!(rx_other.try_recv(), Some(OtherEvent("hello".into())));
        assert_eq!(rx_test.try_recv(), None);
    }
}
