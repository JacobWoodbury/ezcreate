use bevy::prelude::*;

use crate::{
    components::GhostPreview,
    resources::{
        set_game_mode, BindingId, GameMode, GameModeChanged, GamePreferences, KeyBindings,
        PaintState, PlacementState,
    },
    ui::{GameplayAfterUi, UiInputCapture},
};

pub struct InputRouterPlugin;

impl Plugin for InputRouterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                mode_hotkeys,
                shift_toggle_place_select,
                on_mode_changed,
            )
                .in_set(GameplayAfterUi),
        );
    }
}

fn mode_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    capture: Res<UiInputCapture>,
    mut mode: ResMut<GameMode>,
    mut events: MessageWriter<GameModeChanged>,
) {
    if capture.block_game_keyboard {
        return;
    }
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

fn shift_toggle_place_select(
    keys: Res<ButtonInput<KeyCode>>,
    prefs: Res<GamePreferences>,
    capture: Res<UiInputCapture>,
    mut mode: ResMut<GameMode>,
    mut events: MessageWriter<GameModeChanged>,
) {
    if capture.block_game_keyboard {
        return;
    }
    if !prefs.shift_toggles_place_select {
        return;
    }

    let shift_tap = keys.just_pressed(KeyCode::ShiftLeft) || keys.just_pressed(KeyCode::ShiftRight);
    if !shift_tap {
        return;
    }

    match *mode {
        GameMode::Place | GameMode::Select => {
            let next = mode.toggle_place_select();
            set_game_mode(&mut mode, &mut events, next);
        }
        GameMode::Paint => {}
    }
}

fn on_mode_changed(
    mut reader: MessageReader<GameModeChanged>,
    mut paint: ResMut<PaintState>,
    mut placement: ResMut<PlacementState>,
    mut commands: Commands,
    ghosts: Query<Entity, With<GhostPreview>>,
) {
    for GameModeChanged { .. } in reader.read() {
        paint.hover_hit = None;
        for ghost in &ghosts {
            commands.entity(ghost).despawn();
        }
        placement.ghost_entity = None;
        placement.anchor_cell = None;
        placement.placement_valid = false;
    }
}
