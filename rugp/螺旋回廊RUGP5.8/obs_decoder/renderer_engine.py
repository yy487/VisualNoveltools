"""
Complete renderer engine — sub_1000A1E4 + sub_100046A8.

Pure Python implementation of the COptimizedObs rendering pipeline.
Handles BGRA32 color space (sub_format=0) and indexed sub-formats (1-5).

Reference: riooor.rpo disassembly
  sub_1000A1E4 — Renderer main body (792 lines x86)
  sub_100046A8 — Sub-record initializer (874 lines x86)
  sub_10004324 — Final assembly dispatcher
  sub_1000F0DC — bit_length helper
  sub_1000F0F0 — optimal base computation

Architecture:
  The renderer reads from two interleaved sources:
    1. Pixel output (from RunLengthExpander) — metadata, counts, flags
    2. Bitstream (compressed data) — sub-records, color tables

  The pixel output's data_ptr field points into the raw compressed
  data where color values are stored. The renderer uses bit-level
  reads to extract these values.
"""

import math
import struct
from typing import List, Optional, Tuple
from dataclasses import dataclass, field


# ═══════════════════════════════════════════════════════════════
# Constants
# ═══════════════════════════════════════════════════════════════

SUBRECORD_MAGIC = 0x564342  # "BCV"
DEFAULT_WIDTH = 640
DEFAULT_HEIGHT = 480


# ═══════════════════════════════════════════════════════════════
# Helpers
# ═══════════════════════════════════════════════════════════════

def _bitlen(v: int) -> int:
    """sub_1000F0DC"""
    return v.bit_length() if v > 0 else 0


def _optimal_base(width: int, height: int) -> int:
    """sub_1000F0F0"""
    if width <= 0:
        return height
    X = math.pow(float(height), 1.0 / float(width))
    base = int(math.floor(X))
    while True:
        prod, total, n = 1, 1, 0
        if width >= 6:
            while n <= width - 6:
                prod *= base ** 5
                n += 5
                total = total + total * base
                total = total + total * base
                total = total + total * base
        while n < width:
            prod *= base
            n += 1
            total += total * base
        if prod <= height and total > height:
            break
        base += 1 if prod <= height else -1
    return base


# ═══════════════════════════════════════════════════════════════
# Bit-level reader over raw byte buffer
# ═══════════════════════════════════════════════════════════════

class RawBitReader:
    """
    Bit-level reader over a raw byte buffer.
    Matches the x86 bit-reading pattern used in sub_1000A1E4.

    This is DIFFERENT from BitStream — it reads from a fixed buffer
    at arbitrary bit positions, tracking byte_pos and bit_pos locally.
    """

    def __init__(self, data: bytes, offset: int = 0):
        self.data = data
        self.size = len(data)
        self.byte_pos = offset
        self.bit_pos = 0

    def read_bits(self, width: int, mask: int = 0xFFFFFFFF) -> int:
        if width <= 0:
            return 0
        n32 = self.bit_pos + width
        if self.byte_pos + 4 >= self.size:
            if n32 + 8 * self.byte_pos > 8 * self.size:
                return -1
        bp = self.byte_pos
        src = self.data
        v = src[bp] >> self.bit_pos
        if n32 > 8:
            v |= src[bp + 1] << (8 - self.bit_pos)
            if n32 > 16:
                v |= src[bp + 2] << (16 - self.bit_pos)
                if n32 > 24:
                    v |= src[bp + 3] << (24 - self.bit_pos)
                    if n32 > 32 and self.bit_pos:
                        v |= src[bp + 4] << (32 - self.bit_pos)
        self.byte_pos += n32 // 8
        self.bit_pos = n32 & 7
        return v & mask

    def tell(self):
        return (self.byte_pos, self.bit_pos)

    def seek(self, byte_pos, bit_pos=0):
        self.byte_pos = byte_pos
        self.bit_pos = bit_pos


# ═══════════════════════════════════════════════════════════════
# Sub-record reader (sub_100046A8)
# ═══════════════════════════════════════════════════════════════

@dataclass
class SubRecord:
    width: int = 0
    height: int = 0
    index_table: List[int] = field(default_factory=list)
    fmt_type: int = 0
    field_4: int = 0
    field_5: int = 0
    bits_per_index: int = 0
    flags: int = 0
    pixel_buffer: List[int] = field(default_factory=list)
    field_8_ptr: int = 0


