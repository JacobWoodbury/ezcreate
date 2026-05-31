use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{
    components::{FacePaintDecal, GhostPreview, PlacedBlock, PlacedRoot},
    content::{section_anchor_offset_for_yaw, SectionBlueprintFile},
    resources::{
        ActiveSection, BindingId, GameMode, GridConfig, GridEdit, OccupancyMap,
        GameInput, KeyBindings, PlacementState, PlacedBlockSnapshot, UndoStack,
    },
    systems::{
        paint::{apply_blueprint_face_paint, remove_all_face_paint_on_block},
        raycast_util::{cursor_ray, raycast_placed_block},
    },
    ui::{GameplayAfterUi, UiInputCapture},
};

pub struct PlacementPlugin;

#[derive(SystemParam)]
struct PlaceDeleteQueries<'w, 's> {
    placed_root: Query<'w, 's, Entity, With<PlacedRoot>>,
    blocks: Query<'w, 's, (Entity, &'static PlacedBlock, &'static GlobalTransform)>,
    block_entities: Query<'w, 's, Entity, With<PlacedBlock>>,
    decals: Query<'w, 's, (Entity, &'static FacePaintDecal)>,
    spatial_query: SpatialQuery,
    windows: Query<'w, 's, &'static Window>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera3d>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, resolve_selected_section)
            .add_systems(Update, sync_ghost_preview)
            .add_systems(PostUpdate, update_placement_target.in_set(GameplayAfterUi))
            .add_systems(PostUpdate, handle_place_and_delete.in_set(GameplayAfterUi));
    }
}

/// When the selected library item changes to one with a sectionSpecPath, load the JSON.
fn resolve_selected_section(mut placement: ResMut<PlacementState>) {
    let Some(ref item) = placement.selected_item else {
        placement.active_section = None;
        return;
    };

    let spec_rel = match item.section_spec_path.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            placement.active_section = None;
            return;
        }
    };

    // Already resolved for this spec.
    if let Some(ref existing) = placement.active_section {
        let _ = existing;
        return;
    }

    let spec_path = item.manifest_dir.join(&spec_rel);
    let Ok(text) = std::fs::read_to_string(&spec_path) else {
        warn!("Section spec not found: {}", spec_path.display());
        placement.active_section = None;
        return;
    };
    let Ok(blueprint) = serde_json::from_str::<SectionBlueprintFile>(&text) else {
        warn!("Could not parse section spec: {}", spec_path.display());
        placement.active_section = None;
        return;
    };

    placement.active_section = Some(ActiveSection { blueprint });
}

fn section_pivot(section: &ActiveSection, yaw: f32) -> Vec3 {
    section_anchor_offset_for_yaw(&section.blueprint.pieces, yaw)
}

