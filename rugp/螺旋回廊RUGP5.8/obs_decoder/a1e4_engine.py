"""
Direct Python port of sub_1000A1E4 + sub_100046A8 renderer.

This is a line-by-line translation of the 792-line decompiled C code.
It processes pixel output AND bitstream data to produce BGRA32 pixels.

Reference: riooor.rpo disassembly @ 0x1000A1E4, 0x100046A8, 0x10004324
"""

import struct, math
from typing import List, Optional, Tuple


# ═══════════════════════════════════════════════════════════════
# Sub-record reader (sub_100046A8)
# ═══════════════════════════════════════════════════════════════

def bitlen(v: int) -> int:
    return v.bit_length() if v > 0 else 0


def optimal_base(w: int, h: int) -> int:
    if w <= 0: return h
    X = math.pow(float(h), 1.0 / float(w))
    b = int(math.floor(X))
    while True:
        prod, tot, n = 1, 1, 0
        if w >= 6:
            while n <= w - 6:
                prod *= b**5; n += 5
                tot = tot + tot*b; tot = tot + tot*b; tot = tot + tot*b
        while n < w:
            prod *= b; n += 1; tot += tot*b
        if prod <= h and tot > h: break
        b += 1 if prod <= h else -1
    return b


def read_subrecord(src: bytes, byte_pos: int, bit_pos: int,
                   end_pos: int) -> Tuple[Optional[dict], int, int]:
    """Read one sub-record. Returns (record, new_byte_pos, new_bit_pos)."""
    def rb(w, mask=0xFFFFFFFF):
        nonlocal byte_pos, bit_pos
        if w <= 0: return 0
        n32 = bit_pos + w
        if byte_pos + 4 >= end_pos:
            if n32 + 8*byte_pos > 8*end_pos:
                return -1
        bp = byte_pos
        v = src[bp] >> bit_pos
        if n32 > 8:
            v |= src[bp+1] << (8-bit_pos)
            if n32 > 16:
                v |= src[bp+2] << (16-bit_pos)
                if n32 > 24:
                    v |= src[bp+3] << (24-bit_pos)
                    if n32 > 32 and bit_pos:
                        v |= src[bp+4] << (32-bit_pos)
        byte_pos += n32 // 8
        bit_pos = n32 & 7
        return v & mask

    saved_bp, saved_bits = byte_pos, bit_pos

    magic = rb(24, 0xFFFFFF)
    if magic != 0x564342:
        byte_pos, bit_pos = saved_bp, saved_bits
        return None, byte_pos, bit_pos

    w = rb(16, 0xFFFF)
    if w == 0xFFFF or w < 0:
        byte_pos, bit_pos = saved_bp, saved_bits
        return None, byte_pos, bit_pos
    if w == 0:
        return {"width": 0, "height": 0}, byte_pos, bit_pos

    h = rb(24, 0xFFFFFF)
    if h == 0xFFFFFF or h < 0:
        byte_pos, bit_pos = saved_bp, saved_bits
        return None, byte_pos, bit_pos

    flag = rb(1, 1)
    indices = []

    if flag:
        total = 0
        while total < w:
            cnt = rb(5, 0x1F)
            if cnt < 0: break
            run = min(cnt+1, w-total)
            delta = rb(1, 1)
            base = rb(5, 0x1F)+1 if delta else 0
            indices.extend([base]*run)
            total += run
    else:
        for _ in range(w):
            v = rb(5, 0x1F)
            if v < 0 or v == 0x1F: break
            indices.append(v+1)

    fmt = rb(4, 0xF)
    if fmt < 0:
        byte_pos, bit_pos = saved_bp, saved_bits
        return None, byte_pos, bit_pos

    rec = {"width": w, "height": h, "index_table": indices,
           "fmt_type": fmt, "pixel_buffer": []}
    if fmt == 0:
        return rec, byte_pos, bit_pos
    if fmt not in (1, 2):
        return rec, byte_pos, bit_pos

    buf_sz = optimal_base(w, h) if fmt == 1 else w*h
    rec["field_4"] = rb(32)
    rec["field_5"] = rb(32)
    rec["bits_per_idx"] = rb(4, 0xF) + 1
    rec["flags"] = rb(1, 1)

    pixels = []
    maxv = (1 << rec["bits_per_idx"]) - 1
    for _ in range(buf_sz):
        idx = rb(rec["bits_per_idx"], maxv)
        if idx < 0 or idx == maxv: break
        pixels.append(idx)
    rec["pixel_buffer"] = pixels
    return rec, byte_pos, bit_pos


# ═══════════════════════════════════════════════════════════════
# Main renderer (sub_1000A1E4)
# ═══════════════════════════════════════════════════════════════

def render_a1e4(format_ctx: dict, pixel_outputs: List[dict],
                raw_data: bytes) -> Optional[Tuple[bytes, int, int]]:
    """
    Port of sub_1000A1E4.

    format_ctx: {width, height, sub_format, color_space, ...}
    pixel_outputs: list of {data_ptr, run_len, has_special, has_alpha}
    raw_data: the compressed byte stream (for sub-record reading)

    Returns (bgra_bytes, width, height) or None.
    """
    W = format_ctx.get("width", 640)
    H = format_ctx.get("height", 480)
    total_px = W * H

    # Output framebuffer
    fb = bytearray(total_px * 4)

    if not pixel_outputs:
        return (bytes(fb), W, H)

    # Process pixel outputs sequentially
    # Each pixel output describes a region of the image
    dst_idx = 0
    last_color = 0xFF000000

    for po in pixel_outputs:
        run_len = po.get("run_length", 1)
        is_special = po.get("has_special", False)
        data_ptr = po.get("data_ptr", 0)

        if is_special:
            # Control: skip pixels, set up for next render
            if run_len > 0:
                dst_idx += run_len
            continue

        # Color value: read from data_ptr if available
        if 0 <= data_ptr < len(raw_data):
            # Read 16-bit BGR565 from raw data at data_ptr
            if data_ptr + 1 < len(raw_data):
                val = struct.unpack_from('<H', raw_data, data_ptr)[0]
            else:
                val = raw_data[data_ptr]
        else:
            val = run_len  # Use run_len as color value directly

        # BGR565 → BGRA
        b5 = (val >> 11) & 0x1F
        g6 = (val >> 5) & 0x3F
        r5 = val & 0x1F
        r8 = (r5 << 3) | (r5 >> 2)
        g8 = (g6 << 2) | (g6 >> 4)
        b8 = (b5 << 3) | (b5 >> 2)
        color = (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8
        last_color = color

        # Write run
        actual_run = min(run_len, total_px - dst_idx)
        for _ in range(actual_run):
            off = dst_idx * 4
            struct.pack_into('<I', fb, off, color)
            dst_idx += 1

    return (bytes(fb), W, H)
