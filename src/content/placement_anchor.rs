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

fn piece_offset(piece: &SectionBlueprintPiece) -> IVec3 {
    IVec3::new(piece.offset[0], piece.offset[1], piece.offset[2])
}

/// +90° steps around Y (Bevy: X → +Z for positive yaw... verify with Bevy convention).
fn rotate_cell_y(offset: IVec3, quarter_turns: i32) -> IVec3 {
    let mut v = offset;
    let steps = quarter_turns.rem_euclid(4);
    for _ in 0..steps {
        // (x, y, z) --+90° Y--> (z, y, -x)
        v = IVec3::new(v.z, v.y, -v.x);
    }
    v
}

fn yaw_to_quarter_turns(yaw: f32) -> i32 {
    let steps = (yaw / std::f32::consts::FRAC_PI_2).round() as i32;
    steps.rem_euclid(4)
}

/// Blueprint offset (unrotated grid cells) of the piece that sits on the placement anchor for `yaw`.
///
/// The bottom corner is chosen after rotating the footprint; the returned offset is in the same
/// space as blueprint `piece.offset` (used in `R * (offset - pivot)`).
pub fn section_anchor_offset_for_yaw(pieces: &[SectionBlueprintPiece], yaw: f32) -> Vec3 {
    if pieces.is_empty() {
        return Vec3::ZERO;
    }

    let quarter = yaw_to_quarter_turns(yaw);
    let originals: Vec<IVec3> = pieces.iter().map(piece_offset).collect();
    let rotated: Vec<IVec3> = originals
        .iter()
        .map(|o| rotate_cell_y(*o, quarter))
        .collect();

    let rotated_f: Vec<Vec3> = rotated
        .iter()
        .map(|o| o.as_vec3())
        .collect();
    let pivot_rotated = bottom_pivot_offset(&rotated_f);

    for (i, r) in rotated.iter().enumerate() {
        if (*r).as_vec3().distance_squared(pivot_rotated) < 0.01 {
            return originals[i].as_vec3();
        }
    }

    // Fallback: min corner of rotated footprint mapped back (should not happen).
    originals[0].as_vec3()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece_at(x: i32, y: i32, z: i32) -> SectionBlueprintPiece {
        SectionBlueprintPiece {
            scene_path: String::new(),
            item_id: String::new(),
            offset: [x, y, z],
            albedo_texture_path: None,
            face_paints: vec![],
        }
    }

    #[test]
    fn anchor_offset_at_yaw_zero_is_bottom_corner() {
        let pieces = vec![piece_at(0, 0, 0), piece_at(1, 0, 0)];
        let anchor = section_anchor_offset_for_yaw(&pieces, 0.0);
        assert!(anchor == Vec3::new(0.0, 0.0, 0.0) || anchor == Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn anchor_offset_matches_a_real_piece_after_90_degrees() {
        let pieces = vec![piece_at(0, 0, 0), piece_at(1, 0, 0)];
        let anchor = section_anchor_offset_for_yaw(&pieces, std::f32::consts::FRAC_PI_2);
        let valid: Vec<Vec3> = pieces
            .iter()
            .map(|p| Vec3::new(p.offset[0] as f32, p.offset[1] as f32, p.offset[2] as f32))
            .collect();
        assert!(valid.iter().any(|o| o.distance_squared(anchor) < 0.01));
    }
}
