use bevy::prelude::*;

use crate::{
    components::{FacePaintDecal, PlacedBlock, PlacedRoot},
    resources::{BindingId, GridConfig, GridEdit, KeyBindings, OccupancyMap, PlacedBlockSnapshot, UndoStack},
    systems::{
        paint::{apply_face_paint_snapshot, remove_all_face_paint_on_block, remove_face_paint_on_block},
        placement::spawn_block,
    },
    ui::{GameplayAfterUi, UiInputCapture},
};

pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, handle_undo_redo.in_set(GameplayAfterUi));
    }
}

fn handle_undo_redo(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    capture: Res<UiInputCapture>,
    mut undo: ResMut<UndoStack>,
    grid: Res<GridConfig>,
    mut occupancy: ResMut<OccupancyMap>,
    placed_root: Query<Entity, With<PlacedRoot>>,
    blocks: Query<Entity, With<PlacedBlock>>,
    block_globals: Query<&GlobalTransform, With<PlacedBlock>>,
    decals: Query<(Entity, &FacePaintDecal)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if capture.block_game_keyboard {
        return;
    }
    if !KeyBindings::ctrl_pressed(&keys) {        return;
    }

    let Ok(root) = placed_root.single() else {
        return;
    };

    if bindings.just_pressed(&keys, BindingId::Undo) {
        if let Some(edit) = undo.pop_undo() {
            apply_inverse(
                &mut commands,
                &grid,
                &mut occupancy,
                &blocks,
                &decals,
                root,
                &mut meshes,
                &mut materials,
                edit,
            );
        }
    }

    if bindings.just_pressed(&keys, BindingId::Redo) {
        if let Some(edit) = undo.pop_redo() {
            apply_forward(
                &mut commands,
                &grid,
                &mut occupancy,
                &blocks,
                &block_globals,
                &decals,
                root,
                &mut meshes,
                &mut materials,
                edit,
            );
        }
    }
}

fn apply_inverse(
    commands: &mut Commands,
    grid: &GridConfig,
    occupancy: &mut OccupancyMap,
    blocks: &Query<Entity, With<PlacedBlock>>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    root: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    edit: GridEdit,
) {
    match edit {
        GridEdit::Place { snapshot } => {
            despawn_at(commands, occupancy, blocks, decals, snapshot.grid_key);
        }
        GridEdit::Delete { snapshot } => {
            respawn(commands, grid, occupancy, root, meshes, materials, snapshot);
        }
        GridEdit::BulkPlace { snapshots } => {
            for snapshot in snapshots {
                despawn_at(commands, occupancy, blocks, decals, snapshot.grid_key);
            }
        }
        GridEdit::BulkDelete { snapshots } => {
            for snapshot in snapshots {
                respawn(commands, grid, occupancy, root, meshes, materials, snapshot);
            }
        }
        GridEdit::FacePaint { snapshot } => {
            remove_face_paint_on_block(commands, decals, snapshot.parent_block, snapshot.face_normal);
        }
    }
}

fn apply_forward(
    commands: &mut Commands,
    grid: &GridConfig,
    occupancy: &mut OccupancyMap,
    blocks: &Query<Entity, With<PlacedBlock>>,
    block_globals: &Query<&GlobalTransform, With<PlacedBlock>>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    root: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    edit: GridEdit,
) {
    match edit {
        GridEdit::Place { snapshot } => {
            respawn(commands, grid, occupancy, root, meshes, materials, snapshot);
        }
        GridEdit::Delete { snapshot } => {
            despawn_at(commands, occupancy, blocks, decals, snapshot.grid_key);
        }
        GridEdit::BulkPlace { snapshots } => {
            for snapshot in snapshots {
                respawn(commands, grid, occupancy, root, meshes, materials, snapshot);
            }
        }
        GridEdit::BulkDelete { snapshots } => {
            for snapshot in snapshots {
                despawn_at(commands, occupancy, blocks, decals, snapshot.grid_key);
            }
        }
        GridEdit::FacePaint { snapshot } => {
            apply_face_paint_snapshot(commands, meshes, materials, decals, block_globals, &snapshot);
        }
    }
}

fn despawn_at(
    commands: &mut Commands,
    occupancy: &mut OccupancyMap,
    blocks: &Query<Entity, With<PlacedBlock>>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    cell: IVec3,
) {
    if let Some(entity) = occupancy.remove(cell) {
        if blocks.get(entity).is_ok() {
            remove_all_face_paint_on_block(commands, decals, entity);
            commands.entity(entity).despawn();
        }
    }
}

fn respawn(
    commands: &mut Commands,
    grid: &GridConfig,
    occupancy: &mut OccupancyMap,
    root: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    snapshot: PlacedBlockSnapshot,
) {
    if occupancy.contains(snapshot.grid_key) {
        return;
    }
    let world = grid.grid_to_world(snapshot.grid_key);
    let entity = spawn_block(
        commands,
        meshes,
        materials,
        root,
        &snapshot.item_id,
        &snapshot.scene_path,
        snapshot.grid_key,
        world,
        snapshot.rotation,
        grid.grid_size,
    );
    occupancy.insert(snapshot.grid_key, entity);
}
