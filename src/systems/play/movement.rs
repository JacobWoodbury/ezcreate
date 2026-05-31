use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    components::{OrbitCameraRig, PlayCharacter},
    resources::{PlayCharacterRegistry, PlaySession, PlayWorldState},
    systems::camera_orbit::OrbitCameraState,
    ui::UiInputCapture,
};

pub fn character_movement(
    capture: Res<UiInputCapture>,
    session: Res<PlaySession>,
    world: Res<PlayWorldState>,
    registry: Res<PlayCharacterRegistry>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    rigs: Query<&OrbitCameraState, With<OrbitCameraRig>>,
    mut characters: Query<(&PlayCharacter, &mut LinearVelocity), With<PlayCharacter>>,
) {
    if session.is_inactive() {
        return;
    }

    if capture.block_play_movement {
        return;
    }

    let Some(spawned) = world.active_character.as_ref() else {
        return;
    };

    let Some(def) = registry.get(&spawned.id) else {
        return;
    };

    let Ok((_, mut velocity)) = characters.get_mut(spawned.entity) else {
        return;
    };

    let Ok(cam_state) = rigs.single() else {
        return;
    };

    let yaw = Quat::from_rotation_y(cam_state.rotation.y);
    let forward = (yaw * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    let right = forward.cross(Vec3::Y).normalize_or_zero();

    let mut wish = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        wish += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        wish -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        wish += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        wish -= right;
    }

    let mut target = if wish.length_squared() > 1e-6 {
        wish.normalize() * def.move_speed
    } else {
        Vec3::ZERO
    };
    target.y = velocity.0.y;

    if keys.just_pressed(KeyCode::Space) && velocity.0.y.abs() < 0.15 {
        target.y = def.jump_speed;
    }

    let blend = (time.delta_secs() * 12.0).clamp(0.0, 1.0);
    velocity.0 = velocity.0.lerp(target, blend);
}
