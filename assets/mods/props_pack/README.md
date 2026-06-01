# Props Pack

Sample multi-block modules built from unit cubes with per-face solid colors.

| Module | Shape | Colors |
|--------|-------|--------|
| **Bush** | Trunk + cross foliage + top tuft (7 blocks) | Brown bark stem, green leaf clusters (lighter on top) |
| **Fence Section** | Two posts + two horizontal rails (6 blocks, 3×2) | Weathered wood; darker grain on post bases |
| **Small Car** | 4-block chassis + 2-block cabin (6 blocks) | Red body, black tires, blue glass windows, yellow headlight |

## Layout (grid cells, Y up)

### Bush
```
      [L]        y=2  leaf top
  [L][L][L]      y=1  foliage ring
    [T]          y=0  trunk
```

### Fence (X →)
```
P R P   y=1  posts + top rail
P   P   y=0  posts
```

### Car (X → front)
```
  [C][C]     y=1  cabin + windshield (+X on front cabin)
[W][B][B][H] y=0  wheels, body, headlight
```

Place from the **Place** tab in the builder. Section modules rotate with **Q/E** like other multi-block items.

## Thumbnails

Run `python generate_thumbnails.py` (requires Pillow) to regenerate sidebar PNGs.

## Files

- `mod.json` — catalog entries
- `sections/*.json` — block layouts + `facePaints` (local face normals, RGBA brush colors)
- `thumbnails/*.png` — library preview images
