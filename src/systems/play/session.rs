use bevy::prelude::*;

use crate::{
    components::OrbitCameraRig,
    resources::{
        GameMode, GameModeChanged, PlaySession, PlaySessionSnapshot, PlaySessionStorage,
        PlayUiActions, PlayWorldState,
    },
    systems::camera_orbit::OrbitCameraState,
};

/// Systems that run only during an active 3rd-person play session.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlaySessionActive;

pub fn play_session_inactive(session: Res<PlaySession>) -> bool {
    session.is_inactive()
}

pub fn play_session_is_active(session: Res<PlaySession>) -> bool {
    session.is_active()
}

pub fn enter_play_session(
    session: &mut PlaySession,
    storage: &mut PlaySessionStorage,
    rigs: &Query<(&Transform, &OrbitCameraState, &Children), With<OrbitCameraRig>>,
    cameras: &Query<&Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) -> bool {
    if session.is_active() {
        return true;
    }

    let Ok((rig_tf, cam_state, children)) = rigs.single() else {
        return false;
    };

    let camera_local = children
        .iter()
        .find_map(|child| cameras.get(child).ok().map(|t| t.translation))
        .unwrap_or(Vec3::new(0.0, 0.0, 10.0));

    storage.snapshot = Some(PlaySessionSnapshot {
        rig_translation: rig_tf.translation,
        rig_rotation: rig_tf.rotation,
        camera_state: cam_state.clone(),
        camera_local,
    });

    *session = PlaySession::Active;
    true
}

pub fn exit_play_session(
    session: &mut PlaySession,
    storage: &mut PlaySessionStorage,
    rigs: &mut Query<(&mut Transform, &mut OrbitCameraState, &Children), With<OrbitCameraRig>>,
    cameras: &mut Query<&mut Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    if session.is_inactive() {
        return;
    }

    if let Some(snapshot) = storage.snapshot.take() {
        if let Ok((mut rig_tf, mut cam_state, children)) = rigs.single_mut() {
            rig_tf.translation = snapshot.rig_translation;
            rig_tf.rotation = snapshot.rig_rotation;
            *cam_state = snapshot.camera_state;
            for child in children.iter() {
                if let Ok(mut cam) = cameras.get_mut(child) {
                    cam.translation = snapshot.camera_local;
                }
            }
        }
    }

    *session = PlaySession::Inactive;
}

pub fn play_session_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<PlaySession>,
    mut storage: ResMut<PlaySessionStorage>,
    mut rigs: Query<(&mut Transform, &mut OrbitCameraState, &Children), With<OrbitCameraRig>>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    if session.is_inactive() {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        exit_play_session(&mut session, &mut storage, &mut rigs, &mut cameras);
    }
}

pub fn auto_exit_session_on_mode_change(
    mut reader: MessageReader<GameModeChanged>,
    mut session: ResMut<PlaySession>,
    mut storage: ResMut<PlaySessionStorage>,
    mut rigs: Query<(&mut Transform, &mut OrbitCameraState, &Children), With<OrbitCameraRig>>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    for GameModeChanged { mode: new_mode } in reader.read() {
        if *new_mode != GameMode::Play && session.is_active() {
            exit_play_session(&mut session, &mut storage, &mut rigs, &mut cameras);
        }
    }
}

pub fn apply_play_ui_session_actions(
    mut actions: ResMut<PlayUiActions>,
    world: Res<PlayWorldState>,
    mut session: ResMut<PlaySession>,
    mut storage: ResMut<PlaySessionStorage>,
    rigs: Query<(&Transform, &OrbitCameraState, &Children), With<OrbitCameraRig>>,
    cameras: Query<&Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    if !actions.start_session {
        return;
    }
    actions.start_session = false;

    if world.active_character.is_none() {
        return;
    }

    enter_play_session(
        &mut session,
        &mut storage,
        &rigs,
        &cameras,
    );
}
