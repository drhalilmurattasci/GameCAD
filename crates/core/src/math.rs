//! Math primitives and glam re-exports.
//!
//! Provides [`Ray`], [`AABB`], [`Plane`], [`Color`], and [`Transform`] on top of
//! the re-exported glam vector/matrix types.

// ── glam re-exports ──────────────────────────────────────────────────
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

use std::fmt;

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// Ray
// ─────────────────────────────────────────────────────────────────────

/// An infinite ray defined by an origin point and a direction vector.
///
/// # Examples
///
/// ```
/// use core::math::{Ray, Vec3};
///
/// let ray = Ray::new(Vec3::ZERO, Vec3::X);
/// assert_eq!(ray.at(5.0), Vec3::new(5.0, 0.0, 0.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    /// Starting point of the ray.
    pub origin: Vec3,
    /// Direction the ray travels. Not required to be normalized, but many
    /// intersection routines assume unit length; prefer [`Ray::normalized`].
    pub direction: Vec3,
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ray(origin: [{:.3}, {:.3}, {:.3}], dir: [{:.3}, {:.3}, {:.3}])",
            self.origin.x, self.origin.y, self.origin.z,
            self.direction.x, self.direction.y, self.direction.z,
        )
    }
}

impl Ray {
    /// Creates a new ray. `direction` is used as-is (not automatically normalized).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Ray, Vec3};
    ///
    /// let ray = Ray::new(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0));
    /// // direction is NOT normalized
    /// assert_eq!(ray.direction.length(), 2.0);
    /// ```
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Creates a new ray with a normalized direction.
    ///
    /// Returns `None` if the direction vector is zero-length (or very close to it).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Ray, Vec3};
    ///
    /// let ray = Ray::normalized(Vec3::ZERO, Vec3::new(3.0, 0.0, 0.0)).unwrap();
    /// assert!((ray.direction.length() - 1.0).abs() < 1e-6);
    ///
    /// assert!(Ray::normalized(Vec3::ZERO, Vec3::ZERO).is_none());
    /// ```
    #[inline]
    pub fn normalized(origin: Vec3, direction: Vec3) -> Option<Self> {
        let len = direction.length();
        if len < 1e-10 {
            return None;
        }
        Some(Self {
            origin,
            direction: direction / len,
        })
    }

    /// Returns the point at parameter `t` along the ray: `origin + t * direction`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Ray, Vec3};
    ///
    /// let ray = Ray::new(Vec3::ZERO, Vec3::X);
    /// assert_eq!(ray.at(3.0), Vec3::new(3.0, 0.0, 0.0));
    /// assert_eq!(ray.at(0.0), Vec3::ZERO);
    /// ```
    #[inline]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

// ─────────────────────────────────────────────────────────────────────
// AABB
// ─────────────────────────────────────────────────────────────────────

/// Axis-Aligned Bounding Box.
///
/// # Examples
///
/// ```
/// use core::math::{AABB, Vec3};
///
/// let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
/// assert!(aabb.contains_point(Vec3::splat(0.5)));
/// assert!(!aabb.contains_point(Vec3::splat(2.0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
}

impl fmt::Display for AABB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AABB(min: [{:.3}, {:.3}, {:.3}], max: [{:.3}, {:.3}, {:.3}])",
            self.min.x, self.min.y, self.min.z,
            self.max.x, self.max.y, self.max.z,
        )
    }
}

