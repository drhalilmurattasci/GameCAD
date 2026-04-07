//! # Forge Editor -- Core
//!
//! Foundational crate for the Forge Editor providing math primitives, ECS wrappers,
//! a typed event bus, an undo/redo command system, UUID-based identifiers, and
//! frame-timing utilities.
//!
//! All public types are gathered in the [`prelude`] module for convenience.
//!
//! # Examples
//!
//! ```
//! use core::prelude::*;
//!
//! let t = Transform::IDENTITY;
//! assert_eq!(t.matrix(), glam::Mat4::IDENTITY);
//!
//! let bus = EventBus::new();
//! let rx = bus.subscribe::<String>();
//! bus.publish("hello".to_string());
//! assert_eq!(rx.try_recv(), Some("hello".to_string()));
//! ```

pub mod commands;
pub mod ecs;
pub mod events;
pub mod id;
pub mod math;
pub mod time;

/// Convenience re-exports of the most commonly used types.
pub mod prelude {
    pub use crate::commands::{Command, CommandContext, CommandHistory};
    pub use crate::ecs::{EntityId, World};
    pub use crate::events::{Event, EventBus};
    pub use crate::id::{AssetId, MaterialId, NodeId, ScriptId};
    pub use crate::math::{Color, Plane, Transform, AABB, Ray};
    pub use crate::time::{Clock, DeltaTime, FrameCount, TotalTime};
}
