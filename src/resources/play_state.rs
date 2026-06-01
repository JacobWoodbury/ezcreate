use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    character_storage::{delete_preset_from_disk, load_all_presets, save_preset_to_disk},
    PlaceableDef, PlaceableId, PlaceableKind,
};
use crate::systems::camera_orbit::OrbitCameraState;

/// Identifies a character archetype in the registry (extensible for multiple types).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayCharacterId(pub String);

impl PlayCharacterId {
    pub fn default_character() -> Self {
        Self("default".into())
    }
}

/// Runtime spawn/movement stats for a character preset.
#[derive(Clone, Debug)]
pub struct PlayCharacterDef {
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub move_speed: f32,
    pub jump_speed: f32,
    pub linear_damping: f32,
    pub color: Color,
}

/// Named, saveable character preset (persisted as JSON).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayCharacterPreset {
    pub id: String,
    pub name: String,
    #[serde(rename = "moveSpeed")]
    pub move_speed: f32,
    #[serde(rename = "jumpSpeed")]
    pub jump_speed: f32,
    #[serde(rename = "linearDamping")]
    pub linear_damping: f32,
    #[serde(rename = "capsuleRadius")]
    pub capsule_radius: f32,
    #[serde(rename = "capsuleHalfHeight")]
    pub capsule_half_height: f32,
    #[serde(rename = "colorRgb")]
    pub color_rgb: [f32; 3],
}

impl PlayCharacterPreset {
    pub fn builtin_default() -> Self {
        Self {
            id: PlayCharacterId::default_character().0,
            name: "Default".into(),
            move_speed: 6.0,
            jump_speed: 7.0,
            linear_damping: 8.0,
            capsule_radius: 0.35,
            capsule_half_height: 0.55,
            color_rgb: [0.85, 0.55, 0.35],
        }
    }

    pub fn is_builtin(&self) -> bool {
        self.id == PlayCharacterId::default_character().0
    }

    pub fn to_def(&self) -> PlayCharacterDef {
        PlayCharacterDef {
            capsule_radius: self.capsule_radius,
            capsule_half_height: self.capsule_half_height,
            move_speed: self.move_speed,
            jump_speed: self.jump_speed,
            linear_damping: self.linear_damping,
            color: Color::srgb(self.color_rgb[0], self.color_rgb[1], self.color_rgb[2]),
        }
    }

    pub fn new_unique(name: &str) -> Self {
        let slug = sanitize_id(name);
        let id = if slug.is_empty() {
            format!(
                "char_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            )
        } else {
            format!("char_{slug}")
        };
        let mut preset = Self::builtin_default();
        preset.id = id;
        preset.name = name.trim().to_string();
        preset
    }
}

fn sanitize_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

#[derive(Resource)]
pub struct PlayCharacterRegistry {
    presets: Vec<PlayCharacterPreset>,
    defs: HashMap<PlayCharacterId, PlayCharacterDef>,
}

impl Default for PlayCharacterRegistry {
    fn default() -> Self {
        let mut registry = Self {
            presets: vec![PlayCharacterPreset::builtin_default()],
            defs: HashMap::new(),
        };
        registry.rebuild_defs();
        registry
    }
}

impl PlayCharacterRegistry {
    pub fn load_from_disk(&mut self) {
        self.presets = load_all_presets();
        self.rebuild_defs();
    }

    pub fn presets(&self) -> &[PlayCharacterPreset] {
        &self.presets
    }

    pub fn preset(&self, id: &str) -> Option<&PlayCharacterPreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    pub fn get(&self, id: &PlayCharacterId) -> Option<&PlayCharacterDef> {
        self.defs.get(id)
    }

    pub fn rebuild_defs(&mut self) {
        self.defs.clear();
        for preset in &self.presets {
            self.defs.insert(
                PlayCharacterId(preset.id.clone()),
                preset.to_def(),
            );
        }
    }

    pub fn placeable_defs(&self) -> Vec<PlaceableDef> {
        self.presets
            .iter()
            .map(|p| PlaceableDef {
                id: PlaceableId(p.id.clone()),
                label: p.name.clone(),
                kind: PlaceableKind::PlayCharacter(PlayCharacterId(p.id.clone())),
            })
            .collect()
    }

    pub fn upsert_preset(&mut self, preset: PlayCharacterPreset) -> Result<(), String> {
        save_preset_to_disk(&preset)?;
        if let Some(existing) = self.presets.iter_mut().find(|p| p.id == preset.id) {
            *existing = preset;
        } else {
            self.presets.push(preset);
        }
        self.rebuild_defs();
        Ok(())
    }

    pub fn delete_preset(&mut self, id: &str) -> Result<(), String> {
        delete_preset_from_disk(id)?;
        self.presets.retain(|p| p.id != id);
        self.rebuild_defs();
        Ok(())
    }
}

/// Draft being edited in the Play sidebar before saving.
#[derive(Clone, Debug)]
pub struct PlayCharacterDraft {
    pub id: String,
    pub name: String,
    pub move_speed: f32,
    pub jump_speed: f32,
    pub linear_damping: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub color_rgb: [f32; 3],
    pub is_builtin: bool,
}

impl PlayCharacterDraft {
    pub fn from_preset(preset: &PlayCharacterPreset) -> Self {
        Self {
            id: preset.id.clone(),
            name: preset.name.clone(),
            move_speed: preset.move_speed,
            jump_speed: preset.jump_speed,
            linear_damping: preset.linear_damping,
            capsule_radius: preset.capsule_radius,
            capsule_half_height: preset.capsule_half_height,
            color_rgb: preset.color_rgb,
            is_builtin: preset.is_builtin(),
        }
    }

    pub fn to_preset(&self) -> PlayCharacterPreset {
        PlayCharacterPreset {
            id: self.id.clone(),
            name: self.name.trim().to_string(),
            move_speed: self.move_speed,
            jump_speed: self.jump_speed,
            linear_damping: self.linear_damping,
            capsule_radius: self.capsule_radius,
            capsule_half_height: self.capsule_half_height,
            color_rgb: self.color_rgb,
        }
    }

    pub fn new_from_defaults(name: &str) -> Self {
        Self::from_preset(&PlayCharacterPreset::new_unique(name))
    }
}

#[derive(Resource, Default)]
pub struct PlayCharacterEditor {
    pub selected_id: Option<String>,
    pub draft: Option<PlayCharacterDraft>,
    pub new_name_buffer: String,
    pub error: Option<String>,
}

impl PlayCharacterEditor {
    pub fn select_preset(&mut self, preset: &PlayCharacterPreset) {
        self.selected_id = Some(preset.id.clone());
        self.draft = Some(PlayCharacterDraft::from_preset(preset));
        self.error = None;
    }

    pub fn start_new(&mut self, name: &str) {
        let draft = PlayCharacterDraft::new_from_defaults(name);
        self.selected_id = Some(draft.id.clone());
        self.draft = Some(draft);
        self.error = None;
    }
}

#[derive(Clone, Debug)]
pub struct SpawnedCharacter {
    pub id: PlayCharacterId,
    pub entity: Entity,
    pub grid_key: IVec3,
}

#[derive(Resource, Default)]
pub struct PlayWorldState {
    pub active_character: Option<SpawnedCharacter>,
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

#[derive(Resource, Default)]
pub struct PlayUiActions {
    pub remove_character: bool,
    pub start_session: bool,
}

pub fn sync_placeables_from_characters(
    registry: &PlayCharacterRegistry,
    placeables: &mut super::PlaceableRegistry,
) {
    placeables.items = registry.placeable_defs();
}
