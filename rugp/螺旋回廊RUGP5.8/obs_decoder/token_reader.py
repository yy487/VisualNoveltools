"""
Token reader — sub_1000127C and sub_10001244 wrapper.

Reads control tokens from the compressed bitstream.
Each token describes parameters for decoding a batch of pixel values.

Reference: riooor.rpo, sub_1000127C (186 lines of x86 asm)
Callers: sub_100022A0 (pipeline), sub_100024F4 (alt pipeline)
"""

from typing import Optional

from .bitstream import BitStream
from .types import TokenData


class TokenReader:
    """
    Reads control tokens from the bitstream.

    Each token provides:
      count   — number of pixel values to decode
      width   — bits per value in the bitstream
      base    — base offset added to each decoded value
      bit_start — starting bit position for the value data
      first_byte_flags — control flags from the first source byte
    """

    _MAX_TOKEN_BYTES = 65536  # safety limit (was too small at 256)

    def __init__(self, bs: BitStream):
        self._bs: BitStream = bs

    def read(self) -> Optional[TokenData]:
        """
        Read one token from the bitstream.

        Returns None if insufficient data remains, or a TokenData object.

        Token format (sub_1000127C):
          SHORT MODE (byte0 & 8):
            byte0: consumed (contains flags)
            count=1, width=0, base=2, bit_start=6

          STANDARD MODE:
            byte0: flags (low 3 bits used for width encoding)
            byte1: width_high(2) | count_low(5)
            byte2: count_high(3) | mode(2) | reserved(3)
            MODE 0: base=0, bit_start=21
            MODE 1: +byte3 → base computation, bit_start=29
            MODE 2: +byte3 → base computation, bit_start=32
            MODE 3: invalid
        """
        bs: BitStream = self._bs

        if bs.remaining < 3:
            return None

        # Save position for rollback on incomplete reads
        saved_byte, saved_bit = bs.tell()

        byte0: int = bs.data[bs.byte_pos]

        if byte0 & 8:
            # ── Short / special mode ──
            bs.byte_pos += 1
            bs.bit_pos = 0
            # Short mode: count=1, width=0, base=2, bit_start=6
            # Data is 1 byte (the byte we just consumed contains everything)
            token = TokenData(
                data_ptr_offset=bs.byte_pos,
                bit_start=6,
                base_value=2,
                count=1,
                first_byte_flags=byte0,
                width=0,
                raw_size=1,
            )
            # Skip past the 1-byte data region
            bs.byte_pos += 1
            bs.bit_pos = 0
            return token

        # ── Standard mode: read 3 header bytes ──
        if bs.remaining < 4:  # need at least 3 header + possibly 1 extra
            return None

        b0: int = bs.data[bs.byte_pos]
        b1: int = bs.data[bs.byte_pos + 1]
        b2: int = bs.data[bs.byte_pos + 2]
        bs.byte_pos += 3
        bs.bit_pos = 0

        width: int = (b1 >> 6) | (4 * (b0 & 3))
        count: int = (b2 >> 5) | (8 * (b1 & 0x1F))
        mode: int = (b2 >> 3) & 3

        if mode == 0:
            base: int = 0
            bit_start: int = 21
        elif mode == 1:
            bs.byte_pos += 1
            b3: int = bs.data[bs.byte_pos - 1]
            base = (b3 >> 3) | (32 * (b2 & 7))
            bit_start = 29
        elif mode == 2:
            bs.byte_pos += 1
            b3 = bs.data[bs.byte_pos - 1]
            base = b3 | ((b2 & 7) << 8)
            bit_start = 32
        else:
            # Mode 3 is invalid
            bs.seek(saved_byte, saved_bit)
            return None

        # Calculate how many bytes the value data occupies
        total_bits: int = count * width + bit_start
        total_bytes: int = (total_bits + 7) // 8

        if total_bytes > self._MAX_TOKEN_BYTES:
            bs.seek(saved_byte, saved_bit)
            return None

        if bs.remaining < total_bytes:
            # Not enough data — rollback
            bs.seek(saved_byte, saved_bit)
            return None

        token = TokenData(
            data_ptr_offset=bs.byte_pos,
            bit_start=bit_start,
            base_value=base,
            count=count,
            first_byte_flags=byte0,
            width=width,
            raw_size=total_bytes,
        )

        # Skip past the token's data region so the next read()
        # starts at the next token. The pixel decoder will seek
        # back to data_ptr_offset to read the actual pixel data.
        bs.byte_pos += total_bytes
        bs.bit_pos = 0

        return token
