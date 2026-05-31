use avian3d::prelude::*;
use bevy::prelude::*;

use crate::components::PlacedBlock;

/// Closest physics hit on a placed block (skips ground and other colliders).
pub fn raycast_placed_block(
    spatial_query: &SpatialQuery,
    blocks: &Query<Entity, With<PlacedBlock>>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RayHitData> {
    let dir = Dir3::new(direction.normalize()).ok()?;
    spatial_query.cast_ray_predicate(
        origin,
        dir,
        max_distance,
        true,
        &SpatialQueryFilter::default(),
        &|entity| blocks.get(entity).is_ok(),
    )
}

pub fn cursor_ray(
    window: &Window,
    camera: &Camera,
    cam_transform: &GlobalTransform,
) -> Option<Ray3d> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world(cam_transform, cursor).ok()
}

/// Snap a physics hit normal to the nearest axis (block faces are axis-aligned).
pub fn snap_axis_normal(normal: Vec3) -> Vec3 {
    let n = normal.normalize_or_zero();
    let abs = n.abs();
    if abs.x >= abs.y && abs.x >= abs.z {
        Vec3::X * n.x.signum()
    } else if abs.y >= abs.z {
        Vec3::Y * n.y.signum()
    } else {
        Vec3::Z * n.z.signum()
    }
}

/// World-space center of the block face struck by a ray.
pub fn block_face_center(block_center: Vec3, grid_size: f32, face_normal: Vec3) -> Vec3 {
    block_center + face_normal * (grid_size * 0.5)
}

/// Hover / paint tint: brush color blended with blue so the targeted face reads clearly.
pub fn paint_preview_color(base: Color) -> Color {
    let mut s = base.to_srgba();
    const BLUE_WEIGHT: f32 = 0.4;
    s.red = s.red * (1.0 - BLUE_WEIGHT) + 0.15 * BLUE_WEIGHT;
    s.green = s.green * (1.0 - BLUE_WEIGHT) + 0.45 * BLUE_WEIGHT;
    s.blue = s.blue * (1.0 - BLUE_WEIGHT) + 1.0 * BLUE_WEIGHT;
    s.alpha = (s.alpha * 0.5).clamp(0.35, 0.75);
    Color::from(s)
}
