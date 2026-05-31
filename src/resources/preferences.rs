use bevy::prelude::*;

#[derive(Resource)]
pub struct GamePreferences {
    /// Tap Shift to toggle Place ↔ Select (ignored in Paint mode).
    pub shift_toggles_place_select: bool,
    /// When true, W pans toward +Z and S toward −Z (swapped from the legacy mapping).
    pub invert_ws_pan: bool,
}

impl Default for GamePreferences {
    fn default() -> Self {
        Self {
            shift_toggles_place_select: true,
            invert_ws_pan: true,
        }
    }
}
