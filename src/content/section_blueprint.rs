use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::resources::FacePaintKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBlueprintFile {
    pub pieces: Vec<SectionBlueprintPiece>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBlueprintPiece {
    #[serde(rename = "scenePath")]
    pub scene_path: String,
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub offset: [i32; 3],
    #[serde(rename = "albedoTexturePath", skip_serializing_if = "Option::is_none")]
    pub albedo_texture_path: Option<String>,
    #[serde(rename = "facePaints", default, skip_serializing_if = "Vec::is_empty")]
    pub face_paints: Vec<BlueprintFacePaint>,
}

/// Face paint on one block face, stored in local space so section rotation can be applied on load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintFacePaint {
    #[serde(rename = "localNormal")]
    pub local_normal: [i32; 3],
    #[serde(rename = "brushColor")]
    pub brush_color: [u8; 4],
    #[serde(flatten)]
    pub kind: FacePaintKind,
}

pub fn axis_normal_to_i32(n: Vec3) -> [i32; 3] {
    let n = n.normalize_or_zero();
    [
        n.x.round() as i32,
        n.y.round() as i32,
        n.z.round() as i32,
    ]
}

pub fn i32_to_axis_normal(n: [i32; 3]) -> Vec3 {
    Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32).normalize_or_zero()
}

pub fn color_to_rgba8(c: Color) -> [u8; 4] {
    let s = c.to_srgba();
    [
        (s.red.clamp(0.0, 1.0) * 255.0) as u8,
        (s.green.clamp(0.0, 1.0) * 255.0) as u8,
        (s.blue.clamp(0.0, 1.0) * 255.0) as u8,
        (s.alpha.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

pub fn rgba8_to_color(c: [u8; 4]) -> Color {
    Color::srgba(
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    )
}

/// World-space face normal -> local axis normal (inverse block rotation).
pub fn world_face_normal_to_local(world: Vec3, block_rotation: Quat) -> [i32; 3] {
    axis_normal_to_i32(block_rotation.inverse() * world)
}

/// Local axis normal -> world-space face normal (block rotation at placement).
pub fn local_face_normal_to_world(local: [i32; 3], block_rotation: Quat) -> Vec3 {
    snap_axis_normal(block_rotation * i32_to_axis_normal(local))
}

fn snap_axis_normal(normal: Vec3) -> Vec3 {
    let n = normal.normalize_or_zero();
    let abs = n.abs();
    if abs.x >= abs.y && abs.x >= abs.z {
        Vec3::X * n.x.signum()
    } else if abs.y >= abs.z {
        Vec3::Y * n.y.signum()
    } else {
        Vec3::Z * n.z.signum()
    }
}
