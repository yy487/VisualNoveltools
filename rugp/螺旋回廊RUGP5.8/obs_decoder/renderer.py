"""
Main renderer — processes decoded pixel values into BGRA32 output.

Reference functions:
  sub_1000A1E4 — Renderer setup (792 lines)
  sub_10004324 — Final assembly / rendering dispatch
  sub_100046A8 — Sub-record initializer (874 lines)
"""

import math
import struct
from typing import List, Optional, Tuple

from .bitstream import BitStream
from .types import SubRecord
from .constants import (
    SUBRECORD_MAGIC,
    SUBRECORD_FLAG_MASK,
    PIXEL_FLAG_RUNLEN,
    PIXEL_FLAG_END,
    PIXEL_FLAG_ALPHA,
    PIXEL_FLAG_SPECIAL,
    PIXEL_VALUE_MASK,
    DEFAULT_IMAGE_WIDTH,
    DEFAULT_IMAGE_HEIGHT,
)


def bit_length(value: int) -> int:
    if value <= 0:
        return 0
    return value.bit_length()


def compute_optimal_base(width: int, height: int) -> int:
    if width <= 0:
        return height
    X: float = math.pow(float(height), 1.0 / float(width))
    base: int = int(math.floor(X))
    while True:
        product: int = 1
        total: int = 1
        n: int = 0
        if width >= 6:
            while n <= width - 6:
                product *= base ** 5
                n += 5
            total = total + total * base
            total = total + total * base
            total = total + total * base
        while n < width:
            product *= base
            n += 1
            total += total * base
        if product <= height and total > height:
            break
        if product <= height:
            base += 1
        else:
            base -= 1
    return base


# ═══════════════════════════════════════════════════════════════
# Color format conversions
# ═══════════════════════════════════════════════════════════════

def value_to_bgra(value: int) -> int:
    """
    Convert a decoded 16-bit pixel value to BGRA32.

    rUGP commonly uses BGR565 format for BGRA32 color space:
      bits 15:11 = Blue  (5 bits)
      bits 10:5  = Green (6 bits)
      bits 4:0   = Red   (5 bits)

    Returns a 32-bit BGRA value (0xAABBGGRR in memory = struct '<I').
    """
    b5: int = (value >> 11) & 0x1F
    g6: int = (value >> 5) & 0x3F
    r5: int = value & 0x1F

    # Expand 5/6-bit to 8-bit
    r8: int = (r5 << 3) | (r5 >> 2)
    g8: int = (g6 << 2) | (g6 >> 4)
    b8: int = (b5 << 3) | (b5 >> 2)

    return (0xFF << 24) | (r8 << 16) | (g8 << 8) | b8


# ═══════════════════════════════════════════════════════════════
# Sub-record reader — sub_100046A8
# ═══════════════════════════════════════════════════════════════

class SubRecordReader:
    """Reads a SubRecord from the bitstream (sub_100046A8)."""

    def __init__(self, bs: BitStream):
        self._bs: BitStream = bs

    def read(self) -> Optional[SubRecord]:
        bs = self._bs
        saved = bs.tell()

        magic: int = bs.read_bits_masked(24, SUBRECORD_FLAG_MASK)
        if magic < 0 or magic != SUBRECORD_MAGIC:
            bs.seek(*saved)
            return None

        width: int = bs.read_bits_masked(16, 0xFFFF)
        if width < 0 or width == 0xFFFF:
            bs.seek(*saved)
            return None
        if width == 0:
            return SubRecord(width=0, height=0)

        height: int = bs.read_bits_masked(24, SUBRECORD_FLAG_MASK)
        if height < 0 or height == 0xFFFFFF:
            bs.seek(*saved)
            return None

        flag: bool = bs.read_flag()

        index_values: List[int] = []
        if flag:
            total: int = 0
            while total < width:
                count: int = bs.read_bits_masked(5, 0x1F)
                if count < 0:
                    break
                run_count: int = min(count + 1, width - total)
                delta_flag: bool = bs.read_flag()
                base_val: int = bs.read_bits_masked(5, 0x1F) + 1 if delta_flag else 0
                index_values.extend([base_val] * run_count)
                total += run_count
        else:
            for _ in range(width):
                val: int = bs.read_bits_masked(5, 0x1F)
                if val < 0 or val == 0x1F:
                    break
                index_values.append(val + 1)

        fmt_type: int = bs.read_bits_masked(4, 0xF)
        if fmt_type < 0:
            bs.seek(*saved)
            return None

        rec = SubRecord(width=width, height=height,
                        index_table=index_values, format_type=fmt_type)
        if fmt_type == 0:
            return rec

        if fmt_type == 1:
            buffer_size = compute_optimal_base(width, height)
        elif fmt_type == 2:
            buffer_size = width * height
        else:
            return rec

        rec.field_4 = bs.read_bits_masked(32, 0xFFFFFFFF)
        rec.field_5 = bs.read_bits_masked(32, 0xFFFFFFFF)
        rec.bits_per_index = bs.read_bits_masked(4, 0xF) + 1
        rec.flags = 1 if bs.read_flag() else 0

        pixel_indices: List[int] = []
        max_val: int = (1 << rec.bits_per_index) - 1
        for _ in range(buffer_size):
            idx: int = bs.read_bits_masked(rec.bits_per_index, max_val)
            if idx < 0 or idx == max_val:
                break
            pixel_indices.append(idx)
        rec.pixel_buffer = pixel_indices
        return rec


