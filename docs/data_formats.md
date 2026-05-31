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
    "albedoTexturePath": "optional",
    "facePaints": [{
      "localNormal": [0, 1, 0],
      "brushColor": [220, 80, 60, 255],
      "paintType": "solid"
    }, {
      "localNormal": [1, 0, 0],
      "brushColor": [255, 255, 255, 255],
      "paintType": "stamp",
      "stamp": { "width": 4, "height": 4, "pixels": [] }
    }]
  }]
}
```

Offsets are **grid cells relative to anchor** (minimum cell when saved). Placement rotates around the **bottom corner** farthest from the footprint center (recomputed on each 90° rotation so the new bottom stays on the anchor).

**Face paint:** `facePaints` stores per-face decoration in **block-local** axis normals (`localNormal`, each component −1, 0, or 1). `paintType` is `solid` (flat brush color) or `stamp` (pixel grid). Saved automatically when using **Save selection as module** in Select mode.

## Entity meta (Godot) → Bevy components (target)

| Godot meta | Bevy |
|------------|------|
| `library_scene_path` | `LibraryMeta.scene_path` |
| `library_item_id` | `PlacedBlock.item_id` |
| `ez_section_assembly` | `SectionAssembly` component |
| `ez_section_clipboard_json` | serialized on component for undo |
| `ez_section_placement_euler` | `Vec3` on section wrapper |
| `ez_grouped_module` | `GroupedModuleRoot` component |
