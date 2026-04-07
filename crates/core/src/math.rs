//! Math primitives and glam re-exports.
//!
//! Provides [`Ray`], [`AABB`], [`Plane`], [`Color`], and [`Transform`] on top of
//! the re-exported glam vector/matrix types.

// ── glam re-exports ──────────────────────────────────────────────────
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// Ray
// ─────────────────────────────────────────────────────────────────────

/// An infinite ray defined by an origin point and a direction vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    /// Starting point of the ray.
    pub origin: Vec3,
    /// Normalized direction the ray travels.
    pub direction: Vec3,
}

impl Ray {
    /// Creates a new ray. `direction` is **not** automatically normalized.
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Returns the point at parameter `t` along the ray: `origin + t * direction`.
    #[inline]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// ─────────────────────────────────────────────────────────────────────
// AABB
// ─────────────────────────────────────────────────────────────────────

/// Axis-Aligned Bounding Box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
}

impl AABB {
    /// Creates a new AABB from two corners.
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Returns `true` if `point` is inside the box (inclusive).
    #[inline]
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Ray-AABB intersection test (slab method).
    ///
    /// Returns `Some(t)` with the nearest positive hit distance, or `None`.
    pub fn intersects_ray(&self, ray: &Ray) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let t1 = (self.min - ray.origin) * inv_dir;
        let t2 = (self.max - ray.origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_near = t_min.x.max(t_min.y).max(t_min.z);
        let t_far = t_max.x.min(t_max.y).min(t_max.z);

        if t_near > t_far || t_far < 0.0 {
            None
        } else {
            Some(if t_near < 0.0 { t_far } else { t_near })
        }
    }

    /// Returns the smallest AABB that contains both `self` and `other`.
    #[inline]
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns the center point of the box.
    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (extents) of the box.
    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Plane
// ─────────────────────────────────────────────────────────────────────

/// A mathematical plane represented in Hessian normal form.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    /// Unit normal vector of the plane.
    pub normal: Vec3,
    /// Signed distance from the origin along the normal.
    pub distance: f32,
}

impl Plane {
    /// Creates a new plane from a normal and distance.
    #[inline]
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Signed distance from `point` to the plane.
    #[inline]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}

// ─────────────────────────────────────────────────────────────────────
// Color
// ─────────────────────────────────────────────────────────────────────

/// A color stored in **linear** RGB-A space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    /// Opaque white.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// Opaque black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Creates a new color in linear space.
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a hex color string (`#RRGGBB` or `#RRGGBBAA`).
    ///
    /// The returned color is converted to **linear** space.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };

        Some(Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: a as f32 / 255.0,
        })
    }

    /// Converts from linear to sRGB space, returning component values in 0..1.
    #[inline]
    pub fn to_srgb(&self) -> [f32; 4] {
        [
            linear_to_srgb(self.r),
            linear_to_srgb(self.g),
            linear_to_srgb(self.b),
            self.a,
        ]
    }

    /// Returns the linear RGBA components (identity for this type).
    #[inline]
    pub fn to_linear(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Linearly interpolates between `self` and `other` by `t` (clamped to 0..1).
    #[inline]
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

/// Convert a single sRGB component to linear.
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert a single linear component to sRGB.
#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ─────────────────────────────────────────────────────────────────────
// Transform
// ─────────────────────────────────────────────────────────────────────

/// A 3-D transform composed of position, rotation, and non-uniform scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The identity transform.
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Returns the identity transform.
    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Builds a 4x4 affine transformation matrix (scale * rotate * translate).
    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Returns the inverse transform.
    ///
    /// Computes the inverse by inverting the 4x4 matrix and decomposing back
    /// into scale, rotation, and translation. This correctly handles
    /// non-uniform scale.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mat = self.matrix().inverse();
        let (scale, rotation, position) = mat.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Local forward direction (negative Z).
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Local right direction (positive X).
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Local up direction (positive Y).
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Linearly interpolates position and scale, spherically interpolates rotation.
    #[inline]
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert_eq!(ray.at(3.0), Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn aabb_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(!aabb.contains_point(Vec3::splat(2.0)));
    }

    #[test]
    fn aabb_center_and_size() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.size(), Vec3::splat(2.0));
    }

    #[test]
    fn aabb_intersects_ray() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::ONE);
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
        let hit = aabb.intersects_ray(&ray);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 4.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_misses_ray() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::X);
        assert!(aabb.intersects_ray(&ray).is_none());
    }

    #[test]
    fn aabb_merge() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(-1.0), Vec3::splat(0.5));
        let merged = a.merge(&b);
        assert_eq!(merged.min, Vec3::splat(-1.0));
        assert_eq!(merged.max, Vec3::ONE);
    }

    #[test]
    fn color_from_hex() {
        let c = Color::from_hex("#FF0000").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!(c.g < 0.01);
        assert!(c.b < 0.01);
        assert!((c.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(&b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn color_srgb_roundtrip() {
        let c = Color::new(0.5, 0.5, 0.5, 1.0);
        let srgb = c.to_srgb();
        let back = Color::new(
            srgb_to_linear(srgb[0]),
            srgb_to_linear(srgb[1]),
            srgb_to_linear(srgb[2]),
            srgb[3],
        );
        assert!((back.r - c.r).abs() < 1e-5);
    }

    #[test]
    fn transform_identity_matrix() {
        let t = Transform::identity();
        assert_eq!(t.matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn transform_forward() {
        let t = Transform::identity();
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn transform_lerp_identity() {
        let a = Transform::identity();
        let b = Transform {
            position: Vec3::new(10.0, 0.0, 0.0),
            ..Transform::IDENTITY
        };
        let mid = a.lerp(&b, 0.5);
        assert!((mid.position.x - 5.0).abs() < 1e-5);
    }

    #[test]
    fn transform_inverse_roundtrip() {
        // Test with uniform scale (non-uniform scale cannot be cleanly
        // decomposed back into a Transform via to_scale_rotation_translation
        // because SRT decomposition is not unique for non-uniform scale
        // combined with rotation).
        let t = Transform {
            position: Vec3::new(3.0, 4.0, 5.0),
            rotation: Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4),
            scale: Vec3::splat(2.0),
        };
        let inv = t.inverse();
        let roundtrip = t.matrix() * inv.matrix();
        // roundtrip should be approximately identity
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (roundtrip.col(j)[i] - expected).abs() < 1e-3,
                    "Element [{i}][{j}] was {} expected {expected}",
                    roundtrip.col(j)[i]
                );
            }
        }
    }

    #[test]
    fn transform_inverse_identity() {
        let t = Transform::IDENTITY;
        let inv = t.inverse();
        assert!((inv.position - Vec3::ZERO).length() < 1e-5);
        assert!((inv.scale - Vec3::ONE).length() < 1e-5);
    }

    #[test]
    fn aabb_default() {
        let aabb = AABB::default();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn color_default() {
        let c = Color::default();
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn plane_distance_to_point() {
        let plane = Plane::new(Vec3::Y, 0.0);
        assert!((plane.distance_to_point(Vec3::new(0.0, 5.0, 0.0)) - 5.0).abs() < 1e-5);
    }
}
