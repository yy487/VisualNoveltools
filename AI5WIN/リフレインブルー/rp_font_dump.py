# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import math
from pathlib import Path
from PIL import Image

from rp_font_common import GLYPH_HEIGHT, GLYPH_WIDTH, decode_glyph_rgba, read_fnt, read_palette, read_tbl


def dump_grid(fnt_path: Path, pal_path: Path, out_png: Path, *, columns: int = 32,
              start: int = 0, count: int | None = None, scale: int = 1) -> None:
    fnt = read_fnt(fnt_path)
    palette = read_palette(pal_path)
    total = len(fnt) // (GLYPH_WIDTH * GLYPH_HEIGHT // 2)
    if count is None:
        count = total - start
    count = max(0, min(count, total - start))
    rows = math.ceil(count / columns) if count else 1
    img = Image.new("RGBA", (columns * GLYPH_WIDTH, rows * GLYPH_HEIGHT), (0, 0, 0, 0))
    for n in range(count):
        idx = start + n
        glyph = Image.new("RGBA", (GLYPH_WIDTH, GLYPH_HEIGHT))
        glyph.putdata(decode_glyph_rgba(fnt, idx, palette))
        x = (n % columns) * GLYPH_WIDTH
        y = (n // columns) * GLYPH_HEIGHT
        img.alpha_composite(glyph, (x, y))
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.Resampling.NEAREST)
    out_png.parent.mkdir(parents=True, exist_ok=True)
    img.save(out_png)
    print(f"dumped glyphs={count}/{total} -> {out_png}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Dump FONT.FNT/FONT.PAL to a PNG grid.")
    ap.add_argument("fnt")
    ap.add_argument("pal")
    ap.add_argument("out_png")
    ap.add_argument("--columns", type=int, default=32)
    ap.add_argument("--start", type=int, default=0)
    ap.add_argument("--count", type=int, default=None)
    ap.add_argument("--scale", type=int, default=1)
    args = ap.parse_args()
    dump_grid(Path(args.fnt), Path(args.pal), Path(args.out_png), columns=args.columns,
              start=args.start, count=args.count, scale=args.scale)


if __name__ == "__main__":
    main()
