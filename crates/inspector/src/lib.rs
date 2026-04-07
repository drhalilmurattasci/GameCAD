//! # Inspector
//!
//! Property inspection panel for the Forge Editor. Provides typed property
//! editing widgets and an inspector panel that renders editable fields for
//! scene nodes (transforms, lights, cameras, meshes) using egui with the
//! Crystalline theme.
//!
//! ## Modules
//!
//! - [`panel`] -- the main [`InspectorPanel`](panel::InspectorPanel) widget.
//! - [`property`] -- dynamically-typed [`PropertyValue`](property::PropertyValue) enum.
//! - [`widgets`] -- low-level egui draw helpers for individual value types.

pub mod panel;
pub mod property;
pub mod widgets;
