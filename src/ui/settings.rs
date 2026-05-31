use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::{BindingId, GamePreferences, GridConfig, KeyBindings};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Keybindings,
}

pub struct SettingsUiState {
    pub tab: SettingsTab,
    pub rebinding: Option<BindingId>,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            tab: SettingsTab::default(),
            rebinding: None,
        }
    }
}

pub fn draw_settings_window(
    ui: &mut egui::Ui,
    grid: &mut GridConfig,
    prefs: &mut GamePreferences,
    bindings: &mut KeyBindings,
    settings: &mut SettingsUiState,
    keys: &ButtonInput<KeyCode>,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut settings.tab, SettingsTab::General, "General");
        ui.selectable_value(&mut settings.tab, SettingsTab::Keybindings, "Keybindings");
    });
    ui.separator();

    match settings.tab {
        SettingsTab::General => draw_general_tab(ui, grid, prefs),
        SettingsTab::Keybindings => draw_keybindings_tab(ui, bindings, settings, keys),
    }
}

fn draw_general_tab(ui: &mut egui::Ui, grid: &mut GridConfig, prefs: &mut GamePreferences) {
    ui.heading("Placement");
    ui.checkbox(&mut grid.prevent_overlapping, "Prevent block overlap");

    ui.add_space(8.0);
    ui.heading("Selection");
    ui.checkbox(
        &mut prefs.shift_toggles_place_select,
        "Shift toggles Place / Select",
    );

    ui.add_space(8.0);
    ui.heading("Camera");
    ui.checkbox(
        &mut prefs.invert_ws_pan,
        "Invert W / S pan (forward / back)",
    );

    ui.add_space(8.0);
    ui.heading("Grid");
    ui.horizontal(|ui| {
        ui.label("Grid size:");
        ui.add(egui::DragValue::new(&mut grid.grid_size).range(0.25..=8.0).speed(0.05));
    });
    ui.horizontal(|ui| {
        ui.label("Ray length:");
        ui.add(
            egui::DragValue::new(&mut grid.ray_length)
                .range(10.0..=2000.0)
                .speed(1.0),
        );
    });
}

fn draw_keybindings_tab(
    ui: &mut egui::Ui,
    bindings: &mut KeyBindings,
    settings: &mut SettingsUiState,
    keys: &ButtonInput<KeyCode>,
) {
    if let Some(id) = settings.rebinding {
        ui.label(format!("Press a key for: {}", id.label()));
        ui.label("Esc — cancel");
        if let Some(key) = KeyBindings::capture_rebind_key(keys) {
            bindings.set(id, key);
            settings.rebinding = None;
        }
        if keys.just_pressed(KeyCode::Escape) {
            settings.rebinding = None;
        }
        return;
    }

    ui.label("Click Change to rebind. Undo/redo also require Ctrl.");
    ui.add_space(4.0);

    egui::Grid::new("keybind_grid")
        .num_columns(3)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Action").strong());
            ui.label(egui::RichText::new("Key").strong());
            ui.label("");
            ui.end_row();

            for id in BindingId::ALL {
                let key = bindings.get(id);
                let display = if matches!(id, BindingId::Undo | BindingId::Redo) {
                    format!("Ctrl + {}", KeyBindings::key_label(key))
                } else {
                    KeyBindings::key_label(key)
                };

                ui.label(id.label());
                ui.label(&display);
                if ui.button("Change").clicked() {
                    settings.rebinding = Some(id);
                }
                ui.end_row();

                let hint = id.hint();
                if !hint.is_empty() && !matches!(id, BindingId::Undo | BindingId::Redo) {
                    ui.label("");
                    ui.small(hint);
                    ui.label("");
                    ui.end_row();
                }
            }
        });

    ui.add_space(12.0);
    ui.heading("Mouse");
    ui.label("Place block / paint face — Left click");
    ui.label("Delete block (Place mode) — Alt + Right click");
    ui.label("Orbit camera — Right click drag");
    ui.label("Zoom — Mouse wheel");

    ui.add_space(8.0);
    ui.heading("Camera");
    ui.label("Pan — W A S D");
}
