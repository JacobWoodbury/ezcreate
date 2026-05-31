use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::content::{LibraryCatalog, LibraryItemRef};

/// Stable cache key for a library row (`mod_id` + `item_id`).
pub fn library_item_cache_key(item: &LibraryItemRef) -> String {
    format!("{}/{}", item.mod_id, item.item_id)
}

/// Maps library cache key → swatch color for display in the sidebar.
#[derive(Resource, Default)]
pub struct ThumbnailCache {
    pub colors: HashMap<String, [u8; 4]>,
    /// Decoded PNG pixels keyed by cache key; uploaded to egui lazily in the HUD.
    pub images: HashMap<String, egui::ColorImage>,
    pub texture_handles: HashMap<String, egui::TextureHandle>,
}

pub struct ThumbnailPlugin;

impl Plugin for ThumbnailPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThumbnailCache>()
            .add_systems(Update, build_thumbnails);
    }
}

/// Registers swatch colors and optional PNG thumbnails for catalog items.
fn build_thumbnails(catalog: Res<LibraryCatalog>, mut cache: ResMut<ThumbnailCache>) {
    for item in &catalog.items {
        let key = library_item_cache_key(item);
        cache
            .colors
            .entry(key.clone())
            .or_insert_with(|| hash_color(&item.item_id));

        if cache.images.contains_key(&key) {
            continue;
        }

        let Some(rel) = item.thumbnail_path.as_deref() else {
            continue;
        };
        let path = item.manifest_dir.join(rel);
        if let Some(image) = load_png_as_color_image(&path) {
            cache.images.insert(key, image);
        }
    }
}

fn load_png_as_color_image(path: &Path) -> Option<egui::ColorImage> {
    let img = image::open(path).ok()?.into_rgba8();
    let width = img.width() as usize;
    let height = img.height() as usize;
    let size = [width, height];
    let pixels: Vec<egui::Color32> = img
        .chunks_exact(4)
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    Some(egui::ColorImage {
        size,
        source_size: egui::vec2(width as f32, height as f32),
        pixels,
    })
}

/// Derive a stable, pleasant color from a string ID (FNV-1a hash).
pub fn hash_color(id: &str) -> [u8; 4] {
    let h = id
        .bytes()
        .fold(0x811c9dc5u32, |acc, b| acc.wrapping_mul(0x01000193).wrapping_add(b as u32));
    let r = 0x55 | ((h >> 16) & 0x6a) as u8;
    let g = 0x45 | ((h >> 8) & 0x6a) as u8;
    let b = 0x55 | (h & 0x6a) as u8;
    [r, g, b, 255]
}
