//! Forge Editor - Main Application Binary.
//!
//! A Crystalline dark-themed, Fusion-style tabbed 3D editor built on egui/eframe.
//! This crate provides the top-level entry point that initializes tracing, configures
//! the native window, and launches the [`ForgeEditorApp`] via eframe.

use eframe::egui;

// The tc!() macro must be defined before module declarations so all modules can use it.
#[macro_use]
mod theme;

mod app;
mod commands;
mod entity;
mod input;
mod panels;
pub(crate) mod settings;
mod types;
mod viewport;

use app::ForgeEditorApp;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Initializes logging, configures the native window, and runs the editor.
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Forge Editor");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(settings::APP_NAME)
            .with_inner_size([settings::DEFAULT_WINDOW_WIDTH, settings::DEFAULT_WINDOW_HEIGHT])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Forge Editor",
        options,
        Box::new(|_cc| Ok(Box::new(ForgeEditorApp::default()))),
    )
}
