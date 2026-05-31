use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::{AppScreen, FTUE_STEPS, KeyBindings};
use crate::ui::overlay::dim_fullscreen_overlay;

/// Full-screen FTUE slideshow with Back / Next / Skip.
pub fn draw_ftue(ctx: &egui::Context, screen: &mut AppScreen, bindings: &KeyBindings) {
    let step_index = match screen {
        AppScreen::Ftue { step } => (*step).min(FTUE_STEPS.len().saturating_sub(1)),
        _ => return,
    };

    let total = FTUE_STEPS.len();
    let ftue_step = &FTUE_STEPS[step_index];
    let body = (ftue_step.body)(bindings);

    dim_fullscreen_overlay(ctx, "ftue_overlay");

    egui::Window::new("Tutorial")
        .title_bar(true)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(480.0)
        .show(ctx, |ui| {
            ui.label(format!("Step {} / {}", step_index + 1, total));
            ui.separator();
            ui.heading(ftue_step.title);
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    ui.label(body);
                });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if step_index > 0 {
                    if ui.button("Back").clicked() {
                        AppScreen::ftue_back(screen);
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("Back"));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Skip tutorial").clicked() {
                        AppScreen::ftue_skip(screen);
                    }
                    let next_label = if step_index + 1 >= total {
                        "Finish"
                    } else {
                        "Next"
                    };
                    if ui.button(next_label).clicked() {
                        AppScreen::ftue_next(screen);
                    }
                });
            });
        });
}
