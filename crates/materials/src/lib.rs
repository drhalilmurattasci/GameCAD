//! # Materials
//!
//! PBR material definitions, a material library, a node-based material graph
//! with an egui editor, and a WGSL shader compiler.
//!
//! ## Modules
//!
//! - [`material`] -- [`PbrMaterial`](material::PbrMaterial) struct and TOML persistence.
//! - [`library`] -- [`MaterialLibrary`](library::MaterialLibrary) for managing collections.
//! - [`graph`] -- data model for the node-based material graph.
//! - [`editor`] -- egui visual editor for the node graph.
//! - [`compiler`] -- compiles a [`MaterialGraph`](graph::MaterialGraph) to WGSL.

pub mod compiler;
pub mod editor;
pub mod graph;
pub mod library;
pub mod material;

// Backwards-compatible aliases for the old module names.
pub use editor as node_editor;
pub use graph as node_graph;
