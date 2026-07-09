#!/usr/bin/env python3
# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


GLYPH_W = 24
GLYPH_H = 34
GLYPH_SIZE = GLYPH_W * GLYPH_H // 2

TBL_OFF = 0x18C600
FNT_OFF = 0x18FE00
FNT_REGION_SIZE = 0x2C0400


def read_tbl(path: Path) -> list[str]:
    data = path.read_bytes()
    chars: list[str] = []
    for i in range(0, len(data) - 1, 2):
        lo, hi = data[i], data[i + 1]
        if lo == 0 and hi == 0:
            break
        chars.append(bytes([hi, lo]).decode("cp932", errors="replace"))
    return chars


def load_palette(path: Path) -> bytes:
    data = path.read_bytes()
    if len(data) < 48:
        raise ValueError(f"PAL too small: {path}")
    return data[:48]


def palette_list(pal48: bytes) -> list[tuple[int, int, int]]:
    return [tuple(pal48[i : i + 3]) for i in range(0, 48, 3)]


def reverse_mapping(path: Path, extra_path: Path | None = None) -> dict[str, str]:
    cn_to_jp = json.loads(path.read_text(encoding="utf-8"))
    if extra_path and extra_path.exists():
        extra = json.loads(extra_path.read_text(encoding="utf-8"))
        cn_to_jp.update(extra)
    return {jp: cn for cn, jp in cn_to_jp.items()}


def font_cmap(path: Path) -> set[int]:
    try:
        from fontTools.ttLib import TTFont
    except ImportError as exc:
        raise RuntimeError("fontTools is required for missing-glyph detection") from exc
    try:
        tt = TTFont(str(path), fontNumber=0, lazy=True)
    except TypeError:
        tt = TTFont(str(path), lazy=True)
    out: set[int] = set()
    for table in tt["cmap"].tables:
        if table.platformID != 1:
            out.update(table.cmap.keys())
    return out


def parse_font_list(spec: str) -> list[Path]:
    return [Path(item.strip()) for item in spec.split(",") if item.strip()]


def fill_idx_oldlike(alpha: int) -> int:
    # Keep white only in the dense stroke center. Low-alpha fill edges are left
    # to the black outline, matching the original FNT more closely.
    if alpha >= 235:
        return 1
    if alpha >= 205:
        return 2
    if alpha >= 175:
        return 3
    if alpha >= 145:
        return 4
    return 0


def fill_idx_strong(alpha: int) -> int:
    # Brighter fill used by the earlier "simhei strong" full-font preview.
    if alpha >= 220:
        return 1
    if alpha >= 170:
        return 2
    if alpha >= 120:
        return 4
    if alpha >= 70:
        return 7
    if alpha >= 28:
        return 10
    return 0


def edge_idx(alpha: int) -> int:
    if alpha >= 180:
        return 15
    if alpha >= 125:
        return 14
    if alpha >= 80:
        return 13
    if alpha >= 45:
        return 11
    if alpha >= 20:
        return 9
    return 0


class GlyphRenderer:
    def __init__(self, font_path: Path, font_size: int = 22, outline: int = 2, scale: int = 4, style: str = "oldlike"):
        self.scale = scale
        self.outline = outline
        if style not in {"oldlike", "strong"}:
            raise ValueError("style must be oldlike or strong")
        self.style = style
        self.font = ImageFont.truetype(str(font_path), font_size * scale)
        probe = Image.new("L", (GLYPH_W * scale * 4, GLYPH_H * scale * 4), 0)
        self.probe_draw = ImageDraw.Draw(probe)
        stroke = outline * scale
        ref = self.probe_draw.textbbox((0, 0), "中", font=self.font, stroke_width=stroke)
        self.common_y = 1 * scale - ref[1]

    def render_indices(self, ch: str) -> list[int]:
        w, h = GLYPH_W * self.scale, GLYPH_H * self.scale
        outline = Image.new("L", (w, h), 0)
        fill = Image.new("L", (w, h), 0)
        outline_draw = ImageDraw.Draw(outline)
        fill_draw = ImageDraw.Draw(fill)
        stroke = self.outline * self.scale
        bbox = self.probe_draw.textbbox((0, 0), ch, font=self.font, stroke_width=stroke)
        text_w = bbox[2] - bbox[0]
        x = (w - text_w) // 2 - bbox[0]
        outline_draw.text((x, self.common_y), ch, font=self.font, fill=255, stroke_width=stroke, stroke_fill=255)
        fill_draw.text((x, self.common_y), ch, font=self.font, fill=255)
        outline = outline.resize((GLYPH_W, GLYPH_H), Image.Resampling.LANCZOS)
        fill = fill.resize((GLYPH_W, GLYPH_H), Image.Resampling.LANCZOS)

        pixels: list[int] = []
        for y in range(GLYPH_H):
            for x0 in range(GLYPH_W):
                f = fill.getpixel((x0, y))
                o = outline.getpixel((x0, y))
                pi = fill_idx_strong(f) if self.style == "strong" else fill_idx_oldlike(f)
                if pi == 0:
                    pi = edge_idx(o)
                pixels.append(pi)
        return pixels


