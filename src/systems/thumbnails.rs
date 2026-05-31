use std::collections::HashMap;

use bevy::prelude::*;

use crate::content::LibraryCatalog;

/// Maps library item_id → swatch color for display in the sidebar.
/// PNG thumbnails can be layered on top of this later when asset loading is wired up.
#[derive(Resource, Default)]
pub struct ThumbnailCache {
    pub colors: HashMap<String, [u8; 4]>,
}

pub struct ThumbnailPlugin;

impl Plugin for ThumbnailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThumbnailCache>()
            .add_systems(Update, build_thumbnails);
    }
}

/// Registers swatch colors for newly-discovered catalog items.
fn build_thumbnails(catalog: Res<LibraryCatalog>, mut cache: ResMut<ThumbnailCache>) {
    for item in &catalog.items {
        cache
            .colors
            .entry(item.item_id.clone())
            .or_insert_with(|| hash_color(&item.item_id));
    }
}

/// Derive a stable, pleasant color from a string ID (FNV-1a hash).
pub fn hash_color(id: &str) -> [u8; 4] {
    let h = id
        .bytes()
        .fold(0x811c9dc5u32, |acc, b| acc.wrapping_mul(0x01000193).wrapping_add(b as u32));
    // Bias toward medium-bright saturated hues.
    let r = 0x55 | ((h >> 16) & 0x6a) as u8;
    let g = 0x45 | ((h >> 8) & 0x6a) as u8;
    let b = 0x55 | (h & 0x6a) as u8;
    [r, g, b, 255]
}
