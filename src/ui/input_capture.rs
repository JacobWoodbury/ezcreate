use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::AppScreen;

/// Updated each frame after egui is drawn; gameplay systems read this to avoid input passthrough.
#[derive(Resource, Default)]
pub struct UiInputCapture {
    pub block_game_keyboard: bool,
    pub block_game_pointer: bool,
}

impl UiInputCapture {
    pub fn sync(
        ctx: &egui::Context,
        screen: &AppScreen,
        settings_open: bool,
        capture: &mut Self,
    ) {
        let menu_blocking = !matches!(screen, AppScreen::Playing);
        let text_focus = ctx.wants_keyboard_input();
        capture.block_game_keyboard = menu_blocking || settings_open || text_focus;
        capture.block_game_pointer =
            menu_blocking || settings_open || text_focus || ctx.is_pointer_over_area();
    }
}
