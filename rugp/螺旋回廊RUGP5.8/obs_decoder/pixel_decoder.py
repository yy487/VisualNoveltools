"""
Pixel decoder — sub_10001594.

Decodes DWORD pixel values from token parameters and bitstream data.
Each value = base + read_bits(width) from the bitstream region.

Reference: riooor.rpo, sub_10001594 (143 lines of x86 asm)
Callers: sub_100022A0 (pipeline), sub_100024F4 (alt pipeline)

Pixel flags applied during decode:
  0x10000  — special_flag (from byte0 & 8, applied once to first pixel)
  0x80000  — run_length continuation (from byte0 & 0x10)
  0x20000  — alpha_flag (from byte0 & 4, applied to last pixel)
"""

from typing import List, Optional

from .bitstream import BitStream
from .types import TokenData
from .constants import (
    PIXEL_FLAG_RUNLEN,
    PIXEL_FLAG_ALPHA,
    PIXEL_FLAG_SPECIAL,
)


class PixelDecoder:
    """
    Decodes pixel value DWORDs from a token + bitstream region.

    Internal state (matching a1[0..7] in sub_10001594):
      a1[0] = raw_data_ptr
      a1[4] = pixel_buf (DWORD array)
      a1[6] = pixel_count (write index)
      a1[7] = pixel_limit (read limit for RunLengthExpander)
    """

    def __init__(self):
        self._pixels: List[int] = []     # a1[4] — DWORD output array
        self._count: int = 0              # a1[6] — write cursor
        self._limit: int = 0              # a1[7] — read limit
        self._special_pending: bool = False

    # ── public properties ───────────────────────────────────

    @property
    def pixels(self) -> List[int]:
        return self._pixels

    @property
    def count(self) -> int:
        return self._count

    @property
    def limit(self) -> int:
        return self._limit

    def set_limit(self, value: int):
        self._limit = value

    # ── decode ──────────────────────────────────────────────

    def decode(self, token: TokenData, bs: BitStream) -> int:
        """
        Decode pixel values from token specification.

        Returns 0 on success, -1 on error.
        """
        count: int = token.count
        width: int = token.width
        base: int = token.base_value
        bit_start: int = token.bit_start
        data_ptr: int = token.data_ptr_offset

        # Save and reposition bitstream to token data region
        saved_byte, saved_bit = bs.tell()

        # Position at data_ptr + bit_start/8 bytes, bit_start%8 bits
        bs.byte_pos = data_ptr + bit_start // 8
        bs.bit_pos = bit_start & 7

        flags: int = token.first_byte_flags

        for i in range(count):
            if width > 0:
                value: int = bs.read_bits(width)
                if value < 0:
                    bs.seek(saved_byte, saved_bit)
                    return -1
            else:
                value = 0

            pixel: int = base + value

            # Apply flags (matching sub_10001594 logic)
            if flags & 8:
                pixel |= PIXEL_FLAG_SPECIAL
                flags &= ~8  # v20 consumed after first use

            if (flags & 0x10) and i >= count - 1:
                pixel |= PIXEL_FLAG_RUNLEN

            # Append to output array
            if self._count >= len(self._pixels):
                self._pixels.append(pixel)
            else:
                self._pixels[self._count] = pixel
            self._count += 1

        # Apply alpha flag to last pixel (v19 logic)
        if flags & 4:
            if self._count > 0:
                last_idx = self._count - 1
                self._pixels[last_idx] |= PIXEL_FLAG_ALPHA

        # Restore bitstream position
        bs.seek(saved_byte, saved_bit)
        return 0

    def reset(self):
        """Clear decoder state for a fresh pipeline run."""
        self._pixels.clear()
        self._count = 0
        self._limit = 0
        self._special_pending = False
