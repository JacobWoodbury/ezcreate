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
