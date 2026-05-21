# -*- coding: utf-8 -*-
from __future__ import annotations

from pathlib import Path
from typing import Iterable

GLYPH_WIDTH = 24
GLYPH_HEIGHT = 34
GLYPH_SIZE = GLYPH_WIDTH * GLYPH_HEIGHT // 2


def read_palette(path: Path) -> list[tuple[int, int, int]]:
    data = path.read_bytes()
    if len(data) < 16 * 3:
        raise ValueError(f"PAL too small: {path} size={len(data)}")
    return [(data[i], data[i + 1], data[i + 2]) for i in range(0, 16 * 3, 3)]


def write_palette(path: Path, palette: Iterable[int] | bytes | bytearray) -> None:
    raw = bytes(palette)
    if len(raw) < 16 * 3:
        raw = raw + b"\x00" * (16 * 3 - len(raw))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw[:16 * 3])


def read_fnt(path: Path) -> bytes:
    data = path.read_bytes()
    if len(data) % GLYPH_SIZE != 0:
        raise ValueError(f"FNT size is not a multiple of {GLYPH_SIZE}: {path} size={len(data)}")
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


def read_tbl(path: Path, encoding: str = "cp932") -> list[str]:
    """Read FONT.TBL. Each entry is stored as reversed CP932 two-byte code."""
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


def write_tbl(path: Path, chars: Iterable[str], encoding: str = "cp932") -> None:
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
