#!/usr/bin/env python3
"""Render 4 KiB NES CHR banks as indexed tile contact sheets."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


TILE_BYTES = 16
BANK_BYTES = 0x1000
TILES_PER_BANK = BANK_BYTES // TILE_BYTES
TILE_SIDE = 8
SCALE = 3
PALETTE = ((238, 238, 238), (160, 160, 160), (82, 82, 82), (12, 12, 12))


def decode_tile(data: bytes) -> list[list[int]]:
    if len(data) != TILE_BYTES:
        raise ValueError("an NES 2bpp tile must be exactly 16 bytes")
    pixels = [[0] * TILE_SIDE for _ in range(TILE_SIDE)]
    for y in range(TILE_SIDE):
        low, high = data[y], data[y + TILE_SIDE]
        for x in range(TILE_SIDE):
            bit = 7 - x
            pixels[y][x] = ((low >> bit) & 1) | (((high >> bit) & 1) << 1)
    return pixels


def render_group(chr_data: bytes, first_bank: int, output: Path) -> None:
    tile_gap = 1
    tile_step = TILE_SIDE + tile_gap
    page_width = 16 * tile_step - tile_gap
    page_height = 18 + 16 * tile_step - tile_gap
    margin, page_gap = 8, 10
    sheet_width = 4 * page_width + 3 * page_gap + 2 * margin
    sheet_height = 4 * page_height + 3 * page_gap + 2 * margin
    sheet = Image.new("RGB", (sheet_width, sheet_height), (48, 48, 48))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.load_default()

    page_count = min(16, len(chr_data) // BANK_BYTES - first_bank)
    for page in range(page_count):
        bank = first_bank + page
        col, row = page % 4, page // 4
        x0 = margin + col * (page_width + page_gap)
        y0 = margin + row * (page_height + page_gap)
        draw.text((x0, y0), f"B{bank:02X} offset {bank * BANK_BYTES:05X}", font=font, fill=(255, 255, 255))
        bank_offset = bank * BANK_BYTES
        for local_tile in range(TILES_PER_BANK):
            tile_offset = bank_offset + local_tile * TILE_BYTES
            tile = decode_tile(chr_data[tile_offset : tile_offset + TILE_BYTES])
            tx = x0 + (local_tile % 16) * tile_step
            ty = y0 + 18 + (local_tile // 16) * tile_step
            for py, line in enumerate(tile):
                for px, color in enumerate(line):
                    sheet.putpixel((tx + px, ty + py), PALETTE[color])

    sheet.resize((sheet_width * SCALE, sheet_height * SCALE), Image.Resampling.NEAREST).save(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("chr_rom", type=Path)
    parser.add_argument("output_dir", type=Path)
    args = parser.parse_args()

    chr_data = args.chr_rom.read_bytes()
    if not chr_data or len(chr_data) % BANK_BYTES:
        raise SystemExit("CHR-ROM size must be a non-zero multiple of 4 KiB")
    if len(chr_data) % TILE_BYTES:
        raise SystemExit("CHR-ROM size must be a multiple of 16 bytes")

    bank_count = len(chr_data) // BANK_BYTES
    outputs = [
        f"chr_4k_banks_{first_bank:02x}-{min(first_bank + 15, bank_count - 1):02x}.png"
        for first_bank in range(0, bank_count, 16)
    ]
    summary_path = args.output_dir / "chr_summary.json"
    targets = [args.output_dir / name for name in outputs] + [summary_path]
    existing = [path for path in targets if path.exists()]
    if existing:
        raise SystemExit("refusing to overwrite " + ", ".join(str(path) for path in existing))
    args.output_dir.mkdir(parents=True, exist_ok=True)
    for first_bank in range(0, bank_count, 16):
        output = args.output_dir / outputs[first_bank // 16]
        render_group(chr_data, first_bank, output)

    banks = []
    for bank in range(bank_count):
        start = bank * BANK_BYTES
        tiles = [chr_data[start + i * TILE_BYTES : start + (i + 1) * TILE_BYTES] for i in range(TILES_PER_BANK)]
        banks.append(
            {
                "bank_4k": bank,
                "offset": start,
                "tile_start": start // TILE_BYTES,
                "tile_count": len(tiles),
                "blank_tiles": sum(not any(tile) for tile in tiles),
                "unique_patterns": len(set(tiles)),
            }
        )
    summary = {
        "format": "NES 2bpp planar tiles",
        "chr_size": len(chr_data),
        "tile_size": TILE_BYTES,
        "tile_count": len(chr_data) // TILE_BYTES,
        "bank_size": BANK_BYTES,
        "bank_count_4k": bank_count,
        "blank_tiles": sum(bank["blank_tiles"] for bank in banks),
        "unique_patterns": len(set(chr_data[i : i + TILE_BYTES] for i in range(0, len(chr_data), TILE_BYTES))),
        "palette": [list(color) for color in PALETTE],
        "atlases": outputs,
        "banks": banks,
    }
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"tiles={summary['tile_count']} banks_4k={bank_count} blank_tiles={summary['blank_tiles']} unique_patterns={summary['unique_patterns']}")
    for name in outputs:
        print(args.output_dir / name)
    print(summary_path)


if __name__ == "__main__":
    main()
