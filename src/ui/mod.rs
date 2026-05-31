mod hud;
mod input_capture;
mod settings;

pub use hud::UiPlugin;
pub use input_capture::UiInputCapture;

use bevy::prelude::*;

/// Gameplay systems that must run after egui so [`UiInputCapture`] is current.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameplayAfterUi;

pub fn configure_gameplay_after_ui(app: &mut App) {
    use bevy_egui::EguiPostUpdateSet;

    app.init_resource::<UiInputCapture>().configure_sets(
        PostUpdate,
        GameplayAfterUi.after(EguiPostUpdateSet::EndPass),
    );
}
