use bevy::prelude::*;

use crate::{components::OrbitCameraRig, resources::GamePreferences, ui::{GameplayAfterUi, UiInputCapture}};

#[derive(Component)]
pub struct OrbitCameraState {
    pub rotation: Vec2,
    pub zoom: f32,
    pub target_zoom: f32,
}

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, orbit_apply)
            .add_systems(PostUpdate, orbit_mouse_input.in_set(GameplayAfterUi))
            .add_systems(PostUpdate, orbit_keyboard_pan.in_set(GameplayAfterUi));
    }
}

fn orbit_mouse_input(
    capture: Res<UiInputCapture>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut scroll: MessageReader<bevy::input::mouse::MouseWheel>,
    mut rigs: Query<&mut OrbitCameraState, With<OrbitCameraRig>>,
) {
    if capture.block_game_pointer {
        return;
    }
    for mut state in &mut rigs {
        if mouse.pressed(MouseButton::Right) {
            for ev in motion.read() {
                state.rotation.y -= ev.delta.x * 0.01 * 0.5;
                state.rotation.x -= ev.delta.y * 0.01 * 0.5;
                state.rotation.x = state.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_4);
            }
        }

        for ev in scroll.read() {
            state.target_zoom -= ev.y * 0.8;
            state.target_zoom = state.target_zoom.clamp(2.0, 50.0);
        }
    }
}

fn orbit_keyboard_pan(
    keys: Res<ButtonInput<KeyCode>>,
    prefs: Res<GamePreferences>,
    capture: Res<UiInputCapture>,
    time: Res<Time>,
    mut rigs: Query<(&mut Transform, &OrbitCameraState), With<OrbitCameraRig>>,
) {
    if capture.block_game_keyboard {
        return;
    }    let (w_dir, s_dir) = if prefs.invert_ws_pan {
        (Vec3::Z, Vec3::NEG_Z)
    } else {
        (Vec3::NEG_Z, Vec3::Z)
    };

    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_dir += w_dir;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir += s_dir;
    }
    if keys.pressed(KeyCode::KeyD) {
        move_dir += Vec3::X;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_dir += Vec3::NEG_X;
    }

    if move_dir.length_squared() < 1e-6 {
        return;
    }

    for (mut transform, state) in &mut rigs {
        let yaw = Quat::from_rotation_y(state.rotation.y);
        let forward = (yaw * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let delta = (forward * move_dir.z + right * move_dir.x).normalize() * 12.0 * time.delta_secs();
        transform.translation += delta;
    }
}

fn orbit_apply(
    time: Res<Time>,
    mut rigs: Query<
        (&mut Transform, &mut OrbitCameraState, &Children),
        (With<OrbitCameraRig>, Without<Camera3d>),
    >,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<OrbitCameraRig>)>,
) {
    let lerp = time.delta_secs() * 10.0;

    for (mut rig_transform, mut state, children) in &mut rigs {
        state.zoom = state.zoom.lerp(state.target_zoom, lerp);

        let target_rot = Quat::from_euler(
            EulerRot::YXZ,
            state.rotation.y,
            state.rotation.x,
            0.0,
        );
        rig_transform.rotation = rig_transform.rotation.slerp(target_rot, lerp);

        for child in children.iter() {
            if let Ok(mut cam) = cameras.get_mut(child) {
                cam.translation.z = cam.translation.z.lerp(state.zoom, lerp);
            }
        }
    }
}
