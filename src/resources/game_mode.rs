use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Place,
    Select,
    Paint,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Place => "Place",
            GameMode::Select => "Select",
            GameMode::Paint => "Paint",
        }
    }

    pub fn toggle_place_select(self) -> Self {
        match self {
            GameMode::Place => GameMode::Select,
            GameMode::Select => GameMode::Place,
            GameMode::Paint => self,
        }
    }
}

#[derive(Message)]
pub struct GameModeChanged {
    pub mode: GameMode,
}
