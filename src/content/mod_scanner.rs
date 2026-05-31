use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde_json;

use super::manifest::ModManifest;
use crate::resources::GridConfig;

#[derive(Clone, Debug)]
pub struct LibraryItemRef {
    pub mod_id: String,
    pub item_id: String,
    pub display_name: String,
    pub scene_path: String,
    /// Relative path to the section blueprint JSON (from manifest_dir), if this is a section item.
    pub section_spec_path: Option<String>,
    pub manifest_dir: PathBuf,
}

#[derive(Resource, Default)]
pub struct LibraryCatalog {
    pub items: Vec<LibraryItemRef>,
}

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LibraryCatalog>()
            .init_resource::<GridConfig>()
            .add_systems(Startup, scan_mods);
    }
}

fn scan_mods(mut catalog: ResMut<LibraryCatalog>) {
    catalog.items.clear();
    let mut seen_ids = std::collections::HashSet::new();

    for root in mod_roots() {
        if !root.exists() {
            continue;
        }
        walk_mod_json(&root, &mut catalog.items, &mut seen_ids);
    }

    info!(
        "Library: loaded {} items from {} mod roots",
        catalog.items.len(),
        mod_roots().len()
    );
}

fn mod_roots() -> Vec<PathBuf> {
    let mut roots = vec![PathBuf::from("assets/mods")];
    if let Some(data) = dirs::data_dir() {
        roots.push(data.join("ezcreate/mods"));
    }
    roots
}

fn walk_mod_json(dir: &Path, out: &mut Vec<LibraryItemRef>, seen: &mut std::collections::HashSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("mod.json");
            if manifest_path.is_file() {
                load_manifest(&manifest_path, &path, out, seen);
            } else {
                walk_mod_json(&path, out, seen);
            }
        }
    }
}

fn load_manifest(
    path: &Path,
    manifest_dir: &Path,
    out: &mut Vec<LibraryItemRef>,
    seen: &mut std::collections::HashSet<String>,
) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to read {}: {e}", path.display());
            return;
        }
    };

    let manifest: ModManifest = match serde_json::from_str(&text) {
        Ok(m) => m,
        Err(e) => {
            warn!("Invalid mod.json at {}: {e}", path.display());
            return;
        }
    };

    for item in manifest.items {
        if !seen.insert(item.id.clone()) {
            warn!("Duplicate library item id '{}', skipping", item.id);
            continue;
        }
        out.push(LibraryItemRef {
            mod_id: manifest.id.clone(),
            item_id: item.id,
            display_name: item.display_name,
            scene_path: item.scene_path,
            section_spec_path: item.section_spec_path,
            manifest_dir: manifest_dir.to_path_buf(),
        });
    }
}
