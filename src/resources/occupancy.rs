use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct OccupancyMap {
    cells: HashMap<IVec3, Entity>,
}

impl OccupancyMap {
    pub fn get(&self, cell: IVec3) -> Option<Entity> {
        self.cells.get(&cell).copied()
    }

    pub fn insert(&mut self, cell: IVec3, entity: Entity) {
        self.cells.insert(cell, entity);
    }

    pub fn remove(&mut self, cell: IVec3) -> Option<Entity> {
        self.cells.remove(&cell)
    }

    pub fn contains(&self, cell: IVec3) -> bool {
        self.cells.contains_key(&cell)
    }
}
