use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    components::{FacePaintDecal, PaintPreview, PlacedBlock},
    resources::{FacePaintSnapshot, GameMode, GridConfig, GridEdit, PaintState, StampPainter, UndoStack},
    systems::{
        raycast_util::{
            block_face_center, cursor_ray, paint_preview_color, raycast_placed_block, snap_axis_normal,
        },
        stamp_mesh::{face_transform, spawn_stamp_decal},
    },
};

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StampPainter>().add_systems(
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
    block_transforms: Query<(Entity, &GlobalTransform), With<PlacedBlock>>,
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

    let Ok((block, block_transform)) = block_transforms.get(hit.entity) else {
        return;
    };

    let face_normal = snap_axis_normal(hit.normal);
    let face_center = block_face_center(block_transform.translation(), grid.grid_size, face_normal);

    paint.hover_hit = Some(crate::resources::PaintFaceHit {
        block,
        position: face_center,
        normal: face_normal,
    });
}

fn sync_paint_preview(
    mode: Res<GameMode>,
    paint: Res<PaintState>,
    stamp_painter: Res<StampPainter>,
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

    let bias = grid.grid_size * 0.02;
    let face_size = grid.grid_size * 0.98;
    let brush = stamp_painter.brush_color_bevy();
    let preview_tint = paint_preview_color(brush);

    if stamp_painter.apply_mode && stamp_painter.has_pattern_cutouts() {
        let mut alpha_stamp = stamp_painter.stamp.clone();
        for px in &mut alpha_stamp.pixels {
            px[3] = (px[3] / 2).max(60);
        }

        let root = spawn_stamp_decal(
            &mut commands,
            &mut meshes,
            &mut materials,
            &alpha_stamp,
            hit.normal,
            hit.position,
            face_size,
            bias,
        );
        commands.entity(root).insert(PaintPreview);
    }

    spawn_face_overlay(
        &mut commands,
        &mut meshes,
        &mut materials,
        hit.position,
        hit.normal,
        face_size,
        bias,
        preview_tint,
    );
}

fn spawn_face_overlay(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    face_center: Vec3,
    face_normal: Vec3,
    face_size: f32,
    bias: f32,
    color: Color,
) {
    let half = face_size * 0.5;
    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(half)));
    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        ..default()
    });
    let transform = face_transform(face_center, face_normal, face_size, bias);
    commands.spawn((
        PaintPreview,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        Visibility::default(),
    ));
}

fn handle_face_paint(
    mut egui: EguiContexts,
    mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    paint: Res<PaintState>,
    stamp_painter: Res<StampPainter>,
    grid: Res<GridConfig>,
    mut commands: Commands,
    mut undo: ResMut<UndoStack>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    children: Query<&Children>,
    decals: Query<(Entity, &FacePaintDecal)>,
) {
    if *mode != GameMode::Paint || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if !stamp_painter.apply_mode {
        return;
    }

    if egui.ctx_mut().is_ok_and(|ctx| ctx.is_pointer_over_area()) {
        return;
    }

    let Some(hit) = paint.hover_hit else {
        return;
    };

    let bias = grid.grid_size * 0.025;
    let face_size = grid.grid_size * 0.98;
    let brush = stamp_painter.brush_color_bevy();

    clear_face_paint(&mut commands, &children, &decals, hit.block, hit.normal);

    let decal = if stamp_painter.has_pattern_cutouts() {
        spawn_stamp_decal(
            &mut commands,
            &mut meshes,
            &mut materials,
            &stamp_painter.stamp,
            hit.normal,
            hit.position,
            face_size,
            bias,
        )
    } else {
        spawn_solid_face_decal(
            &mut commands,
            &mut meshes,
            &mut materials,
            hit.position,
            hit.normal,
            face_size,
            bias,
            brush,
        )
    };

    commands.entity(hit.block).add_child(decal);
    undo.push(GridEdit::FacePaint {
        snapshot: FacePaintSnapshot {
            parent_block: hit.block,
            decal_entity: decal,
            color: brush,
        },
    });
}

fn clear_face_paint(
    commands: &mut Commands,
    children: &Query<&Children>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    block: Entity,
    face_normal: Vec3,
) {
    let Ok(kids) = children.get(block) else {
        return;
    };
    for child in kids.iter() {
        let Ok((entity, decal)) = decals.get(child) else {
            continue;
        };
        if decal.face_normal.dot(face_normal) > 0.9 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_solid_face_decal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    face_center: Vec3,
    face_normal: Vec3,
    face_size: f32,
    bias: f32,
    color: Color,
) -> Entity {
    let half = face_size * 0.5;
    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(half)));
    let rotation = Quat::from_rotation_arc(Vec3::Y, face_normal);
    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        ..default()
    });

    commands
        .spawn((
            FacePaintDecal {
                color,
                face_normal,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(face_center + face_normal * bias).with_rotation(rotation),
            Visibility::default(),
        ))
        .id()
}
