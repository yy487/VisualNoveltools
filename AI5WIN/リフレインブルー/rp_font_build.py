# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from PIL import Image

from rp_font_common import GLYPH_HEIGHT, GLYPH_WIDTH, split_char_list_text, write_palette, write_tbl


def build_font(char_list_path: Path, indexed_png: Path, out_fnt: Path, out_pal: Path, out_tbl: Path,
               *, start_x: int = 12, start_y: int = 12, margin_x: int = 4, margin_y: int = 0,
               columns: int | None = None, encoding: str = "cp932") -> None:
    text = char_list_path.read_text(encoding="utf-8")
    lines = [line.rstrip("\r\n") for line in text.splitlines() if line.strip()]
    flat_chars = split_char_list_text(text)
    img = Image.open(indexed_png)
    if img.mode != "P":
        raise ValueError("input PNG must be an indexed-color image (mode P) with a 16-color palette")
    pal = img.getpalette() or []
    write_palette(out_pal, pal[:16 * 3])

    pixels = img.load()
    font = bytearray()
    chars_for_tbl = []
    cell_w = GLYPH_WIDTH + margin_x
    cell_h = GLYPH_HEIGHT + margin_y

    if columns is not None:
        # Read a flat char list laid out in fixed columns.
        for idx, ch in enumerate(flat_chars):
            row = idx // columns
            col = idx % columns
            chars_for_tbl.append(ch)
            x0 = start_x + col * cell_w
            y0 = start_y + row * cell_h
            for y in range(y0, y0 + GLYPH_HEIGHT):
                for x in range(x0, x0 + GLYPH_WIDTH, 2):
                    hi = pixels[x, y] & 0x0F
                    lo = pixels[x + 1, y] & 0x0F
                    font.append((hi << 4) | lo)
    else:
        # Preserve the line breaks in char_list.txt.
        for row, line in enumerate(lines):
            line_chars = split_char_list_text(line)
            for col, ch in enumerate(line_chars):
                chars_for_tbl.append(ch)
                x0 = start_x + col * cell_w
                y0 = start_y + row * cell_h
                for y in range(y0, y0 + GLYPH_HEIGHT):
                    for x in range(x0, x0 + GLYPH_WIDTH, 2):
                        hi = pixels[x, y] & 0x0F
                        lo = pixels[x + 1, y] & 0x0F
                        font.append((hi << 4) | lo)

    out_fnt.parent.mkdir(parents=True, exist_ok=True)
    out_fnt.write_bytes(bytes(font))
    write_tbl(out_tbl, chars_for_tbl, encoding)
    print(f"chars={len(chars_for_tbl)} fnt_size={len(font)} -> {out_fnt}, {out_pal}, {out_tbl}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Build FONT.FNT/FONT.PAL/FONT.TBL from indexed char_list.png.")
    ap.add_argument("char_list_txt")
    ap.add_argument("indexed_png")
    ap.add_argument("out_fnt")
    ap.add_argument("out_pal")
    ap.add_argument("out_tbl")
    ap.add_argument("--start-x", type=int, default=12)
    ap.add_argument("--start-y", type=int, default=12)
    ap.add_argument("--margin-x", type=int, default=4)
    ap.add_argument("--margin-y", type=int, default=0)
    ap.add_argument("--columns", type=int, default=None, help="Use fixed flat column layout instead of existing line breaks")
    ap.add_argument("--encoding", default="cp932")
    args = ap.parse_args()
    build_font(Path(args.char_list_txt), Path(args.indexed_png), Path(args.out_fnt), Path(args.out_pal), Path(args.out_tbl),
               start_x=args.start_x, start_y=args.start_y, margin_x=args.margin_x, margin_y=args.margin_y,
               columns=args.columns, encoding=args.encoding)


if __name__ == "__main__":
    main()
