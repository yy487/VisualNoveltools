"""
Format handler — sub_10009F9C.

Routes decoded pixel data to the appropriate color-space renderer.
The first byte of pixel data encodes:
  bits 7:6 = color space (0=BGRA32, 1=RGB24, 2=indexed, 3=special)
  bits 5:0 = sub-format (1-5 valid)

Reference: riooor.rpo, sub_10009F9C (43 lines of x86 asm)
Caller: sub_100022A0 (pipeline)
"""

from typing import Optional

from .types import PixelRun, FormatState
from .constants import (
    FORMAT_COLORSPACE_MASK,
    FORMAT_SUBFORMAT_MASK,
    FORMAT_TABLE,
)


class FormatHandler:
    """
    Handles format detection and color-space routing.

    State machine:
      First call:  read format byte, store color_space + sub_format
      Later calls: route pixel run to renderer
    """

    def __init__(self):
        self._state: FormatState = FormatState()
        self._first_call: bool = True

    @property
    def color_space(self) -> int:
        return getattr(self._state, 'color_space', 0)

    @property
    def sub_format(self) -> int:
        return getattr(self._state, 'sub_format', 0)

    def process_first_byte(self, pixel_data_ptr: int, raw_data: bytes) -> bool:
        """
        Process the first format byte from pixel data.

        Args:
          pixel_data_ptr: offset into raw_data where format byte lives
          raw_data: the full compressed data buffer

        Returns True if initialization succeeded.
        """
        if not self._first_call:
            return True

        if pixel_data_ptr >= len(raw_data):
            return False

        fmt_byte: int = raw_data[pixel_data_ptr]

        self._state.color_space = (fmt_byte & FORMAT_COLORSPACE_MASK) >> 6
        self._state.sub_format = fmt_byte & FORMAT_SUBFORMAT_MASK
        self._state.initialized = True
        self._first_call = False

        return True

    def get_format_info(self) -> dict:
        """
        Get decoded format information.
        """
        sub_fmt: int = self._state.sub_format
        table_entry = FORMAT_TABLE.get(sub_fmt, {})
        return {
            "color_space": self._state.color_space,
            "sub_format": sub_fmt,
            "color_space_name": ["BGRA32", "RGB24", "INDEXED", "SPECIAL"][
                min(self._state.color_space, 3)
            ],
            "table_param": table_entry.get("param", 0),
        }
