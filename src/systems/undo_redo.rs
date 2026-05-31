use bevy::prelude::*;

use crate::{
    components::PlacedBlock,
    components::PlacedRoot,
    resources::{GridConfig, GridEdit, OccupancyMap, PlacedBlockSnapshot, UndoStack},
    systems::placement::spawn_block,
};

pub struct UndoRedoPlugin;

impl Plugin for UndoRedoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_undo_redo);
    }
}

fn handle_undo_redo(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut undo: ResMut<UndoStack>,
    grid: Res<GridConfig>,
    mut occupancy: ResMut<OccupancyMap>,
    placed_root: Query<Entity, With<PlacedRoot>>,
    blocks: Query<Entity, With<PlacedBlock>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    let Ok(root) = placed_root.single() else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyZ) {
        if let Some(edit) = undo.pop_undo() {
            match &edit {
                GridEdit::Place { snapshot } => {
                    despawn_at(&mut commands, &mut occupancy, &blocks, snapshot.grid_key);
                }
                GridEdit::Delete { snapshot } => {
                    respawn(
                        &mut commands,
                        &grid,
                        &mut occupancy,
                        root,
                        &mut meshes,
                        &mut materials,
                        snapshot.clone(),
                    );
                }
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyY) {
        if let Some(edit) = undo.pop_redo() {
            match edit {
                GridEdit::Place { snapshot } => {
                    respawn(
                        &mut commands,
                        &grid,
                        &mut occupancy,
                        root,
                        &mut meshes,
                        &mut materials,
                        snapshot,
                    );
                }
                GridEdit::Delete { snapshot } => {
                    despawn_at(&mut commands, &mut occupancy, &blocks, snapshot.grid_key);
                }
            }
        }
    }
}

fn despawn_at(
    commands: &mut Commands,
    occupancy: &mut OccupancyMap,
    blocks: &Query<Entity, With<PlacedBlock>>,
    cell: IVec3,
) {
    if let Some(entity) = occupancy.remove(cell) {
        if blocks.get(entity).is_ok() {
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
        snapshot.grid_key,
        world,
        snapshot.rotation,
        grid.grid_size,
    );
    occupancy.insert(snapshot.grid_key, entity);
}
