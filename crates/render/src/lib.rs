//! # Forge Editor -- Render
//!
//! GPU rendering crate for the Forge Editor.
//!
//! Provides a wgpu-based renderer with multiple render styles (PBR, wireframe,
//! unlit, normals, depth, grid), camera and light management, glTF mesh loading,
//! and a pipeline cache.
//!
//! The main entry point is [`Renderer`], which orchestrates a full frame pass
//! against a [`GpuContext`] and a target texture view.
//!
//! # Examples
//!
//! ```
//! use render::{RenderStyle, Camera, LightSet};
//!
//! let cam = Camera::default();
//! let lights = LightSet::default();
//! let style = RenderStyle::Pbr;
//! assert!(style.needs_lighting());
//! ```

pub mod camera;
pub mod gpu;
pub mod lights;
pub mod mesh_loader;
pub mod pipeline;
pub mod render_style;
pub mod renderer;
pub mod texture;
pub mod vertex;

pub use camera::{Camera, CameraBuffer, CameraUniform, Projection};
pub use gpu::{GpuContext, SurfaceState};
pub use lights::{DirectionalLight, LightSet, LightUniform, LightsBuffer, PointLight, SpotLight};
pub use mesh_loader::{load_glb, load_glb_from_bytes};
pub use pipeline::PipelineCache;
pub use render_style::RenderStyle;
pub use renderer::Renderer;
pub use texture::Texture;
pub use vertex::{GpuMesh, Vertex};
