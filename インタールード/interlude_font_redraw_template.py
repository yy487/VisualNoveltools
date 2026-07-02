# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import unicodedata
from pathlib import Path
from typing import Optional

from PIL import Image, ImageDraw, ImageFont

ENTRY_COUNT = 11808
GLYPH_W = 24
GLYPH_H = 24
RAW_SIZE = GLYPH_W * GLYPH_H // 4
TABLE_SIZE = ENTRY_COUNT * 4
FONT_BUFFER_LIMIT = 0x145000


def index_to_char(index: int) -> Optional[str]:
    if 0 <= index < 224:
        try:
            return bytes([index + 0x20]).decode("cp932")
        except UnicodeDecodeError:
            return None
    if 224 <= index < 288:
        return None
    if 288 <= index < 6240:
        n = index - 288
        lead = 0x81 + n // 192
        trail = 0x40 + n % 192
        try:
            return bytes([lead, trail]).decode("cp932")
        except UnicodeDecodeError:
            return None
    if 6240 <= index < ENTRY_COUNT:
        n = index - 6240
        lead = 0xE0 + n // 192
        trail = 0x40 + n % 192
        try:
            return bytes([lead, trail]).decode("cp932")
        except UnicodeDecodeError:
            return None
    return None


def load_cnjp(path: Optional[Path]) -> dict[str, str]:
    if path is None:
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    rev: dict[str, str] = {}
    for k, v in data.items():
        if isinstance(k, str) and isinstance(v, str) and len(k) == 1 and len(v) == 1:
            rev.setdefault(v, k)
    return rev


def decode_chunk(data: bytes, off: int, flag: int) -> bytes:
    lit_ptr = off + int.from_bytes(data[off:off + 4], "little")
    ctrl_ptr = off + 4
    out = bytearray()
    remain = RAW_SIZE
    bits = 0xFFFF
    shift = flag & 0xF
    mask = (1 << shift) - 1
    ext_mark = -1 if flag < 128 else mask
    ext_ptr = lit_ptr
    while remain > 0:
        if bits == 0xFFFF:
            bits = int.from_bytes(data[ctrl_ptr:ctrl_ptr + 2], "little") | 0xFFFF0000
            ctrl_ptr += 2
        if bits & 1:
            out.append(data[lit_ptr])
            lit_ptr += 1
            ext_ptr = lit_ptr
            remain -= 1
        else:
            code = int.from_bytes(data[ctrl_ptr:ctrl_ptr + 2], "little")
            ctrl_ptr += 2
            length = code & mask
            dist = code >> shift
            if dist == 0:
                dist = int.from_bytes(data[ctrl_ptr:ctrl_ptr + 2], "little")
                ctrl_ptr += 2
            if length == ext_mark:
                length = ext_mark + data[ext_ptr]
                ext_ptr += 1
            count = length + 3
            src = len(out) - dist
            for i in range(count):
                out.append(out[src + i])
            remain -= count
            lit_ptr = ext_ptr
        bits >>= 1
    return bytes(out)


def compress_raw_lzss(raw: bytes) -> bytes:
    tokens = []
    literals = bytearray()
    i = 0
    n = len(raw)
    while i < n:
        best_len = 0
        best_dist = 0
        max_dist = min(i, 511)
        for dist in range(1, max_dist + 1):
            j = i - dist
            l = 0
            while i + l < n and raw[j + l] == raw[i + l] and l < 130:
                l += 1
            if l > best_len and l >= 3:
                best_len = l
                best_dist = dist
                if l >= 130:
                    break
        if best_len >= 3:
            tokens.append((0, best_dist, best_len))
            i += best_len
        else:
            tokens.append((1, raw[i], 0))
            literals.append(raw[i])
            i += 1

    cmd = bytearray()
    for gstart in range(0, len(tokens), 16):
        group = tokens[gstart:gstart + 16]
        control = 0
        refs = bytearray()
        for bit, tok in enumerate(group):
            if tok[0] == 1:
                control |= 1 << bit
            else:
                _, dist, length = tok
                refs += ((dist << 7) | (length - 3)).to_bytes(2, "little")
        cmd += control.to_bytes(2, "little") + refs
    lit_start = 4 + len(cmd)
    return lit_start.to_bytes(4, "little") + bytes(cmd) + bytes(literals)


def raw_to_img(raw: bytes) -> Image.Image:
    img = Image.new("L", (GLYPH_W, GLYPH_H), 0)
    px = img.load()
    i = 0
    for y in range(GLYPH_H):
        for xb in range(6):
            b = raw[i]
            i += 1
            for j in range(4):
                px[xb * 4 + j, y] = ((b >> (j * 2)) & 3) * 85
    return img


