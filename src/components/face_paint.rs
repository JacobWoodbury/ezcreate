use bevy::prelude::*;

use crate::resources::FacePaintKind;

#[derive(Component, Clone)]
pub struct FacePaintDecal {
    pub color: Color,
    /// Axis-aligned outward normal of the painted face (for replacing prior paint on the same face).
    pub face_normal: Vec3,
    /// Block this decal belongs to (decals live in world space, not as mesh children).
    pub parent_block: Entity,
    pub kind: FacePaintKind,
}
