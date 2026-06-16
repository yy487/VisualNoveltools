#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""
Martopia / side-B SjisFont cn_jp 映射字库工具

用途：
  1) 根据 Martopia.exe 中的 SjisFont 字形索引表，定位 CP932 宿主字符所在的 32x32 字格。
  2) 按 subs_cn_jp.json 的 {中文: CP932宿主字符} 映射，把中文 glyph 重绘到宿主字符字格。
  3) 把翻译 JSON 中的 message/name/message_parts 按 cn_jp 映射转换成可 CP932 编码文本。

典型流程：
  python martopia_cnjp_font_tool.py check-map Martopia.exe subs_cn_jp.json
  python martopia_cnjp_font_tool.py patch-font Martopia.exe font_in subs_cn_jp.json font_out --ttf C:\Windows\Fonts\msyh.ttc --size 28
  python martopia_cnjp_font_tool.py convert-json json_trans json_cp932 subs_cn_jp.json

font_in/font_out 目录应包含：
  SjisFont01.png, SjisFont01_b.png, SjisFont02.png, SjisFont02_b.png
"""
from __future__ import annotations

import argparse
import json
import os
import struct
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw, ImageFilter, ImageFont

BASE = 0x400000
WORD_5337D8 = 0x5337D8
WORD_535F58 = 0x535F58
TABLE1_WORDS = 5056
# word_535F58 到 aSjisfont 之前，足够覆盖 0x9B80+、E0-EE、FA-FC 的查表范围。
TABLE2_WORDS = 10936
CELL = 32

PNG_NAMES = {
    "01": "SjisFont01.png",
    "01_b": "SjisFont01_b.png",
    "02": "SjisFont02.png",
    "02_b": "SjisFont02_b.png",
}


def read_file(path: str | Path) -> bytes:
    return Path(path).read_bytes()


def rva_to_offset(exe: bytes, rva: int) -> int:
    pe = struct.unpack_from("<I", exe, 0x3C)[0]
    num_sections = struct.unpack_from("<H", exe, pe + 6)[0]
    opt_size = struct.unpack_from("<H", exe, pe + 20)[0]
    sec_base = pe + 24 + opt_size
    for i in range(num_sections):
        s = exe[sec_base + i * 40: sec_base + (i + 1) * 40]
        virtual_size, virtual_addr, raw_size, raw_ptr = struct.unpack_from("<IIII", s, 8)
        span = max(virtual_size, raw_size)
        if virtual_addr <= rva < virtual_addr + span:
            return raw_ptr + (rva - virtual_addr)
    raise ValueError(f"RVA not mapped: 0x{rva:X}")


class SjisFontIndex:
    def __init__(self, exe_path: str | Path):
        self.exe_path = Path(exe_path)
        self.exe = read_file(self.exe_path)
        off1 = rva_to_offset(self.exe, WORD_5337D8 - BASE)
        off2 = rva_to_offset(self.exe, WORD_535F58 - BASE)
        self.table1 = struct.unpack_from(f"<{TABLE1_WORDS}H", self.exe, off1)
        self.table2 = struct.unpack_from(f"<{TABLE2_WORDS}H", self.exe, off2)

    def lookup_host(self, ch: str) -> dict[str, Any] | None:
        """返回宿主字符在 SjisFont atlas 中的位置。page 为 01/02，glyph 为 1-based。"""
        if len(ch) != 1:
            return None
        try:
            b = ch.encode("cp932")
        except UnicodeEncodeError:
            return None
        if len(b) != 2:
            return None
        lead, trail = b[0], b[1]
        if trail < 0x40:
            return None

        table = None
        page = None
        idx = None
        code = (lead << 8) | trail

        if 0x81 <= lead <= 0x9F:
            idx = ((3 * lead - 387) << 6) + trail - 64
            if code >= 0x9B80:
                page, table = "02", self.table2
            else:
                page, table = "01", self.table1
        elif 0xE0 <= lead <= 0xEE:
            idx = trail + 192 * (lead - 193) - 64
            page, table = "02", self.table2
        elif 0xFA <= lead <= 0xFC:
            idx = trail + 192 * (lead - 204) - 64
            page, table = "02", self.table2
        else:
            return None

        if idx is None or idx < 0 or idx >= len(table):
            return None
        glyph = table[idx]
        if glyph == 0:
            return None
        return {
            "host": ch,
            "cp932_hex": b.hex(),
            "page": page,
            "table_index": idx,
            "glyph": int(glyph),
            "cell_index": int(glyph) - 1,
        }

    def iter_canonical_hosts(self) -> list[dict[str, Any]]:
        """遍历 EXE 字库索引表可定位到的代表性宿主字符。
        对同一 (page, cell) 若有多个 CP932 编码命中，只保留按编码顺序遇到的第一个。
        """
        out: list[dict[str, Any]] = []
        seen: set[tuple[str, int]] = set()
        ranges = [(0x81, 0x9F), (0xE0, 0xEE), (0xFA, 0xFC)]
        for lo, hi in ranges:
            for lead in range(lo, hi + 1):
                for trail in range(0x40, 0xFD):
                    if trail == 0x7F:
                        continue
                    b = bytes((lead, trail))
                    try:
                        ch = b.decode("cp932")
                    except UnicodeDecodeError:
                        continue
                    if len(ch) != 1:
                        continue
                    loc = self.lookup_host(ch)
                    if loc is None:
                        continue
                    key = (loc["page"], loc["cell_index"])
                    if key in seen:
                        continue
                    seen.add(key)
                    out.append(loc)
        return out


def load_map(path: str | Path) -> dict[str, str]:
    with open(path, "r", encoding="utf-8") as f:
        m = json.load(f)
    if not isinstance(m, dict):
        raise ValueError("mapping JSON root must be an object")
    bad = [(k, v) for k, v in m.items() if not isinstance(k, str) or not isinstance(v, str) or len(k) != 1 or len(v) != 1]
    if bad:
        raise ValueError(f"mapping must be one char -> one char; bad sample={bad[:5]}")
    return m


def load_font(ttf_path: str | Path, size: int, ttc_index: int = 0) -> ImageFont.FreeTypeFont:
    try:
        return ImageFont.truetype(str(ttf_path), size=size, index=ttc_index)
    except TypeError:
        return ImageFont.truetype(str(ttf_path), size=size)


def render_char_mask(ch: str, font: ImageFont.FreeTypeFont, size: int = CELL, xoff: int = 0, yoff: int = 0) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(mask)
    # textbbox 比 getbbox 更接近 PIL 实际绘制区域。
    bbox = d.textbbox((0, 0), ch, font=font)
    w = bbox[2] - bbox[0]
    h = bbox[3] - bbox[1]
    x = int(round((size - w) / 2 - bbox[0] + xoff))
    y = int(round((size - h) / 2 - bbox[1] + yoff))
    d.text((x, y), ch, font=font, fill=255)
    return mask


def paste_glyph(page: Image.Image, x: int, y: int, mask: Image.Image, alpha_scale: float = 1.0) -> None:
    page.paste((0, 0, 0, 0), (x, y, x + CELL, y + CELL))
    if alpha_scale != 1.0:
        mask = mask.point(lambda p: max(0, min(255, int(p * alpha_scale))))
    glyph = Image.new("RGBA", (CELL, CELL), (255, 255, 255, 0))
    glyph.putalpha(mask)
    page.alpha_composite(glyph, (x, y))


def patch_font(args: argparse.Namespace) -> None:
    m = load_map(args.map_json)
    idx = SjisFontIndex(args.exe)
    in_dir = Path(args.font_dir)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    pages: dict[str, Image.Image] = {}
    for k, name in PNG_NAMES.items():
        p = in_dir / name
        if not p.exists():
            raise FileNotFoundError(p)
        pages[k] = Image.open(p).convert("RGBA")
        if pages[k].size[0] % CELL or pages[k].size[1] % CELL:
            raise ValueError(f"{p}: image size must be multiple of {CELL}, got {pages[k].size}")

    font = load_font(args.ttf, args.size, args.ttc_index)
    cols = {"01": pages["01"].size[0] // CELL, "02": pages["02"].size[0] // CELL}

    errors = []
    report = []
    patched = 0

    host_to_cn: dict[str, str] = {}
    for cn, host in m.items():
        old = host_to_cn.get(host)
        if old is not None and old != cn:
            errors.append({"cn": cn, "host": host, "error": f"duplicate host mapped by both {old} and {cn}"})
            continue
        host_to_cn[host] = cn

    if args.full_redraw:
        items = []
        for loc in idx.iter_canonical_hosts():
            host = loc["host"]
            target = host_to_cn.get(host, host)
            row = dict(loc)
            row["target"] = target
            row["mapped"] = host in host_to_cn
            items.append(row)
    else:
        items = []
        used_cells: dict[tuple[str, int], str] = {}
        for cn, host in m.items():
            loc = idx.lookup_host(host)
            if loc is None:
                errors.append({"cn": cn, "host": host, "error": "host char not found in EXE font index"})
                continue
            page = loc["page"]
            cell_index = loc["cell_index"]
            key = (page, cell_index)
            if key in used_cells:
                errors.append({"cn": cn, "host": host, "error": f"cell collision with {used_cells[key]}"})
                continue
            used_cells[key] = cn
            row = dict(loc)
            row["target"] = cn
            row["mapped"] = True
            items.append(row)

    for loc in items:
        page = loc["page"]
        cell_index = loc["cell_index"]
        x = (cell_index % cols[page]) * CELL
        y = (cell_index // cols[page]) * CELL
        mask = render_char_mask(loc["target"], font, CELL, args.xoff, args.yoff)

        paste_glyph(pages[page], x, y, mask, 1.0)
        if args.b_mode == "normal":
            paste_glyph(pages[page + "_b"], x, y, mask, args.b_alpha)
        elif args.b_mode == "blur":
            bmask = mask.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.GaussianBlur(args.b_blur))
            paste_glyph(pages[page + "_b"], x, y, bmask, args.b_alpha)
        elif args.b_mode == "clear":
            pages[page + "_b"].paste((0, 0, 0, 0), (x, y, x + CELL, y + CELL))
        else:
            raise ValueError(f"unknown b_mode: {args.b_mode}")

        patched += 1
        loc.update({"x": x, "y": y})
        report.append(loc)

    for k, name in PNG_NAMES.items():
        pages[k].save(out_dir / name)

    with open(out_dir / "cnjp_font_patch_report.json", "w", encoding="utf-8") as f:
        json.dump({
            "mode": "full_redraw" if args.full_redraw else "mapped_only",
            "patched": patched,
            "errors": errors,
            "items": report,
        }, f, ensure_ascii=False, indent=2)

    print(f"[patch-font] mode={'full_redraw' if args.full_redraw else 'mapped_only'} patched={patched} errors={len(errors)} out={out_dir}")
    if errors:
        print(f"[patch-font] report: {out_dir / 'cnjp_font_patch_report.json'}")


def check_map(args: argparse.Namespace) -> None:
    m = load_map(args.map_json)
    idx = SjisFontIndex(args.exe)
    dup_host = {}
    for cn, host in m.items():
        dup_host.setdefault(host, []).append(cn)
    dup_host = {h: cns for h, cns in dup_host.items() if len(cns) > 1}

    bad_encode = []
    missing = []
    counts = {"01": 0, "02": 0}
    for cn, host in m.items():
        try:
            host.encode("cp932")
        except UnicodeEncodeError:
            bad_encode.append((cn, host))
            continue
        loc = idx.lookup_host(host)
        if loc is None:
            missing.append((cn, host))
        else:
            counts[loc["page"]] += 1

    print(f"[check-map] total={len(m)} page01={counts['01']} page02={counts['02']} missing={len(missing)} bad_cp932={len(bad_encode)} duplicate_host={len(dup_host)}")
    if bad_encode:
        print("  bad_cp932 sample:", bad_encode[:10])
    if missing:
        print("  missing sample:", missing[:10])
    if dup_host:
        print("  duplicate_host sample:", list(dup_host.items())[:10])


def map_text(s: str, m: dict[str, str], strict: bool = False) -> str:
    out = []
    missing = []
    for ch in s:
        if ch in m:
            out.append(m[ch])
        else:
            out.append(ch)
            if strict:
                try:
                    ch.encode("cp932")
                except UnicodeEncodeError:
                    missing.append(ch)
    if missing:
        sample = "".join(dict.fromkeys(missing))[:80]
        raise UnicodeEncodeError("cp932-map", s, 0, len(s), f"unmapped non-cp932 chars: {sample}")
    return "".join(out)


def convert_obj(obj: Any, m: dict[str, str], fields: set[str], strict: bool) -> Any:
    if isinstance(obj, dict):
        res = {}
        for k, v in obj.items():
            if k in fields and isinstance(v, str):
                res[k] = map_text(v, m, strict)
            elif k in fields and isinstance(v, list):
                res[k] = [map_text(x, m, strict) if isinstance(x, str) else x for x in v]
            else:
                res[k] = convert_obj(v, m, fields, strict)
        return res
    if isinstance(obj, list):
        return [convert_obj(x, m, fields, strict) for x in obj]
    return obj


def convert_json_file(src: Path, dst: Path, m: dict[str, str], fields: set[str], strict: bool) -> None:
    with open(src, "r", encoding="utf-8") as f:
        data = json.load(f)
    new_data = convert_obj(data, m, fields, strict)
    dst.parent.mkdir(parents=True, exist_ok=True)
    with open(dst, "w", encoding="utf-8", newline="\n") as f:
        json.dump(new_data, f, ensure_ascii=False, indent=2)


def convert_json(args: argparse.Namespace) -> None:
    m = load_map(args.map_json)
    inp = Path(args.input)
    out = Path(args.output)
    fields = set(args.fields.split(","))
    total = 0
    if inp.is_dir():
        for src in inp.rglob("*.json"):
            rel = src.relative_to(inp)
            convert_json_file(src, out / rel, m, fields, args.strict)
            total += 1
    else:
        convert_json_file(inp, out, m, fields, args.strict)
        total = 1
    print(f"[convert-json] files={total} output={out}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Martopia SjisFont cn_jp 字库映射工具")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("check-map", help="检查 cn_jp 映射是否能落到 SjisFont 字格")
    p.add_argument("exe")
    p.add_argument("map_json")
    p.set_defaults(func=check_map)

    p = sub.add_parser("patch-font", help="按 cn_jp 映射重绘四张 SjisFont PNG")
    p.add_argument("exe")
    p.add_argument("font_dir")
    p.add_argument("map_json")
    p.add_argument("out_dir")
    p.add_argument("--ttf", required=True, help="中文字体 .ttf/.ttc，例如 C:\\Windows\\Fonts\\msyh.ttc")
    p.add_argument("--ttc-index", type=int, default=0)
    p.add_argument("--size", type=int, default=28)
    p.add_argument("--xoff", type=int, default=0)
    p.add_argument("--yoff", type=int, default=-1)
    p.add_argument("--b-mode", choices=["normal", "blur", "clear"], default="blur")
    p.add_argument("--b-alpha", type=float, default=0.55)
    p.add_argument("--b-blur", type=float, default=0.45)
    p.add_argument("--full-redraw", action="store_true", help="全量重绘：映射表中有的画中文；没有的按宿主字符原字形重绘")
    p.set_defaults(func=patch_font)

    p = sub.add_parser("convert-json", help="把翻译 JSON 的 message 等字段转换为 CP932 宿主字符")
    p.add_argument("input")
    p.add_argument("output")
    p.add_argument("map_json")
    p.add_argument("--fields", default="message,message_parts", help="逗号分隔，默认只转换 message/message_parts")
    p.add_argument("--strict", action="store_true", help="发现未映射且不能 cp932 编码的字符就报错")
    p.set_defaults(func=convert_json)

    args = ap.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