def img_to_raw(img: Image.Image) -> bytes:
    img = img.convert("L")
    px = img.load()
    out = bytearray()
    for y in range(GLYPH_H):
        for x in range(0, GLYPH_W, 4):
            b = 0
            for i in range(4):
                q = max(0, min(3, int((px[x + i, y] + 42) // 85)))
                b |= q << (i * 2)
            out.append(b)
    return bytes(out)


def bbox_nonzero(img: Image.Image, region: tuple[int, int, int, int] | None = None) -> Optional[tuple[int, int, int, int]]:
    if region is None:
        bb = img.getbbox()
        return bb
    x0, y0, x1, y1 = region
    crop = img.crop(region)
    bb = crop.getbbox()
    if bb is None:
        return None
    return (x0 + bb[0], y0 + bb[1], x0 + bb[2], y0 + bb[3])


def render_glyph(font: ImageFont.FreeTypeFont, ch: str) -> Optional[Image.Image]:
    if ch in {"", " ", "\u3000", "\t", "\r", "\n"}:
        return None
    temp = Image.new("L", (96, 96), 0)
    draw = ImageDraw.Draw(temp)
    draw.text((24, 24), ch, font=font, fill=255)
    bb = temp.getbbox()
    if bb is None:
        return None
    return temp.crop(bb)


def fit_into_box(glyph: Image.Image, box_w: int, box_h: int) -> Image.Image:
    gw, gh = glyph.size
    if gw <= 0 or gh <= 0:
        return glyph
    scale = min(box_w / gw, box_h / gh, 1.0)
    if scale < 1.0:
        glyph = glyph.resize((max(1, int(round(gw * scale))), max(1, int(round(gh * scale)))), Image.Resampling.LANCZOS)
    return glyph


def is_private_use(ch: str) -> bool:
    return len(ch) == 1 and unicodedata.category(ch) == "Co"


def redraw_using_template(orig_raw: bytes, font: ImageFont.FreeTypeFont, draw_ch: Optional[str], *, single_byte: bool) -> bytes:
    # preserve blank / private-use or unknown slots
    if draw_ch is None or draw_ch in {"", " ", "\u3000"} or is_private_use(draw_ch):
        return orig_raw if draw_ch is None or is_private_use(draw_ch) else bytes(RAW_SIZE)

    orig_img = raw_to_img(orig_raw)
    if single_byte:
        box = bbox_nonzero(orig_img, (12, 0, 24, 24))
    else:
        box = bbox_nonzero(orig_img)
    if box is None:
        # preserve odd slots we cannot template safely
        return orig_raw

    glyph = render_glyph(font, draw_ch)
    if glyph is None:
        return bytes(RAW_SIZE)

    bx0, by0, bx1, by1 = box
    bw, bh = bx1 - bx0, by1 - by0
    glyph = fit_into_box(glyph, bw, bh)
    gw, gh = glyph.size
    x = bx0 + max(0, (bw - gw) // 2)
    y = by0 + max(0, (bh - gh) // 2)

    if single_byte:
        canvas = orig_img.copy()
        draw = ImageDraw.Draw(canvas)
        draw.rectangle((12, 0, 23, 23), fill=0)
    else:
        canvas = Image.new("L", (24, 24), 0)
    canvas.paste(glyph, (x, y))
    return img_to_raw(canvas)


def build_from_template(orig_font_path: Path, out_path: Path, font: ImageFont.FreeTypeFont, cnjp_rev: dict[str, str]) -> dict:
    data = orig_font_path.read_bytes()
    table = bytearray()
    body = bytearray()
    preserved = 0
    remapped = 0
    max_chunk = 0
    min_chunk = 10**9
    examples = []

    for idx in range(ENTRY_COUNT):
        ent = int.from_bytes(data[idx * 4:idx * 4 + 4], "little")
        off = ent & 0xFFFFFF
        flag = (ent >> 24) & 0xFF
        slot_ch = index_to_char(idx)
        draw_ch = cnjp_rev.get(slot_ch, slot_ch) if slot_ch is not None else None
        orig_raw = decode_chunk(data, off, flag)
        new_raw = redraw_using_template(orig_raw, font, draw_ch, single_byte=(idx < 224))
        if new_raw == orig_raw:
            preserved += 1
        if slot_ch is not None and draw_ch is not None and draw_ch != slot_ch:
            remapped += 1
        chunk = compress_raw_lzss(new_raw)
        # roundtrip validate with same flag structure expected by engine
        if decode_chunk(chunk, 0, 0x07) != new_raw:
            raise AssertionError(f"LZ roundtrip failed at idx={idx}")
        new_off = TABLE_SIZE + len(body)
        table += (new_off | (0x07 << 24)).to_bytes(4, "little")
        body += chunk
        max_chunk = max(max_chunk, len(chunk))
        min_chunk = min(min_chunk, len(chunk))
        if idx in (33, 59, 512, 341, 342, 290, 291):
            examples.append({"index": idx, "slot": slot_ch, "draw": draw_ch})

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(bytes(table) + bytes(body))
    return {
        "source": str(orig_font_path),
        "output": str(out_path),
        "file_size": out_path.stat().st_size,
        "buffer_limit": FONT_BUFFER_LIMIT,
        "fits_buffer": out_path.stat().st_size <= FONT_BUFFER_LIMIT,
        "preserved_slots": preserved,
        "remapped_slots": remapped,
        "min_chunk": min_chunk,
        "max_chunk": max_chunk,
        "examples": examples,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Interlude FONT.DAT redraw using original layout templates")
    ap.add_argument("input_dir", help="folder containing original FONT1.DAT..FONT4.DAT")
    ap.add_argument("font", help="TTF/TTC path")
    ap.add_argument("output_dir")
    ap.add_argument("--cnjp", default=None, help="subs_cn_jp.json, render key glyph at value slot")
    ap.add_argument("--size", type=int, default=22)
    args = ap.parse_args()

    in_dir = Path(args.input_dir)
    out_dir = Path(args.output_dir)
    font = ImageFont.truetype(args.font, size=args.size)
    cnjp_rev = load_cnjp(Path(args.cnjp)) if args.cnjp else {}

    results = []
    for name in ["FONT1.DAT", "FONT2.DAT", "FONT3.DAT", "FONT4.DAT"]:
        results.append(build_from_template(in_dir / name, out_dir / name, font, cnjp_rev))
    print(json.dumps(results, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
