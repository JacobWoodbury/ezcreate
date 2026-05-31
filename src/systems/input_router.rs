use bevy::prelude::*;

use crate::resources::{GameMode, GameModeChanged, GamePreferences};

pub struct InputRouterPlugin;

impl Plugin for InputRouterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mode_hotkeys, shift_select_override));
    }
}

fn mode_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<GameMode>,
    mut events: MessageWriter<GameModeChanged>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        set_mode(&mut *mode, &mut events, GameMode::Place);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        set_mode(&mut *mode, &mut events, GameMode::Select);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        set_mode(&mut *mode, &mut events, GameMode::Paint);
    }
    if keys.just_pressed(KeyCode::Tab) {
        let next = mode.toggle_place_select();
        set_mode(&mut *mode, &mut events, next);
    }
}

fn shift_select_override(
    keys: Res<ButtonInput<KeyCode>>,
    prefs: Res<GamePreferences>,
    mut mode: ResMut<GameMode>,
    mut events: MessageWriter<GameModeChanged>,
) {
    if !prefs.select_mode_hold_shift {
        return;
    }

    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        if *mode != GameMode::Select {
            set_mode(&mut *mode, &mut events, GameMode::Select);
        }
    } else if *mode == GameMode::Select {
        set_mode(&mut *mode, &mut events, GameMode::Place);
    }
}

fn set_mode(mode: &mut GameMode, events: &mut MessageWriter<GameModeChanged>, next: GameMode) {
    if *mode == next {
        return;
    }
    *mode = next;
    events.write(GameModeChanged { mode: next });
}
