use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::components::{FacePaintDecal, PaintPreview, PlacedBlock};
use crate::resources::{FacePaintSnapshot, GameMode, GridConfig, GridEdit, PaintState, UndoStack};
use crate::systems::raycast_util::{cursor_ray, raycast_placed_block};

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_paint_hover, sync_paint_preview, handle_face_paint).chain(),
        );
    }
}

fn update_paint_hover(
    mut egui: EguiContexts,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    mut paint: ResMut<PaintState>,
    spatial_query: avian3d::prelude::SpatialQuery,
    blocks: Query<Entity, With<PlacedBlock>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    paint.hover_hit = None;

    if *mode != GameMode::Paint {
        return;
    }

    if egui.ctx_mut().is_ok_and(|ctx| ctx.is_pointer_over_area()) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(ray) = cursor_ray(window, camera, cam_transform) else {
        return;
    };

    let Some(hit) = raycast_placed_block(
        &spatial_query,
        &blocks,
        ray.origin,
        *ray.direction,
        grid.ray_length,
    ) else {
        return;
    };

    let hit_pos = ray.origin + *ray.direction * hit.distance;
    paint.hover_hit = Some(crate::resources::PaintFaceHit {
        block: hit.entity,
        position: hit_pos,
        normal: hit.normal,
    });
}

fn sync_paint_preview(
    mode: Res<GameMode>,
    paint: Res<PaintState>,
    mut commands: Commands,
    previews: Query<Entity, With<PaintPreview>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<GridConfig>,
) {
    for entity in &previews {
        commands.entity(entity).despawn();
    }

    if *mode != GameMode::Paint {
        return;
    }

    let Some(hit) = paint.hover_hit else {
        return;
    };

    let half = (grid.grid_size * 0.45).max(0.1);
    let size = Vec2::splat(half * 2.0);
    let mesh = meshes.add(Plane3d::new(Vec3::Y, size));
    let mut preview_color = paint.brush_color.to_srgba();
    preview_color.alpha *= 0.55;
    let material = materials.add(StandardMaterial {
        base_color: Color::from(preview_color),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        ..default()
    });

    let offset = hit.normal * grid.grid_size.max(0.01) * 0.02;
    let rotation = Quat::from_rotation_arc(Vec3::Y, hit.normal);

    commands.spawn((
        PaintPreview,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(hit.position + offset).with_rotation(rotation),
    ));
}

fn handle_face_paint(
    mut egui: EguiContexts,
    mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    paint: Res<PaintState>,
    grid: Res<GridConfig>,
    mut commands: Commands,
    mut undo: ResMut<UndoStack>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *mode != GameMode::Paint || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if egui.ctx_mut().is_ok_and(|ctx| ctx.is_pointer_over_area()) {
        return;
    }

    let Some(hit) = paint.hover_hit else {
        return;
    };

    let half = (grid.grid_size * 0.45).max(0.1);
    let size = Vec2::splat(half * 2.0);
    let mesh = meshes.add(Plane3d::new(Vec3::Y, size));
    let material = materials.add(StandardMaterial {
        base_color: paint.brush_color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        ..default()
    });

    let offset = hit.normal * grid.grid_size.max(0.01) * 0.02;
    let rotation = Quat::from_rotation_arc(Vec3::Y, hit.normal);

    let decal = commands
        .spawn((
            FacePaintDecal {
                color: paint.brush_color,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(hit.position + offset).with_rotation(rotation),
        ))
        .id();

    commands.entity(hit.block).add_child(decal);

    undo.push(GridEdit::FacePaint {
        snapshot: FacePaintSnapshot {
            parent_block: hit.block,
            decal_entity: decal,
            color: paint.brush_color,
        },
    });
}
