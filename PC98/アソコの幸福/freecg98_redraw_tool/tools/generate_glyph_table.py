#!/usr/bin/env python3
"""Generate the deterministic 16x16 monochrome glyph table embedded by Rust."""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


MAGIC = b"FCG1"
EXPECTED_FONT_SHA256 = "b9c054fd31c2125dba9e8cd3d269146937de69f07dcc10f5f53204c4d98e6b39"
EXPECTED_MAPPING_ENTRIES = 3025
EXPECTED_YING_ROWS = (
    ".......#........",
    "........#.......",
    "..#############.",
    "..#.............",
    "..#.............",
    "..#....#.....#..",
    "..#.#...#....#..",
    "..#..#..#....#..",
    "..#..#...#..#...",
    "..#...#..#..#...",
    "..#...#....#....",
    "..#...#....#....",
    ".#........#.....",
    ".#.......#......",
    "#..############.",
    "................",
)


def render_tile(font: ImageFont.FreeTypeFont, character: str) -> bytes:
    bounds = font.getbbox(character)
    if bounds[0] < 0 or bounds[1] < 0 or bounds[2] > 16 or bounds[3] > 16:
        raise ValueError(f"U+{ord(character):04X} {character!r} exceeds 16x16: {bounds}")
    tile = Image.new("1", (16, 16), 1)
    ImageDraw.Draw(tile).text((0, 0), character, font=font, fill=0, stroke_width=0)
    rows = bytearray()
    black_pixels = 0
    for y in range(16):
        row = 0
        for x in range(16):
            if tile.getpixel((x, y)) == 0:
                row |= 1 << (15 - x)
                black_pixels += 1
        rows.extend(row.to_bytes(2, "big"))
    if black_pixels == 0:
        raise ValueError(f"U+{ord(character):04X} {character!r} rendered empty")
    return bytes(rows)


def rows_as_text(bitmap: bytes) -> tuple[str, ...]:
    rows = []
    for y in range(16):
        value = int.from_bytes(bitmap[y * 2 : y * 2 + 2], "big")
        rows.append("".join("#" if value & (1 << (15 - x)) else "." for x in range(16)))
    return tuple(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mapping", required=True, type=Path)
    parser.add_argument("--font", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    if args.output.exists() and not args.overwrite:
        raise FileExistsError(f"output exists: {args.output}")
    font_bytes = args.font.read_bytes()
    font_sha256 = hashlib.sha256(font_bytes).hexdigest()
    if font_sha256 != EXPECTED_FONT_SHA256:
        raise ValueError(f"unexpected source font SHA-256: {font_sha256}")
    mapping = json.loads(args.mapping.read_text(encoding="utf-8"))
    if len(mapping) != EXPECTED_MAPPING_ENTRIES:
        raise ValueError(f"expected {EXPECTED_MAPPING_ENTRIES} mappings, found {len(mapping)}")

    font = ImageFont.truetype(str(args.font), 16)
    records = []
    for character in sorted(mapping, key=ord):
        if len(character) != 1:
            raise ValueError(f"mapping target is not one character: {character!r}")
        records.append((character, render_tile(font, character)))
    ying = dict(records).get("应")
    if ying is None or rows_as_text(ying) != EXPECTED_YING_ROWS:
        raise ValueError("FreeType monochrome output does not match the approved U+5E94 baseline")

    output = bytearray(MAGIC)
    output.extend(struct.pack("<I", len(records)))
    for character, bitmap in records:
        output.extend(struct.pack("<I", ord(character)))
        output.extend(bitmap)
    temporary = args.output.with_name(f".{args.output.name}.tmp")
    temporary.write_bytes(output)
    temporary.replace(args.output)
    print(f"glyphs={len(records)}")
    print(f"bytes={len(output)}")
    print(f"font_sha256={font_sha256}")
    print(f"output={args.output}")


if __name__ == "__main__":
    main()
