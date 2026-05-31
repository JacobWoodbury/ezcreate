use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    components::{GhostPreview, Ground},
    resources::{
        GameInput, GameMode, GridConfig, PlaceableRegistry, PlacementState, PlayCharacterRegistry,
        PlayWorldState,
    },
    systems::{
        play::spawn_placeable,
        raycast_util::{cursor_ray, raycast_ground, snap_axis_normal},
    },
    ui::{GameplayAfterUi, UiInputCapture},
};

pub struct PlaceablesPlugin;

impl Plugin for PlaceablesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                update_placeable_target,
                sync_placeable_ghost.after(update_placeable_target),
                handle_placeable_placement.after(sync_placeable_ghost),
            )
                .in_set(GameplayAfterUi),
        );
    }
}

fn update_placeable_target(
    capture: Res<UiInputCapture>,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: SpatialQuery,
    ground: Query<Entity, With<Ground>>,
    mut placement: ResMut<PlacementState>,
) {
    if *mode != GameMode::Play || placement.selected_placeable.is_none() {
        return;
    }

    if capture.block_game_pointer {
        placement.anchor_cell = None;
        placement.ghost_pivot_world = None;
        placement.placement_allowed = false;
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(ray) = cursor_ray(window, camera, cam_transform) else {
        return;
    };

    let Some(hit) = raycast_ground(
        &spatial_query,
        &ground,
        ray.origin,
        *ray.direction,
        grid.ray_length,
    ) else {
        placement.anchor_cell = None;
        placement.ghost_pivot_world = None;
        placement.placement_allowed = false;
        return;
    };

    let hit_pos = ray.origin + *ray.direction * hit.distance;
    let face_normal = snap_axis_normal(hit.normal);
    let snapped = grid.snap_to_grid(hit_pos + face_normal * grid.grid_size * 0.5);
    let grid_key = grid.world_to_grid(snapped);

    placement.anchor_cell = Some(grid_key);
    placement.ghost_pivot_world = None;
    placement.placement_allowed = true;
}

fn sync_placeable_ghost(
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    registry: Res<PlaceableRegistry>,
    character_registry: Res<PlayCharacterRegistry>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ghosts: Query<Entity, With<GhostPreview>>,
    mut placement: ResMut<PlacementState>,
) {
    let show = *mode == GameMode::Play
        && placement.selected_placeable.is_some()
        && placement.anchor_cell.is_some();

    if !show {
        if placement.ghost_entity.is_some() {
            for entity in &ghosts {
                commands.entity(entity).despawn();
            }
            placement.ghost_entity = None;
        }
        return;
    }

    let Some(placeable_id) = placement.selected_placeable.as_ref() else {
        return;
    };
    let Some(def) = registry.get(placeable_id) else {
        return;
    };

    let cell = placement.anchor_cell.unwrap();
    let world_pos = grid.grid_to_world(cell);

    // Rebuild ghost when target moves.
    for entity in &ghosts {
        commands.entity(entity).despawn();
    }
    placement.ghost_entity = None;

    let ghost_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.55, 0.35, 0.45),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let entity = match &def.kind {
        crate::resources::PlaceableKind::PlayCharacter(id) => {
            let Some(char_def) = character_registry.get(id) else {
                return;
            };
            let spawn_y = char_def.capsule_radius + char_def.capsule_half_height;
            let mesh = meshes.add(Capsule3d::new(
                char_def.capsule_radius,
                char_def.capsule_half_height * 2.0,
            ));
            commands
                .spawn((
                    GhostPreview,
                    Mesh3d(mesh),
                    MeshMaterial3d(ghost_material),
                    Transform::from_translation(world_pos + Vec3::Y * spawn_y),
                ))
                .id()
        }
    };

    placement.ghost_entity = Some(entity);
}

fn handle_placeable_placement(
    mut commands: Commands,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    input: GameInput,
    capture: Res<UiInputCapture>,
    registry: Res<PlaceableRegistry>,
    character_registry: Res<PlayCharacterRegistry>,
    placement: Res<PlacementState>,
    mut world: ResMut<PlayWorldState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *mode != GameMode::Play {
        return;
    }

    if capture.block_game_pointer {
        return;
    }

    if !input.mouse.just_pressed(MouseButton::Left) || !placement.placement_allowed {
        return;
    }

    let Some(placeable_id) = placement.selected_placeable.clone() else {
        return;
    };
    let Some(cell) = placement.anchor_cell else {
        return;
    };
    let Some(def) = registry.get(&placeable_id) else {
        return;
    };

    if let Err(err) = spawn_placeable(
        &mut commands,
        &mut meshes,
        &mut materials,
        &grid,
        &character_registry,
        &mut world,
        def,
        cell,
    ) {
        warn!("Placeable spawn failed: {err}");
    }
}
