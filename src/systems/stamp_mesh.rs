use bevy::prelude::*;

use crate::{components::FacePaintDecal, resources::Stamp};

/// Spawn a stamp decal as a hierarchy: one root entity + one `Plane3d` child per pixel.
/// Returns the root entity (which should be added as a child of the hit block).
///
/// `face_normal` must be unit-length and axis-aligned.
/// `face_size`   = the full side length of the block face in world units.
/// `bias`        = tiny offset along the normal to avoid Z-fighting.
pub fn spawn_stamp_decal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    stamp: &Stamp,
    face_normal: Vec3,
    hit_pos: Vec3,
    face_size: f32,
    bias: f32,
) -> Entity {
    let cols = stamp.width as f32;
    let rows = stamp.height as f32;
    let cell_w = face_size / cols;
    let cell_h = face_size / rows;
    // Small gap between cells so they don't bleed together.
    let inset = face_size * 0.005;

    // Rotation that maps the local XZ plane onto the hit face.
    let rotation = Quat::from_rotation_arc(Vec3::Y, face_normal.normalize());

    let root = commands
        .spawn((
            FacePaintDecal {
                color: Color::WHITE,
                face_normal,
            },
            Transform::from_translation(hit_pos + face_normal * bias).with_rotation(rotation),
            Visibility::default(),
        ))
        .id();

    for row in 0..stamp.height {
        for col in 0..stamp.width {
            let [r, g, b, a] = stamp.get(col, row);
            if a == 0 {
                continue; // Skip fully transparent pixels.
            }

            let color = Color::srgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            );

            // Local offset in the XZ plane of the root entity.
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

/// Returns a `Transform` that places an XZ-plane mesh onto a block face.
/// Useful for the hover-preview plane.
pub fn face_transform(
    hit_pos: Vec3,
    face_normal: Vec3,
    _face_size: f32,
    bias: f32,
) -> Transform {
    let rotation = Quat::from_rotation_arc(Vec3::Y, face_normal.normalize());
    Transform::from_translation(hit_pos + face_normal * bias).with_rotation(rotation)
}
