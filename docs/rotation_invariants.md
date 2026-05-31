# Rotation invariants

When porting grouped modules and section blueprints from Godot, **ghost preview and committed placement must use the same transform hierarchy**.

## Single block

- Root `Transform.rotation` = `placement_euler` (snapped to 90° steps).
- One occupancy cell at the root grid key.

## Grouped module / section assembly

1. Compute **bottom pivot**: lowest Y layer of piece offsets (at current yaw), then the cell farthest from that layer's XZ center.
2. Insert a **rotation pivot** at that bottom corner (the anchor cell sits on the pivot).
3. Apply `placement_euler` to the pivot, not per-piece local spins.
4. Register occupancy for **each piece cell** after rotation (`CollectGroupedModulePieceRoots` equivalent).

## Wrong approaches (preview ≠ commit)

- Rotating each piece in place without a shared pivot.
- Placing at `anchor + unrotated offset` while the ghost rotates the full assembly.

## Undo

Store `placement_euler` **at place time** on section edits, not the live placement euler, so redo after rotating the ghost does not desync.