impl AABB {
    /// Creates a new AABB from two corners.
    ///
    /// The caller must ensure `min <= max` component-wise; this constructor
    /// does **not** reorder the corners.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
    /// assert_eq!(aabb.center(), Vec3::ZERO);
    /// ```
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates an AABB from two arbitrary points, ensuring `min <= max`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::from_points(Vec3::ONE, Vec3::ZERO);
    /// assert_eq!(aabb.min, Vec3::ZERO);
    /// assert_eq!(aabb.max, Vec3::ONE);
    /// ```
    #[inline]
    pub fn from_points(a: Vec3, b: Vec3) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    /// Returns `true` if `point` is inside the box (inclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    /// assert!(aabb.contains_point(Vec3::ZERO));   // on boundary
    /// assert!(aabb.contains_point(Vec3::ONE));     // on boundary
    /// assert!(!aabb.contains_point(Vec3::splat(2.0)));
    /// ```
    #[inline]
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Returns `true` if `self` and `other` overlap (inclusive of touching).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    /// let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
    /// assert!(a.intersects(&b));
    ///
    /// let c = AABB::new(Vec3::splat(5.0), Vec3::splat(6.0));
    /// assert!(!a.intersects(&c));
    /// ```
    #[inline]
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Ray-AABB intersection test (slab method).
    ///
    /// Returns `Some(t)` with the nearest positive hit distance, or `None`.
    /// Correctly handles rays with zero-component directions (axis-aligned rays)
    /// via IEEE 754 infinity arithmetic.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Ray, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::splat(-1.0), Vec3::ONE);
    /// let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
    /// let t = aabb.intersects_ray(&ray).unwrap();
    /// assert!((t - 4.0).abs() < 1e-5);
    /// ```
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

        // Handle NaN from 0 * inf: if any component is NaN the comparison
        // returns false, which correctly reports no intersection.
        if t_near > t_far || t_far < 0.0 {
            None
        } else {
            Some(if t_near < 0.0 { t_far } else { t_near })
        }
    }

    /// Returns the smallest AABB that contains both `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    /// let b = AABB::new(Vec3::splat(-1.0), Vec3::splat(0.5));
    /// let m = a.merge(&b);
    /// assert_eq!(m.min, Vec3::splat(-1.0));
    /// assert_eq!(m.max, Vec3::ONE);
    /// ```
    #[inline]
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns the center point of the box.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::ZERO, Vec3::splat(2.0));
    /// assert_eq!(aabb.center(), Vec3::ONE);
    /// ```
    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (extents) of the box along each axis.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::ZERO, Vec3::new(3.0, 4.0, 5.0));
    /// assert_eq!(aabb.size(), Vec3::new(3.0, 4.0, 5.0));
    /// ```
    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns the half-extents (half the size along each axis).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::ZERO, Vec3::splat(4.0));
    /// assert_eq!(aabb.half_extents(), Vec3::splat(2.0));
    /// ```
    #[inline]
    pub fn half_extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Returns the volume of the bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{AABB, Vec3};
    ///
    /// let aabb = AABB::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
    /// assert!((aabb.volume() - 24.0).abs() < 1e-6);
    /// ```
    #[inline]
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s.x * s.y * s.z
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
///
/// # Examples
///
/// ```
/// use core::math::{Plane, Vec3};
///
/// // Ground plane (Y = 0)
/// let plane = Plane::new(Vec3::Y, 0.0);
/// assert!((plane.distance_to_point(Vec3::new(0.0, 5.0, 0.0)) - 5.0).abs() < 1e-5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    /// Unit normal vector of the plane.
    pub normal: Vec3,
    /// Signed distance from the origin along the normal.
    pub distance: f32,
}

impl fmt::Display for Plane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Plane(normal: [{:.3}, {:.3}, {:.3}], dist: {:.3})",
            self.normal.x, self.normal.y, self.normal.z, self.distance,
        )
    }
}

