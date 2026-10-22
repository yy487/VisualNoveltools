"""
Bit-accurate bitstream reader — matches x86 bit-reading pattern from riooor.rpo.

Reference functions:
  sub_10001228 - Bitstream initialization
  sub_10001594 - Reading routine (used everywhere in the codebase)
  sub_1000127C - Token reader (uses same bit pattern)

The canonical x86 pattern (repeated in sub_10001594, sub_1000A1E4, sub_100046A8):
    n32 = bit_pos + width;
    if (byte_pos + 4 < end_pos || n32 + 8*byte_pos <= 8*end_pos) {
        byte_ptr = src + byte_pos;
        value = (*byte_ptr >> bit_pos);
        if (n32 > 8) {
            value |= byte_ptr[1] << (8 - bit_pos);
            if (n32 > 16) {
                value |= byte_ptr[2] << (16 - bit_pos);
                if (n32 > 24) {
                    value |= byte_ptr[3] << (24 - bit_pos);
                    if (n32 > 32 && bit_pos)
                        value |= byte_ptr[4] << (32 - bit_pos);
                }
            }
        }
        result = value & mask;
    } else {
        result = -1;
    }
    byte_pos += n32 / 8;
    bit_pos = n32 & 7;
"""

from typing import Optional


class BitStream:
    """
    Bit-accurate stream reader implementing the x86 LSB-first bit
    reading pattern used throughout riooor.rpo.

    The bit position tracks bits WITHIN the current byte (0 = LSB).
    Width can be any positive integer; the reader handles cross-byte
    reads automatically.
    """

    def __init__(self, data: bytes, start_offset: int = 0):
        self._data: bytes = data
        self._size: int = len(data)
        self._byte_pos: int = start_offset   # a1[0] equivalent
        self._bit_pos: int = 0               # a1[5] equivalent (0-7)
        self._src_base: int = 0              # offset of data within parent

    # ── properties ──────────────────────────────────────────

    @property
    def byte_pos(self) -> int:
        return self._byte_pos

    @byte_pos.setter
    def byte_pos(self, value: int):
        self._byte_pos = value

    @property
    def bit_pos(self) -> int:
        return self._bit_pos

    @bit_pos.setter
    def bit_pos(self, value: int):
        self._bit_pos = value

    @property
    def remaining(self) -> int:
        """Bytes remaining in the stream."""
        return max(0, self._size - self._byte_pos)

    @property
    def data(self) -> bytes:
        """Direct access to underlying data (for token reader)."""
        return self._data

    # ── save / restore ──────────────────────────────────────

    def tell(self) -> tuple:
        """Return (byte_pos, bit_pos) for later restore."""
        return (self._byte_pos, self._bit_pos)

    def seek(self, byte_pos: int, bit_pos: int = 0):
        """Restore saved position."""
        self._byte_pos = byte_pos
        self._bit_pos = bit_pos

    # ── core bit reader ─────────────────────────────────────

    def read_bits(self, width: int) -> int:
        """
        Read `width` bits, LSB-first, returning the value.
        The bits are shifted and masked correctly.
        Returns -1 if out of bounds.

        This is the exact Python equivalent of the x86 pattern above.
        """
        if width <= 0:
            return 0

        n32: int = self._bit_pos + width

        # Bounds check matching x86 assembly
        if self._byte_pos + 4 >= self._size:
            if n32 + 8 * self._byte_pos > 8 * self._size:
                return -1

        src: bytes = self._data
        bp: int = self._byte_pos

        # Build value from consecutive bytes
        value: int = src[bp] >> self._bit_pos

        if n32 > 8:
            value |= src[bp + 1] << (8 - self._bit_pos)
            if n32 > 16:
                value |= src[bp + 2] << (16 - self._bit_pos)
                if n32 > 24:
                    value |= src[bp + 3] << (24 - self._bit_pos)
                    if n32 > 32 and self._bit_pos:
                        value |= src[bp + 4] << (32 - self._bit_pos)

        # Advance position
        self._byte_pos += n32 // 8
        self._bit_pos = n32 & 7

        return value & 0xFFFFFFFF

    def read_bits_masked(self, width: int, mask: int) -> int:
        """Read bits and apply a mask.

        Matches the x86 pattern: `result = value & mask;`
        """
        value: int = self.read_bits(width)
        if value < 0:
            return -1
        return value & mask

    def read_u8(self) -> int:
        """Read 8 bits (one byte, aligned or not)."""
        return self.read_bits_masked(8, 0xFF)

    def read_u16(self) -> int:
        """Read 16 bits."""
        return self.read_bits_masked(16, 0xFFFF)

    def read_u24(self) -> int:
        """Read 24 bits."""
        return self.read_bits_masked(24, 0xFFFFFF)

    def read_u32(self) -> int:
        """Read 32 bits."""
        return self.read_bits_masked(32, 0xFFFFFFFF)

    def read_flag(self) -> bool:
        """Read 1 bit, return as bool."""
        return self.read_bits_masked(1, 1) != 0

    # ── convenience ─────────────────────────────────────────

    def skip_bits(self, count: int):
        """Skip `count` bits without reading them."""
        total: int = self._bit_pos + count
        self._byte_pos += total // 8
        self._bit_pos = total & 7

    def align_byte(self):
        """Advance to the next byte boundary."""
        if self._bit_pos:
            self._byte_pos += 1
            self._bit_pos = 0

    def __repr__(self) -> str:
        return (f"BitStream(byte={self._byte_pos}, bit={self._bit_pos}, "
                f"remaining={self.remaining})")
