"""
Type definitions for COptimizedObs decoder data structures.

Matches the C++ structures used in riooor.rpo.
"""

from dataclasses import dataclass, field
from typing import Optional, List


@dataclass
class TokenData:
    """Output of TokenReader (sub_1000127C) — a2[0..6]."""
    data_ptr_offset: int = 0
    bit_start: int = 0       # a2[3] = n6
    base_value: int = 0      # a2[2] = n2
    count: int = 0           # a2[4] = v21
    first_byte_flags: int = 0  # a2[5] low byte
    width: int = 0           # a2[6] = v22
    raw_size: int = 0        # size of raw data block


@dataclass
class PixelRun:
    """Output of RunLengthExpander (sub_10001888) — a2[0..3]."""
    data_ptr: int = 0        # a2[0] — pointer into source
    run_length: int = 0      # a2[1] — accumulated run length
    has_special: bool = False  # a2[2] — 0x10000 flag
    has_alpha: bool = False    # a2[3] — 0x20000 flag


@dataclass
class FormatState:
    """
    Format handler state (sub_10009F9C) — v6[0..8].
    Called `v6` or format_context.
    """
    field_0: int = 0
    field_4: int = 0
    field_8: int = 0
    field_C: int = 0       # v6[3] = format params
    initialized: bool = False


@dataclass
class SubRecord:
    """
    Sub-record initialized by sub_100046A8.
    Size: 0x34 = 52 bytes (13 DWORDs).
    a2[0..12]
    """
    width: int = 0           # a2[0]
    height: int = 0          # a2[1]
    index_table: Optional[List[int]] = None  # a2[2]
    format_type: int = 0     # a2[3]: 0=none, 1=f0f0-computed, 2=raw
    field_4: int = 0         # a2[4]: 32-bit param
    field_5: int = 0         # a2[5]: 32-bit param
    bits_per_index: int = 0  # a2[6]
    flags: int = 0           # a2[7]
    pixel_buffer: Optional[List[int]] = None  # a2[8]
    field_9: int = 0         # a2[9]
    field_10: int = 0        # a2[10]
    field_11: int = 0        # a2[11]
    field_12: int = 1        # a2[12]


@dataclass
class DecoderContext:
    """
    Pipeline context matching `this` in sub_100022A0.
    Tracks all state across the decoding pipeline.
    """
    # Bitstream state (this+56)
    src_data: Optional[bytes] = None
    src_size: int = 0
    byte_pos: int = 0
    bit_pos: int = 0

    # Pixel buffer (this+88)
    pixel_buf: List[int] = field(default_factory=list)
    pixel_count: int = 0
    pixel_read_pos: int = 0
    pixel_dst_offset: int = 0

    # Token data (this+432)
    current_token: Optional[TokenData] = None

    # Pixel run output (this+460)
    current_run: Optional[PixelRun] = None

    # Working buffers (this+480, this+496)
    work_buf_a: Optional[object] = None
    work_buf_b: Optional[object] = None

    # Format state
    format_state: FormatState = field(default_factory=FormatState)

    # Renderer sub-records
    sub_records: List[SubRecord] = field(default_factory=list)

    # Output
    output_width: int = 640
    output_height: int = 480
    output_pixels: bytearray = field(default_factory=bytearray)