impl Plane {
    /// Creates a new plane from a normal and distance.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Plane, Vec3};
    ///
    /// let plane = Plane::new(Vec3::Y, 2.0);
    /// assert_eq!(plane.normal, Vec3::Y);
    /// ```
    #[inline]
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Creates a plane from a normal and a point on the plane.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Plane, Vec3};
    ///
    /// let plane = Plane::from_normal_and_point(Vec3::Y, Vec3::new(0.0, 3.0, 0.0));
    /// assert!((plane.distance - 3.0).abs() < 1e-6);
    /// ```
    #[inline]
    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Self {
        Self {
            normal,
            distance: normal.dot(point),
        }
    }

    /// Signed distance from `point` to the plane.
    ///
    /// Positive values mean the point is on the side the normal points towards.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Plane, Vec3};
    ///
    /// let plane = Plane::new(Vec3::Y, 0.0);
    /// assert!((plane.distance_to_point(Vec3::Y) - 1.0).abs() < 1e-5);
    /// assert!((plane.distance_to_point(Vec3::NEG_Y) + 1.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// Ray-plane intersection. Returns `Some(t)` if the ray hits the plane
    /// at `ray.at(t)`, or `None` if the ray is parallel.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Plane, Ray, Vec3};
    ///
    /// let plane = Plane::new(Vec3::Y, 0.0);
    /// let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
    /// let t = plane.intersect_ray(&ray).unwrap();
    /// assert!((t - 5.0).abs() < 1e-5);
    /// ```
    pub fn intersect_ray(&self, ray: &Ray) -> Option<f32> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < 1e-7 {
            return None; // Ray is parallel to the plane
        }
        let t = (self.distance - self.normal.dot(ray.origin)) / denom;
        Some(t)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Color
// ─────────────────────────────────────────────────────────────────────

