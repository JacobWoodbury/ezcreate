use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::{AppScreen, PlaySession};

/// Updated each frame after egui is drawn; gameplay systems read this to avoid input passthrough.
#[derive(Resource, Default)]
pub struct UiInputCapture {
    pub block_game_keyboard: bool,
    pub block_game_pointer: bool,
    pub block_play_movement: bool,
    pub block_play_look: bool,
}

impl UiInputCapture {
    pub fn sync(
        ctx: &egui::Context,
        screen: &AppScreen,
        settings_open: bool,
        session: &PlaySession,
        capture: &mut Self,
    ) {
        let menu_blocking = !matches!(screen, AppScreen::Playing);
        let text_focus = ctx.wants_keyboard_input();
        let pointer_over_ui = ctx.is_pointer_over_area();

        capture.block_game_keyboard = menu_blocking || settings_open || text_focus;
        capture.block_game_pointer =
            menu_blocking || settings_open || text_focus || pointer_over_ui;

        if session.is_active() {
            // Block builder hotkeys (1–4, undo, etc.) but allow WASD/Space/Esc for play.
            capture.block_play_movement =
                menu_blocking || settings_open || text_focus;
            capture.block_play_look =
                menu_blocking || settings_open || text_focus || pointer_over_ui;
        } else {
            capture.block_play_movement = true;
            capture.block_play_look = true;
        }
    }
}
