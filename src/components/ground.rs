use bevy::prelude::*;

/// World floor collider — excluded from block pick / paint rays.
#[derive(Component)]
pub struct Ground;