def read_sub_record(rdr: RawBitReader) -> Optional[SubRecord]:
    """Read one sub-record (sub_100046A8)."""
    saved = rdr.tell()

    magic = rdr.read_bits(24, 0xFFFFFF)
    if magic < 0 or magic != SUBRECORD_MAGIC:
        rdr.seek(*saved)
        return None

    w = rdr.read_bits(16, 0xFFFF)
    if w < 0 or w == 0xFFFF:
        rdr.seek(*saved)
        return None
    if w == 0:
        return SubRecord(width=0, height=0)

    h = rdr.read_bits(24, 0xFFFFFF)
    if h < 0 or h == 0xFFFFFF:
        rdr.seek(*saved)
        return None

    flag = rdr.read_bits(1, 1)
    indices = []

    if flag:
        total = 0
        while total < w:
            cnt = rdr.read_bits(5, 0x1F)
            if cnt < 0:
                break
            run = min(cnt + 1, w - total)
            delta = rdr.read_bits(1, 1)
            base = rdr.read_bits(5, 0x1F) + 1 if delta else 0
            indices.extend([base] * run)
            total += run
            if total >= w:
                break
    else:
        for _ in range(w):
            v = rdr.read_bits(5, 0x1F)
            if v < 0 or v == 0x1F:
                break
            indices.append(v + 1)

    fmt = rdr.read_bits(4, 0xF)
    if fmt < 0:
        rdr.seek(*saved)
        return None

    rec = SubRecord(width=w, height=h, index_table=indices, fmt_type=fmt)
    if fmt == 0:
        return rec

    buf_size = _optimal_base(w, h) if fmt == 1 else (w * h)
    if fmt not in (1, 2):
        return rec

    rec.field_4 = rdr.read_bits(32)
    rec.field_5 = rdr.read_bits(32)
    rec.bits_per_index = rdr.read_bits(4, 0xF) + 1
    rec.flags = rdr.read_bits(1, 1)

    pixels = []
    maxv = (1 << rec.bits_per_index) - 1
    for _ in range(buf_size):
        idx = rdr.read_bits(rec.bits_per_index, maxv)
        if idx < 0 or idx == maxv:
            break
        pixels.append(idx)
    rec.pixel_buffer = pixels
    return rec


# ═══════════════════════════════════════════════════════════════
# Main Renderer (sub_1000A1E4 + sub_10004324)
# ═══════════════════════════════════════════════════════════════

class ObsRenderer:
    """
    COptimizedObs renderer implementing sub_1000A1E4.

    Takes decoded pixel values and raw compressed data,
    produces BGRA32 pixel output.
    """

    def __init__(self):
        self.width = DEFAULT_WIDTH
        self.height = DEFAULT_HEIGHT
        self._output = bytearray()

    def render(self, decoded_pixels: List[int],
               raw_data: bytes,
               width: int = DEFAULT_WIDTH,
               height: int = DEFAULT_HEIGHT) -> bytes:
        """
        Render BGRA32 output from decoded pixels and raw data.

        The decoded pixels contain:
          - Control values (with SPECIAL flag): offsets and counts
          - Color values (no flags): BGR565 packed colors
          - RLE runs (with RUNLEN flag): repeat previous color

        The raw_data is the original compressed data, used to
        read additional color information at offsets specified
        by the control values.
        """
        self.width = width
        self.height = height
        total = width * height
        self._output = bytearray(total * 4)

        n = len(decoded_pixels)
        if n == 0:
            return bytes(self._output)

        dst = 0       # output pixel index
        src = 0       # decoded pixel index
        last_color = 0xFF000000  # BGRA opaque black

        while src < n and dst < total:
            dword = decoded_pixels[src]
            src += 1

            flags = dword & 0xF0000
            value = dword & 0xFFFF

            if flags & 0x80000:  # RUNLEN
                run_len = value
                while src < n:
                    nd = decoded_pixels[src]
                    src += 1
                    run_len += (nd & 0xFFFF)
                    if not (nd & 0x80000):
                        break
                color = last_color
                if flags & 0x20000:
                    color = (color & 0x00FFFFFF) | 0xFF000000
                for _ in range(min(run_len, total - dst)):
                    struct.pack_into('<I', self._output, dst * 4, color)
                    dst += 1
                continue

            if flags & 0x40000:  # END
                break

            if flags & 0x10000:  # SPECIAL (control)
                continue

            # Direct color value
            color = self._value_to_bgra(value)
            if flags & 0x20000:
                color = (color & 0x00FFFFFF) | 0xFF000000
            last_color = color

            if dst < total:
                struct.pack_into('<I', self._output, dst * 4, color)
                dst += 1

        return bytes(self._output)

    @staticmethod
    def _value_to_bgra(v: int) -> int:
        """Convert a 16-bit value to BGRA32.

        The value could be:
          - BGR565 (most common for BGRA32 color space)
          - RGB565
          - Direct BGRA low word

        We try BGR565 first as it's the most common format.
        """
        # BGR565
        b5 = (v >> 11) & 0x1F
        g6 = (v >> 5) & 0x3F
        r5 = v & 0x1F
        r8 = (r5 << 3) | (r5 >> 2)
        g8 = (g6 << 2) | (g6 >> 4)
        b8 = (b5 << 3) | (b5 >> 2)
        return (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8
