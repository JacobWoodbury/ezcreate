use bevy::prelude::*;

use crate::{
    components::FacePaintDecal,
    resources::{FacePaintKind, Stamp},
};

/// Spawn a stamp decal in world space: root + one `Plane3d` child per opaque pixel.
pub fn spawn_stamp_decal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    stamp: &Stamp,
    face_normal: Vec3,
    face_size: f32,
    world_transform: Transform,
    decal_color: Color,
    parent_block: Entity,
) -> Entity {
    let cols = stamp.width as f32;
    let rows = stamp.height as f32;
    let cell_w = face_size / cols;
    let cell_h = face_size / rows;
    let inset = face_size * 0.005;

    let root = commands
        .spawn((
            FacePaintDecal {
                color: decal_color,
                face_normal,
                parent_block,
                kind: FacePaintKind::Stamp {
                    stamp: stamp.clone(),
                },
            },
            world_transform,
            Visibility::default(),
        ))
        .id();

    for row in 0..stamp.height {
        for col in 0..stamp.width {
            let [r, g, b, a] = stamp.get(col, row);
            if a == 0 {
                continue;
            }

            let color = Color::srgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            );

            let local_x = (col as f32 + 0.5) * cell_w - face_size * 0.5;
            let local_z = (row as f32 + 0.5) * cell_h - face_size * 0.5;

            let mesh = meshes.add(Plane3d::new(
                Vec3::Y,
                Vec2::new(cell_w * 0.5 - inset, cell_h * 0.5 - inset),
            ));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                double_sided: true,
                ..default()
            });

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(Vec3::new(local_x, 0.0, local_z)),
                ChildOf(root),
            ));
        }
    }

    root
}

/// Returns a world-space `Transform` for a face overlay (hover preview).
pub fn face_transform(hit_pos: Vec3, face_normal: Vec3, _face_size: f32, bias: f32) -> Transform {
    crate::systems::raycast_util::face_transform_world(hit_pos, face_normal, bias)
}
