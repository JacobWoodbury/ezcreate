use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::AppScreen;
use crate::ui::overlay::dim_fullscreen_overlay;

/// Full-screen dim overlay with a centered launch panel.
pub fn draw_launch_menu(
    ctx: &egui::Context,
    screen: &mut AppScreen,
    mut exit: MessageWriter<AppExit>,
) {
    dim_fullscreen_overlay(ctx, "launch_menu_overlay");

    egui::Window::new("ezcreate")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .fixed_size([320.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("ezcreate");
                ui.label("Modular building sandbox");
                ui.add_space(12.0);

                if ui.button("Play").clicked() {
                    AppScreen::enter_playing(screen);
                }
                if ui.button("Tutorial").clicked() {
                    AppScreen::start_ftue(screen);
                }
                if ui.button("Quit").clicked() {
                    exit.write(AppExit::Success);
                }
            });
        });
}
