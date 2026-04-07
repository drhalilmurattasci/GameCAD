//! Ray-casting and mouse-picking utilities for selecting objects in the viewport.

use forge_core::ecs::EntityId;
use forge_core::math::{Ray, AABB};
use glam::{Mat4, Vec2, Vec4};

/// Converts a 2D screen-space mouse position to a world-space ray.
///
/// `mouse_pos` is in pixels (origin top-left).
/// `viewport_size` is the viewport dimensions in pixels.
/// `view_proj_inv` is the inverse of the combined view-projection matrix.
pub fn screen_to_ray(mouse_pos: Vec2, viewport_size: Vec2, view_proj_inv: Mat4) -> Ray {
    // Convert to normalized device coordinates [-1, 1].
    let ndc_x = (mouse_pos.x / viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (mouse_pos.y / viewport_size.y) * 2.0; // flip Y

    let near_ndc = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let near_world = view_proj_inv * near_ndc;
    let far_world = view_proj_inv * far_ndc;

    let near_point = near_world.truncate() / near_world.w;
    let far_point = far_world.truncate() / far_world.w;

    let direction = (far_point - near_point).normalize();
    Ray::new(near_point, direction)
}

/// Tests a ray against an axis-aligned bounding box.
///
/// Returns `Some(t)` with the nearest positive intersection distance, or `None`.
pub fn ray_aabb_intersect(ray: &Ray, aabb: &AABB) -> Option<f32> {
    aabb.intersects_ray(ray)
}

/// Picks the closest entity whose AABB intersects the ray.
///
/// `entities` is a slice of `(EntityId, AABB)` pairs.
/// Returns the closest hit as `(entity, distance)`.
pub fn pick_entity(ray: &Ray, entities: &[(EntityId, AABB)]) -> Option<(EntityId, f32)> {
    let mut closest: Option<(EntityId, f32)> = None;

    for (entity, aabb) in entities {
        if let Some(t) = ray_aabb_intersect(ray, aabb)
            && t >= 0.0
            && closest.is_none_or(|(_, prev_t)| t < prev_t)
        {
            closest = Some((*entity, t));
        }
    }

    closest
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn screen_to_ray_center() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let vp_inv = (proj * view).inverse();

        let ray = screen_to_ray(Vec2::new(400.0, 300.0), Vec2::new(800.0, 600.0), vp_inv);
        // Ray should point roughly along -Z from position (0,0,5)
        assert!(ray.direction.z < 0.0);
    }

    #[test]
    fn ray_aabb_hit() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let t = ray_aabb_intersect(&ray, &aabb);
        assert!(t.is_some());
        assert!((t.unwrap() - 4.0).abs() < 1e-4);
    }

    #[test]
    fn ray_aabb_miss() {
        let ray = Ray::new(Vec3::new(0.0, 10.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(ray_aabb_intersect(&ray, &aabb).is_none());
    }
}
