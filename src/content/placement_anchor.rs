use bevy::prelude::*;

use super::section_blueprint::SectionBlueprintPiece;

/// Pivot in grid-cell space: lowest Y layer, then the cell farthest from that layer's XZ center.
pub fn bottom_pivot_offset(piece_offsets: &[Vec3]) -> Vec3 {
    if piece_offsets.is_empty() {
        return Vec3::ZERO;
    }

    let min_y = piece_offsets
        .iter()
        .map(|o| o.y)
        .fold(f32::INFINITY, f32::min);

    let bottom: Vec<Vec3> = piece_offsets
        .iter()
        .copied()
        .filter(|o| (o.y - min_y).abs() < 0.5)
        .collect();

    if bottom.len() == 1 {
        return bottom[0];
    }

    let cx = bottom.iter().map(|o| o.x).sum::<f32>() / bottom.len() as f32;
    let cz = bottom.iter().map(|o| o.z).sum::<f32>() / bottom.len() as f32;
    let center = Vec3::new(cx, min_y, cz);

    bottom
        .iter()
        .copied()
        .max_by(|a, b| {
            let da = Vec2::new(a.x - center.x, a.z - center.z).length_squared();
            let db = Vec2::new(b.x - center.x, b.z - center.z).length_squared();
            da.total_cmp(&db)
        })
        .unwrap_or(bottom[0])
}

pub fn bottom_pivot_offset_from_pieces(pieces: &[SectionBlueprintPiece]) -> Vec3 {
    let offsets: Vec<Vec3> = pieces
        .iter()
        .map(|p| {
            Vec3::new(
                p.offset[0] as f32,
                p.offset[1] as f32,
                p.offset[2] as f32,
            )
        })
        .collect();
    bottom_pivot_offset(&offsets)
}

/// Pivot for the current placement yaw — recomputed so the anchored corner is the bottom one after rotation.
pub fn bottom_pivot_offset_for_yaw(pieces: &[SectionBlueprintPiece], yaw: f32) -> Vec3 {
    let rotation = Quat::from_rotation_y(yaw);
    let rotated: Vec<Vec3> = pieces
        .iter()
        .map(|p| {
            let offset = Vec3::new(
                p.offset[0] as f32,
                p.offset[1] as f32,
                p.offset[2] as f32,
            );
            let v = rotation * offset;
            Vec3::new(v.x.round(), v.y.round(), v.z.round())
        })
        .collect();
    bottom_pivot_offset(&rotated)
}
