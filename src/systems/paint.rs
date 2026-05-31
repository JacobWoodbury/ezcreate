use bevy::prelude::*;

use crate::{
    components::{FacePaintDecal, PaintPreview, PlacedBlock},
    resources::{
        FacePaintKind, FacePaintSnapshot, GameMode, GridConfig, GridEdit, PaintState, StampPainter,
        UndoStack,
    },
    systems::{
        raycast_util::{
            block_face_center, cursor_ray, face_transform_world, paint_preview_color,
            raycast_placed_block, snap_axis_normal,
        },
        stamp_mesh::{face_transform, spawn_stamp_decal},
    },
    ui::{GameplayAfterUi, UiInputCapture},
};

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StampPainter>()
            .add_systems(Startup, load_saved_stamps)
            .add_systems(PostUpdate, update_paint_hover.in_set(GameplayAfterUi))
            .add_systems(PostUpdate, handle_face_paint.in_set(GameplayAfterUi))
            .add_systems(PostUpdate, sync_paint_preview.after(update_paint_hover));
    }
}

fn load_saved_stamps(mut stamp_painter: ResMut<StampPainter>) {
    stamp_painter.reload_stamps();
}

/// Raycast the block face under the cursor; returns block entity, world face center, and normal.
fn face_hit_under_cursor(
    block_pointer: bool,
    grid: &GridConfig,
    spatial_query: &avian3d::prelude::SpatialQuery,
    block_globals: &Query<&GlobalTransform, With<PlacedBlock>>,
    blocks: &Query<Entity, With<PlacedBlock>>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) -> Option<crate::resources::PaintFaceHit> {
    if block_pointer {
        return None;
    }

    let window = windows.single().ok()?;
    let (camera, cam_transform) = cameras.single().ok()?;
    let ray = cursor_ray(window, camera, cam_transform)?;
    let hit = raycast_placed_block(
        spatial_query,
        blocks,
        ray.origin,
        *ray.direction,
        grid.ray_length,
    )?;
    let block_global = block_globals.get(hit.entity).ok()?;
    let face_normal = snap_axis_normal(hit.normal);
    let face_center = block_face_center(block_global.translation(), grid.grid_size, face_normal);

    Some(crate::resources::PaintFaceHit {
        block: hit.entity,
        position: face_center,
        normal: face_normal,
    })
}

