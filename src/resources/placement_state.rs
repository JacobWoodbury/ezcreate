use bevy::prelude::*;

use crate::content::{LibraryItemRef, SectionBlueprintFile};
use crate::resources::PlaceableId;

/// Resolved once when a library item with `sectionSpecPath` is selected.
#[derive(Clone)]
pub struct ActiveSection {
    pub blueprint: SectionBlueprintFile,
}

#[derive(Resource)]
pub struct PlacementState {
    pub selected_item: Option<LibraryItemRef>,
    /// Selected placeable in Play mode (characters, NPCs, …). Mutually exclusive with `selected_item`.
    pub selected_placeable: Option<PlaceableId>,
    /// Set when the selected item has a `sectionSpecPath`.
    pub active_section: Option<ActiveSection>,
    pub placement_euler: Vec3,
    pub anchor_cell: Option<IVec3>,
    /// When set, ghost pivot uses this world position instead of anchor cell center.
    pub ghost_pivot_world: Option<Vec3>,
    /// Whether clicking would place without overlapping (when prevent overlap is on).
    pub placement_allowed: bool,
    /// Root ghost entity (single block or section pivot).
    pub ghost_entity: Option<Entity>,
    /// Cached preview state to avoid despawn/spawn every frame.
    pub ghost_signature: Option<u64>,
}

impl Default for PlacementState {
    fn default() -> Self {
        Self {
            selected_item: None,
            selected_placeable: None,
            active_section: None,
            placement_euler: Vec3::ZERO,
            anchor_cell: None,
            ghost_pivot_world: None,
            placement_allowed: false,
            ghost_entity: None,
            ghost_signature: None,
        }
    }
}

impl PlacementState {
    pub fn clear_ghost_cache(&mut self) {
        self.ghost_entity = None;
        self.ghost_signature = None;
    }

    pub fn block_ghost_signature(&self) -> Option<u64> {
        let anchor = self.anchor_cell?;
        let mut sig = anchor.x as u64;
        sig = sig.wrapping_mul(31).wrapping_add(anchor.y as u64);
        sig = sig.wrapping_mul(31).wrapping_add(anchor.z as u64);
        sig = sig.wrapping_mul(31).wrapping_add(self.placement_allowed as u64);
        sig = sig
            .wrapping_mul(31)
            .wrapping_add(self.placement_euler.y.to_bits() as u64);
        sig = sig
            .wrapping_mul(31)
            .wrapping_add(self.active_section.is_some() as u64);
        if let Some(pivot) = self.ghost_pivot_world {
            sig = sig.wrapping_mul(31).wrapping_add(pivot.x.to_bits() as u64);
            sig = sig.wrapping_mul(31).wrapping_add(pivot.y.to_bits() as u64);
            sig = sig.wrapping_mul(31).wrapping_add(pivot.z.to_bits() as u64);
        }
        Some(sig)
    }

    pub fn placeable_ghost_signature(&self, placeable_id: &str) -> Option<u64> {
        let anchor = self.anchor_cell?;
        let mut sig = anchor.x as u64;
        sig = sig.wrapping_mul(31).wrapping_add(anchor.y as u64);
        sig = sig.wrapping_mul(31).wrapping_add(anchor.z as u64);
        for b in placeable_id.bytes() {
            sig = sig.wrapping_mul(31).wrapping_add(b as u64);
        }
        Some(sig)
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

    pub fn select_block_item(&mut self, item: LibraryItemRef) {
        self.selected_item = Some(item);
        self.selected_placeable = None;
        self.active_section = None;
    }

    pub fn select_placeable(&mut self, id: PlaceableId) {
        self.selected_placeable = Some(id);
        self.selected_item = None;
        self.active_section = None;
    }

    pub fn clear_placeable(&mut self) {
        self.selected_placeable = None;
    }
}