# ═══════════════════════════════════════════════════════════════
# Renderer — processes decoded pixels directly
# ═══════════════════════════════════════════════════════════════

class BGRARenderer:
    """
    Renders BGRA32 output from decoded pixel values.

    The decoded pixel DWORD array contains interleaved control and color data:
      - Control: 0x10000 flag → data pointer / count
      - RLE:     0x80000 flag → run-length repeat
      - Alpha:   0x20000 flag → alpha info
      - End:     0x40000 flag → stop
      - No flags: the value is a BGR565 color (write 1 pixel)
    """

    def __init__(self):
        self._output: bytearray = bytearray()
        self._width: int = DEFAULT_IMAGE_WIDTH
        self._height: int = DEFAULT_IMAGE_HEIGHT

    def render_final(
        self,
        sub_records: List[SubRecord],
        decoded_pixels: List[int],
        width: int = DEFAULT_IMAGE_WIDTH,
        height: int = DEFAULT_IMAGE_HEIGHT,
    ) -> bytes:
        self._width = width
        self._height = height
        total_pixels: int = width * height
        self._output = bytearray(total_pixels * 4)

        if not decoded_pixels:
            return bytes(self._output)

        dst_idx: int = 0
        src_idx: int = 0
        last_color: int = 0xFF000000  # opaque black
        n: int = len(decoded_pixels)

        while src_idx < n and dst_idx < total_pixels:
            dword: int = decoded_pixels[src_idx]
            src_idx += 1

            if dword & PIXEL_FLAG_END:
                break

            value: int = dword & PIXEL_VALUE_MASK
            is_special: bool = bool(dword & PIXEL_FLAG_SPECIAL)
            is_alpha: bool = bool(dword & PIXEL_FLAG_ALPHA)
            is_rle: bool = bool(dword & PIXEL_FLAG_RUNLEN)

            if is_special:
                # Control value: offset/count — skip for now
                # The next pixel(s) will have the actual color data
                if is_rle:
                    # Accumulate run length
                    run_accum: int = value
                    while src_idx < n:
                        nd: int = decoded_pixels[src_idx]
                        src_idx += 1
                        run_accum += (nd & PIXEL_VALUE_MASK)
                        if not (nd & PIXEL_FLAG_RUNLEN):
                            break
                continue

            if is_rle:
                # RLE run: repeat last_color
                run_len: int = value
                while src_idx < n:
                    nd: int = decoded_pixels[src_idx]
                    src_idx += 1
                    run_len += (nd & PIXEL_VALUE_MASK)
                    if not (nd & PIXEL_FLAG_RUNLEN):
                        break

                color = last_color
                if is_alpha:
                    color = (color & 0x00FFFFFF) | 0xFF000000

                for _ in range(min(run_len, total_pixels - dst_idx)):
                    off: int = dst_idx * 4
                    struct.pack_into('<I', self._output, off, color)
                    dst_idx += 1
                continue

            # Direct pixel: value is BGR565 color
            color: int = value_to_bgra(value)
            if is_alpha:
                color = (color & 0x00FFFFFF) | 0xFF000000

            last_color = color
            off: int = dst_idx * 4
            struct.pack_into('<I', self._output, off, color)
            dst_idx += 1

        return bytes(self._output)
