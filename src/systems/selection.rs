use bevy::prelude::*;

use crate::components::{PlacedBlock, SelectionOutline};
use crate::resources::{
    BindingId, GameMode, GridConfig, GridEdit, KeyBindings, OccupancyMap, PlacedBlockSnapshot,
    SelectionState, UndoStack,
};
use crate::systems::{
    paint::remove_all_face_paint_on_block,
    raycast_util::{cursor_ray, raycast_placed_block},
};
use crate::ui::{GameplayAfterUi, UiInputCapture};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionOutlineAssets>()
            .add_systems(Update, sync_selection_outlines)
            .add_systems(PostUpdate, handle_selection_input.in_set(GameplayAfterUi));
    }
}

#[derive(Resource)]
struct SelectionOutlineAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl FromWorld for SelectionOutlineAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::new(1.02, 1.02, 1.02))
        };
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.35, 0.75, 1.0, 0.55),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        Self { mesh, material }
    }
}

fn handle_selection_input(
    capture: Res<UiInputCapture>,
    mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut selection: ResMut<SelectionState>,
    grid: Res<GridConfig>,
    mut commands: Commands,
    mut occupancy: ResMut<OccupancyMap>,
    mut undo: ResMut<UndoStack>,
    blocks: Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    block_entities: Query<Entity, With<PlacedBlock>>,
    decals: Query<(Entity, &crate::components::FacePaintDecal)>,
    spatial_query: avian3d::prelude::SpatialQuery,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if *mode != GameMode::Select {
        selection.marquee_dragging = false;
        selection.marquee_start = None;
        selection.marquee_current = None;
        return;
    }

    if capture.block_game_pointer && !selection.marquee_dragging {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        selection.marquee_dragging = true;
        selection.marquee_start = Some(cursor);
        selection.marquee_current = Some(cursor);
    }

    if selection.marquee_dragging {
        selection.marquee_current = Some(cursor);
    }

    if mouse.just_released(MouseButton::Left) && selection.marquee_dragging {
        selection.marquee_dragging = false;
        let shift = KeyBindings::shift_pressed(&keys);

        if selection.marquee_drag_distance() > 8.0 {
            if let Some(rect) = selection.marquee_rect() {
                let picked = blocks_in_screen_rect(&cameras, &blocks, rect);
                if shift {
                    for entity in picked {
                        selection.selected.insert(entity);
                    }
                } else {
                    selection.selected = picked.into_iter().collect();
                }
            }
        } else if let Some(entity) = pick_block_at_cursor(
            &grid,
            &windows,
            &cameras,
            &spatial_query,
            &block_entities,
        ) {
            if shift {
                selection.toggle(entity);
            } else {
                selection.set_single(entity);
            }
        } else if !shift {
            selection.clear();
        }

        selection.marquee_start = None;
        selection.marquee_current = None;
    }

    if !capture.block_game_keyboard {
        if bindings.delete_pressed(&keys) {
            delete_selection(
                &mut commands,
                &mut occupancy,
                &mut undo,
                &mut selection,
                &blocks,
                &decals,
            );
        }

        if bindings.just_pressed(&keys, BindingId::RotateCcw) {
            rotate_selection_y(
                &mut commands,
                &mut occupancy,
                &grid,
                &selection,
                &blocks,
                -std::f32::consts::FRAC_PI_2,
            );
        }
        if bindings.just_pressed(&keys, BindingId::RotateCw) {
            rotate_selection_y(
                &mut commands,
                &mut occupancy,
                &grid,
                &selection,
                &blocks,
                std::f32::consts::FRAC_PI_2,
            );
        }
    }
}

fn pick_block_at_cursor(
    grid: &GridConfig,
    windows: &Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: &avian3d::prelude::SpatialQuery,
    blocks: &Query<Entity, With<PlacedBlock>>,
) -> Option<Entity> {
    let window = windows.single().ok()?;
    let (camera, cam_transform) = cameras.single().ok()?;
    let ray = cursor_ray(window, camera, cam_transform)?;
    let hit = raycast_placed_block(
        spatial_query,
        blocks,
        ray.origin,
        *ray.direction,
        grid.ray_length,
    )?;
    Some(hit.entity)
}

