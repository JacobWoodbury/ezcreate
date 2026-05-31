use bevy::prelude::*;

#[derive(Resource)]
pub struct GridConfig {
    pub grid_size: f32,
    pub prevent_overlapping: bool,
    pub ray_length: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            grid_size: 1.0,
            prevent_overlapping: false,
            ray_length: 2500.0,
        }
    }
}

impl GridConfig {
    pub fn snap_to_grid(&self, world: Vec3) -> Vec3 {
        let g = self.grid_size.max(f32::EPSILON);
        Vec3::new(
            (world.x / g).round() * g,
            (world.y / g).round() * g,
            (world.z / g).round() * g,
        )
    }

    pub fn world_to_grid(&self, world: Vec3) -> IVec3 {
        let g = self.grid_size.max(f32::EPSILON);
        IVec3::new(
            (world.x / g).round() as i32,
            (world.y / g).round() as i32,
            (world.z / g).round() as i32,
        )
    }

    pub fn grid_to_world(&self, key: IVec3) -> Vec3 {
        let g = self.grid_size.max(f32::EPSILON);
        key.as_vec3() * g
    }
}
