use std::path::PathBuf;

use super::{PlayCharacterId, PlayCharacterPreset};

pub fn characters_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ezcreate/characters"))
}

pub fn load_all_presets() -> Vec<PlayCharacterPreset> {
    let Some(dir) = characters_dir() else {
        return vec![PlayCharacterPreset::builtin_default()];
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![PlayCharacterPreset::builtin_default()];
    };

    let mut presets = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(preset) = serde_json::from_str::<PlayCharacterPreset>(&text) {
            presets.push(preset);
        }
    }

    presets.sort_by(|a, b| a.name.cmp(&b.name));

    if presets.iter().any(|p| p.id == PlayCharacterId::default_character().0) {
        presets
    } else {
        let mut all = vec![PlayCharacterPreset::builtin_default()];
        all.extend(presets);
        all
    }
}

pub fn save_preset_to_disk(preset: &PlayCharacterPreset) -> Result<(), String> {
    let dir = characters_dir().ok_or("Cannot resolve user data directory.")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", preset.id));
    let json = serde_json::to_string_pretty(preset).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn delete_preset_from_disk(id: &str) -> Result<(), String> {
    if id == PlayCharacterId::default_character().0 {
        return Err("Cannot delete the built-in default character.".into());
    }
    if let Some(dir) = characters_dir() {
        let path = dir.join(format!("{id}.json"));
        if path.is_file() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
