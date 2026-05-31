use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    components::{GhostPreview, PlacedBlock, PlacedRoot},
    content::LibraryCatalog,
    resources::{
        GameMode, GridConfig, GridEdit, OccupancyMap, PlacementState, PlacedBlockSnapshot, UndoStack,
    },
};

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_placement_target,
                sync_ghost_preview,
                handle_place_and_delete,
            )
                .chain(),
        );
    }
}

fn update_placement_target(
    mut egui: EguiContexts,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: SpatialQuery,
    mut placement: ResMut<PlacementState>,
    occupancy: Res<OccupancyMap>,
) {
    placement.anchor_cell = None;
    placement.placement_valid = false;

    if *mode != GameMode::Place || placement.selected_item.is_none() {
        return;
    }

    if egui
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_area())
    {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        return;
    };

    let Ok(dir) = Dir3::new(ray.direction.normalize()) else {
        return;
    };

    let filter = SpatialQueryFilter::default();
    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        dir,
        grid.ray_length,
        true,
        &filter,
    ) else {
        return;
    };

    let half = grid.grid_size * 0.5;
    let hit_pos = ray.origin + *ray.direction * hit.distance;
    let world = hit_pos + hit.normal * half;
    let snapped = grid.snap_to_grid(world);
    let cell = grid.world_to_grid(snapped);

    placement.anchor_cell = Some(cell);
    placement.placement_valid = check_placement_valid(&grid, &occupancy, cell);
}

fn sync_ghost_preview(
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    mut placement: ResMut<PlacementState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ghosts: Query<Entity, With<GhostPreview>>,
) {
    let show = *mode == GameMode::Place
        && placement.selected_item.is_some()
        && placement.anchor_cell.is_some();

    if !show {
        for entity in &ghosts {
            commands.entity(entity).despawn();
        }
        placement.ghost_entity = None;
        return;
    }

    let cell = placement.anchor_cell.unwrap();
    let world = grid.grid_to_world(cell);
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        placement.placement_euler.x,
        placement.placement_euler.y,
        placement.placement_euler.z,
    );

    let color = if placement.placement_valid {
        Color::srgba(0.35, 0.85, 0.45, 0.45)
    } else {
        Color::srgba(0.9, 0.25, 0.2, 0.45)
    };

    if let Some(entity) = placement.ghost_entity {
        if let Ok(mut entity_cmds) = commands.get_entity(entity) {
            entity_cmds.insert((
                Transform::from_translation(world).with_rotation(rotation),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
            ));
        }
    } else {
        let mesh = meshes.add(Cuboid::new(grid.grid_size, grid.grid_size, grid.grid_size));
        let material = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let entity = commands
            .spawn((
                GhostPreview,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(world).with_rotation(rotation),
            ))
            .id();
        placement.ghost_entity = Some(entity);
    }
}

fn handle_place_and_delete(
    mut commands: Commands,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    _catalog: Res<LibraryCatalog>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<PlacementState>,
    mut occupancy: ResMut<OccupancyMap>,
    mut undo: ResMut<UndoStack>,
    placed_root: Query<Entity, With<PlacedRoot>>,
    blocks: Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *mode == GameMode::Place {
        if keys.just_pressed(KeyCode::KeyQ) {
            placement.rotate_yaw_reverse();
        }
        if keys.just_pressed(KeyCode::KeyE) {
            placement.rotate_yaw_forward();
        }
    }

    let Ok(root) = placed_root.single() else {
        return;
    };

    if *mode == GameMode::Place
        && mouse.just_pressed(MouseButton::Left)
        && placement.placement_valid
    {
        if let (Some(cell), Some(item)) = (placement.anchor_cell, placement.selected_item.clone()) {
            if grid.prevent_overlapping && occupancy.contains(cell) {
                return;
            }

            let world = grid.grid_to_world(cell);
            let rotation = Quat::from_euler(
                EulerRot::XYZ,
                placement.placement_euler.x,
                placement.placement_euler.y,
                placement.placement_euler.z,
            );

            let entity = spawn_block(
                &mut commands,
                &mut meshes,
                &mut materials,
                root,
                &item.item_id,
                &item.scene_path,
                cell,
                world,
                rotation,
                grid.grid_size,
            );
            occupancy.insert(cell, entity);

            undo.push(GridEdit::Place {
                snapshot: PlacedBlockSnapshot {
                    item_id: item.item_id,
                    grid_key: cell,
                    rotation,
                    scene_path: item.scene_path,
                },
            });
        }
    }

    let delete_pressed =
        mouse.just_pressed(MouseButton::Right) && keys.pressed(KeyCode::AltLeft);

    if delete_pressed {
        let Some(cell) = raycast_cell_under_cursor(
            &grid,
            &windows,
            &cameras,
            &spatial_query,
        ) else {
            return;
        };

        if let Some(entity) = occupancy.get(cell) {
            if let Ok((_, block, transform)) = blocks.get(entity) {
                let snapshot = PlacedBlockSnapshot {
                    item_id: block.item_id.clone(),
                    grid_key: block.grid_key,
                    rotation: transform.rotation(),
                    scene_path: block.scene_path.clone(),
                };
                commands.entity(entity).despawn();
                occupancy.remove(cell);
                undo.push(GridEdit::Delete { snapshot });
            }
        }
    }
}

pub fn spawn_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    item_id: &str,
    scene_path: &str,
    grid_key: IVec3,
    world: Vec3,
    rotation: Quat,
    grid_size: f32,
) -> Entity {
    let mesh = meshes.add(Cuboid::new(grid_size, grid_size, grid_size));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.65, 0.75),
        ..default()
    });

    commands
        .spawn((
            PlacedBlock {
                item_id: item_id.to_string(),
                grid_key,
                scene_path: scene_path.to_string(),
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(world).with_rotation(rotation),
            RigidBody::Static,
            Collider::cuboid(grid_size * 0.5, grid_size * 0.5, grid_size * 0.5),
        ))
        .insert(ChildOf(parent))
        .id()
}

fn raycast_cell_under_cursor(
    grid: &GridConfig,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: &SpatialQuery,
) -> Option<IVec3> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, cam_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(cam_transform, cursor).ok()?;
    let dir = Dir3::new(ray.direction.normalize()).ok()?;
    let hit = spatial_query.cast_ray(
        ray.origin,
        dir,
        grid.ray_length,
        true,
        &SpatialQueryFilter::default(),
    )?;
    let hit_pos = ray.origin + *ray.direction * hit.distance;
    let world = hit_pos + hit.normal * (grid.grid_size * 0.5);
    Some(grid.world_to_grid(grid.snap_to_grid(world)))
}

pub fn check_placement_valid(
    grid: &GridConfig,
    occupancy: &OccupancyMap,
    cell: IVec3,
) -> bool {
    !grid.prevent_overlapping || !occupancy.contains(cell)
}
