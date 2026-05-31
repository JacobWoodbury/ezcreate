use bevy::prelude::*;

use crate::resources::PlayCharacterId;

/// Marker for a dropped playable character entity.
#[derive(Component)]
pub struct PlayCharacter {
    pub id: PlayCharacterId,
}
