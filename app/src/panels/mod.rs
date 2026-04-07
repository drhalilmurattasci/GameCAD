//! UI panels surrounding the central viewport.
//!
//! Each sub-module implements one panel as an `impl ForgeEditorApp` block:
//! toolbar (tab bar + tools), outliner (scene tree), inspector (properties),
//! bottom (content browser / console / agent progress / timeline), and
//! status bar.

pub(crate) mod bottom;
pub(crate) mod inspector;
pub(crate) mod outliner;
pub(crate) mod status_bar;
pub(crate) mod toolbar;
