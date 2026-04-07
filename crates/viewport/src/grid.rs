//! Configuration for the viewport reference grid overlay.

use forge_core::math::Color;
use serde::{Deserialize, Serialize};

/// Configuration for the viewport reference grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Distance between minor grid lines.
    pub spacing: f32,
    /// Draw a major line every N minor lines.
    pub major_every: u32,
    /// Color of minor grid lines.
    pub color: Color,
    /// Color of major grid lines.
    pub major_color: Color,
    /// Distance at which the grid fades out.
    pub fade_distance: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            spacing: 1.0,
            major_every: 10,
            color: Color::new(0.3, 0.3, 0.3, 0.5),
            major_color: Color::new(0.5, 0.5, 0.5, 0.8),
            fade_distance: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid_is_sane() {
        let grid = GridConfig::default();
        assert!(grid.spacing > 0.0);
        assert!(grid.major_every > 0);
        assert!(grid.fade_distance > 0.0);
    }
}
