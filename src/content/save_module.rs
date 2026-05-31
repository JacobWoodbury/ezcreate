use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

use super::section_blueprint::{
    BlueprintFacePaint, SectionBlueprintFile, SectionBlueprintPiece, color_to_rgba8,
    world_face_normal_to_local,
};
use super::{LibraryCatalog, LibraryItemRef};
use crate::components::{FacePaintDecal, PlacedBlock};
use crate::content::manifest::{ModManifest, ModManifestItem};
use crate::resources::{GridConfig, SelectionState};

pub fn user_blueprints_root() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ezcreate/mods/user_blueprints"))
}

/// Save the current selection as a section blueprint + mod.json entry.
pub fn register_grouped_module(
    selection: &SelectionState,
    blocks: &Query<(Entity, &PlacedBlock, &GlobalTransform)>,
    decals: &Query<(Entity, &FacePaintDecal)>,
    grid: &GridConfig,
    display_name: &str,
    catalog: &mut LibraryCatalog,
) -> Result<LibraryItemRef, String> {
    if selection.selected.is_empty() {
        return Err("Nothing selected.".into());
    }

    let mut pieces = Vec::new();
    for entity in &selection.selected {
        let (_, block, transform) = blocks
            .get(*entity)
            .map_err(|_| "Selected block was despawned.".to_string())?;
        pieces.push((
            grid.world_to_grid(transform.translation()),
            block.scene_path.clone(),
            block.item_id.clone(),
        ));
    }

    let min_key = pieces
        .iter()
        .map(|(k, _, _)| *k)
        .reduce(|a, b| a.min(b))
        .unwrap();

    let mut face_paints_by_offset: HashMap<[i32; 3], Vec<BlueprintFacePaint>> = HashMap::new();

    for (_, decal) in decals.iter() {
        if !selection.selected.contains(&decal.parent_block) {
            continue;
        }
        let Ok((_, _, block_transform)) = blocks.get(decal.parent_block) else {
            continue;
        };
        let grid_key = grid.world_to_grid(block_transform.translation());
        let offset = [
            grid_key.x - min_key.x,
            grid_key.y - min_key.y,
            grid_key.z - min_key.z,
        ];
        face_paints_by_offset
            .entry(offset)
            .or_default()
            .push(BlueprintFacePaint {
                local_normal: world_face_normal_to_local(
                    decal.face_normal,
                    block_transform.rotation(),
                ),
                brush_color: color_to_rgba8(decal.color),
                kind: decal.kind.clone(),
            });
    }

    pieces.sort_by_key(|(k, _, _)| (k.y, k.x, k.z));

    let blueprint = SectionBlueprintFile {
        pieces: pieces
            .iter()
            .map(|(key, scene_path, item_id)| {
                let offset = [
                    key.x - min_key.x,
                    key.y - min_key.y,
                    key.z - min_key.z,
                ];
                SectionBlueprintPiece {
                    scene_path: scene_path.clone(),
                    item_id: item_id.clone(),
                    offset,
                    albedo_texture_path: None,
                    face_paints: face_paints_by_offset.remove(&offset).unwrap_or_default(),
                }
            })
            .collect(),
    };

    let root = user_blueprints_root().ok_or("Could not resolve user data directory.")?;
    let sections_dir = root.join("sections");
    std::fs::create_dir_all(&sections_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(root.join("grouped")).map_err(|e| e.to_string())?;

    let item_id = format!("grouped_{}", uuid_simple());
    let section_filename = format!("{item_id}.json");
    let section_path = sections_dir.join(&section_filename);
    let section_rel = format!("sections/{section_filename}");

    let json = serde_json::to_string_pretty(&blueprint).map_err(|e| e.to_string())?;
    std::fs::write(&section_path, json).map_err(|e| e.to_string())?;

    let representative_scene = pieces[0].1.clone();
    let manifest_path = root.join("mod.json");
    let mut manifest = if manifest_path.exists() {
        let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<ModManifest>(&text).unwrap_or(ModManifest {
            id: "user_blueprints".into(),
            name: "User Blueprints".into(),
            items: vec![],
        })
    } else {
        ModManifest {
            id: "user_blueprints".into(),
            name: "User Blueprints".into(),
            items: vec![],
        }
    };

    manifest.items.push(ModManifestItem {
        id: item_id.clone(),
        display_name: display_name.to_string(),
        scene_path: representative_scene.clone(),
        thumbnail_path: None,
        category: Some("User".into()),
        section_spec_path: Some(section_rel.clone()),
    });

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, manifest_json).map_err(|e| e.to_string())?;

    let item_ref = LibraryItemRef {
        mod_id: manifest.id.clone(),
        item_id: item_id.clone(),
        display_name: display_name.to_string(),
        scene_path: representative_scene,
        thumbnail_path: None,
        section_spec_path: Some(section_rel),
        manifest_dir: root.clone(),
    };
    catalog.items.push(item_ref.clone());
    Ok(item_ref)
}

/// Removes a user-saved module from disk and the in-memory catalog.
pub fn delete_user_module(item: &LibraryItemRef, catalog: &mut LibraryCatalog) -> Result<(), String> {
    if !item.is_user_deletable() {
        return Err("Built-in library items cannot be deleted.".into());
    }

    let manifest_path = item.manifest_dir.join("mod.json");
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut manifest: ModManifest = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let removed = manifest
        .items
        .iter()
        .position(|i| i.id == item.item_id)
        .ok_or_else(|| format!("Item '{}' not found in mod.json", item.item_id))?;
    let removed = manifest.items.remove(removed);

    if let Some(spec_rel) = removed.section_spec_path.as_ref() {
        let spec_path = item.manifest_dir.join(spec_rel);
        if spec_path.is_file() {
            std::fs::remove_file(&spec_path).map_err(|e| e.to_string())?;
        }
    }

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, manifest_json).map_err(|e| e.to_string())?;

    catalog.items.retain(|i| {
        !(i.mod_id == item.mod_id && i.item_id == item.item_id)
    });

    Ok(())
}

/// Renames a user-saved module (`displayName` in mod.json and in-memory catalog).
pub fn rename_user_module(
    item: &LibraryItemRef,
    new_display_name: &str,
    catalog: &mut LibraryCatalog,
) -> Result<(), String> {
    if !item.is_user_deletable() {
        return Err("Built-in library items cannot be renamed.".into());
    }

    let name = new_display_name.trim();
    if name.is_empty() {
        return Err("Name cannot be empty.".into());
    }

    let manifest_path = item.manifest_dir.join("mod.json");
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut manifest: ModManifest = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let entry = manifest
        .items
        .iter_mut()
        .find(|i| i.id == item.item_id)
        .ok_or_else(|| format!("Item '{}' not found in mod.json", item.item_id))?;
    entry.display_name = name.to_string();

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, manifest_json).map_err(|e| e.to_string())?;

    for lib_item in catalog.items.iter_mut() {
        if lib_item.mod_id == item.mod_id && lib_item.item_id == item.item_id {
            lib_item.display_name = name.to_string();
        }
    }

    Ok(())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}
