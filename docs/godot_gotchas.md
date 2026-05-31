# Godot gotchas (preserve in Bevy)

1. **Placement input** — Godot uses `_Input` + `InputMap.EventIsAction` on the mouse event so LMB place still works after the library picker steals UI focus. In Bevy: allow viewport placement when egui does not consume the pointer, or clear focus on pick.

2. **`SetMeta` on scenes** — Runtime meta on packed scenes is not persisted (#76366). Use explicit Bevy components on prefabs (`GroupedModuleRoot`, etc.).

3. **Fresh scene load** — `LoadPackedSceneFresh` bypasses cache for `user://` blueprints after save. Bevy: reload asset by path or version stamp.

4. **`PreventOverlappingBlocks`** — Defaults **false**; occupancy map still tracks cells for delete/select.

5. **Copy binding** — `copy` saves selection as a grouped module JSON, not clipboard paste.

6. **Paint** — Only face decals are committed; `PaintTextureEdit` exists in Godot but is unused.

7. **Half-cell ray offset** — `world_hit = hit.position + hit.normal * (GridSize * 0.5)` places on the adjacent cell for stacking.
