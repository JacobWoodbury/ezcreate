use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::components::OrbitCameraRig;

#[derive(Component)]
pub struct OrbitCameraState {
    pub rotation: Vec2,
    pub zoom: f32,
    pub target_zoom: f32,
}

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (orbit_mouse_input, orbit_keyboard_pan, orbit_apply));
    }
}

fn orbit_mouse_input(
    mut egui: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut scroll: MessageReader<bevy::input::mouse::MouseWheel>,
    mut rigs: Query<&mut OrbitCameraState, With<OrbitCameraRig>>,
) {
    if egui
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_area())
    {
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
    time: Res<Time>,
    mut rigs: Query<(&mut Transform, &OrbitCameraState), With<OrbitCameraRig>>,
) {
    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        move_dir += Vec3::NEG_Z;
    }
    if keys.pressed(KeyCode::KeyS) {
        move_dir += Vec3::Z;
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
