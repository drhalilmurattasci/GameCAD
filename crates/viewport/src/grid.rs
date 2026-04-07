//! Configuration and fade logic for the viewport reference grid overlay.

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
    /// Distance at which the grid starts fading out.
    pub fade_distance: f32,
    /// Distance beyond `fade_distance` at which the grid is fully invisible.
    /// Defaults to `fade_distance * 1.5`.
    pub fade_end: f32,
    /// Whether the grid is visible at all.
    pub visible: bool,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            spacing: 1.0,
            major_every: 10,
            color: Color::new(0.3, 0.3, 0.3, 0.5),
            major_color: Color::new(0.5, 0.5, 0.5, 0.8),
            fade_distance: 100.0,
            fade_end: 150.0,
            visible: true,
        }
    }
}

impl GridConfig {
    /// Computes the alpha multiplier for a grid line at the given distance
    /// from the camera.
    ///
    /// Returns a value in `[0.0, 1.0]`:
    /// - `1.0` when `distance <= fade_distance`
    /// - `0.0` when `distance >= fade_end`
    /// - Linearly interpolated between
    #[inline]
    pub fn fade_alpha(&self, distance: f32) -> f32 {
        if !self.visible || self.fade_end <= self.fade_distance {
            if distance <= self.fade_distance {
                return if self.visible { 1.0 } else { 0.0 };
            }
            return 0.0;
        }

        if distance <= self.fade_distance {
            1.0
        } else if distance >= self.fade_end {
            0.0
        } else {
            1.0 - (distance - self.fade_distance) / (self.fade_end - self.fade_distance)
        }
    }

    /// Returns the minor grid line color with alpha multiplied by the fade factor.
    #[inline]
    pub fn faded_color(&self, distance: f32) -> Color {
        let alpha = self.fade_alpha(distance);
        Color::new(self.color.r, self.color.g, self.color.b, self.color.a * alpha)
    }

    /// Returns the major grid line color with alpha multiplied by the fade factor.
    #[inline]
    pub fn faded_major_color(&self, distance: f32) -> Color {
        let alpha = self.fade_alpha(distance);
        Color::new(
            self.major_color.r,
            self.major_color.g,
            self.major_color.b,
            self.major_color.a * alpha,
        )
    }

    /// Returns `true` if the grid line at `index` is a major line.
    #[inline]
    pub fn is_major_line(&self, index: i32) -> bool {
        self.major_every > 0 && index % self.major_every as i32 == 0
    }

    /// Returns the world-space coordinate for grid line at the given index.
    #[inline]
    pub fn line_position(&self, index: i32) -> f32 {
        index as f32 * self.spacing
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
        assert!(grid.fade_end > grid.fade_distance);
        assert!(grid.visible);
    }

    #[test]
    fn fade_alpha_within_range() {
        let grid = GridConfig::default();
        // Well within fade_distance
        assert!((grid.fade_alpha(50.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fade_alpha_at_edge() {
        let grid = GridConfig::default();
        // Exactly at fade_distance
        assert!((grid.fade_alpha(100.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fade_alpha_beyond_fade_end() {
        let grid = GridConfig::default();
        // Beyond fade_end
        assert!((grid.fade_alpha(200.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn fade_alpha_midpoint() {
        let grid = GridConfig::default();
        // Midpoint between fade_distance (100) and fade_end (150) -> alpha 0.5
        let alpha = grid.fade_alpha(125.0);
        assert!((alpha - 0.5).abs() < 1e-5);
    }

    #[test]
    fn fade_alpha_invisible_grid() {
        let mut grid = GridConfig::default();
        grid.visible = false;
        assert_eq!(grid.fade_alpha(0.0), 0.0);
        assert_eq!(grid.fade_alpha(50.0), 0.0);
    }

    #[test]
    fn faded_color_zero_at_distance() {
        let grid = GridConfig::default();
        let c = grid.faded_color(200.0);
        assert!((c.a).abs() < f32::EPSILON);
    }

    #[test]
    fn faded_color_full_at_close() {
        let grid = GridConfig::default();
        let c = grid.faded_color(10.0);
        assert!((c.a - grid.color.a).abs() < f32::EPSILON);
    }

    #[test]
    fn faded_major_color_preserves_rgb() {
        let grid = GridConfig::default();
        let c = grid.faded_major_color(125.0);
        assert!((c.r - grid.major_color.r).abs() < f32::EPSILON);
        assert!((c.g - grid.major_color.g).abs() < f32::EPSILON);
        assert!((c.b - grid.major_color.b).abs() < f32::EPSILON);
    }

    #[test]
    fn is_major_line_works() {
        let grid = GridConfig::default(); // major_every = 10
        assert!(grid.is_major_line(0));
        assert!(grid.is_major_line(10));
        assert!(grid.is_major_line(-10));
        assert!(!grid.is_major_line(1));
        assert!(!grid.is_major_line(5));
    }

    #[test]
    fn line_position_correct() {
        let grid = GridConfig::default(); // spacing = 1.0
        assert!((grid.line_position(5) - 5.0).abs() < f32::EPSILON);
        assert!((grid.line_position(-3) - (-3.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn line_position_with_custom_spacing() {
        let mut grid = GridConfig::default();
        grid.spacing = 2.5;
        assert!((grid.line_position(4) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fade_alpha_negative_distance() {
        let grid = GridConfig::default();
        // Negative distance (shouldn't happen but should not panic)
        let alpha = grid.fade_alpha(-10.0);
        assert!((alpha - 1.0).abs() < f32::EPSILON);
    }
}
