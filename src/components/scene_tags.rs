use bevy::prelude::*;

#[derive(Component)]
pub struct PlacedRoot;

#[derive(Component)]
pub struct OrbitCameraRig;

#[derive(Component)]
pub struct GhostPreview;

/// Separate tag for the paint-mode face preview, so it doesn't clash with the placement ghost.
#[derive(Component)]
pub struct PaintPreview;

/// World-space highlight mesh for a selected block (not parented — blocks use `ChildOf(PlacedRoot)`).
#[derive(Component)]
pub struct SelectionOutline {
    pub block: Entity,
}
