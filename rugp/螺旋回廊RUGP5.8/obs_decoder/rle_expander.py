"""
Run-length expander — sub_10001888.

Processes decoded DWORD pixel array into (color, length, flags) tuples.
Correctly distinguishes between:
  - Control values (0x10000 flag): data_ptr + count info
  - Direct pixels (no flags): the value IS the color, length=1
  - RLE runs (0x80000 flag): accumulated run length
  - End marker (0x40000 flag): stop processing
  - Alpha flag (0x20000): alpha channel info

Reference: riooor.rpo, sub_10001888 (69 lines of x86 asm)
"""

from typing import Optional, List

from .types import PixelRun
from .constants import (
    PIXEL_FLAG_RUNLEN,
    PIXEL_FLAG_END,
    PIXEL_FLAG_ALPHA,
    PIXEL_FLAG_SPECIAL,
    PIXEL_VALUE_MASK,
)


class RunLengthExpander:
    """Expands the DWORD pixel array into (color, length, flags) tuples."""

    def __init__(self, pixel_list: List[int]):
        self._pixels: List[int] = pixel_list
        self._read_pos: int = 0
        self._dst_offset: int = 0

    @property
    def read_pos(self) -> int:
        return self._read_pos

    @property
    def dst_offset(self) -> int:
        return self._dst_offset

    def reset(self, pixel_list: List[int]):
        self._pixels = pixel_list
        self._read_pos = 0
        self._dst_offset = 0

    def expand(self) -> Optional[PixelRun]:
        """
        Expand one run of pixels.

        Returns PixelRun where:
          data_ptr  — accumulated pixel offset in output
          run_length — how many output pixels this run covers
          has_special — if True, value is a control/pointer, not a color
          has_alpha  — if True, alpha channel data follows
        """
        if self._read_pos >= len(self._pixels):
            return None

        pixel: int = self._pixels[self._read_pos]

        # End marker
        if pixel & PIXEL_FLAG_END:
            self._read_pos += 1
            return None

        has_alpha: bool = bool(pixel & PIXEL_FLAG_ALPHA)
        has_special: bool = bool(pixel & PIXEL_FLAG_SPECIAL)
        value: int = pixel & PIXEL_VALUE_MASK

        if pixel & PIXEL_FLAG_RUNLEN:
            # ── RLE run: accumulate run length ──
            run_len: int = value
            self._read_pos += 1
            while self._read_pos < len(self._pixels):
                next_pixel: int = self._pixels[self._read_pos]
                if next_pixel & PIXEL_FLAG_ALPHA:
                    has_alpha = True
                run_len += (next_pixel & PIXEL_VALUE_MASK)
                self._read_pos += 1
                if not (next_pixel & PIXEL_FLAG_RUNLEN):
                    break

            run = PixelRun(
                data_ptr=self._dst_offset,
                run_length=run_len,
                has_special=has_special,
                has_alpha=has_alpha,
            )
            self._dst_offset += run_len
            return run

        else:
            # ── Single pixel or control ──
            self._read_pos += 1
            run = PixelRun(
                data_ptr=self._dst_offset,
                run_length=1,           # Single pixel
                has_special=has_special,
                has_alpha=has_alpha,
            )
            self._dst_offset += 1
            return run
