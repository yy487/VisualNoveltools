"""
Top-level decoder API for COptimizedObs resources.

COptimizedObs stores rendering command streams. The data consists of:
  1. 8-byte MFC header (08 15 17 10 40 44 44 20)
  2. Token-based control data (sub_1000127C format)
  3. Encrypted/compressed pixel data (requires riooor.rpo rendering pipeline)

Current status:
  - Token pipeline: Implemented (reads ~12 tokens per file)
  - BGR565 raw interpretation: Produces random noise (data is encrypted)
  - Full rendering (sub_1000A1E4): Requires further reverse engineering
  - Sub-record reading (sub_100046A8): Magic 0x564342 not found in test data

The complete rendering pipeline uses:
  sub_1000127C → Token reader ✓
  sub_10001594 → Pixel decoder ✓
  sub_10001888 → Run-length expander ✓
  sub_10009F9C → Format handler ✓
  sub_1000A1E4 → Renderer setup (792 lines x86 asm)
  sub_100046A8 → Sub-record init (874 lines x86 asm)
  sub_10004324 → Final assembly / dispatch
  5 sub-format renderers (function pointers at 0x10014620-0x10017CE0)

For production use, loading riooor.rpo via ctypes (32-bit Python on Windows)
is recommended over a full Python port.
"""

import struct
import zlib
from pathlib import Path
from typing import Optional, Tuple

from .constants import DEFAULT_IMAGE_WIDTH, DEFAULT_IMAGE_HEIGHT


def decode_obs_bytes(
    data: bytes,
    width: int = DEFAULT_IMAGE_WIDTH,
    height: int = DEFAULT_IMAGE_HEIGHT,
) -> Tuple[Optional[bytes], int, int]:
    """Decode COptimizedObs data using the multi-strategy palette pipeline."""
    from .full_pipeline import decode_optimizedobs
    return decode_optimizedobs(data, width, height)


def _decode_bgr565(
    payload: bytes, width: int, height: int
) -> Tuple[Optional[bytes], int, int]:
    """Decode as raw BGR565 pixels."""
    total_pixels: int = width * height
    needed: int = total_pixels * 2

    if len(payload) < needed:
        payload = payload + b'\x00' * (needed - len(payload))

    output: bytearray = bytearray(total_pixels * 4)
    for i in range(total_pixels):
        bgr565: int = struct.unpack_from('<H', payload, i * 2)[0]
        b5: int = (bgr565 >> 11) & 0x1F
        g6: int = (bgr565 >> 5) & 0x3F
        r5: int = bgr565 & 0x1F
        r8: int = (r5 << 3) | (r5 >> 2)
        g8: int = (g6 << 2) | (g6 >> 4)
        b8: int = (b5 << 3) | (b5 >> 2)
        dst: int = i * 4
        output[dst:dst + 4] = bytes([b8, g8, r8, 0xFF])

    return (bytes(output), width, height)


def _decode_via_tokens(
    payload: bytes, width: int, height: int
) -> Optional[Tuple[bytes, int, int]]:
    """
    Decode via token pipeline + renderer engine.

    Full pipeline:
      1. Read tokens from payload (sub_1000127C)
      2. Decode pixel values (sub_10001594)
      3. Render BGRA32 via ObsRenderer (sub_1000A1E4 path)
    """
    from .bitstream import BitStream
    from .token_reader import TokenReader
    from .pixel_decoder import PixelDecoder
    from .renderer_engine import ObsRenderer

    bs = BitStream(payload, 0)
    tr = TokenReader(bs)
    pd = PixelDecoder()

    # Read all tokens
    for _ in range(50000):
        token = tr.read()
        if token is None:
            break
        pd.decode(token, bs)

    pixels = pd.pixels
    if not pixels:
        return None

    # Render
    renderer = ObsRenderer()
    result = renderer.render(pixels, payload, width, height)

    return (result, width, height)


def decode_obs_file(
    filepath: str,
    width: int = DEFAULT_IMAGE_WIDTH,
    height: int = DEFAULT_IMAGE_HEIGHT,
) -> Tuple[Optional[bytes], int, int]:
    """Decode a COptimizedObs file to BGRA32 pixels."""
    data: bytes = Path(filepath).read_bytes()
    return decode_obs_bytes(data, width, height)


def save_png(filepath: str, pixels: bytes, width: int, height: int):
    """Save BGRA32 pixels as a PNG file."""

    def _chunk(ctype: bytes, data: bytes) -> bytes:
        c = ctype + data
        crc = struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack('>I', len(data)) + c + crc

    # BGRA → RGBA
    rgba = bytearray(len(pixels))
    for i in range(0, len(pixels), 4):
        rgba[i] = pixels[i + 2]
        rgba[i + 1] = pixels[i + 1]
        rgba[i + 2] = pixels[i]
        rgba[i + 3] = pixels[i + 3]

    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    row_size = width * 4
    raw = b''
    for y in range(height):
        raw += b'\x00' + bytes(rgba[y * row_size:(y + 1) * row_size])

    png = b'\x89PNG\r\n\x1a\n'
    png += _chunk(b'IHDR', ihdr)
    png += _chunk(b'IDAT', zlib.compress(raw))
    png += _chunk(b'IEND', b'')
    Path(filepath).write_bytes(png)
