use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    components::{Ground, OrbitCameraRig, PlacedRoot},
    systems::camera_orbit::OrbitCameraState,
};

pub struct WorldSetupPlugin;

impl Plugin for WorldSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 0.8, 0.0)),
    ));

    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.42, 0.38),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::IDENTITY,
        RigidBody::Static,
        Collider::cuboid(50.0, 0.05, 50.0),
    ));

    commands.spawn(PlacedRoot);

    commands
        .spawn((
            OrbitCameraRig,
            OrbitCameraState {
                rotation: Vec2::new(-0.35, 0.9),
                zoom: 10.0,
                target_zoom: 10.0,
            },
            Transform::from_xyz(4.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 10.0),
            ));
        });
}
