#!/usr/bin/env python3
"""Build a nearest-neighbour visual sheet for 16x16 FREECG98 glyphs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


SAMPLES = "我的你是不有这想空健康身体幸福环境黑赶键盘环顾四周"


def carrier_slot(character: str) -> tuple[int, int]:
    encoded = character.encode("cp932")
    if len(encoded) != 2:
        raise ValueError(f"carrier is not double-byte CP932: {character!r}")
    lead, trail = encoded
    row = (lead - (0x81 if lead <= 0x9F else 0xC1)) * 2 + 0x21
    if trail >= 0x9F:
        row += 1
        cell = trail - 0x7E
    elif trail > 0x7F:
        cell = trail - 0x20
    else:
        cell = trail - 0x1F
    return row - 0x20, cell


def mapped_tiles(image: Image.Image, mapping: dict[str, str], text: str) -> list[Image.Image]:
    tiles = []
    for character in text:
        tile_x, tile_y = carrier_slot(mapping[character])
        tiles.append(image.crop((tile_x * 16, tile_y * 16, tile_x * 16 + 16, tile_y * 16 + 16)).convert("1"))
    return tiles


def hinted_tiles(font_path: Path, text: str, size: int) -> list[Image.Image]:
    font = ImageFont.truetype(str(font_path), size)
    tiles = []
    for character in text:
        tile = Image.new("1", (16, 16), 1)
        ImageDraw.Draw(tile).text((0, 0), character, font=font, fill=0, stroke_width=0)
        tiles.append(tile)
    return tiles


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mapping", required=True, type=Path)
    parser.add_argument("--font", required=True, type=Path)
    parser.add_argument("--base", required=True, type=Path)
    parser.add_argument("--before", type=Path)
    parser.add_argument("--rendered", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--text", default=SAMPLES)
    args = parser.parse_args()

    mapping = json.loads(args.mapping.read_text(encoding="utf-8"))
    text = "".join(character for character in args.text if character in mapping)
    rows = [("base carrier", mapped_tiles(Image.open(args.base), mapping, text))]
    if args.before is not None:
        rows.append(("old unhinted", mapped_tiles(Image.open(args.before), mapping, text)))
    rows.extend(
        [
            ("fixed output", mapped_tiles(Image.open(args.rendered), mapping, text)),
            ("approved FT mono 16", hinted_tiles(args.font, text, 16)),
        ]
    )
    scale = 6
    label_width = 150
    row_height = 16 * scale + 28
    sheet = Image.new("RGB", (label_width + len(text) * 16 * scale, len(rows) * row_height), "white")
    draw = ImageDraw.Draw(sheet)
    for row_index, (label, tiles) in enumerate(rows):
        y = row_index * row_height
        draw.text((4, y + 6), label, fill="black")
        for column, tile in enumerate(tiles):
            x = label_width + column * 16 * scale
            enlarged = tile.resize((16 * scale, 16 * scale), Image.Resampling.NEAREST).convert("RGB")
            sheet.paste(enlarged, (x, y))
            draw.rectangle((x, y, x + 16 * scale - 1, y + 16 * scale - 1), outline=(210, 210, 210))
            draw.text((x + 4, y + 16 * scale + 2), f"U+{ord(text[column]):04X}", fill="black")
    sheet.save(args.output)
    print(f"text={text}")
    print(f"output={args.output}")


if __name__ == "__main__":
    main()
