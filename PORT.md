# ezcreate — Bevy port

Rust/Bevy rebuild of **ezcreate** (Architect-Builder-X): a Townscaper-style, god-view modular building sandbox with JSON-driven mods, 1m grid placement, Place/Select/Paint modes, and undo.

**Godot source:** `/Users/jacobwoodbury/Documents/builder` (ignore `addons/ziva_agent`).

## Status

| Phase | Scope | Status |
|-------|--------|--------|
| **A — MVP** | Orbit camera, ground, mod scan, single-block place/delete, ghost, Q/E yaw, egui modes, undo | **In progress** (scaffolded in this repo) |
| **B — Parity** | Selection, sections, grouped modules, paint, settings | Not started |
| **C — Roadmap** | Sockets, WFC, texture paint | Not started |

## Run

```sh
cargo run
```

## Controls (Phase A)

| Input | Action |
|-------|--------|
| WASD | Pan camera (XZ) |
| RMB drag | Orbit |
| Scroll | Zoom |
| 1 / 2 / 3 | Place / Select / Paint mode |
| Tab | Toggle Place ↔ Select |
| Shift (hold) | Temporary Select when `SelectModeHoldShift` is on |
| Library click | Pick block |
| LMB | Place (Place mode) |
| Alt + RMB | Delete block |
| Q / E | Rotate placement yaw |
| Ctrl+Z / Ctrl+Y | Undo / redo place & delete |

## Crate layout

See plan: `src/app.rs`, `resources/`, `systems/`, `content/`, `ui/`, `components/`.

## Docs

- [`docs/data_formats.md`](docs/data_formats.md) — `mod.json`, section JSON, meta keys
- [`docs/rotation_invariants.md`](docs/rotation_invariants.md) — pivot rules for grouped/section assemblies
- [`docs/godot_gotchas.md`](docs/godot_gotchas.md) — porting pitfalls from Godot

## Mods

Shipped: `assets/mods/sample_pack/mod.json`. User mods: `~/Library/Application Support/ezcreate/mods/` (via `dirs::data_dir()`).

Scene paths in manifests still reference `.tscn` in Godot; Bevy MVP uses procedural 1m cubes until glTF prefabs land.
