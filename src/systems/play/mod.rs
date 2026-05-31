mod camera;
mod movement;
mod session;
mod spawn;

use bevy::prelude::*;

pub use session::{play_session_inactive, PlaySessionActive};
pub use spawn::{remove_active_character, spawn_placeable};

pub struct PlayPlugin;
impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        use bevy_egui::EguiPostUpdateSet;

        app.init_resource::<crate::resources::PlayUiActions>()
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
