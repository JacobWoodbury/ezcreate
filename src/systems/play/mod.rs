use bevy::prelude::*;

pub use session::{play_session_inactive, PlaySessionActive};
pub use spawn::{remove_active_character, spawn_placeable};

mod camera;
mod movement;
mod session;
mod spawn;

pub struct PlayPlugin;

fn init_character_presets(
    mut registry: ResMut<crate::resources::PlayCharacterRegistry>,
    mut placeables: ResMut<crate::resources::PlaceableRegistry>,
    mut editor: ResMut<crate::resources::PlayCharacterEditor>,
) {
    registry.load_from_disk();
    crate::resources::sync_placeables_from_characters(&registry, &mut placeables);
    if let Some(first) = registry.presets().first() {
        editor.select_preset(first);
    }
}

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        use bevy_egui::EguiPostUpdateSet;

        app.init_resource::<crate::resources::PlayUiActions>()
            .add_systems(Startup, init_character_presets)
            .configure_sets(
                PostUpdate,
                PlaySessionActive
                    .run_if(session::play_session_is_active)
                    .after(EguiPostUpdateSet::EndPass),
            )
            .add_systems(
                Update,
                (
                    spawn::apply_play_ui_remove_action,
                    session::apply_play_ui_session_actions,
                    session::auto_exit_session_on_mode_change,
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    session::play_session_input,
                    movement::character_movement,
                    camera::play_camera_follow,
                    camera::play_camera_mouse_look,
                )
                    .in_set(PlaySessionActive),
            );
    }
}
