use bevy::prelude::*;

#[derive(Resource)]
pub struct GamePreferences {
    pub select_mode_hold_shift: bool,
}

impl Default for GamePreferences {
    fn default() -> Self {
        Self {
            select_mode_hold_shift: true,
        }
    }
}
