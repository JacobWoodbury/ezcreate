use bevy::prelude::*;

use crate::resources::{
    set_game_mode, BindingId, GameMode, GameModeChanged, GamePreferences, KeyBindings,
    PaintState, PlacementState,
};

pub struct InputRouterPlugin;

impl Plugin for InputRouterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mode_hotkeys, shift_select_override, on_mode_changed));
    }
}

fn mode_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut mode: ResMut<GameMode>,
    mut events: MessageWriter<GameModeChanged>,
) {
    if bindings.just_pressed(&keys, BindingId::ModePlace) {
        set_game_mode(&mut mode, &mut events, GameMode::Place);
    }
    if bindings.just_pressed(&keys, BindingId::ModeSelect) {
        set_game_mode(&mut mode, &mut events, GameMode::Select);
    }
    if bindings.just_pressed(&keys, BindingId::ModePaint) {
        set_game_mode(&mut mode, &mut events, GameMode::Paint);
    }
    if bindings.just_pressed(&keys, BindingId::TogglePlaceSelect) {
        let next = mode.toggle_place_select();
        set_game_mode(&mut mode, &mut events, next);
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

    if KeyBindings::shift_pressed(&keys) {
        if *mode != GameMode::Select {
            set_game_mode(&mut mode, &mut events, GameMode::Select);
        }
    } else if *mode == GameMode::Select {
        set_game_mode(&mut mode, &mut events, GameMode::Place);
    }
}

fn on_mode_changed(
    mut reader: MessageReader<GameModeChanged>,
    mut paint: ResMut<PaintState>,
    mut placement: ResMut<PlacementState>,
    mut commands: Commands,
) {
    for GameModeChanged { mode } in reader.read() {
        paint.hover_hit = None;
        placement.anchor_cell = None;
        placement.placement_valid = false;
        if *mode != GameMode::Place {
            placement.ghost_entity = None;
        } else if let Some(ghost) = placement.ghost_entity.take() {
            commands.entity(ghost).despawn();
        }
    }
}
