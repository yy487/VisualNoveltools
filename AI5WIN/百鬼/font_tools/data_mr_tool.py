# -*- coding: utf-8 -*-
"""Tools for Baigui / AI5WIN DATA.MR font archives.

DATA.MR is an AI5 LZSS stream. The decompressed payload starts with:

    u32 count = 42
    u32 offsets[count]

The first offset points to the character table. The character table starts
with a u16 glyph count followed by glyph_count CP932 double-byte character
codes. Segment 3 is the main 24x24 16-bit grayscale glyph atlas, one 1152-byte
record per glyph (24 * 24 * 2).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import struct
import unicodedata
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


FRAME_SIZE = 0x1000
FRAME_INIT = 0xFEE
FRAME_MASK = 0xFFF
FRAME_FILL = 0x00
MAX_MATCH = 18
MIN_MATCH = 3
PUNCTUATION_LIKE_CHARS = "ー"


def lzss_decompress(src: bytes, expected_size: int | None = None) -> bytes:
    frame = bytearray([FRAME_FILL]) * FRAME_SIZE
    frame_pos = FRAME_INIT
    flags = 0
    i = 0
    out = bytearray()

    while i < len(src):
        flags >>= 1
        if (flags & 0x100) == 0:
            flags = src[i] | 0xFF00
            i += 1
        if flags & 1:
            if i >= len(src):
                break
            b = src[i]
            i += 1
            out.append(b)
            frame[frame_pos] = b
            frame_pos = (frame_pos + 1) & FRAME_MASK
        else:
            if i + 1 >= len(src):
                break
            lo = src[i]
            hi = src[i + 1]
            i += 2
            pos = lo | ((hi & 0xF0) << 4)
            length = (hi & 0x0F) + MIN_MATCH
            for k in range(length):
                b = frame[(pos + k) & FRAME_MASK]
                out.append(b)
                frame[frame_pos] = b
                frame_pos = (frame_pos + 1) & FRAME_MASK
                if expected_size is not None and len(out) >= expected_size:
                    return bytes(out)
    return bytes(out)


def _key_at(frame: bytearray, pos: int) -> bytes:
    return bytes((frame[pos], frame[(pos + 1) & FRAME_MASK], frame[(pos + 2) & FRAME_MASK]))


def _find_match(frame: bytearray, frame_pos: int, look: bytes, pos_index: dict[bytes, set[int]]) -> tuple[int, int]:
    limit = min(MAX_MATCH, len(look))
    if limit < MIN_MATCH:
        return 0, 0
    candidates = pos_index.get(look[:3], ())
    best_pos = 0
    best_len = 0

    for cand in candidates:
        overlay: dict[int, int] = {}
        sim_fp = frame_pos & FRAME_MASK
        length = 0
        while length < limit:
            rp = (cand + length) & FRAME_MASK
            b = overlay.get(rp, frame[rp])
            if b != look[length]:
                break
            overlay[sim_fp] = look[length]
            sim_fp = (sim_fp + 1) & FRAME_MASK
            length += 1
        if length > best_len:
            best_pos = cand
            best_len = length
            if best_len == limit:
                break
    if best_len < MIN_MATCH:
        return 0, 0
    return best_pos, best_len


def _build_index(frame: bytearray) -> tuple[dict[bytes, set[int]], list[bytes]]:
    index: dict[bytes, set[int]] = {}
    pos_keys: list[bytes] = []
    for pos in range(FRAME_SIZE):
        key = _key_at(frame, pos)
        pos_keys.append(key)
        index.setdefault(key, set()).add(pos)
    return index, pos_keys


def _put_byte(frame: bytearray, pos_index: dict[bytes, set[int]], pos_keys: list[bytes], frame_pos: int, value: int) -> None:
    fp = frame_pos & FRAME_MASK
    frame[fp] = value
    for off in (-2, -1, 0):
        p = (fp + off) & FRAME_MASK
        old_key = pos_keys[p]
        new_key = _key_at(frame, p)
        if old_key == new_key:
            continue
        bucket = pos_index.get(old_key)
        if bucket is not None:
            bucket.discard(p)
            if not bucket:
                pos_index.pop(old_key, None)
        pos_keys[p] = new_key
        pos_index.setdefault(new_key, set()).add(p)


def lzss_compress(src: bytes) -> bytes:
    # Simple and deterministic. Rebuilding the 4 KiB index after each token is
    # slower than a mutable linked index, but this keeps the compressor compact
    # and exact enough for the 5-8 MiB DATA.MR payloads.
    frame = bytearray([FRAME_FILL]) * FRAME_SIZE
    frame_pos = FRAME_INIT
    i = 0
    out = bytearray()
    pos_index, pos_keys = _build_index(frame)

    while i < len(src):
        flag_pos = len(out)
        out.append(0)
        flags = 0
        for bit in range(8):
            if i >= len(src):
                break
            match_pos, match_len = _find_match(frame, frame_pos, src[i:i + MAX_MATCH], pos_index)
            if match_len:
                out.append(match_pos & 0xFF)
                out.append(((match_pos >> 4) & 0xF0) | ((match_len - MIN_MATCH) & 0x0F))
                for k in range(match_len):
                    _put_byte(frame, pos_index, pos_keys, frame_pos, src[i + k])
                    frame_pos = (frame_pos + 1) & FRAME_MASK
                i += match_len
            else:
                b = src[i]
                i += 1
                flags |= 1 << bit
                out.append(b)
                _put_byte(frame, pos_index, pos_keys, frame_pos, b)
                frame_pos = (frame_pos + 1) & FRAME_MASK
        out[flag_pos] = flags
    return bytes(out)


def lzss_compress_external(raw_path: Path, mr_path: Path, helper: Path | None = None) -> dict[str, object] | None:
    if helper is None:
        helper = Path(__file__).with_name("baigui_lzss_pack.exe")
    if not helper.exists():
        return None

    proc = subprocess.run(
        [str(helper), str(raw_path), str(mr_path)],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(proc.stdout)


class DataMr:
    def __init__(self, raw: bytes):
        self.raw = raw
        self.count = struct.unpack_from("<I", raw, 0)[0]
        self.offsets = [struct.unpack_from("<I", raw, 4 + i * 4)[0] for i in range(self.count)]
        if self.count != 42:
            raise ValueError(f"unexpected section count: {self.count}")
        if self.offsets[0] != 4 + self.count * 4:
            raise ValueError("bad first section offset")
        if self.offsets[-1] != len(raw):
            raise ValueError("last offset does not equal file size")

    @property
    def glyph_count(self) -> int:
        return struct.unpack_from("<H", self.raw, self.offsets[0])[0]

    @property
    def chars(self) -> list[str]:
        base = self.offsets[0] + 2
        return [self.raw[base + i * 2:base + i * 2 + 2].decode("cp932") for i in range(self.glyph_count)]

    @property
    def glyph_segment(self) -> bytes:
        return self.raw[self.offsets[3]:self.offsets[4]]

    def section(self, index: int) -> bytes:
        return self.raw[self.offsets[index]:self.offsets[index + 1]]

    def info(self) -> dict[str, object]:
        sizes = [self.offsets[i + 1] - self.offsets[i] for i in range(self.count - 1)]
        return {
            "raw_size": len(self.raw),
            "section_count": self.count,
            "glyph_count": self.glyph_count,
            "section_sizes": sizes,
            "glyph_segment_bytes": len(self.glyph_segment),
            "glyph_record_bytes": len(self.glyph_segment) // max(1, self.glyph_count),
        }


def read_mr(path: Path, expected_size: int | None = None) -> DataMr:
    return DataMr(lzss_decompress(path.read_bytes(), expected_size))


def is_punctuation_like(char: str) -> bool:
    return unicodedata.category(char)[0] in "PS" or char in PUNCTUATION_LIKE_CHARS


def render_glyph(char: str, font: ImageFont.FreeTypeFont, canvas: int = 24, offset_x: int = 0, offset_y: int = 0) -> bytes:
    hi = Image.new("L", (canvas * 4, canvas * 4), 0)
    draw = ImageDraw.Draw(hi)
    bbox = draw.textbbox((0, 0), char, font=font)
    w = bbox[2] - bbox[0]
    h = bbox[3] - bbox[1]
    scale = hi.width // canvas
    x = (hi.width - w) // 2 - bbox[0] + offset_x * scale
    y = (hi.height - h) // 2 - bbox[1] + offset_y * scale
    draw.text((x, y), char, font=font, fill=255)
    small = hi.resize((canvas, canvas), Image.Resampling.LANCZOS)
    out = bytearray()
    for value in small.tobytes():
        v = max(0, min(0x20, round(value * 0x20 / 255)))
        out.extend((v, v))
    return bytes(out)


def render_glyph_mask(char: str, font: ImageFont.FreeTypeFont, canvas: int = 24, offset_x: int = 0, offset_y: int = 0) -> bytes:
    hi = Image.new("L", (canvas * 4, canvas * 4), 0)
    draw = ImageDraw.Draw(hi)
    bbox = draw.textbbox((0, 0), char, font=font)
    w = bbox[2] - bbox[0]
    h = bbox[3] - bbox[1]
    scale = hi.width // canvas
    x = (hi.width - w) // 2 - bbox[0] + offset_x * scale
    y = (hi.height - h) // 2 - bbox[1] + offset_y * scale
    draw.text((x, y), char, font=font, fill=255)
    small = hi.resize((canvas, canvas), Image.Resampling.LANCZOS)
    return bytes(max(0, min(0x20, round(value * 0x20 / 255))) for value in small.tobytes())


def read_char_table(section: bytes) -> list[str]:
    count = struct.unpack_from("<H", section, 0)[0]
    chars = []
    for i in range(count):
        chars.append(section[2 + i * 2:2 + i * 2 + 2].decode("cp932"))
    return chars


def invert_cn_jp_mapping(mapping: dict[str, str]) -> dict[str, str]:
    inverted: dict[str, str] = {}
    collisions: list[tuple[str, str, str]] = []
    for cn, borrowed in mapping.items():
        previous = inverted.get(borrowed)
        if previous is not None and previous != cn:
            collisions.append((borrowed, previous, cn))
        inverted[borrowed] = cn
    if collisions:
        sample = ", ".join(f"{borrowed}:{old}/{new}" for borrowed, old, new in collisions[:8])
        raise ValueError(f"duplicate borrowed characters in mapping: {sample}")
    return inverted


def build_redrawn(
    base: DataMr,
    borrowed_to_source: dict[str, str],
    font_path: Path,
    font_size: int,
    preserve_punctuation: bool = True,
) -> bytes:
    chars = base.chars
    font = ImageFont.truetype(str(font_path), font_size)
    glyphs = bytearray()
    cache: dict[str, bytes] = {}
    base_glyphs = base.glyph_segment
    for idx, char in enumerate(chars):
        source_char = borrowed_to_source.get(char, char)
        if (
            preserve_punctuation
            and is_punctuation_like(char)
            and (char not in borrowed_to_source or is_punctuation_like(source_char))
        ):
            glyphs.extend(base_glyphs[idx * 1152:(idx + 1) * 1152])
            continue
        rec = cache.get(source_char)
        if rec is None:
            rec = render_glyph(source_char, font)
            cache[source_char] = rec
        glyphs.extend(rec)

    sections = [base.section(i) for i in range(base.count - 1)]
    sections[3] = bytes(glyphs)

    # Sections 4 and 5 participate in runtime resource lookup. Rebuilding them
    # as independent glyph atlases makes DATA.MR loading fail, even though their
    # lengths resemble 24x24 image records. Preserve all non-main sections.
    offsets = []
    cursor = 4 + base.count * 4
    for section in sections:
        offsets.append(cursor)
        cursor += len(section)
    offsets.append(cursor)

    out = bytearray()
    out.extend(struct.pack("<I", base.count))
    for off in offsets:
        out.extend(struct.pack("<I", off))
    for section in sections:
        out.extend(section)
    return bytes(out)


def dump_sheet(mr: DataMr, out_path: Path, limit: int = 256) -> None:
    chars = mr.chars
    glyphs = mr.glyph_segment
    n = min(limit, len(chars), len(glyphs) // 1152)
    cell = 32
    cols = 16
    rows = (n + cols - 1) // cols
    img = Image.new("L", (cols * cell, rows * cell), 0)
    for idx in range(n):
        rec = glyphs[idx * 1152:(idx + 1) * 1152]
        tile = Image.new("L", (24, 24), 0)
        tile.putdata([min(255, rec[i] * 8) for i in range(0, len(rec), 2)])
        img.paste(tile, ((idx % cols) * cell + 4, (idx // cols) * cell + 2))
    out_path.parent.mkdir(parents=True, exist_ok=True)
    img.save(out_path)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_info = sub.add_parser("info")
    p_info.add_argument("mr", type=Path)
    p_info.add_argument("--expected-size", type=lambda s: int(s, 0))

    p_unpack = sub.add_parser("unpack")
    p_unpack.add_argument("mr", type=Path)
    p_unpack.add_argument("raw", type=Path)
    p_unpack.add_argument("--expected-size", type=lambda s: int(s, 0))

    p_pack = sub.add_parser("pack")
    p_pack.add_argument("raw", type=Path)
    p_pack.add_argument("mr", type=Path)
    p_pack.add_argument("--python-compressor", action="store_true")

    p_sheet = sub.add_parser("sheet")
    p_sheet.add_argument("mr", type=Path)
    p_sheet.add_argument("png", type=Path)
    p_sheet.add_argument("--expected-size", type=lambda s: int(s, 0))
    p_sheet.add_argument("--limit", type=int, default=256)

    p_redraw = sub.add_parser("redraw")
    p_redraw.add_argument("base_mr", type=Path)
    p_redraw.add_argument("mapping_json", type=Path)
    p_redraw.add_argument("font", type=Path)
    p_redraw.add_argument("out_mr", type=Path)
    p_redraw.add_argument("--expected-size", type=lambda s: int(s, 0), default=0x77053E)
    p_redraw.add_argument("--font-size", type=int, default=88)
    p_redraw.add_argument("--raw-out", type=Path)
    p_redraw.add_argument("--python-compressor", action="store_true")
    p_redraw.add_argument(
        "--redraw-punctuation",
        action="store_true",
        help="redraw punctuation/symbol glyphs instead of preserving their original game layout",
    )

    args = parser.parse_args()
    if args.cmd == "info":
        mr = read_mr(args.mr, args.expected_size)
        print(json.dumps(mr.info(), ensure_ascii=False, indent=2))
    elif args.cmd == "unpack":
        raw = lzss_decompress(args.mr.read_bytes(), args.expected_size)
        args.raw.write_bytes(raw)
        print(json.dumps({"raw_size": len(raw), "sha256": sha256(raw)}, indent=2))
    elif args.cmd == "pack":
        raw = args.raw.read_bytes()
        if not args.python_compressor:
            report = lzss_compress_external(args.raw, args.mr)
        else:
            report = None
        if report is None:
            packed = lzss_compress(raw)
            args.mr.write_bytes(packed)
        else:
            packed = args.mr.read_bytes()
        check = lzss_decompress(packed, len(raw))
        if check != raw:
            raise RuntimeError("compressed stream failed round-trip verification")
        print(json.dumps({"packed_size": len(packed), "raw_size": len(raw), "sha256": sha256(packed)}, indent=2))
    elif args.cmd == "sheet":
        mr = read_mr(args.mr, args.expected_size)
        dump_sheet(mr, args.png, args.limit)
        print(args.png)
    elif args.cmd == "redraw":
        base = read_mr(args.base_mr, args.expected_size)
        mapping = json.loads(args.mapping_json.read_text(encoding="utf-8"))
        borrowed_to_source = invert_cn_jp_mapping(mapping)
        raw = build_redrawn(
            base,
            borrowed_to_source,
            args.font,
            args.font_size,
            preserve_punctuation=not args.redraw_punctuation,
        )
        raw_path = args.raw_out or args.out_mr.with_suffix(args.out_mr.suffix + ".raw")
        raw_path.write_bytes(raw)
        if not args.python_compressor:
            report = lzss_compress_external(raw_path, args.out_mr)
        else:
            report = None
        if report is None:
            packed = lzss_compress(raw)
            args.out_mr.write_bytes(packed)
        else:
            packed = args.out_mr.read_bytes()
        if lzss_decompress(packed, len(raw)) != raw:
            raise RuntimeError("redrawn stream failed round-trip verification")
        print(json.dumps({
            "raw_size": len(raw),
            "packed_size": len(packed),
            "sha256": sha256(packed),
            "mapped_glyphs": sum(1 for ch in base.chars if ch in borrowed_to_source),
            "preserved_punctuation_glyphs": sum(
                1
                for ch in base.chars
                if is_punctuation_like(ch)
                and (ch not in borrowed_to_source or is_punctuation_like(borrowed_to_source[ch]))
            ) if not args.redraw_punctuation else 0,
            "raw_out": str(raw_path),
        }, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
