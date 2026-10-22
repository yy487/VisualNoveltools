"""
Constants extracted from riooor.rpo via IDA Pro reverse engineering.

Reference: riooor.rpo .rdata section, sub_10009F9C format tables.
"""

# ═══════════════════════════════════════════════════════════════
# MFC / RIO header constants
# ═══════════════════════════════════════════════════════════════

# MFC CArchive tag values found in COptimizedObs headers
MFC_CSHORTSTRING_TAG = 0x08  # CString with 1-byte length prefix (len=8)

# Format byte masks (sub_10009F9C)
FORMAT_COLORSPACE_MASK = 0xC0   # High 2 bits
FORMAT_SUBFORMAT_MASK = 0x3F    # Low 6 bits

# Color space types
COLORSPACE_BGRA32 = 0
COLORSPACE_RGB24 = 1
COLORSPACE_INDEXED = 2
COLORSPACE_SPECIAL = 3

# ═══════════════════════════════════════════════════════════════
# Format dimension table (dword_100189B0 / dword_100189B4)
# Indexed by (sub_format - 1) * 2, sub_format ∈ {1,2,3,4,5}
# ═══════════════════════════════════════════════════════════════

# Table entries: (param, renderer_func_ptr)
# The function pointers are internal to riooor.rpo and aren't
# exported as named functions. They correspond to addresses:
#   0x10017CE0, 0x10017160, 0x10016360, 0x10015240, 0x10014620
FORMAT_TABLE = {
    1: {"param": 0x00000CD8, "renderer_rva": 0x10017CE0},
    2: {"param": 0x00000B72, "renderer_rva": 0x10017160},
    3: {"param": 0x00000DF7, "renderer_rva": 0x10016360},
    4: {"param": 0x00001025, "renderer_rva": 0x10015240},
    5: {"param": 0x00000C01, "renderer_rva": 0x10014620},
}

# ═══════════════════════════════════════════════════════════════
# Magic constants from sub_100046A8
# ═══════════════════════════════════════════════════════════════

SUBRECORD_MAGIC = 0x564342  # "BCV" in ASCII (bytes: 42 43 56)
SUBRECORD_FLAG_MASK = 0xFFFFFF

# ═══════════════════════════════════════════════════════════════
# Pixel flag bits (sub_10001594, sub_10001888)
# ═══════════════════════════════════════════════════════════════

PIXEL_FLAG_RUNLEN = 0x80000     # Run-length continuation
PIXEL_FLAG_END = 0x40000         # End marker
PIXEL_FLAG_ALPHA = 0x20000       # Alpha present
PIXEL_FLAG_SPECIAL = 0x10000     # Special/auxiliary flag
PIXEL_VALUE_MASK = 0xFFFF        # Low 16 bits = value

# ═══════════════════════════════════════════════════════════════
# Token reader modes (sub_1000127C)
# ═══════════════════════════════════════════════════════════════

TOKEN_MODE_SHORT = -1   # Short mode (byte0 & 8)
TOKEN_MODE_0 = 0        # base=0, bit_start=21
TOKEN_MODE_1 = 1        # base from stream, bit_start=29
TOKEN_MODE_2 = 2        # base from stream, bit_start=32
TOKEN_MODE_3 = 3        # invalid

# ═══════════════════════════════════════════════════════════════
# Buffer sizes (sub_10001994, sub_1000B1EC)
# ═══════════════════════════════════════════════════════════════

RAW_BUFFER_SIZE = 0x4000     # 16384 bytes
PIXEL_BUFFER_INIT = 0x1000   # 4096 bytes (1024 DWORDs)
WORK_BUFFER_SIZE = 0xE78     # 3704 bytes (sub_1000B1EC allocation)

# ═══════════════════════════════════════════════════════════════
# Default image dimensions (from CRip metadata)
# ═══════════════════════════════════════════════════════════════

DEFAULT_IMAGE_WIDTH = 640
DEFAULT_IMAGE_HEIGHT = 480