fn update_placement_target(
    capture: Res<UiInputCapture>,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: SpatialQuery,
    mut placement: ResMut<PlacementState>,
    occupancy: Res<OccupancyMap>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
) {
    placement.anchor_cell = None;
    placement.placement_valid = false;

    if *mode != GameMode::Place || placement.selected_item.is_none() {
        return;
    }

    if capture.block_game_pointer {
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

    // Allow hitting both ground and blocks so placement always has a surface.
    let Ok(dir) = Dir3::new(ray.direction.normalize()) else {
        return;
    };
    let Some(hit) = spatial_query.cast_ray(ray.origin, dir, grid.ray_length, true, &SpatialQueryFilter::default())
    else {
        return;
    };

    let hit_pos = ray.origin + *ray.direction * hit.distance;
    let world = hit_pos + hit.normal * (grid.grid_size * 0.5);
    let snapped = grid.snap_to_grid(world);
    let cell = grid.world_to_grid(snapped);

    placement.anchor_cell = Some(cell);
    placement.placement_valid = section_footprint_valid(&placement, &grid, &occupancy, cell);

    let _ = placed_blocks;
}

fn section_footprint_valid(
    placement: &PlacementState,
    grid: &GridConfig,
    occupancy: &OccupancyMap,
    anchor: IVec3,
) -> bool {
    if !grid.prevent_overlapping {
        return true;
    }
    if let Some(ref section) = placement.active_section {
        let yaw = placement.placement_euler.y;
        let rotation = Quat::from_rotation_y(yaw);
        let pivot = section_pivot(section, yaw);
        for piece in &section.blueprint.pieces {
            let offset = Vec3::new(
                piece.offset[0] as f32,
                piece.offset[1] as f32,
                piece.offset[2] as f32,
            );
            let delta = rotation * (offset - pivot);
            let cell = anchor
                + IVec3::new(
                    delta.x.round() as i32,
                    delta.y.round() as i32,
                    delta.z.round() as i32,
                );
            if occupancy.contains(cell) {
                return false;
            }
        }
        true
    } else {
        check_placement_valid(grid, occupancy, anchor)
    }
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

    // Always rebuild when state changes — despawn old first.
    for entity in &ghosts {
        commands.entity(entity).despawn();
    }
    placement.ghost_entity = None;

    let cell = placement.anchor_cell.unwrap();
    let anchor_world = grid.grid_to_world(cell);
    let yaw = placement.placement_euler.y;
    let rotation = Quat::from_rotation_y(yaw);

    let ghost_color = if placement.placement_valid {
        Color::srgba(0.35, 0.85, 0.45, 0.45)
    } else {
        Color::srgba(0.9, 0.25, 0.2, 0.45)
    };

    let ghost_material = materials.add(StandardMaterial {
        base_color: ghost_color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    if let Some(ref section) = placement.active_section.clone() {
        let pivot = section_pivot(section, yaw);
        let pivot_entity = commands
            .spawn((
                GhostPreview,
                Transform::from_translation(anchor_world).with_rotation(rotation),
                Visibility::default(),
            ))
            .id();

        for piece in &section.blueprint.pieces {
            let offset = Vec3::new(
                piece.offset[0] as f32,
                piece.offset[1] as f32,
                piece.offset[2] as f32,
            );
            let local_offset = (offset - pivot) * grid.grid_size;
            let mesh = meshes.add(Cuboid::new(grid.grid_size, grid.grid_size, grid.grid_size));
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(ghost_material.clone()),
                Transform::from_translation(local_offset),
                ChildOf(pivot_entity),
            ));
        }
        placement.ghost_entity = Some(pivot_entity);
    } else {
        // Single-block ghost.
        let mesh = meshes.add(Cuboid::new(grid.grid_size, grid.grid_size, grid.grid_size));
        let entity = commands
            .spawn((
                GhostPreview,
                Mesh3d(mesh),
                MeshMaterial3d(ghost_material),
                Transform::from_translation(anchor_world).with_rotation(rotation),
            ))
            .id();
        placement.ghost_entity = Some(entity);
    }
}

fn handle_place_and_delete(
    mut commands: Commands,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    input: GameInput,
    capture: Res<UiInputCapture>,
    mut placement: ResMut<PlacementState>,
    mut occupancy: ResMut<OccupancyMap>,
    mut undo: ResMut<UndoStack>,
    mut world: PlaceDeleteQueries,
) {
    if *mode == GameMode::Place && !capture.block_game_keyboard {
        if input.bindings.just_pressed(&input.keys, BindingId::RotateCcw) {
            placement.rotate_yaw_reverse();
        }
        if input.bindings.just_pressed(&input.keys, BindingId::RotateCw) {
            placement.rotate_yaw_forward();
        }
    }

    let Ok(root) = placed_root.single() else {
        return;
    };

    if capture.block_game_pointer {
        return;
    }

    if *mode == GameMode::Place
        && input.mouse.just_pressed(MouseButton::Left)
        && placement.placement_valid
    {
        if let Some(cell) = placement.anchor_cell {
            if let Some(ref section) = placement.active_section.clone() {
                place_section(
                    &mut commands,
                    &grid,
                    &mut occupancy,
                    &mut undo,
                    &mut meshes,
                    &mut materials,
                    root,
                    cell,
                    placement.placement_euler.y,
                    section,
                );
            } else if let Some(item) = placement.selected_item.clone() {
                if grid.prevent_overlapping && occupancy.contains(cell) {
                    return;
                }
                let world = grid.grid_to_world(cell);
                let rotation = Quat::from_rotation_y(placement.placement_euler.y);
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
    }

    let delete_pressed = input.mouse.just_pressed(MouseButton::Right)
        && KeyBindings::alt_pressed(&input.keys);

    if delete_pressed {
        let Some(cell) = raycast_cell_under_cursor(
            &grid, &windows, &cameras, &spatial_query, &block_entities,
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
                remove_all_face_paint_on_block(&mut commands, &decals, entity);
                commands.entity(entity).despawn();
                occupancy.remove(cell);
                undo.push(GridEdit::Delete { snapshot });
            }
        }
    }
}

fn place_section(
    commands: &mut Commands,
    grid: &GridConfig,
    occupancy: &mut OccupancyMap,
    undo: &mut UndoStack,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    root: Entity,
    anchor: IVec3,
    yaw: f32,
    section: &ActiveSection,
) {
    let rotation = Quat::from_rotation_y(yaw);
    let pivot = section_pivot(section, yaw);
    let anchor_world = grid.grid_to_world(anchor);
    let mut snapshots = Vec::new();

    for piece in &section.blueprint.pieces {
        let offset = Vec3::new(
            piece.offset[0] as f32,
            piece.offset[1] as f32,
            piece.offset[2] as f32,
        );
        let world_pos =
            anchor_world + rotation * ((offset - pivot) * grid.grid_size);
        let cell = grid.world_to_grid(world_pos);

        if grid.prevent_overlapping && occupancy.contains(cell) {
            continue;
        }

        let entity = spawn_block(
            commands,
            meshes,
            materials,
            root,
            &piece.item_id,
            &piece.scene_path,
            cell,
            world_pos,
            rotation,
            grid.grid_size,
        );
        for face_paint in &piece.face_paints {
            apply_blueprint_face_paint(
                commands,
                meshes,
                materials,
                entity,
                world_pos,
                grid.grid_size,
                rotation,
                face_paint,
            );
        }
        occupancy.insert(cell, entity);
        snapshots.push(PlacedBlockSnapshot {
            item_id: piece.item_id.clone(),
            grid_key: cell,
            rotation,
            scene_path: piece.scene_path.clone(),
        });
    }

    if !snapshots.is_empty() {
        undo.push(GridEdit::BulkPlace { snapshots });
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
            Visibility::default(),
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
    blocks: &Query<Entity, With<PlacedBlock>>,
) -> Option<IVec3> {
    let window = windows.single().ok()?;
    let (camera, cam_transform) = cameras.single().ok()?;
    let ray = cursor_ray(window, camera, cam_transform)?;
    let hit = raycast_placed_block(spatial_query, blocks, ray.origin, *ray.direction, grid.ray_length)?;
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