/// A color stored in **linear** RGB-A space.
///
/// # Examples
///
/// ```
/// use core::math::Color;
///
/// let c = Color::from_hex("#FF0000").unwrap();
/// assert!((c.r - 1.0).abs() < 0.01);
/// assert!(c.g < 0.01);
///
/// let mid = Color::BLACK.lerp(&Color::WHITE, 0.5);
/// assert!((mid.r - 0.5).abs() < 1e-5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Red channel (linear space).
    pub r: f32,
    /// Green channel (linear space).
    pub g: f32,
    /// Blue channel (linear space).
    pub b: f32,
    /// Alpha channel (0.0 = fully transparent, 1.0 = fully opaque).
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color(r: {:.3}, g: {:.3}, b: {:.3}, a: {:.3})",
            self.r, self.g, self.b, self.a,
        )
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
    /// Opaque red.
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// Opaque green.
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    /// Opaque blue.
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    /// Creates a new color in linear space.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// let c = Color::new(0.5, 0.5, 0.5, 1.0);
    /// assert_eq!(c.a, 1.0);
    /// ```
    #[inline]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a hex color string (`#RRGGBB` or `#RRGGBBAA`).
    ///
    /// The returned color is converted to **linear** space.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// let c = Color::from_hex("#FF0000").unwrap();
    /// assert!((c.r - 1.0).abs() < 0.01);
    ///
    /// let c = Color::from_hex("00FF00FF").unwrap();  // no '#' prefix is fine
    /// assert!(c.g > 0.9);
    ///
    /// assert!(Color::from_hex("invalid").is_none());
    /// assert!(Color::from_hex("#FFF").is_none()); // too short
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// let srgb = Color::WHITE.to_srgb();
    /// assert!((srgb[0] - 1.0).abs() < 1e-6);
    /// ```
    #[inline]
    pub fn to_srgb(&self) -> [f32; 4] {
        [
            linear_to_srgb(self.r),
            linear_to_srgb(self.g),
            linear_to_srgb(self.b),
            self.a,
        ]
    }

    /// Returns the linear RGBA components as an array.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// let c = Color::new(0.1, 0.2, 0.3, 1.0);
    /// assert_eq!(c.to_linear(), [0.1, 0.2, 0.3, 1.0]);
    /// ```
    #[inline]
    pub fn to_linear(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Linearly interpolates between `self` and `other` by `t` (clamped to 0..1).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// let mid = Color::BLACK.lerp(&Color::WHITE, 0.5);
    /// assert!((mid.r - 0.5).abs() < 1e-5);
    /// assert!((mid.a - 1.0).abs() < 1e-5);
    /// ```
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

    /// Returns `true` if all components are finite (not NaN or infinite).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::Color;
    ///
    /// assert!(Color::WHITE.is_finite());
    /// assert!(!Color::new(f32::NAN, 0.0, 0.0, 1.0).is_finite());
    /// ```
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
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
///
/// # Examples
///
/// ```
/// use core::math::{Transform, Vec3, Quat, Mat4};
///
/// let t = Transform::IDENTITY;
/// assert_eq!(t.matrix(), Mat4::IDENTITY);
/// assert!((t.forward() - Vec3::NEG_Z).length() < 1e-5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// World-space position.
    pub position: Vec3,
    /// Orientation as a unit quaternion.
    pub rotation: Quat,
    /// Non-uniform scale along each axis.
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transform(pos: [{:.3}, {:.3}, {:.3}], rot: [{:.3}, {:.3}, {:.3}, {:.3}], scale: [{:.3}, {:.3}, {:.3}])",
            self.position.x, self.position.y, self.position.z,
            self.rotation.x, self.rotation.y, self.rotation.z, self.rotation.w,
            self.scale.x, self.scale.y, self.scale.z,
        )
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
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Mat4};
    ///
    /// assert_eq!(Transform::identity().matrix(), Mat4::IDENTITY);
    /// ```
    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Creates a transform from just a position, with identity rotation and
    /// unit scale.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3};
    ///
    /// let t = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
    /// assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
    /// assert_eq!(t.scale, Vec3::ONE);
    /// ```
    #[inline]
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    /// Builds a 4x4 affine transformation matrix (scale * rotate * translate).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3, Mat4};
    ///
    /// let t = Transform::from_position(Vec3::new(1.0, 0.0, 0.0));
    /// let m = t.matrix();
    /// // Translation is in the last column
    /// assert!((m.col(3).x - 1.0).abs() < 1e-6);
    /// ```
    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Returns the inverse transform.
    ///
    /// Computes the inverse by inverting the 4x4 matrix and decomposing back
    /// into scale, rotation, and translation. This correctly handles
    /// uniform scale; non-uniform scale combined with rotation may produce
    /// imprecise results due to ambiguity in SRT decomposition.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3, Quat};
    ///
    /// let t = Transform {
    ///     position: Vec3::new(3.0, 0.0, 0.0),
    ///     rotation: Quat::IDENTITY,
    ///     scale: Vec3::splat(2.0),
    /// };
    /// let inv = t.inverse();
    /// let roundtrip = t.matrix() * inv.matrix();
    /// // Should be approximately identity
    /// for i in 0..4 {
    ///     assert!((roundtrip.col(i)[i] - 1.0).abs() < 1e-3);
    /// }
    /// ```
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
    ///
    /// The parameter `t` is **not** clamped; values outside 0..1 will extrapolate.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3};
    ///
    /// let a = Transform::IDENTITY;
    /// let b = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
    /// let mid = a.lerp(&b, 0.5);
    /// assert!((mid.position.x - 5.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Transforms a point by this transform (applies scale, rotation, then translation).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3};
    ///
    /// let t = Transform::from_position(Vec3::new(1.0, 0.0, 0.0));
    /// assert_eq!(t.transform_point(Vec3::ZERO), Vec3::new(1.0, 0.0, 0.0));
    /// ```
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * (self.scale * point) + self.position
    }

    /// Transforms a direction vector (applies rotation only, no scale/translation).
    ///
    /// # Examples
    ///
    /// ```
    /// use core::math::{Transform, Vec3, Quat};
    ///
    /// let t = Transform {
    ///     rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    ///     ..Transform::IDENTITY
    /// };
    /// let dir = t.transform_direction(Vec3::Z);
    /// assert!((dir.x - 1.0).abs() < 1e-5); // Z rotated 90 deg around Y -> X
    /// ```
    #[inline]
    pub fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * direction
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Ray ──────────────────────────────────────────────────────────

    #[test]
    fn ray_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert_eq!(ray.at(3.0), Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn ray_at_negative_t() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert_eq!(ray.at(-2.0), Vec3::new(-2.0, 0.0, 0.0));
    }

    #[test]
    fn ray_at_zero_t() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::X);
        assert_eq!(ray.at(0.0), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn ray_normalized_constructor() {
        let ray = Ray::normalized(Vec3::ZERO, Vec3::new(3.0, 0.0, 0.0)).unwrap();
        assert!((ray.direction.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ray_normalized_zero_direction() {
        assert!(Ray::normalized(Vec3::ZERO, Vec3::ZERO).is_none());
    }

    #[test]
    fn ray_display() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let s = format!("{ray}");
        assert!(s.contains("Ray"));
    }

    // ── AABB ─────────────────────────────────────────────────────────

    #[test]
    fn aabb_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(!aabb.contains_point(Vec3::splat(2.0)));
    }

    #[test]
    fn aabb_contains_point_on_boundary() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::ZERO));
        assert!(aabb.contains_point(Vec3::ONE));
    }

    #[test]
    fn aabb_center_and_size() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.size(), Vec3::splat(2.0));
    }

    #[test]
    fn aabb_half_extents() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::splat(4.0));
        assert_eq!(aabb.half_extents(), Vec3::splat(2.0));
    }

    #[test]
    fn aabb_volume() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        assert!((aabb.volume() - 24.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_from_points_swapped() {
        let aabb = AABB::from_points(Vec3::ONE, Vec3::ZERO);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
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
    fn aabb_ray_inside() {
        // Ray origin inside the AABB should still return a hit (the far intersection)
        let aabb = AABB::new(Vec3::splat(-1.0), Vec3::ONE);
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let hit = aabb.intersects_ray(&ray);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_misses_ray() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::X);
        assert!(aabb.intersects_ray(&ray).is_none());
    }

    #[test]
    fn aabb_ray_behind() {
        // Ray pointing away from the AABB
        let aabb = AABB::new(Vec3::new(2.0, -1.0, -1.0), Vec3::new(3.0, 1.0, 1.0));
        let ray = Ray::new(Vec3::ZERO, Vec3::NEG_X);
        assert!(aabb.intersects_ray(&ray).is_none());
    }

    #[test]
    fn aabb_ray_axis_aligned_zero_component() {
        // Axis-aligned ray with zero direction components (tests inf handling)
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::ONE);
        let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
        assert!(aabb.intersects_ray(&ray).is_some());
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
    fn aabb_intersects_other() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b));

        let c = AABB::new(Vec3::splat(5.0), Vec3::splat(6.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn aabb_intersects_touching() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 1.0, 1.0));
        assert!(a.intersects(&b)); // touching at boundary
    }

    #[test]
    fn aabb_default() {
        let aabb = AABB::default();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn aabb_display() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        let s = format!("{aabb}");
        assert!(s.contains("AABB"));
    }

    // ── Plane ────────────────────────────────────────────────────────

    #[test]
    fn plane_distance_to_point() {
        let plane = Plane::new(Vec3::Y, 0.0);
        assert!((plane.distance_to_point(Vec3::new(0.0, 5.0, 0.0)) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn plane_distance_negative() {
        let plane = Plane::new(Vec3::Y, 0.0);
        assert!((plane.distance_to_point(Vec3::new(0.0, -3.0, 0.0)) + 3.0).abs() < 1e-5);
    }

    #[test]
    fn plane_from_normal_and_point() {
        let plane = Plane::from_normal_and_point(Vec3::Y, Vec3::new(0.0, 3.0, 0.0));
        assert!((plane.distance - 3.0).abs() < 1e-6);
        assert!((plane.distance_to_point(Vec3::new(0.0, 3.0, 0.0))).abs() < 1e-5);
    }

    #[test]
    fn plane_intersect_ray() {
        let plane = Plane::new(Vec3::Y, 0.0);
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
        let t = plane.intersect_ray(&ray).unwrap();
        assert!((t - 5.0).abs() < 1e-5);
    }

    #[test]
    fn plane_intersect_ray_parallel() {
        let plane = Plane::new(Vec3::Y, 0.0);
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::X); // parallel to plane
        assert!(plane.intersect_ray(&ray).is_none());
    }

    #[test]
    fn plane_display() {
        let plane = Plane::new(Vec3::Y, 1.0);
        let s = format!("{plane}");
        assert!(s.contains("Plane"));
    }

    // ── Color ────────────────────────────────────────────────────────

    #[test]
    fn color_from_hex() {
        let c = Color::from_hex("#FF0000").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!(c.g < 0.01);
        assert!(c.b < 0.01);
        assert!((c.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn color_from_hex_with_alpha() {
        let c = Color::from_hex("#FF000080").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.a - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn color_from_hex_invalid() {
        assert!(Color::from_hex("#FFF").is_none());
        assert!(Color::from_hex("").is_none());
        assert!(Color::from_hex("#GGGGGG").is_none());
    }

    #[test]
    fn color_from_hex_no_prefix() {
        let c = Color::from_hex("00FF00").unwrap();
        assert!(c.g > 0.9);
    }

    #[test]
    fn color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(&b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn color_lerp_clamped() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let over = a.lerp(&b, 1.5);
        assert!((over.r - 1.0).abs() < 1e-5);
        let under = a.lerp(&b, -0.5);
        assert!((under.r - 0.0).abs() < 1e-5);
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
    fn color_srgb_boundary_values() {
        // Values at the sRGB threshold boundary
        assert!((srgb_to_linear(0.0)).abs() < 1e-6);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
        assert!((linear_to_srgb(0.0)).abs() < 1e-6);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_is_finite() {
        assert!(Color::WHITE.is_finite());
        assert!(!Color::new(f32::NAN, 0.0, 0.0, 1.0).is_finite());
        assert!(!Color::new(0.0, f32::INFINITY, 0.0, 1.0).is_finite());
    }

    #[test]
    fn color_default() {
        let c = Color::default();
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn color_display() {
        let c = Color::RED;
        let s = format!("{c}");
        assert!(s.contains("Color"));
    }

    // ── Transform ────────────────────────────────────────────────────

    #[test]
    fn transform_identity_matrix() {
        let t = Transform::identity();
        assert_eq!(t.matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn transform_from_position() {
        let t = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn transform_forward() {
        let t = Transform::identity();
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn transform_right() {
        let t = Transform::identity();
        let right = t.right();
        assert!((right - Vec3::X).length() < 1e-5);
    }

    #[test]
    fn transform_up() {
        let t = Transform::identity();
        let up = t.up();
        assert!((up - Vec3::Y).length() < 1e-5);
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
        let t = Transform {
            position: Vec3::new(3.0, 4.0, 5.0),
            rotation: Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4),
            scale: Vec3::splat(2.0),
        };
        let inv = t.inverse();
        let roundtrip = t.matrix() * inv.matrix();
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
    fn transform_transform_point() {
        let t = Transform::from_position(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(t.transform_point(Vec3::ZERO), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(t.transform_point(Vec3::X), Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn transform_transform_point_with_scale() {
        let t = Transform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        };
        assert_eq!(t.transform_point(Vec3::ONE), Vec3::splat(2.0));
    }

    #[test]
    fn transform_transform_direction() {
        let t = Transform {
            position: Vec3::new(100.0, 0.0, 0.0), // position should not affect direction
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(5.0), // scale should not affect direction
        };
        let dir = t.transform_direction(Vec3::X);
        assert!((dir - Vec3::X).length() < 1e-5);
    }

    #[test]
    fn transform_display() {
        let t = Transform::IDENTITY;
        let s = format!("{t}");
        assert!(s.contains("Transform"));
    }
}
