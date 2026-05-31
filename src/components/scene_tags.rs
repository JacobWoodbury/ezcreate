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

/// Slightly larger child mesh shown when the parent block is selected.
#[derive(Component)]
pub struct SelectionOutline;
