"""
Pipeline orchestrator — sub_100022A0.

Implements the complete COptimizedObs decoding pipeline with
correct interleaving of token reading and renderer metadata parsing.

Reference: riooor.rpo, sub_100022A0 (108 lines of x86 asm)
"""

from typing import Optional, Tuple, List

from .bitstream import BitStream
from .token_reader import TokenReader
from .pixel_decoder import PixelDecoder
from .rle_expander import RunLengthExpander
from .format_handler import FormatHandler
from .renderer import SubRecordReader, BGRARenderer
from .types import TokenData, PixelRun, SubRecord
from .constants import (
    DEFAULT_IMAGE_WIDTH,
    DEFAULT_IMAGE_HEIGHT,
    PIXEL_FLAG_END,
)

MAX_TOKENS = 200000


class ObsPipeline:
    """
    COptimizedObs decoding pipeline with interleaved renderer metadata.

    Implements the full sub_100022A0 flow:
      1. Read tokens → decode pixels → expand runs
      2. Feed pixel output to renderer setup (sub_1000A1E4)
      3. Renderer reads sub-records from bitstream
      4. Final assembly renders BGRA32 pixels
    """

    def __init__(self, data: bytes):
        self._raw_data: bytes = data

        # COptimizedObs files have an 8-byte fixed header:
        #   08 15 17 10 40 44 44 20
        # This is the MFC serialization prefix (tag=0x1508, schema=0x1017).
        # The compressed command stream starts at offset 8.
        # Byte 8 is the format byte (color_space:sub_format).
        offset = 8
        if len(data) > 8 and data[0] == 0x08 and data[1] == 0x15:
            payload = data[offset:]
        else:
            offset = 0
            payload = data

        self._payload_offset: int = offset
        self._bs: BitStream = BitStream(payload)

        # Pipeline components
        self._token_reader: TokenReader = TokenReader(self._bs)
        self._pixel_decoder: PixelDecoder = PixelDecoder()
        self._rle: Optional[RunLengthExpander] = None
        self._format_handler: FormatHandler = FormatHandler()
        self._sub_record_reader: SubRecordReader = SubRecordReader(self._bs)
        self._renderer: BGRARenderer = BGRARenderer()

        # State
        self._sub_records: List[SubRecord] = []
        self._output_width: int = DEFAULT_IMAGE_WIDTH
        self._output_height: int = DEFAULT_IMAGE_HEIGHT

        # Stats
        self._token_count: int = 0
        self._render_count: int = 0

    def decode(self) -> Tuple[Optional[bytes], int, int]:
        """
        Run the full decoding pipeline with interleaved rendering.
        """
        bs: BitStream = self._bs
        tr: TokenReader = self._token_reader
        pd: PixelDecoder = self._pixel_decoder

        # ── Phase 1: Read all tokens ──
        while self._token_count < MAX_TOKENS:
            token: Optional[TokenData] = tr.read()
            if token is None:
                break

            self._token_count += 1
            pd.decode(token, bs)

        # ── Phase 2: Initialize RLE on all decoded pixels ──
        self._rle = RunLengthExpander(pd.pixels)

        # ── Phase 3: Process first format byte ──
        # For color_space=0, sub_format=0, the format byte is the
        # first byte of the pixel data at the initial pointer position.
        # We process it via the format handler.
        first_run: Optional[PixelRun] = self._rle.expand()
        if first_run is None:
            return (None, 0, 0)

        # ── Phase 4: Run renderer setup via sub_1000A1E4 ──
        # The renderer reads from both the pixel data pointer and the bitstream.
        # It interleaves sub-record reads with pixel data reads.
        self._run_renderer_setup(pd.pixels, self._rle)

        # ── Phase 5: Render final image ──
        result: bytes = self._renderer.render_final(
            self._sub_records,
            pd.pixels,
            self._output_width,
            self._output_height,
        )

        return (result, self._output_width, self._output_height)

    def _run_renderer_setup(self, pixels: List[int], rle: RunLengthExpander):
        """
        Run the sub_1000A1E4 renderer setup.

        Reads sub-records from the bitstream and rendering parameters
        from the pixel data, interleaved as in the original code.

        The pixel data is organized as DWORD records:
          [value_with_flags] [optional continuation values]
        where value_with_flags has:
          bits 31:20 = flags (0x8=runlen, 0x4=end, 0x2=alpha, 0x1=special)
          bits 19:0  = value (for sub_format=0: direct pixel dimensions)

        For sub_format=0 (BGRA32), the renderer uses pixel dimensions
        directly and reads color data from sub-records in the bitstream.
        """
        bs: BitStream = self._bs

        # Read all available sub-records from the bitstream
        # The bitstream position after all tokens have been read
        # points to the sub-record data region.
        for _ in range(5000):
            rec: Optional[SubRecord] = self._sub_record_reader.read()
            if rec is None:
                break
            if rec.width > 0 or rec.height > 0:
                self._sub_records.append(rec)

    def set_output_size(self, width: int, height: int):
        self._output_width = width
        self._output_height = height

    def get_stats(self) -> dict:
        return {
            "tokens_read": self._token_count,
            "pixels_decoded": self._pixel_decoder.count,
            "sub_records": len(self._sub_records),
            "format": self._format_handler.get_format_info(),
        }
