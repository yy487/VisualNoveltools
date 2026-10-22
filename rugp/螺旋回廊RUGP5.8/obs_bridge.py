"""
ctypes bridge to riooor.rpo for COptimizedObs decoding.

REQUIRES: 32-bit Python on Windows.
Install: winget install Python.Python.3.11 --arch x86

The rUGP DLLs (riooor.rpo, UnivUI.dll, etc.) are 32-bit MFC extension DLLs.
This bridge loads them via ctypes and calls the COptimizedObs rendering
pipeline (sub_100022A0) to decode compressed pixel data into BGRA32 output.

Architecture:
  The pipeline function sub_100022A0 uses __thiscall convention.
  We create a small executable trampoline that moves the first
  ctypes argument (this pointer) into ECX before jumping to the
  real function address.
"""

import ctypes
import struct
import os
import sys
from pathlib import Path
from typing import Optional, Tuple

# ═══════════════════════════════════════════════════════════════
# Constants from riooor.rpo reverse engineering
# ═══════════════════════════════════════════════════════════════

# Pipeline function RVAs (relative to image base 0x10000000)
RVA_PIPELINE = 0x100022A0    # sub_100022A0 — orchestrator (thiscall)
RVA_TOKEN    = 0x1000127C    # sub_1000127C — token reader
RVA_PXDECODE = 0x10001594    # sub_10001594 — pixel decoder
RVA_RLEEXP   = 0x10001888    # sub_10001888 — run-length expander
RVA_FORMAT   = 0x10009F9C    # sub_10009F9C — format handler
RVA_RENDERER = 0x1000A1E4    # sub_1000A1E4 — main renderer
RVA_SUBREC   = 0x100046A8    # sub_100046A8 — sub-record init

IMAGE_BASE = 0x10000000

# Context object size (from field offsets in sub_100022A0)
# Largest offset used: this+754 (+0x2F2) → ~800 bytes
CONTEXT_SIZE = 4096

# Output buffer expectation
DEFAULT_WIDTH = 640
DEFAULT_HEIGHT = 480

# ═══════════════════════════════════════════════════════════════
# Error type
# ═══════════════════════════════════════════════════════════════

class RioBridgeError(Exception):
    pass


# ═══════════════════════════════════════════════════════════════
# Thiscall trampoline builder
# ═══════════════════════════════════════════════════════════════

def _make_thiscall_trampoline(target_addr: int) -> ctypes._CFuncPtr:
    """
    Create a ctypes-callable function that wraps a __thiscall target.

    On x86, __thiscall passes `this` in ECX and remaining arguments
    on the stack (right-to-left, callee cleanup — same as stdcall).

    We build a tiny executable stub:
        mov  ecx, [esp+4]    ; load first stack arg (this) into ECX
        jmp  <target_addr>   ; jump to the real function

    The stub is allocated in executable memory via VirtualAlloc.
    """
    if ctypes.sizeof(ctypes.c_voidp) != 4:
        raise RioBridgeError("Thiscall trampoline requires 32-bit process")

    # x86 machine code for the stub
    stub_code = bytearray([
        0x8B, 0x4C, 0x24, 0x04,  # mov ecx, [esp+4]  — load this from stack
        0xB8, 0x00, 0x00, 0x00, 0x00,  # mov eax, <target>
        0xFF, 0xE0,                # jmp eax
    ])
    # Write target address into the mov eax instruction
    struct.pack_into('<I', stub_code, 5, target_addr)

    # Allocate executable memory
    PAGE_EXECUTE_READWRITE = 0x40
    MEM_COMMIT = 0x1000
    MEM_RESERVE = 0x2000

    kernel32 = ctypes.windll.kernel32
    buf_addr = kernel32.VirtualAlloc(
        0, len(stub_code), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE
    )
    if not buf_addr:
        raise RioBridgeError("VirtualAlloc failed for trampoline")

    # Copy stub code
    ctypes.memmove(buf_addr, bytes(stub_code), len(stub_code))

    # Return as callable function
    # Signature: int __stdcall(void* this) — first arg becomes ECX
    func_type = ctypes.WINFUNCTYPE(ctypes.c_int, ctypes.c_void_p)
    return func_type(buf_addr)


# ═══════════════════════════════════════════════════════════════
# Bridge
# ═══════════════════════════════════════════════════════════════