fn update_paint_hover(
    capture: Res<UiInputCapture>,
    mode: Res<GameMode>,
    grid: Res<GridConfig>,
    mut paint: ResMut<PaintState>,
    spatial_query: avian3d::prelude::SpatialQuery,
    block_globals: Query<&GlobalTransform, With<PlacedBlock>>,
    blocks: Query<Entity, With<PlacedBlock>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    paint.hover_hit = None;

    if *mode != GameMode::Paint {
        return;
    }

    paint.hover_hit = face_hit_under_cursor(
        capture.block_game_pointer,
        &grid,
        &spatial_query,
        &block_globals,
        &blocks,
        &windows,
        &cameras,
    );
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

    if stamp_painter.apply_mode {
        if stamp_painter.apply_uses_stamp_grid() {
            let mut preview_stamp = stamp_painter.stamp.clone();
            for px in &mut preview_stamp.pixels {
                px[3] = (px[3] / 2).max(60);
            }

            let root = spawn_stamp_decal(
                &mut commands,
                &mut meshes,
                &mut materials,
                &preview_stamp,
                hit.normal,
                face_size,
                face_transform(hit.position, hit.normal, face_size, bias),
                brush,
                hit.block,
            );
            commands.entity(root).insert(PaintPreview);
        } else {
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
    }
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
    let transform = face_transform_world(face_center, face_normal, bias);
    commands.spawn((
        PaintPreview,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        Visibility::default(),
    ));
}

fn handle_face_paint(
    capture: Res<UiInputCapture>,
    mode: Res<GameMode>,
    mouse: Res<ButtonInput<MouseButton>>,
    stamp_painter: Res<StampPainter>,
    grid: Res<GridConfig>,
    mut commands: Commands,
    mut undo: ResMut<UndoStack>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    decals: Query<(Entity, &FacePaintDecal)>,
    spatial_query: avian3d::prelude::SpatialQuery,
    block_globals: Query<&GlobalTransform, With<PlacedBlock>>,
    blocks: Query<Entity, With<PlacedBlock>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if *mode != GameMode::Paint || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if capture.block_game_pointer {
        return;
    }

    if !stamp_painter.apply_mode {
        return;
    }

    let Some(hit) = face_hit_under_cursor(
        capture.block_game_pointer,
        &grid,
        &spatial_query,
        &block_globals,
        &blocks,
        &windows,
        &cameras,
    ) else {
        return;
    };

    let bias = grid.grid_size * 0.025;
    let face_size = grid.grid_size * 0.98;
    let brush = stamp_painter.brush_color_bevy();
    let world_transform = face_transform_world(hit.position, hit.normal, bias);

    clear_face_paint(&mut commands, &decals, hit.block, hit.normal);

    let kind = if stamp_painter.apply_uses_stamp_grid() {
        spawn_stamp_decal(
            &mut commands,
            &mut meshes,
            &mut materials,
            &stamp_painter.stamp,
            hit.normal,
            face_size,
            world_transform,
            brush,
            hit.block,
        );
        FacePaintKind::Stamp {
            stamp: stamp_painter.stamp.clone(),
        }
    } else {
        spawn_solid_face_decal(
            &mut commands,
            &mut meshes,
            &mut materials,
            hit.block,
            hit.normal,
            face_size,
            world_transform,
            brush,
        );
        FacePaintKind::Solid
    };

    undo.push(GridEdit::FacePaint {
        snapshot: FacePaintSnapshot {
            parent_block: hit.block,
            color: brush,
            face_normal: hit.normal,
            face_size,
            bias,
            kind,
        },
    });
}

/// Re-applies a face-paint edit (used by redo).
pub fn apply_face_paint_snapshot(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    block_globals: &Query<&GlobalTransform, With<PlacedBlock>>,
    snapshot: &FacePaintSnapshot,
) {
    clear_face_paint(commands, decals, snapshot.parent_block, snapshot.face_normal);

    let Ok(block_global) = block_globals.get(snapshot.parent_block) else {
        return;
    };
    let face_center =
        block_face_center(block_global.translation(), snapshot.face_size, snapshot.face_normal);
    let world_transform = face_transform_world(face_center, snapshot.face_normal, snapshot.bias);

    match &snapshot.kind {
        FacePaintKind::Solid => {
            spawn_solid_face_decal(
                commands,
                meshes,
                materials,
                snapshot.parent_block,
                snapshot.face_normal,
                snapshot.face_size,
                world_transform,
                snapshot.color,
            );
        }
        FacePaintKind::Stamp { stamp } => {
            spawn_stamp_decal(
                commands,
                meshes,
                materials,
                stamp,
                snapshot.face_normal,
                snapshot.face_size,
                world_transform,
                snapshot.color,
                snapshot.parent_block,
            );
        }
    }
}

/// Records undo and removes a face-paint decal (root entity; stamp children are despawned too).
pub fn delete_face_decal_with_undo(
    commands: &mut Commands,
    undo: &mut UndoStack,
    grid: &GridConfig,
    decal_entity: Entity,
    decal: &FacePaintDecal,
) {
    undo.push(GridEdit::FacePaint {
        snapshot: FacePaintSnapshot {
            parent_block: decal.parent_block,
            color: decal.color,
            face_normal: decal.face_normal,
            face_size: grid.grid_size * 0.98,
            bias: grid.grid_size * 0.025,
            kind: decal.kind.clone(),
        },
    });
    commands.entity(decal_entity).despawn();
}

/// Removes any decal on `block` facing `face_normal` (used by undo).
pub fn remove_face_paint_on_block(
    commands: &mut Commands,
    decals: &Query<(Entity, &FacePaintDecal)>,
    block: Entity,
    face_normal: Vec3,
) {
    clear_face_paint(commands, decals, block, face_normal);
}

/// Removes every face-paint decal on `block` (call before despawn).
pub fn remove_all_face_paint_on_block(
    commands: &mut Commands,
    decals: &Query<(Entity, &FacePaintDecal)>,
    block: Entity,
) {
    for (entity, decal) in decals.iter() {
        if decal.parent_block == block {
            commands.entity(entity).despawn();
        }
    }
}

fn clear_face_paint(
    commands: &mut Commands,
    decals: &Query<(Entity, &FacePaintDecal)>,
    block: Entity,
    face_normal: Vec3,
) {
    for (entity, decal) in decals.iter() {
        if decal.parent_block == block && decal.face_normal.dot(face_normal) > 0.9 {
            log::debug!("Replacing face paint (color {:?})", decal.color);
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_solid_face_decal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent_block: Entity,
    face_normal: Vec3,
    face_size: f32,
    world_transform: Transform,
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

    commands.spawn((
        FacePaintDecal {
            color,
            face_normal,
            parent_block,
            kind: FacePaintKind::Solid,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        world_transform,
        Visibility::default(),
    ));
}

/// Applies blueprint face paint after a block is spawned in a section.
pub fn apply_blueprint_face_paint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    block_entity: Entity,
    block_center: Vec3,
    grid_size: f32,
    block_rotation: Quat,
    paint: &crate::content::BlueprintFacePaint,
) {
    use crate::content::{local_face_normal_to_world, rgba8_to_color};
    use crate::systems::raycast_util::{block_face_center, face_transform_world};

    let face_normal = local_face_normal_to_world(paint.local_normal, block_rotation);
    let brush = rgba8_to_color(paint.brush_color);
    let face_size = grid_size * 0.98;
    let bias = grid_size * 0.025;
    let face_center = block_face_center(block_center, grid_size, face_normal);
    let world_transform = face_transform_world(face_center, face_normal, bias);

    match &paint.kind {
        FacePaintKind::Solid => {
            spawn_solid_face_decal(
                commands,
                meshes,
                materials,
                block_entity,
                face_normal,
                face_size,
                world_transform,
                brush,
            );
        }
        FacePaintKind::Stamp { stamp } => {
            spawn_stamp_decal(
                commands,
                meshes,
                materials,
                stamp,
                face_normal,
                face_size,
                world_transform,
                brush,
                block_entity,
            );
        }
    }
}
