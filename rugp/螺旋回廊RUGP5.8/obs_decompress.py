"""
rUGP COptimizedObs decompressor - pure Python implementation
Based on reverse engineering of riooor.rpo (sub_1000127C, sub_10001594, sub_10001888)

The COptimizedObs payload (after 8-byte MFC header) contains:
  [variable bitstream header] [compressed pixel data]

The decompression produces BGRA32 (4 bytes per pixel) output.
"""

import struct
from typing import Optional


class BitStream:
    """Bit-level reader for the rUGP compressed format."""

    def __init__(self, data: bytes):
        self.data = data
        self.byte_pos = 0
        self.bit_pos = 0  # 0-7, next bit to read from current byte

    def remaining_bytes(self) -> int:
        return len(self.data) - self.byte_pos

    def read_bit(self) -> int:
        if self.byte_pos >= len(self.data):
            return 0
        byte = self.data[self.byte_pos]
        bit = (byte >> (7 - self.bit_pos)) & 1
        self.bit_pos += 1
        if self.bit_pos >= 8:
            self.bit_pos = 0
            self.byte_pos += 1
        return bit

    def read_bits(self, count: int) -> int:
        """Read `count` bits, MSB first."""
        value = 0
        for _ in range(count):
            value = (value << 1) | self.read_bit()
        return value

    def read_bits_le(self, count: int) -> int:
        """Read `count` bits, LSB first (bitstream order)."""
        value = 0
        for i in range(count):
            if self.read_bit():
                value |= (1 << i)
        return value

    def align_byte(self):
        if self.bit_pos > 0:
            self.bit_pos = 0
            self.byte_pos += 1

    def read_byte(self) -> int:
        self.align_byte()
        if self.byte_pos >= len(self.data):
            return 0
        b = self.data[self.byte_pos]
        self.byte_pos += 1
        return b

    def peek_byte(self) -> int:
        if self.byte_pos >= len(self.data):
            return 0
        return self.data[self.byte_pos]

    def tell(self) -> int:
        return self.byte_pos * 8 + self.bit_pos


class ObsDecompressContext:
    """State machine for COptimizedObs decompression (matches sub_10001594 context)."""

    def __init__(self, compressed_data: bytes):
        # Raw data buffer (a1[0], a1[4])
        self.raw_buf = bytearray(4096)
        self.raw_size = 4096
        self.raw_pos = 0  # a1[8] - write position
        self.raw_used = 0  # a1[12] - consumed offset

        # Pixel output buffer (a1[16], a1[20])
        self.pixel_buf = None  # will be allocated as list of ints
        self.pixel_capacity = 0
        self.pixel_count = 0  # a1[24] - pixels written
        self.pixel_consumed = 0  # a1[28] - pixels consumed

        # Source compressed data
        self.src = compressed_data
        self.src_pos = 0  # a1[32]

        # Bitstream state
        self.bitstream = BitStream(compressed_data)

        # Token state (a2 equivalent)
        self.token_count = 0
        self.token_flags = 0
        self.token_value = 0
        self.token_base = 0
        self.token_bit_start = 0
        self.token_bit_end = 0
        self.token_data = b''
        self.token_size = 0

    def ensure_pixel_capacity(self, needed: int):
        if self.pixel_capacity < needed:
            new_cap = max(needed, self.pixel_capacity * 2, 256)
            new_buf = [0] * new_cap
            if self.pixel_buf:
                new_buf[:len(self.pixel_buf)] = self.pixel_buf
            self.pixel_buf = new_buf
            self.pixel_capacity = new_cap

    def ensure_raw_capacity(self, needed: int):
        if self.raw_size < needed:
            new_size = max(needed, self.raw_size * 2)
            new_buf = bytearray(new_size)
            new_buf[:len(self.raw_buf)] = self.raw_buf
            self.raw_buf = new_buf
            self.raw_size = new_size


def read_token(ctx: ObsDecompressContext) -> Optional[dict]:
    """
    Implement sub_1000127C - read a control token from the bitstream.
    Returns None if more data is needed, or a dict with token fields.
    """
    bs = ctx.bitstream

    if bs.remaining_bytes() < 3:
        return None

    # Read first byte
    byte0 = bs.peek_byte()

    if byte0 & 8:
        # Short/special mode
        bs.read_byte()  # consume
        count = 1  # v21
        width = 0  # v22
        base = 2   # n2
        bit_start = 6  # n6
    else:
        bs.read_byte()
        byte1 = bs.read_byte()
        byte2 = bs.read_byte()

        width = (byte1 >> 6) | (4 * (byte0 & 3))
        count = (byte2 >> 5) | (8 * (byte1 & 0x1F))
        mode = (byte2 >> 3) & 3

        if mode == 0:
            base = 0
            bit_start = 21
        elif mode == 1:
            if bs.remaining_bytes() < 1:
                return None
            byte3 = bs.read_byte()
            base = (byte3 >> 3) | (32 * (byte2 & 7))
            bit_start = 29
        elif mode == 2:
            if bs.remaining_bytes() < 1:
                return None
            byte3 = bs.read_byte()
            base = byte3 | ((byte2 & 7) << 8)
            bit_start = 32
        else:
            return None  # error

    # Calculate total bits needed
    total_bits = count * width + bit_start
    total_bytes = (total_bits + 7) // 8

    if bs.remaining_bytes() < total_bytes:
        return None

    # Read the values
    values = []
    current_bit = bit_start
    for _ in range(count):
        if width > 0:
            # Read `width` bits from bitstream at current_bit position
            val = 0
            byte_idx = current_bit // 8
            bit_idx = current_bit % 8

            for w in range(width):
                b = bs.data[bs.byte_pos + byte_idx + (bit_idx + w) // 8]
                shift = 7 - ((bit_idx + w) % 8)
                bit = (b >> shift) & 1
                val |= (bit << (width - 1 - w))

            values.append(base + val)
        else:
            values.append(base)

        current_bit += width

    # Advance bitstream past the read data
    bs.byte_pos += total_bytes
    bs.bit_pos = 0

    return {
        'count': count,
        'width': width,
        'base': base,
        'bit_start': bit_start,
        'values': values,
    }


def decompress_obs(compressed_data: bytes) -> Optional[bytes]:
    """
    Decompress COptimizedObs pixel data to BGRA32.

    Args:
        compressed_data: Raw bytes after the 8-byte MFC header

    Returns:
        BGRA32 pixel data as bytes, or None if decompression fails
    """
    ctx = ObsDecompressContext(compressed_data)

    # Initialize
    ctx.ensure_pixel_capacity(640 * 480)

    pixel_idx = 0
    MAX_PIXELS = 640 * 480 * 2  # safety limit

    # Process tokens until we have enough pixels or run out of data
    while ctx.bitstream.remaining_bytes() >= 3 and pixel_idx < MAX_PIXELS:
        token = read_token(ctx)
        if token is None:
            break

        for value in token['values']:
            if pixel_idx >= MAX_PIXELS:
                break

            # Convert value to BGRA32 pixel
            # Based on sub_10001594 and sub_10001888:
            # The decoded value is a packed BGRA pixel
            # Low 16 bits: BG or GB
            # High bits: RA or AR

            ctx.ensure_pixel_capacity(pixel_idx + 1)
            ctx.pixel_buf[pixel_idx] = value
            pixel_idx += 1

    if pixel_idx == 0:
        return None

    # Pack 32-bit pixels into BGRA bytes
    result = bytearray()
    for i in range(pixel_idx):
        px = ctx.pixel_buf[i] & 0xFFFFFFFF
        result.extend(struct.pack('<I', px))

    return bytes(result)
