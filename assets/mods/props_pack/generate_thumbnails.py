#!/usr/bin/env python3
"""Generate isometric-style thumbnail PNGs for props_pack modules."""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parent / "thumbnails"
OUT.mkdir(parents=True, exist_ok=True)

BARK = (101, 67, 33, 255)
BARK_D = (68, 45, 20, 255)
LEAF = (58, 128, 52, 255)
LEAF_L = (88, 158, 68, 255)
WOOD = (168, 128, 82, 255)
WOOD_D = (118, 86, 48, 255)
WOOD_L = (188, 148, 98, 255)
CAR = (205, 52, 45, 255)
CAR_D = (155, 38, 34, 255)
TIRE = (32, 32, 35, 255)
GLASS = (120, 175, 200, 255)
LIGHT = (240, 230, 180, 255)
BG = (48, 58, 52, 255)


def iso_block(draw, ox, oy, size, top, left, right):
    h = size // 2
    w = size
    draw.polygon(
        [(ox, oy - h), (ox + w, oy), (ox, oy + h), (ox - w, oy)],
        fill=top,
    )
    draw.polygon(
        [(ox - w, oy), (ox, oy + h), (ox, oy + h + size), (ox - w, oy + size)],
        fill=left,
    )
    draw.polygon(
        [(ox + w, oy), (ox, oy + h), (ox, oy + h + size), (ox + w, oy + size)],
        fill=right,
    )


def save(name, draw_fn):
    img = Image.new("RGBA", (64, 64), BG)
    draw = ImageDraw.Draw(img)
    draw_fn(draw)
    path = OUT / f"{name}.png"
    img.save(path)
    print(f"Wrote {path}")


def draw_bush(d):
    s = 7
    cx, base = 32, 44
    iso_block(d, cx, base - s * 2, s, BARK, BARK_D, BARK)
    for dx in (-1, 0, 1):
        for dz in (-1, 0, 1):
            if dx == 0 and dz == 0:
                continue
            iso_block(d, cx + dx * s * 2, base - s * 4, s, LEAF_L, LEAF, LEAF)
    iso_block(d, cx, base - s * 4, s, LEAF_L, LEAF, LEAF)
    iso_block(d, cx, base - s * 6, s, LEAF_L, LEAF_L, LEAF)


def draw_fence(d):
    s = 6
    base = 46
    for px in (18, 32, 46):
        iso_block(d, px, base, s, WOOD_L, WOOD_D, WOOD)
        iso_block(d, px, base - s * 2, s, WOOD_L, WOOD, WOOD)
    iso_block(d, 32, base - s, s, WOOD, WOOD_D, WOOD)
    iso_block(d, 32, base - s * 3, s, WOOD_L, WOOD, WOOD)


def draw_car(d):
    s = 5
    base = 48
    iso_block(d, 14, base, s, TIRE, TIRE, TIRE)
    iso_block(d, 50, base, s, LIGHT, TIRE, TIRE)
    for px in (24, 32, 40):
        iso_block(d, px, base, s, CAR, CAR_D, CAR)
    iso_block(d, 28, base - s * 2, s, CAR, GLASS, GLASS)
    iso_block(d, 36, base - s * 2, s, GLASS, CAR, GLASS)


if __name__ == "__main__":
    save("bush", draw_bush)
    save("fence_section", draw_fence)
    save("small_car", draw_car)
