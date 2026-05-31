use bevy::prelude::*;

#[derive(Component)]
pub struct PlacedBlock {
    pub item_id: String,
    pub grid_key: IVec3,
    /// Source scene path from mod manifest (for saving grouped modules).
    pub scene_path: String,
}
