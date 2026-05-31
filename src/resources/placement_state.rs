use bevy::prelude::*;

use crate::content::{LibraryItemRef, SectionBlueprintFile};

/// Resolved once when a library item with `sectionSpecPath` is selected.
#[derive(Clone)]
pub struct ActiveSection {
    pub blueprint: SectionBlueprintFile,
}

#[derive(Resource)]
pub struct PlacementState {
    pub selected_item: Option<LibraryItemRef>,
    /// Set when the selected item has a `sectionSpecPath`.
    pub active_section: Option<ActiveSection>,
    pub placement_euler: Vec3,
    pub anchor_cell: Option<IVec3>,
    pub placement_valid: bool,
    /// Root ghost entity (single block or section pivot).
    pub ghost_entity: Option<Entity>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected_item: None,
            active_section: None,
            placement_euler: Vec3::ZERO,
            anchor_cell: None,
            placement_valid: false,
            ghost_entity: None,
        }
    }
}

impl PlacementState {
    pub fn snap_placement_euler(&mut self) {
        const STEP: f32 = std::f32::consts::FRAC_PI_2;
        self.placement_euler = Vec3::new(
            (self.placement_euler.x / STEP).round() * STEP,
            (self.placement_euler.y / STEP).round() * STEP,
            (self.placement_euler.z / STEP).round() * STEP,
        );
    }

    pub fn rotate_yaw_forward(&mut self) {
        self.placement_euler.y += std::f32::consts::FRAC_PI_2;
        self.snap_placement_euler();
    }

    pub fn rotate_yaw_reverse(&mut self) {
        self.placement_euler.y -= std::f32::consts::FRAC_PI_2;
        self.snap_placement_euler();
    }
}
