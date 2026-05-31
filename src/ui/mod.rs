mod ftue;
mod hud;
mod input_capture;
mod launch_menu;
mod overlay;
mod settings;

pub use hud::UiPlugin;
pub use input_capture::UiInputCapture;

use bevy::prelude::*;

use crate::resources::AppScreen;

/// Gameplay systems that must run after egui so [`UiInputCapture`] is current.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameplayAfterUi;

pub fn gameplay_active(screen: Res<AppScreen>) -> bool {
    matches!(*screen, AppScreen::Playing)
}

pub fn configure_gameplay_after_ui(app: &mut App) {
    use bevy_egui::EguiPostUpdateSet;

    app.init_resource::<UiInputCapture>().configure_sets(
        PostUpdate,
        GameplayAfterUi
            .run_if(gameplay_active)
            .after(EguiPostUpdateSet::EndPass),
    );
}
