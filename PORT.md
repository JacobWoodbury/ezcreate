# ezcreate — Bevy port

Rust/Bevy rebuild of **ezcreate** (Architect-Builder-X): a Townscaper-style, god-view modular building sandbox with JSON-driven mods, 1m grid placement, Place/Select/Paint modes, and undo.

**Godot source:** `/Users/jacobwoodbury/Documents/builder` (ignore `addons/ziva_agent`).

## Status

| Phase | Scope | Status |
|-------|--------|--------|
| **A — MVP** | Orbit camera, ground, mod scan, single-block place/delete, ghost, Q/E yaw, egui modes, undo | **Done** |
| **B — Parity** | Selection, sections, grouped modules, paint, settings | **Partial** (see below) |
| **C — Roadmap** | Sockets, WFC, texture paint | Not started |

## Run

```sh
cargo run
```

## Phase B progress

| Feature | Status |
|---------|--------|
| Select: click + shift-toggle | Done |
| Select: marquee (screen rect) | Done |
| Select: highlight tint | Done |
| Select: Delete / Q/E rotate group | Done |
| Select: save selection → section JSON | Done (`user://…/ezcreate/mods/user_blueprints/`) |
| Paint: face decals + undo delete | Done (redo paint: partial) |
| Recent picks in library UI | Done |
| Section **placement** from `sectionSpecPath` | Not yet |
| Grouped module scenes + `RotationPivot` | Not yet |
| Settings overlay + input persistence | Not yet |

## Controls (Phase A/B)

| Input | Action |
|-------|--------|
| WASD | Pan camera (XZ) |
| RMB drag | Orbit |
| Scroll | Zoom |
| 1 / 2 / 3 | Place / Select / Paint mode |
| Tab | Toggle Place ↔ Select |
| Shift (hold) | Temporary Select when `SelectModeHoldShift` is on |
| Library click | Pick block |
| LMB | Place (Place) / select (Select) / paint face (Paint) |
| LMB drag (>8px) | Marquee select (Select mode) |
| Del | Delete selection (Select mode) |
| Alt + RMB | Delete block under cursor |
| Q / E | Rotate placement (Place) or selection (Select) |
| Ctrl+Z / Ctrl+Y | Undo / redo place, delete, bulk delete, paint |

## Crate layout

See plan: `src/app.rs`, `resources/`, `systems/`, `content/`, `ui/`, `components/`.

## Docs

- [`docs/data_formats.md`](docs/data_formats.md) — `mod.json`, section JSON, meta keys
- [`docs/rotation_invariants.md`](docs/rotation_invariants.md) — pivot rules for grouped/section assemblies
- [`docs/godot_gotchas.md`](docs/godot_gotchas.md) — porting pitfalls from Godot

## Mods

Shipped: `assets/mods/sample_pack/mod.json`. User mods: `~/Library/Application Support/ezcreate/mods/` (via `dirs::data_dir()`).

Scene paths in manifests still reference `.tscn` in Godot; Bevy MVP uses procedural 1m cubes until glTF prefabs land.