fn blocks_in_screen_rect(
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    blocks: &Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    rect: Rect,
) -> Vec<Entity> {
    let Ok((camera, cam_transform)) = cameras.single() else {
        return vec![];
    };

    blocks
        .iter()
        .filter(|(_, _, transform)| {
            block_center_in_marquee(camera, cam_transform, transform, rect)
        })
        .map(|(entity, _, _)| entity)
        .collect()
}

/// Block is selected when its projected center lies inside the marquee (window logical pixels).
fn block_center_in_marquee(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    transform: &GlobalTransform,
    marquee: Rect,
) -> bool {
    let Some(screen) = world_to_window_point(camera, cam_transform, transform.translation()) else {
        return false;
    };
    marquee.contains(screen)
}

fn world_to_window_point(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    world: Vec3,
) -> Option<Vec2> {
    camera.world_to_viewport(cam_transform, world).ok()
}

fn delete_selection(
    commands: &mut Commands,
    occupancy: &mut OccupancyMap,
    undo: &mut UndoStack,
    selection: &mut SelectionState,
    blocks: &Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    decals: &Query<(Entity, &crate::components::FacePaintDecal)>,
) {
    let mut snapshots = Vec::new();
    for &entity in &selection.selected {
        if let Ok((_, block, transform)) = blocks.get(entity) {
            snapshots.push(PlacedBlockSnapshot {
                item_id: block.item_id.clone(),
                grid_key: block.grid_key,
                rotation: transform.rotation(),
                scene_path: block.scene_path.clone(),
            });
            remove_all_face_paint_on_block(commands, decals, entity);
            occupancy.remove(block.grid_key);
            commands.entity(entity).despawn();
        }
    }
    if !snapshots.is_empty() {
        undo.push(GridEdit::BulkDelete { snapshots });
        selection.clear();
    }
}

fn rotate_selection_y(
    commands: &mut Commands,
    occupancy: &mut OccupancyMap,
    grid: &GridConfig,
    selection: &SelectionState,
    blocks: &Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    yaw_delta: f32,
) {
    let entities: Vec<Entity> = selection.selected.iter().copied().collect();
    if entities.is_empty() {
        return;
    }

    let positions: Vec<Vec3> = entities
        .iter()
        .filter_map(|e| blocks.get(*e).ok().map(|(_, _, t)| t.translation()))
        .collect();
    if positions.is_empty() {
        return;
    }

    let centroid = positions.iter().sum::<Vec3>() / positions.len() as f32;
    let rotation = Quat::from_rotation_y(yaw_delta);

    for &entity in &entities {
        let Ok((_, block, transform)) = blocks.get(entity) else {
            continue;
        };
        occupancy.remove(block.grid_key);

        let offset = transform.translation() - centroid;
        let new_pos = centroid + rotation * offset;
        let new_rot = rotation * transform.rotation();
        let new_cell = grid.world_to_grid(new_pos);

        commands.entity(entity).insert((
            Transform::from_translation(new_pos).with_rotation(new_rot),
            PlacedBlock {
                item_id: block.item_id.clone(),
                grid_key: new_cell,
                scene_path: block.scene_path.clone(),
            },
        ));
        occupancy.insert(new_cell, entity);
    }
}

fn sync_selection_outlines(
    mode: Res<GameMode>,
    selection: Res<SelectionState>,
    assets: Res<SelectionOutlineAssets>,
    mut commands: Commands,
    block_globals: Query<&GlobalTransform, With<PlacedBlock>>,
    mut outlines: Query<(Entity, &SelectionOutline, &mut Transform)>,
) {
    let wanted: std::collections::HashSet<Entity> = if *mode == GameMode::Select {
        selection.selected.clone()
    } else {
        std::collections::HashSet::new()
    };

    for (outline_entity, outline, _) in outlines.iter() {
        if !wanted.contains(&outline.block) {
            commands.entity(outline_entity).despawn();
        }
    }

    for &block in &wanted {
        let Ok(block_global) = block_globals.get(block) else {
            continue;
        };

        let world_transform = Transform {
            translation: block_global.translation(),
            rotation: block_global.rotation(),
            scale: Vec3::ONE,
        };

        if let Some((_, _, mut transform)) = outlines
            .iter_mut()
            .find(|(_, o, _)| o.block == block)
        {
            *transform = world_transform;
            continue;
        }

        commands.spawn((
            SelectionOutline { block },
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.material.clone()),
            world_transform,
            Visibility::default(),
        ));
    }
}
