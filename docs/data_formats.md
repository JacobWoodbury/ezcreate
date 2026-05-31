# Data formats

## `mod.json`

```json
{
  "id": "sample_pack",
  "name": "Sample Pack",
  "items": [{
    "id": "sample_block",
    "displayName": "Sample Block",
    "scenePath": "scenes/sample_block.glb",
    "category": "Samples",
    "sectionSpecPath": "sections/foo.json"
  }]
}
```

- Scanned recursively under `assets/mods` and user data `ezcreate/mods`.
- Item `id` must be unique per session (later duplicates are skipped).

## Section blueprint (`sections/*.json`)

```json
{
  "pieces": [{
    "scenePath": "scenes/sample_block.glb",
    "itemId": "sample_block",
    "offset": [0, 0, 0],
    "albedoTexturePath": "optional"
  }]
}
```

Offsets are **grid cells relative to anchor** (minimum cell when saved). Placement applies rigid rotation around the piece centroid before occupancy keys are computed.

## Entity meta (Godot) → Bevy components (target)

| Godot meta | Bevy |
|------------|------|
| `library_scene_path` | `LibraryMeta.scene_path` |
| `library_item_id` | `PlacedBlock.item_id` |
| `ez_section_assembly` | `SectionAssembly` component |
| `ez_section_clipboard_json` | serialized on component for undo |
| `ez_section_placement_euler` | `Vec3` on section wrapper |
| `ez_grouped_module` | `GroupedModuleRoot` component |
