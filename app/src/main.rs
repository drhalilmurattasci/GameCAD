//! Forge Editor - Main Application Binary.
//!
//! A Crystalline dark-themed, Fusion-style tabbed 3D editor built on egui/eframe.
//! This crate provides the top-level entry point that initializes tracing, configures
//! the native window, and launches the [`ForgeEditorApp`] via eframe.

use eframe::egui;

// The tc!() macro must be defined before module declarations so all modules can use it.
#[macro_use]
mod theme;

mod input_handling;
mod panels;
mod state;
mod viewport;

use state::ForgeEditorApp;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Initializes logging, configures the native window, and runs the editor.
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Forge Editor");

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        vsync: true,
        viewport: egui::ViewportBuilder::default()
            .with_title(state::settings::APP_NAME)
            .with_inner_size([state::settings::DEFAULT_WINDOW_WIDTH, state::settings::DEFAULT_WINDOW_HEIGHT])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Forge Editor",
        options,
        Box::new(|cc| Ok(Box::new(ForgeEditorApp::new(cc)))),
    )
}