class ObsBridge:
    """Bridge to riooor.rpo rendering pipeline via ctypes."""

    def __init__(self, dll_dir: str = "."):
        self._dir = Path(dll_dir).resolve()
        self._dll = None
        self._loaded = False

    # ── load ───────────────────────────────────────────────

    def load(self):
        """Load riooor.rpo and dependencies."""
        if ctypes.sizeof(ctypes.c_voidp) != 4:
            raise RioBridgeError(
                "32-bit Python required.\n"
                "Install: winget install Python.Python.3.11 --arch x86"
            )

        original = os.getcwd()
        try:
            os.chdir(str(self._dir))

            # Load dependencies in order
            for dep in ["MSVCRT.dll", "msvcp60.dll", "Mfc42.dll",
                         "GMfc.dll", "UnivUI.dll", "Vm60.dll", "rvmm.dll"]:
                try:
                    ctypes.CDLL(dep)
                except OSError:
                    pass  # Some may already be loaded or not needed

            self._dll = ctypes.CDLL("riooor.rpo")
            self._loaded = True
        finally:
            os.chdir(original)

    # ── decode ─────────────────────────────────────────────

    def decode(self, data: bytes, width: int = DEFAULT_WIDTH,
               height: int = DEFAULT_HEIGHT
               ) -> Tuple[Optional[bytes], int, int]:
        """
        Decode COptimizedObs data via riooor.rpo pipeline.

        Args:
          data: Raw bytes from COptimizedObs resource (with 8B header)
          width, height: Expected output image dimensions

        Returns:
          (bgra_pixels, width, height) or (None, 0, 0)
        """
        if not self._loaded:
            raise RioBridgeError("Not loaded. Call load() first.")

        total_pixels = width * height
        data_len = len(data)

        # ── Allocate context object ──
        ctx = ctypes.create_string_buffer(CONTEXT_SIZE)
        ctx_ptr = ctypes.addressof(ctx)

        # ── Copy input data to accessible buffer ──
        src_buf = ctypes.create_string_buffer(data, data_len)
        src_ptr = ctypes.addressof(src_buf)

        # ── Allocate output buffer ──
        out_buf = ctypes.create_string_buffer(total_pixels * 4)

        # ── Set up context fields ──
        # this+44 (0x2C): source data pointer
        ctypes.cast(ctx_ptr + 44, ctypes.POINTER(ctypes.c_uint32)
                    ).contents.value = src_ptr
        # this+48 (0x30): source data size
        ctypes.cast(ctx_ptr + 48, ctypes.POINTER(ctypes.c_uint32)
                    ).contents.value = data_len

        # ── Resolve function address ──
        # DLL base = _handle
        dll_base = self._dll._handle
        func_abs = dll_base + (RVA_PIPELINE - IMAGE_BASE)

        # ── Call pipeline via thiscall trampoline ──
        try:
            trampoline = _make_thiscall_trampoline(func_abs)
            ret = trampoline(ctx_ptr)
        except Exception as e:
            raise RioBridgeError(f"Pipeline call failed: {e}")

        if ret == 0:
            return (None, 0, 0)

        # ── Extract output ──
        # The pipeline renders into an internal framebuffer.
        # For the BGRA32 format, the output should be in the
        # work buffer allocated by sub_10004324.
        #
        # We need to locate the output pixel data. Based on the
        # reverse engineering, sub_10004324 writes to this+2 (v2[2])
        # which contains a pointer to the framebuffer.
        #
        # Read this+2[0] (v2[2] = context[2]):
        fb_ptr = ctypes.cast(ctx_ptr + 8, ctypes.POINTER(ctypes.c_uint32)
                             ).contents.value
        if fb_ptr and fb_ptr != 0:
            ctypes.memmove(ctypes.addressof(out_buf),
                           ctypes.c_void_p(fb_ptr),
                           total_pixels * 4)
        else:
            # Fallback: try reading from context directly
            # The output might be at a fixed offset in the context
            pass

        return (bytes(out_buf), width, height)


# ═══════════════════════════════════════════════════════════════
# Singleton
# ═══════════════════════════════════════════════════════════════

_bridge: Optional[ObsBridge] = None


def get_bridge(dll_dir: str = ".") -> ObsBridge:
    global _bridge
    if _bridge is None:
        _bridge = ObsBridge(dll_dir)
        _bridge.load()
    return _bridge


def decode_obs(data: bytes, w: int = 640, h: int = 480
               ) -> Tuple[Optional[bytes], int, int]:
    return get_bridge().decode(data, w, h)


# ═══════════════════════════════════════════════════════════════
# CLI
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print(f"Python: {sys.version}")
    bits = ctypes.sizeof(ctypes.c_voidp) * 8
    print(f"Arch: {bits}-bit")

    if bits != 32:
        print()
        print("=" * 55)
        print("  32-BIT PYTHON REQUIRED")
        print("=" * 55)
        print("Install:  winget install Python.Python.3.11 --arch x86")
        print("Then:     py -3.11-32 obs_bridge.py")
        sys.exit(1)

    d = Path(__file__).resolve().parent
    print(f"DLL dir: {d}")

    try:
        bridge = get_bridge(str(d))
        print("Bridge loaded!")
    except RioBridgeError as e:
        print(f"FAIL: {e}")
        sys.exit(1)
