use avian3d::prelude::*;
use bevy::prelude::*;

use crate::components::{Ground, PlacedBlock};
use crate::resources::GridConfig;

/// Closest physics hit on the ground plane (skips blocks and other colliders).
pub fn raycast_ground(
    spatial_query: &SpatialQuery,
    ground: &Query<Entity, With<Ground>>,
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
        &|entity| ground.get(entity).is_ok(),
    )
}

/// Result of a placement surface raycast.
pub struct PlacementSurface {
    pub anchor_cell: IVec3,
    /// Block under the cursor when the ray hit a placed block.
    pub hit_block_center: Option<Vec3>,
    pub hit_face_normal: Option<Vec3>,
}

/// Resolve where placement should target: prefer block faces, then ground.
pub fn resolve_placement_surface(
    grid: &GridConfig,
    spatial_query: &SpatialQuery,
    blocks: &Query<Entity, With<PlacedBlock>>,
    ground: &Query<Entity, With<Ground>>,
    block_transforms: &Query<&GlobalTransform, With<PlacedBlock>>,
    ray: &Ray3d,
    max_distance: f32,
) -> Option<PlacementSurface> {
    let origin = ray.origin;
    let direction = *ray.direction;

    if let Some(hit) = raycast_placed_block(spatial_query, blocks, origin, direction, max_distance) {
        let hit_pos = origin + direction * hit.distance;
        let anchor_cell = placement_anchor_cell(grid, &hit, hit_pos, block_transforms);
        let face_normal = snap_axis_normal(hit.normal);
        let hit_block_center = block_transforms
            .get(hit.entity)
            .ok()
            .map(|tf| tf.translation());
        return Some(PlacementSurface {
            anchor_cell,
            hit_block_center,
            hit_face_normal: Some(face_normal),
        });
    }

    let hit = raycast_ground(spatial_query, ground, origin, direction, max_distance)?;
    let hit_pos = origin + direction * hit.distance;
    Some(PlacementSurface {
        anchor_cell: placement_anchor_cell(grid, &hit, hit_pos, block_transforms),
        hit_block_center: None,
        hit_face_normal: None,
    })
}

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

/// Grid cell to place into when clicking a surface hit by a placement ray.
pub fn placement_anchor_cell(
    grid: &GridConfig,
    hit: &RayHitData,
    hit_pos: Vec3,
    block_transforms: &Query<&GlobalTransform, With<PlacedBlock>>,
) -> IVec3 {
    if let Ok(block_tf) = block_transforms.get(hit.entity) {
        let face_normal = snap_axis_normal(hit.normal);
        let anchor_world = block_tf.translation() + face_normal * grid.grid_size;
        return grid.world_to_grid(anchor_world);
    }

    let face_normal = snap_axis_normal(hit.normal);
    let anchor_world = grid.snap_to_grid(hit_pos + face_normal * grid.grid_size * 0.5);
    grid.world_to_grid(anchor_world)
}

/// World-space transform for a face overlay (preview or painted decals).
pub fn face_transform_world(face_center: Vec3, face_normal: Vec3, bias: f32) -> Transform {
    let normal = face_normal.normalize();
    Transform::from_translation(face_center + normal * bias)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, normal))
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
