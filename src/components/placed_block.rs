use bevy::prelude::*;

#[derive(Component)]
pub struct PlacedBlock {
    pub item_id: String,
    pub grid_key: IVec3,
}