def pack_glyph(indices: list[int]) -> bytes:
    if len(indices) != GLYPH_W * GLYPH_H:
        raise ValueError("bad glyph pixel count")
    out = bytearray()
    for i in range(0, len(indices), 2):
        out.append((indices[i] << 4) | indices[i + 1])
    return bytes(out)


def decode_glyph(fnt: bytes, idx: int) -> list[int]:
    data = fnt[idx * GLYPH_SIZE : (idx + 1) * GLYPH_SIZE]
    if len(data) != GLYPH_SIZE:
        raise IndexError(idx)
    out: list[int] = []
    for b in data:
        out.append((b >> 4) & 0x0F)
        out.append(b & 0x0F)
    return out


def glyph_image(indices: list[int], colors: list[tuple[int, int, int]], scale: int) -> Image.Image:
    rgba = []
    for pi in indices:
        if pi == 0:
            rgba.append((0, 0, 0, 0))
        else:
            r, g, b = colors[pi]
            rgba.append((r, g, b, 255))
    img = Image.new("RGBA", (GLYPH_W, GLYPH_H), (0, 0, 0, 0))
    img.putdata(rgba)
    return img.resize((GLYPH_W * scale, GLYPH_H * scale), Image.Resampling.NEAREST)


def make_checker(w: int, h: int, step: int = 10) -> Image.Image:
    img = Image.new("RGBA", (w, h), (42, 45, 48, 255))
    draw = ImageDraw.Draw(img)
    for y in range(0, h, step):
        for x in range(0, w, step):
            if (x // step + y // step) % 2 == 0:
                draw.rectangle([x, y, x + step - 1, y + step - 1], fill=(58, 61, 64, 255))
    return img


def patch_fnt(
    source_fnt: Path,
    source_tbl: Path,
    mapping: Path,
    font: Path,
    fallback_fonts: list[Path],
    extra_mapping: Path | None,
    out_fnt: Path,
    indices: list[int],
    font_size: int,
    outline: int,
    style: str,
    fallback_missing: bool,
) -> dict:
    chars = read_tbl(source_tbl)
    if source_fnt.stat().st_size != len(chars) * GLYPH_SIZE:
        raise ValueError("FNT/TBL glyph count mismatch")
    jp_to_cn = reverse_mapping(mapping, extra_mapping)
    renderer = GlyphRenderer(font, font_size=font_size, outline=outline, style=style)
    cmap = font_cmap(font)
    fallback_renderers = []
    for fallback_font in fallback_fonts:
        if not fallback_font.exists():
            continue
        fallback_cmap = font_cmap(fallback_font)
        if fallback_cmap:
            fallback_renderers.append(
                (
                    fallback_font,
                    fallback_cmap,
                    GlyphRenderer(fallback_font, font_size=font_size, outline=outline, style=style),
                )
            )
    data = bytearray(source_fnt.read_bytes())
    patched = []
    fallback_font = []
    fallback = []
    for idx in indices:
        tbl_ch = chars[idx]
        draw_ch = jp_to_cn.get(tbl_ch, tbl_ch)
        if fallback_missing and cmap and ord(draw_ch) not in cmap:
            fallback_renderer = None
            for fallback_font_path, fallback_cmap, candidate_renderer in fallback_renderers:
                if ord(draw_ch) in fallback_cmap:
                    fallback_renderer = (fallback_font_path, candidate_renderer)
                    break
            if fallback_renderer is None:
                fallback.append({"idx": idx, "tbl": tbl_ch, "draw": draw_ch})
                continue
            fallback_font_path, candidate_renderer = fallback_renderer
            glyph = pack_glyph(candidate_renderer.render_indices(draw_ch))
            data[idx * GLYPH_SIZE : (idx + 1) * GLYPH_SIZE] = glyph
            fallback_font.append({"idx": idx, "tbl": tbl_ch, "draw": draw_ch, "font": str(fallback_font_path)})
            continue
        glyph = pack_glyph(renderer.render_indices(draw_ch))
        data[idx * GLYPH_SIZE : (idx + 1) * GLYPH_SIZE] = glyph
        patched.append({"idx": idx, "tbl": tbl_ch, "draw": draw_ch})
    out_fnt.parent.mkdir(parents=True, exist_ok=True)
    out_fnt.write_bytes(bytes(data))
    return {
        "patched": patched,
        "fallback_font": fallback_font,
        "fallback_original": fallback,
        "sha256_fnt": hashlib.sha256(bytes(data)).hexdigest(),
    }


def inject_fnt(base_exe: Path, source_tbl: Path, patched_fnt: Path, out_exe: Path) -> dict:
    exe = bytearray(base_exe.read_bytes())
    tbl = source_tbl.read_bytes()
    fnt = patched_fnt.read_bytes()
    if exe[TBL_OFF : TBL_OFF + len(tbl)] != tbl:
        raise ValueError("base EXE TBL region does not match source FONT.TBL")
    if len(fnt) > FNT_REGION_SIZE:
        raise ValueError("patched FNT exceeds reserved EXE region")
    exe[FNT_OFF : FNT_OFF + FNT_REGION_SIZE] = fnt + b"\x00" * (FNT_REGION_SIZE - len(fnt))
    out_exe.parent.mkdir(parents=True, exist_ok=True)
    out_exe.write_bytes(bytes(exe))
    return {"sha256_exe": hashlib.sha256(bytes(exe)).hexdigest(), "exe_size": len(exe)}


def preview_cluster(
    old_fnt: Path,
    new_fnt: Path,
    tbl: Path,
    pal: Path,
    out_png: Path,
    indices: list[int],
    scale: int = 5,
) -> None:
    chars = read_tbl(tbl)
    colors = palette_list(load_palette(pal))
    old_data = old_fnt.read_bytes()
    new_data = new_fnt.read_bytes()
    label_font = ImageFont.load_default()
    try:
        label_font = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 15)
    except OSError:
        pass
    cell_w, cell_h = GLYPH_W * scale, GLYPH_H * scale
    canvas = Image.new("RGBA", (170 + cell_w * 2 + 40, 35 + len(indices) * (cell_h + 16)), (22, 24, 28, 255))
    draw = ImageDraw.Draw(canvas)
    draw.text((170, 10), "old", font=label_font, fill=(235, 235, 235, 255))
    draw.text((170 + cell_w, 10), "fixed", font=label_font, fill=(235, 235, 235, 255))
    for row, idx in enumerate(indices):
        y = 35 + row * (cell_h + 16)
        label = f"{idx} U+{ord(chars[idx]):04X}"
        draw.text((8, y + cell_h // 2 - 8), label, font=label_font, fill=(235, 235, 235, 255))
        for col, data in enumerate((old_data, new_data)):
            bg = make_checker(cell_w, cell_h)
            bg.alpha_composite(glyph_image(decode_glyph(data, idx), colors, scale))
            canvas.alpha_composite(bg, (170 + col * cell_w, y))
    out_png.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert("RGB").save(out_png)


def parse_indices(spec: str) -> list[int]:
    result: list[int] = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            left, right = part.split("-", 1)
            result.extend(range(int(left, 0), int(right, 0) + 1))
        else:
            result.append(int(part, 0))
    return sorted(dict.fromkeys(result))


def compact_report(info: dict) -> dict:
    out = dict(info)
    patched = out.get("patched")
    fallback_font = out.get("fallback_font")
    fallback_original = out.get("fallback_original")
    if isinstance(patched, list) and isinstance(fallback_font, list):
        out["redrawn_count"] = len(patched) + len(fallback_font)
    if isinstance(fallback_original, list):
        out["fallback_original_count"] = len(fallback_original)
    for key in ("patched", "fallback_font", "fallback_original"):
        rows = out.get(key)
        if isinstance(rows, list) and len(rows) > 20:
            out[key + "_count"] = len(rows)
            out[key + "_sample"] = rows[:10]
            del out[key]
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Refrain Blue AI5WIN FNT repair/inject helper.")
    ap.add_argument("--root", default="..", help="Project root. Default is parent of the current working directory.")
    ap.add_argument("--source-dir", default="../source", help="Directory containing FONT.FNT/FONT.TBL/FONT.PAL and subs_cn_jp.json")
    ap.add_argument("--base-exe", default="../source/AI5WIN_chs.exe")
    ap.add_argument("--extra-mapping", default="../source/extra_cn_jp.json", help="Optional extra CN -> JP mapping override")
    ap.add_argument("--font", default="C:/Windows/Fonts/simhei.ttf")
    ap.add_argument("--fallback-fonts", default="C:/Windows/Fonts/msgothic.ttc,C:/Windows/Fonts/meiryo.ttc", help="Comma-separated fonts used when the primary font lacks a glyph")
    ap.add_argument("--indices", default="472-479")
    ap.add_argument("--font-size", type=int, default=22)
    ap.add_argument("--outline", type=int, default=2)
    ap.add_argument("--style", choices=["oldlike", "strong"], default="oldlike")
    ap.add_argument("--full-redraw", action="store_true", help="Redraw every TBL slot with the selected font/style")
    ap.add_argument("--no-fallback-missing", action="store_true", help="Render missing font chars instead of preserving source glyphs")
    ap.add_argument("--out-fnt", default="../build/FONT_fix_detected_cluster.FNT")
    ap.add_argument("--out-exe", default="../output/AI5WIN_chs_oldfnt_fix_detected_cluster.exe")
    ap.add_argument("--preview", default="../preview/detected_cluster_old_vs_fixed.png")
    args = ap.parse_args()

    # Do not call resolve() here. In some non-Unicode console launches,
    # resolved Japanese paths come back as mojibake. Keep relative paths and
    # let the OS cwd handle them. Run this script from tools/.
    root = Path(args.root)
    source_dir = Path(args.source_dir)
    source_fnt = source_dir / "FONT.FNT"
    source_tbl = source_dir / "FONT.TBL"
    source_pal = source_dir / "FONT.PAL"
    mapping = source_dir / "subs_cn_jp.json"
    extra_mapping = Path(args.extra_mapping)
    base_exe = Path(args.base_exe)
    out_fnt = Path(args.out_fnt) if Path(args.out_fnt).is_absolute() else (root / "build" / Path(args.out_fnt).name)
    out_exe = Path(args.out_exe) if Path(args.out_exe).is_absolute() else (root / "output" / Path(args.out_exe).name)
    preview = Path(args.preview) if Path(args.preview).is_absolute() else (root / "preview" / Path(args.preview).name)
    indices = list(range(len(read_tbl(source_tbl)))) if args.full_redraw else parse_indices(args.indices)

    patch_info = patch_fnt(
        source_fnt=source_fnt,
        source_tbl=source_tbl,
        mapping=mapping,
        font=Path(args.font),
        fallback_fonts=parse_font_list(args.fallback_fonts),
        extra_mapping=extra_mapping,
        out_fnt=out_fnt,
        indices=indices,
        font_size=args.font_size,
        outline=args.outline,
        style=args.style,
        fallback_missing=not args.no_fallback_missing,
    )
    inject_info = inject_fnt(base_exe=base_exe, source_tbl=source_tbl, patched_fnt=out_fnt, out_exe=out_exe)
    preview_cluster(source_fnt, out_fnt, source_tbl, source_pal, preview, indices)

    report = compact_report({**patch_info, **inject_info, "out_fnt": str(out_fnt), "out_exe": str(out_exe), "preview": str(preview)})
    print(json.dumps(report, ensure_ascii=True, indent=2))


if __name__ == "__main__":
    main()
