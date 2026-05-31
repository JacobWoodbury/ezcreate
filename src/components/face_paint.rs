use bevy::prelude::*;

#[derive(Component)]
pub struct FacePaintDecal {
    pub color: Color,
    /// Axis-aligned outward normal of the painted face (for replacing prior paint on the same face).
    pub face_normal: Vec3,
}
