use bevy::prelude::*;

use super::PlayCharacterId;

/// Unique id for a placeable entry (characters, NPCs, mod props, …).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaceableId(pub String);

/// What to spawn when a placeable is placed. Extend with new variants as needed.
#[derive(Clone, Debug)]
pub enum PlaceableKind {
    PlayCharacter(PlayCharacterId),
}

/// Sidebar / registry entry for a placeable item.
#[derive(Clone, Debug)]
pub struct PlaceableDef {
    pub id: PlaceableId,
    pub label: String,
    pub kind: PlaceableKind,
}

#[derive(Resource, Default)]
pub struct PlaceableRegistry {
    pub items: Vec<PlaceableDef>,
}

impl PlaceableRegistry {
    pub fn get(&self, id: &PlaceableId) -> Option<&PlaceableDef> {
        self.items.iter().find(|d| d.id == *id)
    }
}
