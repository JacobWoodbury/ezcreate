use bevy::prelude::*;

use crate::{
    components::OrbitCameraRig,
    resources::{PlaySession, PlayWorldState},
    systems::camera_orbit::OrbitCameraState,
    ui::UiInputCapture,
};

pub fn play_camera_follow(
    session: Res<PlaySession>,
    world: Res<PlayWorldState>,
    time: Res<Time>,
    characters: Query<&GlobalTransform, With<crate::components::PlayCharacter>>,
    mut rigs: Query<
        (&mut Transform, &OrbitCameraState, &Children),
        (With<OrbitCameraRig>, Without<Camera3d>),
    >,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    if session.is_inactive() {
        return;
    }

    let Some(spawned) = world.active_character.as_ref() else {
        return;
    };

    let Ok(char_tf) = characters.get(spawned.entity) else {
        return;
    };

    let Ok((mut rig_tf, state, children)) = rigs.single_mut() else {
        return;
    };

    let focus = char_tf.translation() + Vec3::Y * 0.9;
    let lerp = (time.delta_secs() * 8.0).clamp(0.0, 1.0);
    rig_tf.translation = rig_tf.translation.lerp(focus, lerp);

    let target_rot = Quat::from_euler(
        EulerRot::YXZ,
        state.rotation.y,
        state.rotation.x,
        0.0,
    );
    rig_tf.rotation = rig_tf.rotation.slerp(target_rot, lerp);

    for child in children.iter() {
        if let Ok(mut cam) = cameras.get_mut(child) {
            cam.translation.z = cam.translation.z.lerp(state.zoom, lerp);
        }
    }
}

pub fn play_camera_mouse_look(
    capture: Res<UiInputCapture>,
    session: Res<PlaySession>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut rigs: Query<&mut OrbitCameraState, With<OrbitCameraRig>>,
) {
    if session.is_inactive() || capture.block_play_look {
        return;
    }

    for mut state in &mut rigs {
        if mouse.pressed(MouseButton::Right) {
            for ev in motion.read() {
                state.rotation.y -= ev.delta.x * 0.01 * 0.5;
                state.rotation.x -= ev.delta.y * 0.01 * 0.5;
                state.rotation.x = state.rotation.x.clamp(
                    -std::f32::consts::FRAC_PI_2,
                    std::f32::consts::FRAC_PI_4,
                );
            }
        } else {
            motion.clear();
        }
    }
}
