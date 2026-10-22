#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
fontkit.py — AI5WIN 字库一站式工具
====================================
合并原 6 个 rp_font_*.py 的功能，外加 PIL 渲染 TTF 替代 Photoshop，
一条命令完成从 TTF 字体到 EXE 补丁的全流程。

用法:
  python fontkit.py tbl decode  FONT.TBL tbl.txt          # TBL → 可读文本
  python fontkit.py tbl encode  tbl.txt FONT.TBL          # 文本 → TBL
  python fontkit.py charset char_list.txt [--base-tbl FONT.TBL] [--json trans/] [--fullwidth]
  python fontkit.py render  font.ttf char_list.txt out.png [--glyph-w 24] [--glyph-h 34]
  python fontkit.py build   char_list.txt char_list.png FONT.FNT FONT.PAL FONT.TBL
  python fontkit.py dump    FONT.FNT FONT.PAL preview.png
  python fontkit.py inject  game.exe FONT.TBL FONT.FNT -o patched.exe [--patch-u32 ...]
  python fontkit.py full    game.exe font.ttf [--base-tbl FONT.TBL] [--json trans/] [-o out.exe]
"""

from __future__ import annotations
import argparse
import json
import math
import struct
import unicodedata
from pathlib import Path

# ── 常量 ───────────────────────────────────────────────────────────
GLYPH_WIDTH  = 24
GLYPH_HEIGHT = 34
GLYPH_SIZE   = GLYPH_WIDTH * GLYPH_HEIGHT // 2  # 4bpp → 2 pixels per byte

# ── PAL 读写 ───────────────────────────────────────────────────────
def read_palette(path: Path) -> list[tuple[int, int, int]]:
    data = path.read_bytes()
    if len(data) < 16 * 3:
        raise ValueError(f"PAL too small: {path} size={len(data)}")
    return [(data[i], data[i + 1], data[i + 2]) for i in range(0, 16 * 3, 3)]

def write_palette(path: Path, palette) -> None:
    raw = bytes(palette)
    if len(raw) < 16 * 3:
        raw = raw + b"\x00" * (16 * 3 - len(raw))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw[:16 * 3])

# ── FNT 读写 ───────────────────────────────────────────────────────
def read_fnt(path: Path) -> bytes:
    data = path.read_bytes()
    if len(data) % GLYPH_SIZE != 0:
        raise ValueError(f"FNT size not multiple of {GLYPH_SIZE}: {path} size={len(data)}")
    return data

def decode_glyph_rgba(fnt: bytes, index: int, palette: list[tuple[int, int, int]]) -> list[tuple[int, int, int, int]]:
    pos = index * GLYPH_SIZE
    chunk = fnt[pos:pos + GLYPH_SIZE]
    if len(chunk) != GLYPH_SIZE:
        raise IndexError(index)
    out = []
    for b in chunk:
        for shift in (4, 0):
            pi = (b >> shift) & 0x0F
            if pi == 0:
                out.append((0, 0, 0, 0))
            else:
                r, g, bl = palette[pi]
                out.append((r, g, bl, 255))
    return out

# ── TBL 读写 ───────────────────────────────────────────────────────
def read_tbl(path: Path, encoding: str = "cp932") -> list[str]:
    data = path.read_bytes()
    chars: list[str] = []
    for i in range(0, len(data) - 1, 2):
        lo, hi = data[i], data[i + 1]
        if lo == 0 and hi == 0:
            break
        bs = bytes([hi, lo])
        try:
            chars.append(bs.decode(encoding))
        except UnicodeDecodeError:
            chars.append("{%02X%02X}" % (hi, lo))
    return chars

def write_tbl(path: Path, chars, encoding: str = "cp932") -> None:
    out = bytearray()
    for ch in chars:
        if len(ch) == 6 and ch.startswith("{") and ch.endswith("}"):
            bs = bytes.fromhex(ch[1:-1])
        else:
            bs = ch.encode(encoding)
        if len(bs) != 2:
            raise ValueError(f"FONT.TBL supports two-byte codes only: {ch!r} -> {bs.hex()}")
        out.extend([bs[1], bs[0]])
    out.extend(b"\x00\x00")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(bytes(out))

# ── 字符列表解析 ────────────────────────────────────────────────────
def split_char_list_text(text: str) -> list[str]:
    chars: list[str] = []
    i = 0
    while i < len(text):
        if text[i] in "\r\n":
            i += 1
            continue
        if text[i] == "{" and i + 5 < len(text) and text[i + 5] == "}":
            chars.append(text[i:i + 6])
            i += 6
        else:
            chars.append(text[i])
            i += 1
    return chars

# ── CP932 全角字符 ──────────────────────────────────────────────────
def cp932_two_byte_fullwidth_chars() -> list[str]:
    out = []
    first_list = list(range(0x81, 0xA0)) + list(range(0xE0, 0xEB)) + list(range(0xFA, 0xFD))
    second_list = list(range(0x40, 0x7F)) + list(range(0x80, 0x100))
    for hi in first_list:
        for lo in second_list:
            bs = bytes([hi, lo])
            try:
                ch = bs.decode("cp932")
            except UnicodeDecodeError:
                continue
            if len(ch) != 1:
                continue
            if unicodedata.east_asian_width(ch) in ("F", "W", "A"):
                out.append(ch)
    return out

# ── JSON 字符收集 ───────────────────────────────────────────────────
def collect_json_chars(path: Path, fields: list[str]) -> set[str]:
    chars: set[str] = set()
    files = [path] if path.is_file() else sorted(path.rglob("*.json"))
    for p in files:
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        stack = [data]
        while stack:
            cur = stack.pop()
            if isinstance(cur, dict):
                for k, v in cur.items():
                    if k in fields and isinstance(v, str):
                        chars.update(v)
                    elif isinstance(v, (dict, list)):
                        stack.append(v)
            elif isinstance(cur, list):
                stack.extend(cur)
    return chars

# ── PE 工具 ────────────────────────────────────────────────────────
def align_up(v: int, a: int) -> int:
    return (v + a - 1) // a * a

def pe_r16(d, off): return int.from_bytes(d[off:off+2], "little")
def pe_r32(d, off): return int.from_bytes(d[off:off+4], "little")
def pe_w16(d, off, v): d[off:off+2] = v.to_bytes(2, "little")
def pe_w32(d, off, v): d[off:off+4] = v.to_bytes(4, "little")

def add_font_section(exe: bytes, tbl: bytes, fnt: bytes, section_name: bytes = b".font"):
    """追加一个包含 TBL + FNT 的新 PE 段。返回 (new_exe, info_dict)。"""
    data = bytearray(exe)
    if data[:2] != b"MZ":
        raise ValueError("不是有效的 MZ 文件")
    pe = pe_r32(data, 0x3C)
    if data[pe:pe+4] != b"PE\x00\x00":
        raise ValueError("不是有效的 PE 文件")
    coff = pe + 4
    num_sections = pe_r16(data, coff + 2)
    opt_size = pe_r16(data, coff + 16)
    opt = coff + 20
    magic = pe_r16(data, opt)
    if magic != 0x10B:
        raise ValueError("仅支持 PE32 格式")
    section_alignment = pe_r32(data, opt + 0x20)
    file_alignment    = pe_r32(data, opt + 0x24)
    size_of_image_off = opt + 0x38
    size_of_headers   = pe_r32(data, opt + 0x3C)
    sec_table = opt + opt_size
    new_sec_hdr = sec_table + num_sections * 40
    if new_sec_hdr + 40 > size_of_headers:
        raise ValueError("PE 头部空间不足，无法添加新段；请用 PE 编辑器扩展头部")
    last = sec_table + (num_sections - 1) * 40
    last_va   = pe_r32(data, last + 12)
    last_raw_size = pe_r32(data, last + 16)
    last_raw = pe_r32(data, last + 20)
    last_vsize = pe_r32(data, last + 8)
    new_va  = align_up(last_va + max(last_vsize, last_raw_size), section_alignment)
    new_raw = align_up(last_raw + last_raw_size, file_alignment)

    payload = bytearray()
    tbl_rva = new_va
    payload.extend(tbl)
    while len(payload) % file_alignment:
        payload.append(0)
    fnt_rva = new_va + len(payload)
    payload.extend(fnt)
    raw_size  = align_up(len(payload), file_alignment)
    virt_size = len(payload)
    payload.extend(b"\x00" * (raw_size - len(payload)))

    if len(data) < new_raw:
        data.extend(b"\x00" * (new_raw - len(data)))
    data[new_raw:new_raw + raw_size] = payload

    name = section_name[:8].ljust(8, b"\x00")
    data[new_sec_hdr:new_sec_hdr + 8] = name
    pe_w32(data, new_sec_hdr + 8,  virt_size)
    pe_w32(data, new_sec_hdr + 12, new_va)
    pe_w32(data, new_sec_hdr + 16, raw_size)
    pe_w32(data, new_sec_hdr + 20, new_raw)
    pe_w32(data, new_sec_hdr + 24, 0)
    pe_w32(data, new_sec_hdr + 28, 0)
    pe_w16(data, new_sec_hdr + 32, 0)
    pe_w16(data, new_sec_hdr + 34, 0)
    pe_w32(data, new_sec_hdr + 36, 0x40000040)  # 可读 + 已初始化数据

    pe_w16(data, coff + 2, num_sections + 1)
    pe_w32(data, size_of_image_off, align_up(new_va + virt_size, section_alignment))

    info = {
        "section": section_name.decode("ascii", "ignore"),
        "section_rva": new_va, "section_raw": new_raw,
        "section_virtual_size": virt_size, "section_raw_size": raw_size,
        "tbl_rva": tbl_rva, "tbl_size": len(tbl),
        "fnt_rva": fnt_rva, "fnt_size": len(fnt),
        "file_alignment": file_alignment, "section_alignment": section_alignment,
    }
    return bytes(data), info

def apply_patch_u32(data: bytearray, spec: str, values: dict[str, int]) -> None:
    if "=" not in spec:
        raise ValueError("--patch-u32 格式：OFFSET=VALUE")
    left, right = spec.split("=", 1)
    off = int(left, 0)
    key = right.strip()
    val = values[key] if key in values else int(key, 0)
    if not (0 <= off <= len(data) - 4):
        raise ValueError(f"patch 偏移超出范围: {off:#x}")
    pe_w32(data, off, val)

# ══════════════════════════════════════════════════════════════════════
#  命令实现
# ══════════════════════════════════════════════════════════════════════

def cmd_tbl(args):
    if args.tbl_cmd == "decode":
        chars = read_tbl(Path(args.input), args.encoding)
        lines = ["".join(chars[i:i+args.columns]) for i in range(0, len(chars), args.columns)]
        Path(args.output).write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"解码完成：{len(chars)} 个字符 → {args.output}")
    else:
        chars = split_char_list_text(Path(args.input).read_text(encoding="utf-8"))
        write_tbl(Path(args.output), chars, args.encoding)
        print(f"编码完成：{len(chars)} 个字符 → {args.output}")

def cmd_charset(args):
    """生成 char_list.txt"""
    chars: list[str] = []
    seen: set[str] = set()
    def add(ch: str):
        if ch in "\r\n" or ch in seen:
            return
        try:
            if len(ch.encode("cp932")) != 2:
                return
        except UnicodeEncodeError:
            return
        seen.add(ch)
        chars.append(ch)

    if args.base_tbl:
        for c in read_tbl(Path(args.base_tbl)):
            add(c)
    if args.json_dir:
        fields = [x.strip() for x in args.fields.split(",") if x.strip()]
        for c in sorted(collect_json_chars(Path(args.json_dir), fields)):
            add(c)
    if args.fullwidth:
        for c in cp932_two_byte_fullwidth_chars():
            add(c)
    lines = ["".join(chars[i:i+args.columns]) for i in range(0, len(chars), args.columns)]
    Path(args.output).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"生成完成：{len(chars)} 个字符 → {args.output}")

def cmd_render(args):
    """用 TTF 渲染字符网格 PNG，替代 Photoshop。"""
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError:
        print("需要 Pillow: pip install Pillow")
        return

    text = Path(args.chars).read_text(encoding="utf-8")
    chars = split_char_list_text(text)
    glyph_w = args.glyph_w
    glyph_h = args.glyph_h
    cols    = args.columns
    rows    = (len(chars) + cols - 1) // cols

    margin_x = args.margin_x
    margin_y = args.margin_y
    cell_w = glyph_w + margin_x
    cell_h = glyph_h + margin_y
    start_x = args.start_x
    start_y = args.start_y
    img_w = start_x * 2 + cols * cell_w
    img_h = start_y * 2 + rows * cell_h

    # 纯绿背景 (R=0, G=255, B=0) — 调色板索引 0 = 透明
    img = Image.new("RGB", (img_w, img_h), (0, 255, 0))
    draw = ImageDraw.Draw(img)

    # 二分查找合适的字体大小
    font_size = args.font_size
    if font_size <= 0:
        lo, hi = 8, glyph_h * 3
        font_size = glyph_h
        for _ in range(12):
            mid = (lo + hi) // 2
            try:
                f = ImageFont.truetype(str(args.ttf), mid)
            except Exception:
                lo = mid + 1
                continue
            # 用 "あ" 作为全角测试字符
            bbox = draw.textbbox((0, 0), "あ", font=f)
            h = bbox[3] - bbox[1]
            if h <= glyph_h - 4:
                lo = mid + 1
                font_size = mid
            else:
                hi = mid - 1
        print(f"自动计算字体大小: {font_size}pt (字形高度 ≤ {glyph_h-4}px)")

    try:
        font = ImageFont.truetype(str(args.ttf), font_size)
    except Exception as e:
        print(f"无法加载字体 {args.ttf}: {e}")
        return

    # 逐字绘制，居中
    for i, ch in enumerate(chars):
        row, col = divmod(i, cols)
        x_center = start_x + col * cell_w + cell_w // 2
        y_center = start_y + row * cell_h + cell_h // 2
        bbox = draw.textbbox((0, 0), ch, font=font)
        tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
        x = x_center - tw // 2 - bbox[0]
        y = y_center - th // 2 - bbox[1]
        draw.text((x, y), ch, font=font, fill=(255, 255, 255))

    # 转换为索引颜色（16 色），绿色 = 索引 0
    pal_img = img.convert("P", palette=Image.Palette.ADAPTIVE, colors=16)
    # 确保绿色是索引 0：找到绿色像素的索引，交换到位置 0
    palette_data = pal_img.getpalette() or []
    green_idx = None
    for pi in range(16):
        r, g, b = palette_data[pi*3:pi*3+3]
        if r == 0 and g == 255 and b == 0:
            green_idx = pi
            break
    if green_idx is not None and green_idx != 0:
        # 交换索引 0 和 green_idx
        p = palette_data
        p[0:3], p[green_idx*3:green_idx*3+3] = p[green_idx*3:green_idx*3+3], p[0:3]
        pal_img.putpalette(p)
        # 交换像素数据中的值
        px = pal_img.load()
        for y in range(img_h):
            for x in range(img_w):
                v = px[x, y]
                if v == 0:
                    px[x, y] = green_idx
                elif v == green_idx:
                    px[x, y] = 0

    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    pal_img.save(args.output)
    print(f"渲染完成：{len(chars)} 个字符 → {args.output} ({img_w}×{img_h})")

def cmd_build(args):
    """char_list.txt + PNG → FNT + PAL + TBL"""
    from PIL import Image
    text = Path(args.chars).read_text(encoding="utf-8")
    lines = [line.rstrip("\r\n") for line in text.splitlines() if line.strip()]
    flat_chars = split_char_list_text(text)
    img = Image.open(args.png)
    if img.mode != "P":
        raise ValueError("PNG 必须是索引颜色模式 (mode P)，16 色调色板")
    pal = img.getpalette() or []
    write_palette(Path(args.pal), pal[:16 * 3])

    pixels = img.load()
    font = bytearray()
    chars_for_tbl = []

    margin_x = args.margin_x
    margin_y = args.margin_y
    cell_w = GLYPH_WIDTH + margin_x
    cell_h = GLYPH_HEIGHT + margin_y

    if args.columns is not None:
        for idx, ch in enumerate(flat_chars):
            row, col = divmod(idx, args.columns)
            chars_for_tbl.append(ch)
            x0 = args.start_x + col * cell_w
            y0 = args.start_y + row * cell_h
            for y in range(y0, y0 + GLYPH_HEIGHT):
                for x in range(x0, x0 + GLYPH_WIDTH, 2):
                    hi = pixels[x, y] & 0x0F
                    lo = pixels[x + 1, y] & 0x0F
                    font.append((hi << 4) | lo)
    else:
        for row, line in enumerate(lines):
            line_chars = split_char_list_text(line)
            for col, ch in enumerate(line_chars):
                chars_for_tbl.append(ch)
                x0 = args.start_x + col * cell_w
                y0 = args.start_y + row * cell_h
                for y in range(y0, y0 + GLYPH_HEIGHT):
                    for x in range(x0, x0 + GLYPH_WIDTH, 2):
                        hi = pixels[x, y] & 0x0F
                        lo = pixels[x + 1, y] & 0x0F
                        font.append((hi << 4) | lo)

    Path(args.fnt).parent.mkdir(parents=True, exist_ok=True)
    Path(args.fnt).write_bytes(bytes(font))
    write_tbl(Path(args.tbl), chars_for_tbl, args.encoding)
    print(f"构建完成：{len(chars_for_tbl)} 个字符，FNT={len(font)} 字节")

def cmd_dump(args):
    """FNT + PAL → PNG 预览"""
    try:
        from PIL import Image
    except ImportError:
        print("需要 Pillow: pip install Pillow")
        return
    fnt = read_fnt(Path(args.fnt))
    palette = read_palette(Path(args.pal))
    total = len(fnt) // GLYPH_SIZE
    count = args.count if args.count else total - args.start
    count = max(0, min(count, total - args.start))
    rows = math.ceil(count / args.columns) if count else 1
    img = Image.new("RGBA", (args.columns * GLYPH_WIDTH, rows * GLYPH_HEIGHT), (0, 0, 0, 0))
    for n in range(count):
        idx = args.start + n
        glyph = Image.new("RGBA", (GLYPH_WIDTH, GLYPH_HEIGHT))
        glyph.putdata(decode_glyph_rgba(fnt, idx, palette))
        x = (n % args.columns) * GLYPH_WIDTH
        y = (n // args.columns) * GLYPH_HEIGHT
        img.alpha_composite(glyph, (x, y))
    if args.scale != 1:
        img = img.resize((img.width * args.scale, img.height * args.scale), Image.Resampling.NEAREST)
    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    img.save(args.output)
    print(f"预览完成：{count}/{total} 个字 → {args.output}")

def cmd_remap(args):
    """CN→JP 映射替换：不改 TBL/PAL，只替换 FNT 中对应索引的字形"""
    import json
    from PIL import Image, ImageDraw, ImageFont

    # 1. 加载 TBL，构建 JP→索引 反向映射
    tbl = read_tbl(Path(args.tbl_in))
    jp_to_idx = {c: i for i, c in enumerate(tbl)}
    print(f"TBL: {len(tbl)} 个字符")

    # 2. 加载 CN→JP 映射
    mapping = json.loads(Path(args.mapping).read_text(encoding="utf-8"))
    pairs = []  # (cn_char, fnt_index)
    for cn_char, jp_char in mapping.items():
        if jp_char in jp_to_idx:
            pairs.append((cn_char, jp_to_idx[jp_char]))
    print(f"映射: {len(mapping)} 条 CN→JP，{len(pairs)} 条可原地替换 ({100*len(pairs)/len(mapping):.1f}%)")

    # 3. 加载 FNT
    fnt = bytearray(read_fnt(Path(args.fnt_in)))
    total_glyphs = len(fnt) // GLYPH_SIZE
    print(f"FNT: {total_glyphs} 个字形")

    # 4. 加载 PAL
    palette = read_palette(Path(args.pal_in))

    # 5. 用 TTF 渲染每个 CN 字符，替换对应位置的字形
    from PIL import Image, ImageDraw, ImageFont
    gw, gh = args.glyph_w, args.glyph_h

    font_size = args.font_size
    if font_size <= 0:
        # 自动计算：临时画布二分搜索
        tmp_img = Image.new("L", (gw * 2, gh * 2), 0)
        tmp_draw = ImageDraw.Draw(tmp_img)
        lo, hi = 8, gh * 3
        font_size = gh
        for _ in range(12):
            mid = (lo + hi) // 2
            try:
                f = ImageFont.truetype(str(args.ttf), mid)
            except Exception:
                lo = mid + 1; continue
            bbox = tmp_draw.textbbox((0, 0), "中", font=f)
            h = bbox[3] - bbox[1]
            if h <= gh - 4: lo = mid + 1; font_size = mid
            else: hi = mid - 1
        print(f"字体大小: {font_size}pt (高度 ≤ {gh-4}px)")

    font = ImageFont.truetype(str(args.ttf), font_size)

    # 建立颜色 → 调色板索引的映射（基于亮度）
    # 调色板 0=透明绿, 1=白..15=黑
    pal_brightness = [(0, True)]  # index 0 = transparent
    for pi in range(1, 16):
        r, g, b = palette[pi]
        brightness = int(0.299 * r + 0.587 * g + 0.114 * b)
        pal_brightness.append((brightness, False))
    max_bright = max(b for b, _ in pal_brightness)
    min_bright = min(b for b, _ in pal_brightness[1:]) if len(pal_brightness) > 1 else 0

    replaced = 0
    draw = ImageDraw.Draw(Image.new("L", (1, 1)))
    for cn_char, idx in pairs:
        if idx >= total_glyphs:
            continue
        # 渲染 CN 字符到 24×34 灰度图
        glyph_img = Image.new("L", (gw, gh), 0)
        gd = ImageDraw.Draw(glyph_img)
        bbox = gd.textbbox((0, 0), cn_char, font=font)
        tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
        x = (gw - tw) // 2 - bbox[0]
        y = (gh - th) // 2 - bbox[1]
        gd.text((x, y), cn_char, font=font, fill=255)

        # 将灰度像素映射到调色板索引
        glyph_bytes = bytearray()
        for py in range(gh):
            for px in range(0, gw, 2):
                hi_val = glyph_img.getpixel((px, py))
                lo_val = glyph_img.getpixel((px + 1, py))
                hi_idx = _brightness_to_pal(hi_val, pal_brightness, max_bright, min_bright)
                lo_idx = _brightness_to_pal(lo_val, pal_brightness, max_bright, min_bright)
                glyph_bytes.append((hi_idx << 4) | lo_idx)

        # 写入 FNT
        pos = idx * GLYPH_SIZE
        fnt[pos:pos + GLYPH_SIZE] = glyph_bytes
        replaced += 1

    # 6. 保存
    Path(args.fnt_out).write_bytes(bytes(fnt))
    print(f"替换完成: {replaced} 个字 → {args.fnt_out}")


def _brightness_to_pal(value: int, pal_brightness: list, max_b: int, min_b: int) -> int:
    """将灰度值 (0-255) 映射到最近的调色板索引 (跳过索引 0=透明)。"""
    if value < 8:
        return 0  # 接近纯黑=背景 → 透明
    # 将 0-255 线性映射到 min_b..max_b
    t = value / 255.0
    target = min_b + t * (max_b - min_b)
    best_idx, best_dist = 1, float('inf')
    for pi in range(1, 16):
        dist = abs(pal_brightness[pi][0] - target)
        if dist < best_dist:
            best_dist, best_idx = dist, pi
    return best_idx


def cmd_inject(args):
    """追加 TBL+FNT 到 EXE，可选 patch 代码"""
    exe  = Path(args.exe).read_bytes()
    tbl  = Path(args.tbl).read_bytes()
    fnt  = Path(args.fnt).read_bytes()
    patched, info = add_font_section(exe, tbl, fnt, args.section.encode("ascii"))
    data = bytearray(patched)
    vals = {"tbl_rva": info["tbl_rva"], "fnt_rva": info["fnt_rva"]}
    for spec in args.patch_u32 or []:
        apply_patch_u32(data, spec, vals)
    Path(args.output).write_bytes(bytes(data))
    for k, v in info.items():
        print(f"  {k} = 0x{v:X}" if isinstance(v, int) else f"  {k} = {v}")
    print(f"写入: {args.output}")
    if not args.patch_u32:
        print("提示: 段已追加但未 patch 代码。用 --patch-u32 指定代码补丁地址。")

def cmd_full(args):
    """全自动流水线：TTF → PNG → FNT/PAL/TBL → 注入 EXE"""
    import tempfile, os
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        # 1. 生成 char_list.txt
        chars: list[str] = []
        seen: set[str] = set()
        def add(ch: str):
            if ch in "\r\n" or ch in seen: return
            try:
                if len(ch.encode("cp932")) != 2: return
            except UnicodeEncodeError: return
            seen.add(ch); chars.append(ch)
        if args.base_tbl:
            for c in read_tbl(Path(args.base_tbl)): add(c)
        if args.json_dir:
            fields = [x.strip() for x in args.fields.split(",") if x.strip()]
            for c in sorted(collect_json_chars(Path(args.json_dir), fields)): add(c)
        if args.fullwidth:
            for c in cp932_two_byte_fullwidth_chars(): add(c)
        char_txt = tmp / "char_list.txt"
        lines = ["".join(chars[i:i+args.columns]) for i in range(0, len(chars), args.columns)]
        char_txt.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"[1/4] 字符表: {len(chars)} 个字符")

        # 2. TTF → PNG
        png = tmp / "char_list.png"
        # 传给 cmd_render 的参数（手动构造 args 太麻烦，直接用内部函数）
        _render_ttf_to_png(args.ttf, char_txt, png, args)
        print(f"[2/4] 渲染 PNG 完成")

        # 3. PNG → FNT/PAL/TBL
        fnt_path = tmp / "FONT.FNT"
        pal_path = tmp / "FONT.PAL"
        tbl_path = tmp / "FONT.TBL"
        _build_png_to_font(char_txt, png, fnt_path, pal_path, tbl_path, args)
        print(f"[3/4] 构建字库完成")

        # 4. 注入 EXE
        exe = Path(args.exe).read_bytes()
        tbl_data = tbl_path.read_bytes()
        fnt_data = fnt_path.read_bytes()
        patched, info = add_font_section(exe, tbl_data, fnt_data, args.section.encode("ascii"))
        data = bytearray(patched)
        vals = {"tbl_rva": info["tbl_rva"], "fnt_rva": info["fnt_rva"]}
        for spec in args.patch_u32 or []:
            apply_patch_u32(data, spec, vals)
        out = Path(args.output) if args.output else Path(args.exe).with_stem(Path(args.exe).stem + "_chs")
        out.write_bytes(bytes(data))
        print(f"[4/4] 注入完成 → {out}")
        print()
        for k, v in info.items():
            print(f"  {k} = 0x{v:X}" if isinstance(v, int) else f"  {k} = {v}")
        if not args.patch_u32:
            print("提示: 未 patch 代码。用 --patch-u32 指定代码补丁。")


# ── 内部辅助（供 full 命令复用）────────────────────────────────────
def _render_ttf_to_png(ttf_path, char_txt_path, png_out, args):
    from PIL import Image, ImageDraw, ImageFont
    text = Path(char_txt_path).read_text(encoding="utf-8")
    chars = split_char_list_text(text)
    gw, gh = args.glyph_w, args.glyph_h
    cols   = args.columns
    rows   = (len(chars) + cols - 1) // cols
    mx, my = args.margin_x, args.margin_y
    cw, ch = gw + mx, gh + my
    sx, sy = args.start_x, args.start_y
    iw, ih = sx * 2 + cols * cw, sy * 2 + rows * ch
    img = Image.new("RGB", (iw, ih), (0, 255, 0))
    draw = ImageDraw.Draw(img)

    font_size = args.font_size
    if font_size <= 0:
        lo, hi = 8, gh * 3
        font_size = gh
        for _ in range(12):
            mid = (lo + hi) // 2
            try:
                f = ImageFont.truetype(str(ttf_path), mid)
            except Exception:
                lo = mid + 1; continue
            bbox = draw.textbbox((0, 0), "あ", font=f)
            h = bbox[3] - bbox[1]
            if h <= gh - 4: lo = mid + 1; font_size = mid
            else: hi = mid - 1

    font = ImageFont.truetype(str(ttf_path), font_size)
    for i, c in enumerate(chars):
        row, col = divmod(i, cols)
        xc = sx + col * cw + cw // 2
        yc = sy + row * ch + ch // 2
        bb = draw.textbbox((0, 0), c, font=font)
        tw, th = bb[2] - bb[0], bb[3] - bb[1]
        x = xc - tw // 2 - bb[0]
        y = yc - th // 2 - bb[1]
        draw.text((x, y), c, font=font, fill=(255, 255, 255))

    pal_img = img.convert("P", palette=Image.Palette.ADAPTIVE, colors=16)
    pd = pal_img.getpalette() or []
    green_idx = None
    for pi in range(16):
        r, g, b = pd[pi*3:pi*3+3]
        if r == 0 and g == 255 and b == 0:
            green_idx = pi; break
    if green_idx is not None and green_idx != 0:
        p = pd
        p[0:3], p[green_idx*3:green_idx*3+3] = p[green_idx*3:green_idx*3+3], p[0:3]
        pal_img.putpalette(p)
        px = pal_img.load()
        for y in range(ih):
            for x in range(iw):
                v = px[x, y]
                if v == 0: px[x, y] = green_idx
                elif v == green_idx: px[x, y] = 0
    pal_img.save(png_out)

def _build_png_to_font(char_txt_path, png_path, fnt_out, pal_out, tbl_out, args):
    from PIL import Image
    text = Path(char_txt_path).read_text(encoding="utf-8")
    lines = [line.rstrip("\r\n") for line in text.splitlines() if line.strip()]
    flat_chars = split_char_list_text(text)
    img = Image.open(png_path)
    if img.mode != "P":
        raise ValueError("PNG 必须是索引颜色模式")
    pal = img.getpalette() or []
    write_palette(Path(pal_out), pal[:16 * 3])
    pixels = img.load()
    font = bytearray()
    chars_for_tbl = []
    mx, my = args.margin_x, args.margin_y
    cw, c_h = GLYPH_WIDTH + mx, GLYPH_HEIGHT + my
    if args.columns is not None:
        for idx, c in enumerate(flat_chars):
            row, col = divmod(idx, args.columns)
            chars_for_tbl.append(c)
            x0 = args.start_x + col * cw
            y0 = args.start_y + row * c_h
            for y in range(y0, y0 + GLYPH_HEIGHT):
                for x in range(x0, x0 + GLYPH_WIDTH, 2):
                    font.append(((pixels[x, y] & 0xF) << 4) | (pixels[x+1, y] & 0xF))
    else:
        for row, line in enumerate(lines):
            for col, c in enumerate(split_char_list_text(line)):
                chars_for_tbl.append(c)
                x0 = args.start_x + col * cw
                y0 = args.start_y + row * c_h
                for y in range(y0, y0 + GLYPH_HEIGHT):
                    for x in range(x0, x0 + GLYPH_WIDTH, 2):
                        font.append(((pixels[x, y] & 0xF) << 4) | (pixels[x+1, y] & 0xF))
    encoding = getattr(args, 'encoding', 'cp932')
    Path(fnt_out).write_bytes(bytes(font))
    write_tbl(Path(tbl_out), chars_for_tbl, encoding)

# ══════════════════════════════════════════════════════════════════════
#  CLI
# ══════════════════════════════════════════════════════════════════════
def main():
    ap = argparse.ArgumentParser(
        description="fontkit.py — AI5WIN 字库一站式工具",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  python fontkit.py tbl decode FONT.TBL tbl.txt
  python fontkit.py tbl encode tbl.txt FONT.TBL --encoding cp932
  python fontkit.py charset char_list.txt --base-tbl FONT.TBL --fullwidth
  python fontkit.py render font.ttf char_list.txt char_list.png
  python fontkit.py build char_list.txt char_list.png FONT.FNT FONT.PAL FONT.TBL
  python fontkit.py dump FONT.FNT FONT.PAL preview.png --scale 2
  python fontkit.py inject game.exe FONT.TBL FONT.FNT -o patched.exe --patch-u32 0x4898E=tbl_rva
  python fontkit.py full game.exe font.ttf --base-tbl FONT.TBL --fullwidth -o chs.exe --patch-u32 0x4898E=tbl_rva
""")
    sub = ap.add_subparsers(dest="cmd")

    # ── tbl ──────────────────────────────────────────────────────
    p = sub.add_parser("tbl", help="解码/编码 FONT.TBL")
    p.add_argument("tbl_cmd", choices=["decode", "encode"])
    p.add_argument("input")
    p.add_argument("output")
    p.add_argument("--encoding", default="cp932")
    p.add_argument("--columns", type=int, default=80)
    p.set_defaults(func=cmd_tbl)

    # ── charset ──────────────────────────────────────────────────
    p = sub.add_parser("charset", help="生成 char_list.txt")
    p.add_argument("output")
    p.add_argument("--base-tbl", default="")
    p.add_argument("--json-dir", default="")
    p.add_argument("--fields", default="message")
    p.add_argument("--fullwidth", action="store_true")
    p.add_argument("--columns", type=int, default=80)
    p.set_defaults(func=cmd_charset)

    # ── render (NEW!) ────────────────────────────────────────────
    p = sub.add_parser("render", help="用 TTF 渲染字符网格 PNG（替代 Photoshop）")
    p.add_argument("ttf", help="TTF/OTF 字体文件")
    p.add_argument("chars", help="char_list.txt 字符表")
    p.add_argument("output", help="输出 PNG 文件")
    p.add_argument("--glyph-w", type=int, default=24, help="字形宽度 (默认 24)")
    p.add_argument("--glyph-h", type=int, default=34, help="字形高度 (默认 34)")
    p.add_argument("--margin-x", type=int, default=4, help="水平间距 (默认 4)")
    p.add_argument("--margin-y", type=int, default=0, help="垂直间距 (默认 0)")
    p.add_argument("--start-x", type=int, default=12)
    p.add_argument("--start-y", type=int, default=12)
    p.add_argument("--columns", type=int, default=80, help="每行字符数")
    p.add_argument("--font-size", type=int, default=0, help="字体大小 (0=自动计算)")
    p.set_defaults(func=cmd_render)

    # ── build ────────────────────────────────────────────────────
    p = sub.add_parser("build", help="PNG → FNT/PAL/TBL")
    p.add_argument("chars", help="char_list.txt")
    p.add_argument("png", help="char_list.png (索引色)")
    p.add_argument("fnt")
    p.add_argument("pal")
    p.add_argument("tbl")
    p.add_argument("--start-x", type=int, default=12)
    p.add_argument("--start-y", type=int, default=12)
    p.add_argument("--margin-x", type=int, default=4)
    p.add_argument("--margin-y", type=int, default=0)
    p.add_argument("--columns", type=int, default=None)
    p.add_argument("--encoding", default="cp932")
    p.set_defaults(func=cmd_build)

    # ── dump ─────────────────────────────────────────────────────
    p = sub.add_parser("dump", help="FNT → PNG 预览")
    p.add_argument("fnt")
    p.add_argument("pal")
    p.add_argument("output")
    p.add_argument("--columns", type=int, default=32)
    p.add_argument("--start", type=int, default=0)
    p.add_argument("--count", type=int, default=None)
    p.add_argument("--scale", type=int, default=1)
    p.set_defaults(func=cmd_dump)

    # ── remap (NEW!) ────────────────────────────────────────────
    p = sub.add_parser("remap", help="CN→JP 映射替换：不改 TBL，只换 FNT 中对应索引的字形")
    p.add_argument("fnt_in", help="原始 FONT.FNT")
    p.add_argument("pal_in", help="原始 FONT.PAL（保留调色板）")
    p.add_argument("tbl_in", help="原始 FONT.TBL（保留索引映射）")
    p.add_argument("ttf", help="中文字体 TTF")
    p.add_argument("mapping", help="subs_cn_jp.json（CN→JP 字符映射）")
    p.add_argument("fnt_out", help="输出 FNT（仅修改匹配的字形）")
    p.add_argument("--font-size", type=int, default=0, help="字体大小 (0=自动计算)")
    p.add_argument("--glyph-w", type=int, default=24)
    p.add_argument("--glyph-h", type=int, default=34)
    p.set_defaults(func=cmd_remap)

    # ── inject ───────────────────────────────────────────────────
    p = sub.add_parser("inject", help="追加 TBL+FNT 段到 EXE")
    p.add_argument("exe")
    p.add_argument("tbl")
    p.add_argument("fnt")
    p.add_argument("-o", "--output", required=True)
    p.add_argument("--section", default=".font")
    p.add_argument("--patch-u32", action="append", default=[])
    p.set_defaults(func=cmd_inject)

    # ── full (NEW!) ──────────────────────────────────────────────
    p = sub.add_parser("full", help="一条龙：TTF → PNG → FNT/PAL/TBL → 注入 EXE")
    p.add_argument("exe")
    p.add_argument("ttf", help="TTF 字体文件")
    p.add_argument("-o", "--output", default="")
    p.add_argument("--base-tbl", default="")
    p.add_argument("--json-dir", default="")
    p.add_argument("--fields", default="message")
    p.add_argument("--fullwidth", action="store_true")
    p.add_argument("--columns", type=int, default=80)
    p.add_argument("--glyph-w", type=int, default=24)
    p.add_argument("--glyph-h", type=int, default=34)
    p.add_argument("--margin-x", type=int, default=4)
    p.add_argument("--margin-y", type=int, default=0)
    p.add_argument("--start-x", type=int, default=12)
    p.add_argument("--start-y", type=int, default=12)
    p.add_argument("--font-size", type=int, default=0)
    p.add_argument("--section", default=".font")
    p.add_argument("--patch-u32", action="append", default=[])
    p.set_defaults(func=cmd_full)

    args = ap.parse_args()
    if not args.cmd:
        ap.print_help()
        return
    args.func(args)

if __name__ == "__main__":
    main()
