use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    components::PlayCharacter,
    resources::{
        GridConfig, PlaceableDef, PlaceableKind, PlayCharacterId, PlayCharacterRegistry,
        PlayWorldState, SpawnedCharacter,
    },
};

/// Spawn a play character at a grid cell (replaces any existing active character).
pub fn spawn_character_at_cell(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid: &GridConfig,
    registry: &PlayCharacterRegistry,
    world: &mut PlayWorldState,
    character_id: &PlayCharacterId,
    grid_key: IVec3,
) -> Result<Entity, String> {
    let def = registry
        .get(character_id)
        .ok_or_else(|| format!("Unknown character id: {}", character_id.0))?;

    let spawn_y = def.capsule_radius + def.capsule_half_height;
    let world_pos = grid.grid_to_world(grid_key) + Vec3::Y * spawn_y;

    remove_active_character(commands, world);

    let mesh = meshes.add(Capsule3d::new(def.capsule_radius, def.capsule_half_height * 2.0));
    let material = materials.add(StandardMaterial {
        base_color: def.color,
        ..default()
    });

    let entity = commands
        .spawn((
            PlayCharacter {
                id: character_id.clone(),
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(world_pos),
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::capsule(def.capsule_radius, def.capsule_half_height),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(def.linear_damping),
            Friction::new(0.8),
            Restitution::ZERO,
            GravityScale(1.0),
        ))
        .id();

    world.active_character = Some(SpawnedCharacter {
        id: character_id.clone(),
        entity,
        grid_key,
    });
    world.controlled = Some(character_id.clone());

    Ok(entity)
}

pub fn spawn_placeable(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    grid: &GridConfig,
    character_registry: &PlayCharacterRegistry,
    world: &mut PlayWorldState,
    def: &PlaceableDef,
    grid_key: IVec3,
) -> Result<Entity, String> {
    match &def.kind {
        PlaceableKind::PlayCharacter(id) => {
            spawn_character_at_cell(commands, meshes, materials, grid, character_registry, world, id, grid_key)
        }
    }
}

pub fn remove_active_character(commands: &mut Commands, world: &mut PlayWorldState) {
    if let Some(spawned) = world.active_character.take() {
        commands.entity(spawned.entity).despawn();
    }
    world.controlled = None;
}

pub fn apply_play_ui_remove_action(
    mut commands: Commands,
    mut actions: ResMut<crate::resources::PlayUiActions>,
    mut world: ResMut<PlayWorldState>,
) {
    if actions.remove_character {
        actions.remove_character = false;
        remove_active_character(&mut commands, &mut world);
    }
}
