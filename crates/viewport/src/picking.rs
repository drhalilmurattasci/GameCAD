//! Ray-casting and mouse-picking utilities for selecting objects in the viewport.

use forge_core::ecs::EntityId;
use forge_core::math::{Ray, AABB};
use glam::{Mat4, Vec2, Vec4};

/// Converts a 2D screen-space mouse position to a world-space ray.
///
/// `mouse_pos` is in pixels (origin top-left).
/// `viewport_size` is the viewport dimensions in pixels.
/// `view_proj_inv` is the inverse of the combined view-projection matrix.
///
/// Returns `None` if the viewport has zero width or height.
pub fn screen_to_ray(mouse_pos: Vec2, viewport_size: Vec2, view_proj_inv: Mat4) -> Option<Ray> {
    if viewport_size.x < 1.0 || viewport_size.y < 1.0 {
        return None;
    }

    // Convert to normalized device coordinates [-1, 1].
    let ndc_x = (mouse_pos.x / viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (mouse_pos.y / viewport_size.y) * 2.0; // flip Y

    let near_ndc = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let near_world = view_proj_inv * near_ndc;
    let far_world = view_proj_inv * far_ndc;

    // Guard against w being zero (degenerate projection)
    if near_world.w.abs() < 1e-10 || far_world.w.abs() < 1e-10 {
        return None;
    }

    let near_point = near_world.truncate() / near_world.w;
    let far_point = far_world.truncate() / far_world.w;

    let dir = far_point - near_point;
    let len = dir.length();
    if len < 1e-10 {
        return None;
    }

    Some(Ray::new(near_point, dir / len))
}

/// Tests a ray against an axis-aligned bounding box.
///
/// Returns `Some(t)` with the nearest positive intersection distance, or `None`.
#[inline]
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
    use forge_core::ecs::World;
    use glam::Vec3;

    #[test]
    fn screen_to_ray_center() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let vp_inv = (proj * view).inverse();

        let ray = screen_to_ray(Vec2::new(400.0, 300.0), Vec2::new(800.0, 600.0), vp_inv);
        assert!(ray.is_some());
        let ray = ray.unwrap();
        // Ray should point roughly along -Z from position (0,0,5)
        assert!(ray.direction.z < 0.0);
    }

    #[test]
    fn screen_to_ray_zero_viewport_returns_none() {
        let vp_inv = Mat4::IDENTITY;
        assert!(screen_to_ray(Vec2::ZERO, Vec2::ZERO, vp_inv).is_none());
        assert!(screen_to_ray(Vec2::ZERO, Vec2::new(0.0, 600.0), vp_inv).is_none());
        assert!(screen_to_ray(Vec2::ZERO, Vec2::new(800.0, 0.0), vp_inv).is_none());
    }

    #[test]
    fn screen_to_ray_corner() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let vp_inv = (proj * view).inverse();

        // Top-left corner
        let ray = screen_to_ray(Vec2::new(0.0, 0.0), Vec2::new(800.0, 600.0), vp_inv);
        assert!(ray.is_some());
        let ray = ray.unwrap();
        assert!(ray.direction.z < 0.0); // still points forward
        assert!(ray.direction.x < 0.0); // left side
        assert!(ray.direction.y > 0.0); // top
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

    #[test]
    fn ray_aabb_ray_inside_box() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let aabb = AABB::new(Vec3::splat(-2.0), Vec3::splat(2.0));
        let t = ray_aabb_intersect(&ray, &aabb);
        assert!(t.is_some());
        // When inside, should return distance to exit face
        assert!(t.unwrap() > 0.0);
    }

    #[test]
    fn ray_aabb_degenerate_flat_box() {
        // A box with zero thickness on Y
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let aabb = AABB::new(Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 0.0, 1.0));
        // This may or may not hit depending on the slab method, but should not panic
        let _result = ray_aabb_intersect(&ray, &aabb);
    }

    #[test]
    fn pick_entity_closest_wins() {
        let mut world = World::new();
        let e1 = world.spawn_entity((1u32,));
        let e2 = world.spawn_entity((2u32,));

        let ray = Ray::new(Vec3::new(0.0, 0.0, 10.0), Vec3::new(0.0, 0.0, -1.0));

        let entities = vec![
            (e1, AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0))),   // closer: t=9
            (e2, AABB::new(Vec3::new(-1.0, -1.0, 3.0), Vec3::new(1.0, 1.0, 5.0))), // farther: t=5
        ];

        let result = pick_entity(&ray, &entities);
        assert!(result.is_some());
        let (entity, _t) = result.unwrap();
        assert_eq!(entity, e2); // e2 is actually closer (z=3..5 vs z=-1..1)
    }

    #[test]
    fn pick_entity_no_hits() {
        let mut world = World::new();
        let e = world.spawn_entity((1u32,));
        let ray = Ray::new(Vec3::new(100.0, 100.0, 100.0), Vec3::X);
        let entities = vec![(e, AABB::new(Vec3::ZERO, Vec3::ONE))];
        assert!(pick_entity(&ray, &entities).is_none());
    }

    #[test]
    fn pick_entity_empty_list() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert!(pick_entity(&ray, &[]).is_none());
    }
}
