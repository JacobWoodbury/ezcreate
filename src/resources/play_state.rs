use std::collections::HashMap;

use bevy::prelude::*;

use crate::systems::camera_orbit::OrbitCameraState;

/// Identifies a character archetype in the registry (extensible for multiple types).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayCharacterId(pub String);

impl PlayCharacterId {
    pub fn default_character() -> Self {
        Self("default".into())
    }
}

/// Static definition for spawning a character (mesh/collider/physics tuning).
#[derive(Clone, Debug)]
pub struct PlayCharacterDef {
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub move_speed: f32,
    pub jump_speed: f32,
    pub linear_damping: f32,
    pub color: Color,
}

impl Default for PlayCharacterDef {
    fn default() -> Self {
        Self {
            capsule_radius: 0.35,
            capsule_half_height: 0.55,
            move_speed: 6.0,
            jump_speed: 7.0,
            linear_damping: 8.0,
            color: Color::srgb(0.85, 0.55, 0.35),
        }
    }
}

#[derive(Resource)]
pub struct PlayCharacterRegistry {
    pub defs: HashMap<PlayCharacterId, PlayCharacterDef>,
}

impl Default for PlayCharacterRegistry {
    fn default() -> Self {
        let mut defs = HashMap::new();
        defs.insert(PlayCharacterId::default_character(), PlayCharacterDef::default());
        Self { defs }
    }
}

impl PlayCharacterRegistry {
    pub fn get(&self, id: &PlayCharacterId) -> Option<&PlayCharacterDef> {
        self.defs.get(id)
    }
}

#[derive(Clone, Debug)]
pub struct SpawnedCharacter {
    pub id: PlayCharacterId,
    pub entity: Entity,
    pub grid_key: IVec3,
}

/// v1: one active character; later swap for Vec + controlled id.
#[derive(Resource, Default)]
pub struct PlayWorldState {
    pub active_character: Option<SpawnedCharacter>,
    /// Which character receives input during play session (v1: same as active).
    pub controlled: Option<PlayCharacterId>,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaySession {
    #[default]
    Inactive,
    Active,
}

impl PlaySession {
    pub fn is_active(self) -> bool {
        matches!(self, PlaySession::Active)
    }

    pub fn is_inactive(self) -> bool {
        matches!(self, PlaySession::Inactive)
    }
}

/// Saved orbit camera state when entering play session.
#[derive(Clone, Debug)]
pub struct PlaySessionSnapshot {
    pub rig_translation: Vec3,
    pub rig_rotation: Quat,
    pub camera_state: OrbitCameraState,
    pub camera_local: Vec3,
}

#[derive(Resource, Default)]
pub struct PlaySessionStorage {
    pub snapshot: Option<PlaySessionSnapshot>,
}

/// UI / HUD pending actions processed after egui draw.
#[derive(Resource, Default)]
pub struct PlayUiActions {
    pub remove_character: bool,
    pub start_session: bool,
}
