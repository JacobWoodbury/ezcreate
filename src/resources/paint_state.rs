use bevy::prelude::*;

#[derive(Resource)]
pub struct PaintState {
    pub brush_color: Color,
    /// Last face hit while hovering in paint mode (for preview).
    pub hover_hit: Option<PaintFaceHit>,
}

#[derive(Clone, Copy)]
pub struct PaintFaceHit {
    pub block: Entity,
    pub position: Vec3,
    pub normal: Vec3,
}

impl Default for PaintState {
    fn default() -> Self {
        Self {
            brush_color: Color::srgb(0.9, 0.35, 0.25),
            hover_hit: None,
        }
    }
}
