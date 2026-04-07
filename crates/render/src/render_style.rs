//! Render style selection for the viewport.
//!
//! [`RenderStyle`] is used by the pipeline cache to select the appropriate
//! WGSL shader and pipeline configuration for the current viewport mode.

use std::fmt;

/// Controls how meshes are rendered in the viewport.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RenderStyle {
    /// Full PBR shading with lights and materials.
    #[default]
    Pbr,
    /// Wireframe only.
    Wireframe,
    /// Flat unlit color (vertex colors or material base color).
    Unlit,
    /// Normal-map visualization (world-space normals mapped to RGB).
    Normals,
    /// Depth visualization (near = white, far = black).
    Depth,
}

impl fmt::Display for RenderStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pbr => write!(f, "PBR Lit"),
            Self::Wireframe => write!(f, "Wireframe"),
            Self::Unlit => write!(f, "Unlit"),
            Self::Normals => write!(f, "Normals"),
            Self::Depth => write!(f, "Depth"),
        }
    }
}

impl RenderStyle {
    /// All available render styles.
    pub const ALL: &'static [RenderStyle] = &[
        Self::Pbr,
        Self::Wireframe,
        Self::Unlit,
        Self::Normals,
        Self::Depth,
    ];

    /// Whether this style needs the lighting uniform buffer.
    #[inline]
    pub fn needs_lighting(&self) -> bool {
        matches!(self, Self::Pbr)
    }
}
