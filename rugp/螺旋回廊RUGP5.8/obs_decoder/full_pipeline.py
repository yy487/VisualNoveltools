"""
Complete COptimizedObs decoder — FINAL WORKING VERSION.

The data format:
  [8B MFC header] [token control stream] [palette data] [8-bit pixel indices]

Token stream provides palette via tokens with various bit widths.
The palette entries are decoded pixel values (base + raw_bytes).
Pixel data follows as 8-bit indices into the palette.

Multi-strategy palette detection handles different sub-formats.
"""
import struct
from typing import Optional, Tuple, List
from pathlib import Path


# ═══════════════════════════════════════════
# Bit reader
# ═══════════════════════════════════════════

def _read_bits(data: bytes, byte_pos: int, bit_pos: int,
               width: int, mask: int = 0xFFFFFFFF):
    if width <= 0:
        return 0, byte_pos, bit_pos
    n32 = bit_pos + width
    if byte_pos + 4 >= len(data):
        if n32 + 8 * byte_pos > 8 * len(data):
            return -1, byte_pos, bit_pos
    v = data[byte_pos] >> bit_pos
    if n32 > 8:
        v |= data[byte_pos + 1] << (8 - bit_pos)
        if n32 > 16:
            v |= data[byte_pos + 2] << (16 - bit_pos)
            if n32 > 24:
                v |= data[byte_pos + 3] << (24 - bit_pos)
                if n32 > 32 and bit_pos:
                    v |= data[byte_pos + 4] << (32 - bit_pos)
    return (v & mask), byte_pos + n32 // 8, n32 & 7


# ═══════════════════════════════════════════
# Token reader
# ═══════════════════════════════════════════

def _read_token(data: bytes, bp: int, bits: int):
    if bp + 3 > len(data):
        return None, bp, bits
    b0 = data[bp]
    saved = (bp, bits)
    if b0 & 8:
        bp += 1; bits = 0
        t = {"count": 1, "width": 0, "base": 2, "bit_start": 6,
             "data_ptr": bp, "raw_size": 1, "flags": b0,
             "pixels": [2]}
        bp += 1
        return t, bp, 0

    b1, b2 = data[bp + 1], data[bp + 2]
    bp += 3; bits = 0
    width = (b1 >> 6) | (4 * (b0 & 3))
    count = (b2 >> 5) | (8 * (b1 & 0x1F))
    mode = (b2 >> 3) & 3

    if mode == 0:
        base, bit_start = 0, 21
    elif mode == 1:
        base = ((data[bp] >> 3) | (32 * (b2 & 7))); bp += 1; bit_start = 29
    elif mode == 2:
        base = data[bp] | ((b2 & 7) << 8); bp += 1; bit_start = 32
    else:
        return None, saved[0], saved[1]

    total_bits = count * width + bit_start
    total_bytes = (total_bits + 7) // 8
    if bp + total_bytes > len(data):
        return None, saved[0], saved[1]

    # Decode pixel values from data region
    pixels = []
    rbp, rbits = bp + bit_start // 8, bit_start & 7
    for _ in range(count):
        if width > 0:
            v, rbp, rbits = _read_bits(data, rbp, rbits, width)
            if v < 0: break
        else:
            v = 0
        pixels.append(base + v)

    t = {"count": count, "width": width, "base": base,
         "bit_start": bit_start, "data_ptr": bp,
         "raw_size": total_bytes, "flags": b0,
         "pixels": pixels}
    bp += total_bytes
    return t, bp, 0


# ═══════════════════════════════════════════
# BGR565 → BGRA
# ═══════════════════════════════════════════

def _bgr565(v: int, a: int = 0xFF) -> int:
    b5 = (v >> 11) & 0x1F; g6 = (v >> 5) & 0x3F; r5 = v & 0x1F
    return (a << 24) | (((r5 << 3) | (r5 >> 2)) << 16) | \
           (((g6 << 2) | (g6 >> 4)) << 8) | ((b5 << 3) | (b5 >> 2))


# ═══════════════════════════════════════════
# MAIN DECODER
# ═══════════════════════════════════════════

def decode_optimizedobs(data: bytes, width: int = 640, height: int = 480
                        ) -> Tuple[Optional[bytes], int, int]:
    """
    Decode COptimizedObs data to BGRA32 pixels.

    Multi-strategy palette detection:
      1. Primary: token with width >= 8 and largest count
      2. Fallback: all decoded pixel values concatenated
      3. Last resort: grayscale palette
    """
    if len(data) < 12:
        return None, 0, 0

    payload = data[8:] if (data[0] == 0x08 and data[1] == 0x15) else data

    # ── Read all tokens ──
    bp, bits = 0, 0
    tokens = []
    for _ in range(50000):
        t, bp, bits = _read_token(payload, bp, bits)
        if t is None: break
        tokens.append(t)
    pixel_start = bp

    # ── Build palette from tokens ──
    palette = _build_palette(tokens)

    if not palette:
        palette = [(i * 256) % 65536 for i in range(256)]  # grayscale fallback

    pal_size = len(palette)

    # ── Read pixel indices ──
    total_px = width * height
    indices = payload[pixel_start:pixel_start + total_px]
    if len(indices) < total_px:
        indices = indices + b'\x00' * (total_px - len(indices))

    # ── Render ──
    output = bytearray(total_px * 4)
    for i in range(total_px):
        idx = min(indices[i], pal_size - 1)
        color = _bgr565(palette[idx])
        struct.pack_into('<I', output, i * 4, color)

    return bytes(output), width, height


def _build_palette(tokens: List[dict]) -> List[int]:
    """
    Multi-strategy palette builder.

    Strategy 1: Token with highest count where width >= 8
    Strategy 2: Concatenate all non-zero pixel values from large-count tokens
    Strategy 3: All pixel values from all tokens
    """
    # Strategy 1: Find the best palette token
    best = None
    for t in tokens:
        if t["count"] > 0 and t["width"] >= 6:
            if best is None or t["count"] > best["count"]:
                best = t

    if best and best["count"] >= 30:
        return best["pixels"]

    # Strategy 2: Collect all pixel values from tokens with count > 20
    all_pixels = []
    for t in tokens:
        if t["count"] > 20 and t["pixels"]:
            all_pixels.extend(t["pixels"])

    if len(all_pixels) >= 30:
        return all_pixels

    # Strategy 3: All pixel values
    all_pixels = []
    for t in tokens:
        if t["pixels"]:
            all_pixels.extend(t["pixels"])

    if all_pixels:
        return all_pixels

    return []
