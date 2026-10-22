#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
IGF ↔ PNG converter for Silky engine (Angel_chs.exe / FOB2).

IGF is a "ZEUS"-wrapped bitmap container used by the Silky visual novel engine.
It stores 24-bit BGR pixel data with optional LZSS compression, bottom-up row
order, and DWORD-aligned row stride.  Pure green (R=0,G=255,B=0) is the
transparency colour key.

Requirements (Python):
    pip install Pillow numpy

C accelerator (optional but strongly recommended for speed):
    gcc -shared -O3 -o igf_lzss.dll igf_lzss.c
    (MinGW-w64; see build_win.bat)

Usage:
    # Single file
    python igf_tool.py to-png   cal01.igf
    python igf_tool.py to-igf   cal01.png

    # Batch (multi-process)
    python igf_tool.py to-png   ./ansys_unpacked/ ./png_output/ --workers 8
    python igf_tool.py to-igf   ./png_output/     ./igf_output/  --workers 8

    # Info
    python igf_tool.py info     cal01.igf
"""

from __future__ import annotations

import argparse
import ctypes
import multiprocessing as mp
import os
import struct
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import numpy as np
from PIL import Image

# ── Constants ───────────────────────────────────────────────────────────

MAGIC_ZEUS   = 0x5355455A   # b"ZEUS" little-endian
HEADER_SIZE  = 20
FLAG_COMPRESSED = 0x80000000
FLAG_BPP_MASK   = 0x000000FF
DEFAULT_FLAGS   = 0x80000018   # compressed, 24 bpp
BPP             = 24

# ── C DLL loader ────────────────────────────────────────────────────────

_dll: Optional[ctypes.CDLL] = None
_dll_path: Optional[Path] = None


def _find_dll() -> Optional[Path]:
    """Locate igf_lzss.dll next to this script or in PATH."""
    here = Path(__file__).resolve().parent
    candidates = [
        here / "igf_lzss.dll",
        here / "igf_lzss.so",
        here / "igf_lzss.dylib",
    ]
    for p in candidates:
        if p.is_file():
            return p
    # also check cwd
    for p in candidates:
        p2 = Path.cwd() / p.name
        if p2.is_file():
            return p2
    return None


def _load_dll() -> Optional[ctypes.CDLL]:
    global _dll, _dll_path
    if _dll is not None:
        return _dll
    path = _find_dll()
    if path is None:
        return None
    try:
        lib = ctypes.cdll.LoadLibrary(str(path))
    except OSError:
        return None

    # lzss_decompress
    lib.lzss_decompress.argtypes = [
        ctypes.POINTER(ctypes.c_ubyte),  # src
        ctypes.c_size_t,                 # src_len
        ctypes.c_size_t,                 # expected_size
        ctypes.POINTER(ctypes.POINTER(ctypes.c_ubyte)),  # out
        ctypes.POINTER(ctypes.c_size_t), # out_len
    ]
    lib.lzss_decompress.restype = ctypes.c_int

    # lzss_compress
    lib.lzss_compress.argtypes = [
        ctypes.POINTER(ctypes.c_ubyte),  # src
        ctypes.c_size_t,                 # src_len
        ctypes.c_int,                    # level
        ctypes.POINTER(ctypes.POINTER(ctypes.c_ubyte)),  # out
        ctypes.POINTER(ctypes.c_size_t), # out_len
    ]
    lib.lzss_compress.restype = ctypes.c_int

    # lzss_free
    lib.lzss_free.argtypes = [ctypes.c_void_p]
    lib.lzss_free.restype = None

    # bgr_to_rgb
    lib.bgr_to_rgb.argtypes = [
        ctypes.POINTER(ctypes.c_ubyte),  # src
        ctypes.POINTER(ctypes.c_ubyte),  # dst
        ctypes.c_int,                    # width
        ctypes.c_int,                    # height
        ctypes.c_int,                    # src_stride (0=auto)
    ]
    lib.bgr_to_rgb.restype = None

    # rgb_to_bgr
    lib.rgb_to_bgr.argtypes = [
        ctypes.POINTER(ctypes.c_ubyte),  # src
        ctypes.POINTER(ctypes.c_ubyte),  # dst
        ctypes.c_int,                    # width
        ctypes.c_int,                    # height
        ctypes.c_int,                    # dst_stride (0=auto)
    ]
    lib.rgb_to_bgr.restype = None

    _dll = lib
    _dll_path = path
    return lib


def have_c_accel() -> bool:
    return _load_dll() is not None


# ── Python fallback LZSS ────────────────────────────────────────────────

_FRAME_SIZE   = 0x1000
_FRAME_INIT   = 0xFEE
_FRAME_FILL   = 0x20
_FRAME_MASK   = 0xFFF


def _py_lzss_decompress(src: bytes, expected_size: int = 0) -> bytes:
    """Pure-Python LZSS decompressor (compatible with Silky engine)."""
    frame = bytearray([_FRAME_FILL]) * _FRAME_SIZE
    frame_pos = _FRAME_INIT
    flags = 0
    out = bytearray()
    i = 0
    n = len(src)
    while i < n and len(out) < (expected_size or n * 4):
        flags >>= 1
        if (flags & 0x100) == 0:
            if i >= n:
                break
            flags = src[i] | 0xFF00
            i += 1
        if flags & 1:
            if i >= n:
                break
            b = src[i]
            i += 1
            out.append(b)
            frame[frame_pos] = b
            frame_pos = (frame_pos + 1) & _FRAME_MASK
        else:
            if i + 1 >= n:
                break
            lo = src[i]
            hi = src[i + 1]
            i += 2
            pos = lo | ((hi & 0xF0) << 4)
            length = (hi & 0x0F) + 3
            for k in range(length):
                b = frame[(pos + k) & _FRAME_MASK]
                out.append(b)
                frame[frame_pos] = b
                frame_pos = (frame_pos + 1) & _FRAME_MASK
                if expected_size and len(out) >= expected_size:
                    return bytes(out[:expected_size])
    if expected_size and len(out) != expected_size:
        raise ValueError(f"LZSS decompress size mismatch: got {len(out)}, expected {expected_size}")
    return bytes(out)


def _py_lzss_compress(raw: bytes) -> bytes:
    """Pure-Python LZSS compressor (compatible with Silky engine)."""
    from ifl_tool_engine import lzss_compress
    return lzss_compress(raw, level=1)


# ── C-accelerated LZSS wrappers ─────────────────────────────────────────

def _c_lzss_decompress(src: bytes, expected_size: int) -> bytes:
    lib = _load_dll()
    if lib is None:
        raise RuntimeError("C DLL not available")

    src_arr = (ctypes.c_ubyte * len(src)).from_buffer_copy(src)
    out_ptr = ctypes.POINTER(ctypes.c_ubyte)()
    out_len = ctypes.c_size_t(0)

    ret = lib.lzss_decompress(src_arr, len(src), expected_size,
                              ctypes.byref(out_ptr), ctypes.byref(out_len))
    if ret != 0:
        raise RuntimeError(f"lzss_decompress failed: {ret}")

    result = bytes(out_ptr[:out_len.value])
    lib.lzss_free(out_ptr)
    return result


def _c_lzss_compress(raw: bytes, level: int = 1) -> bytes:
    lib = _load_dll()
    if lib is None:
        raise RuntimeError("C DLL not available")

    src_arr = (ctypes.c_ubyte * len(raw)).from_buffer_copy(raw)
    out_ptr = ctypes.POINTER(ctypes.c_ubyte)()
    out_len = ctypes.c_size_t(0)

    ret = lib.lzss_compress(src_arr, len(raw), level,
                            ctypes.byref(out_ptr), ctypes.byref(out_len))
    if ret != 0:
        raise RuntimeError(f"lzss_compress failed: {ret}")

    result = bytes(out_ptr[:out_len.value])
    lib.lzss_free(out_ptr)
    return result


def _c_bgr_to_rgb(src: bytes, width: int, height: int, src_stride: int = 0) -> bytes:
    lib = _load_dll()
    if lib is None:
        raise RuntimeError("C DLL not available")

    dst_len = width * height * 3
    src_arr = (ctypes.c_ubyte * len(src)).from_buffer_copy(src)
    dst_arr = (ctypes.c_ubyte * dst_len)()

    lib.bgr_to_rgb(src_arr, dst_arr, width, height, src_stride)

    return bytes(dst_arr)


def _c_rgb_to_bgr(src: bytes, width: int, height: int, dst_stride: int = 0) -> bytes:
    lib = _load_dll()
    if lib is None:
        raise RuntimeError("C DLL not available")

    if dst_stride <= 0:
        dst_stride = (width * 3 + 3) & ~3
    dst_len = dst_stride * height
    src_arr = (ctypes.c_ubyte * len(src)).from_buffer_copy(src)
    dst_arr = (ctypes.c_ubyte * dst_len)()

    lib.rgb_to_bgr(src_arr, dst_arr, width, height, dst_stride)

    return bytes(dst_arr)


# ── Pixel conversion (numpy fast path) ──────────────────────────────────

def _np_bgr_to_rgb(pixels: np.ndarray, width: int, height: int, stride: int) -> np.ndarray:
    """Convert BGR bottom-up with stride to RGB top-down tight array."""
    rgb = np.zeros((height, width, 3), dtype=np.uint8)
    for y in range(height):
        row_start = (height - 1 - y) * stride
        row = pixels[row_start:row_start + width * 3].reshape(width, 3)
        rgb[y, :, :] = row[:, ::-1]  # BGR -> RGB
    return rgb


def _np_rgb_to_bgr(rgb: np.ndarray, width: int, height: int) -> bytes:
    """Convert RGB top-down to BGR bottom-up with DWORD-aligned stride."""
    stride = (width * 3 + 3) & ~3
    buf = np.zeros(stride * height, dtype=np.uint8)
    for y in range(height):
        dst_start = (height - 1 - y) * stride
        row_bgr = rgb[y, :, ::-1]  # RGB -> BGR
        buf[dst_start:dst_start + width * 3] = row_bgr.flatten()
    return buf.tobytes()


# ── IGF file handling ────────────────────────────────────────────────────

@dataclass
class IGFFile:
    """Parsed IGF (ZEUS) image."""
    width: int
    height: int
    flags: int
    pixels: bytes       # decompressed BGR pixel data, bottom-up, DWORD-padded rows
    pixel_size: int     # decompressed size from header

    @property
    def is_compressed(self) -> bool:
        return bool(self.flags & FLAG_COMPRESSED)

    @property
    def bpp(self) -> int:
        """Effective bits per pixel, auto-detected from flags or data size."""
        raw = self.flags & FLAG_BPP_MASK
        if raw in (8, 24):
            return raw
        # raw is 0 or other — infer from pixel data size
        ratio = self.pixel_size / (self.width * self.height)
        if ratio >= 3.5:
            return 32
        if ratio >= 2.5:
            return 24
        return 8

    @property
    def stride(self) -> int:
        if self.bpp == 8:
            return self.width
        if self.bpp == 32:
            return self.width * 4
        return (self.width * 3 + 3) & ~3

    @property
    def bpp_flags(self) -> int:
        """flags value for this bpp (without compression bit)."""
        return self.flags & FLAG_BPP_MASK


def read_igf(path: Path) -> IGFFile:
    """Read and parse an IGF file, decompressing if needed."""
    data = path.read_bytes()
    if len(data) < HEADER_SIZE:
        raise ValueError(f"IGF file too small: {path} ({len(data)} bytes)")

    magic, width, height, pixel_size, flags = struct.unpack_from('<IIIII', data, 0)
    if magic != MAGIC_ZEUS:
        raise ValueError(f"Not a ZEUS/IGF file (magic=0x{magic:08X}): {path}")

    body = data[HEADER_SIZE:]

    if flags & FLAG_COMPRESSED:
        decompressed = lzss_decompress(body, pixel_size)
    else:
        decompressed = body

    return IGFFile(
        width=width,
        height=height,
        flags=flags,
        pixels=decompressed,
        pixel_size=pixel_size,
    )


def write_igf(igf: IGFFile, path: Path, compress: bool = True, compress_level: int = 1):
    """Write an IGF file, optionally compressing with LZSS."""
    if compress:
        body = lzss_compress(igf.pixels, level=compress_level)
        flags = igf.flags | FLAG_COMPRESSED
    else:
        body = igf.pixels
        flags = igf.flags & ~FLAG_COMPRESSED

    # Ensure bpp is properly set in flags lower byte
    # (only relevant for 8/24 bpp; 32 bpp uses bpp=0)
    if (flags & FLAG_BPP_MASK) not in (8, 24):
        if igf.bpp == 8:
            flags = (flags & ~FLAG_BPP_MASK) | 8
        elif igf.bpp == 24:
            flags = (flags & ~FLAG_BPP_MASK) | 24
        # 32bpp stays as 0 in lower byte

    pixel_size = len(igf.pixels)
    header = struct.pack('<IIIII', MAGIC_ZEUS, igf.width, igf.height, pixel_size, flags)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(header + body)


def lzss_decompress(src: bytes, expected_size: int) -> bytes:
    """Decompress LZSS using C accelerator (fast)."""
    if have_c_accel():
        return _c_lzss_decompress(src, expected_size)
    return _py_lzss_decompress(src, expected_size)


def lzss_compress(raw: bytes, level: int = 1) -> bytes:
    """Compress with LZSS using C accelerator."""
    if have_c_accel():
        return _c_lzss_compress(raw, level)
    return _py_lzss_compress(raw)


# ── Core conversion functions ────────────────────────────────────────────

def igf_to_image(igf: IGFFile) -> Image.Image:
    """Convert an IGFFile to a PIL Image."""
    pixels = igf.pixels

    if igf.bpp == 8:
        # 8bpp indexed (bottom-up), palette stored externally.
        row_bytes = igf.width
        buf = bytearray()
        for y in range(igf.height):
            src_off = (igf.height - 1 - y) * igf.stride
            buf += pixels[src_off:src_off + row_bytes]
        return Image.frombytes('L', (igf.width, igf.height), bytes(buf))

    if igf.bpp == 32:
        # 32bpp BGRA (bottom-up) → RGBA (top-down)
        buf = bytearray()
        stride = igf.stride
        for y in range(igf.height):
            src_off = (igf.height - 1 - y) * stride
            row = pixels[src_off:src_off + igf.width * 4]
            # BGRA → RGBA: swap R↔B, keep G and A
            row_swap = bytearray(len(row))
            for x in range(igf.width):
                row_swap[x*4+0] = row[x*4+2]  # R←B
                row_swap[x*4+1] = row[x*4+1]  # G←G
                row_swap[x*4+2] = row[x*4+0]  # B←R
                row_swap[x*4+3] = row[x*4+3]  # A←A
            buf += row_swap
        return Image.frombytes('RGBA', (igf.width, igf.height), bytes(buf))

    # 24bpp BGR (bottom-up) → RGB (top-down)
    stride = igf.stride
    if have_c_accel():
        rgb_bytes = _c_bgr_to_rgb(pixels, igf.width, igf.height, stride)
        return Image.frombytes('RGB', (igf.width, igf.height), rgb_bytes)

    rgb = _np_bgr_to_rgb(np.frombuffer(pixels, dtype=np.uint8), igf.width, igf.height, stride)
    return Image.fromarray(rgb, 'RGB')


def image_to_igf(img: Image.Image, compress: bool = True,
                 compress_level: int = 1) -> IGFFile:
    """Convert a PIL Image to an IGFFile.  Detects bpp from image mode."""
    width, height = img.size

    if img.mode == 'L' or img.mode == 'P':
        # 8bpp indexed
        if img.mode == 'P':
            img = img.convert('L')
        raw = np.array(img, dtype=np.uint8).tobytes()
        stride = width
        buf = bytearray(stride * height)
        for y in range(height):
            dst_off = (height - 1 - y) * stride
            buf[dst_off:dst_off + width] = raw[y*width:(y+1)*width]
        return IGFFile(width=width, height=height,
                       flags=FLAG_COMPRESSED | 8,
                       pixels=bytes(buf), pixel_size=len(buf))

    if img.mode == 'RGBA':
        # 32bpp BGRA (bottom-up)
        raw = np.array(img, dtype=np.uint8).tobytes()
        stride = width * 4
        buf = bytearray(stride * height)
        for y in range(height):
            dst_off = (height - 1 - y) * stride
            src_off = y * width * 4
            row = raw[src_off:src_off + width * 4]
            # RGBA → BGRA
            for x in range(width):
                buf[dst_off + x*4 + 0] = row[x*4 + 2]  # B←R
                buf[dst_off + x*4 + 1] = row[x*4 + 1]  # G←G
                buf[dst_off + x*4 + 2] = row[x*4 + 0]  # R←B
                buf[dst_off + x*4 + 3] = row[x*4 + 3]  # A←A
        return IGFFile(width=width, height=height,
                       flags=FLAG_COMPRESSED | 32,
                       pixels=bytes(buf), pixel_size=len(buf))

    # 24bpp RGB → BGR
    if img.mode != 'RGB':
        img = img.convert('RGB')
    rgb = np.array(img, dtype=np.uint8)

    if have_c_accel():
        bgr_pixels = _c_rgb_to_bgr(rgb.tobytes(), width, height, 0)
    else:
        bgr_pixels = _np_rgb_to_bgr(rgb, width, height)

    return IGFFile(width=width, height=height,
                   flags=DEFAULT_FLAGS,
                   pixels=bgr_pixels, pixel_size=len(bgr_pixels))


def igf_to_png(igf_path: Path, png_path: Path, quiet: bool = False) -> bool:
    """Convert a single IGF file to PNG."""
    try:
        igf = read_igf(igf_path)
        img = igf_to_image(igf)
        png_path.parent.mkdir(parents=True, exist_ok=True)
        img.save(png_path, 'PNG')
        if not quiet:
            print(f"  {igf_path.name} → {png_path.name}  ({igf.width}×{igf.height}, {igf.bpp}bpp)")
        return True
    except Exception as e:
        print(f"  ERROR {igf_path.name}: {e}", file=sys.stderr)
        return False


def png_to_igf(png_path: Path, igf_path: Path,
               compress: bool = True, compress_level: int = 1,
               quiet: bool = False) -> bool:
    """Convert a single PNG file to IGF."""
    try:
        img = Image.open(png_path)
        igf = image_to_igf(img, compress=compress, compress_level=compress_level)
        write_igf(igf, igf_path, compress=compress, compress_level=compress_level)
        if not quiet:
            flag = "compressed" if compress else "raw"
            print(f"  {png_path.name} → {igf_path.name}  ({igf.width}×{igf.height}, {igf.bpp}bpp, {flag})")
        return True
    except Exception as e:
        print(f"  ERROR {png_path.name}: {e}", file=sys.stderr)
        return False


# ── Batch processing ─────────────────────────────────────────────────────

def _igf2png_worker(args: tuple) -> bool:
    """Worker for multiprocessing pool."""
    igf_path, out_dir, quiet = args
    png_name = Path(igf_path).stem + '.png'
    png_path = Path(out_dir) / png_name
    return igf_to_png(Path(igf_path), png_path, quiet=quiet)


def _png2igf_worker(args: tuple) -> bool:
    """Worker for multiprocessing pool."""
    png_path, out_dir, compress, compress_level, quiet = args
    igf_name = Path(png_path).stem + '.igf'
    igf_path = Path(out_dir) / igf_name
    return png_to_igf(Path(png_path), igf_path,
                      compress=compress, compress_level=compress_level,
                      quiet=quiet)


def batch_igf2png(input_dir: Path, output_dir: Path,
                  workers: int = 0, quiet: bool = False):
    """Batch convert all IGF files in input_dir to PNG in output_dir."""
    igf_files = sorted(input_dir.glob('*.igf'))
    if not igf_files:
        print(f"No .igf files found in {input_dir}")
        return

    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Converting {len(igf_files)} IGF → PNG  (workers={workers or mp.cpu_count()})")

    tasks = [(str(f), str(output_dir), quiet) for f in igf_files]

    t0 = time.time()
    if workers == 1:
        results = [_igf2png_worker(t) for t in tasks]
    else:
        with mp.Pool(processes=workers if workers > 0 else None) as pool:
            results = pool.map(_igf2png_worker, tasks)

    ok = sum(results)
    elapsed = time.time() - t0
    print(f"Done: {ok}/{len(igf_files)} converted in {elapsed:.1f}s "
          f"({len(igf_files)/elapsed:.1f} files/s)")


def batch_png2igf(input_dir: Path, output_dir: Path,
                  compress: bool = True, compress_level: int = 1,
                  workers: int = 0, quiet: bool = False):
    """Batch convert all PNG files in input_dir to IGF in output_dir."""
    png_files = sorted(input_dir.glob('*.png'))
    if not png_files:
        print(f"No .png files found in {input_dir}")
        return

    output_dir.mkdir(parents=True, exist_ok=True)
    print(f"Converting {len(png_files)} PNG → IGF  (workers={workers or mp.cpu_count()})")

    tasks = [(str(f), str(output_dir), compress, compress_level, quiet) for f in png_files]

    t0 = time.time()
    if workers == 1:
        results = [_png2igf_worker(t) for t in tasks]
    else:
        with mp.Pool(processes=workers if workers > 0 else None) as pool:
            results = pool.map(_png2igf_worker, tasks)

    ok = sum(results)
    elapsed = time.time() - t0
    print(f"Done: {ok}/{len(png_files)} converted in {elapsed:.1f}s "
          f"({len(png_files)/elapsed:.1f} files/s)")


# ── Info command ─────────────────────────────────────────────────────────

def print_info(path: Path):
    """Print IGF file metadata."""
    igf = read_igf(path)
    print(f"File:      {path.name}")
    print(f"Size:      {path.stat().st_size:,} bytes")
    print(f"Width:     {igf.width}")
    print(f"Height:    {igf.height}")
    print(f"BPP:       {igf.bpp}")
    print(f"Flags:     0x{igf.flags:08X}")
    print(f"Compressed: {'yes' if igf.is_compressed else 'no'}")
    print(f"Pixels:    {igf.pixel_size:,} bytes (stride={igf.stride})")
    print(f"Ratio:     {path.stat().st_size / igf.pixel_size:.1%}")


# ── CLI ──────────────────────────────────────────────────────────────────

def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="IGF ↔ PNG converter for Silky engine (FOB2 / Angel_chs.exe)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python igf_tool.py to-png   cal01.igf                    # single file
  python igf_tool.py to-png   ./ansys_unpacked/ ./png/     # batch directory
  python igf_tool.py to-igf   image.png                     # single file
  python igf_tool.py to-igf   ./png/ ./igf_out/             # batch directory
  python igf_tool.py info     cal01.igf
        """,
    )
    sub = ap.add_subparsers(dest='cmd', required=True)

    # info
    p_info = sub.add_parser('info', help='show IGF file metadata')
    p_info.add_argument('file', type=Path, help='IGF file to inspect')

    # to-png (igf2png)
    p_topng = sub.add_parser('to-png', help='convert IGF → PNG')
    p_topng.add_argument('input', type=Path, help='IGF file or directory of .igf files')
    p_topng.add_argument('output', type=Path, nargs='?', help='PNG file or output directory')
    p_topng.add_argument('-w', '--workers', type=int, default=0,
                         help='parallel workers (0=auto, 1=serial)')
    p_topng.add_argument('-q', '--quiet', action='store_true')

    # to-igf (png2igf)
    p_toigf = sub.add_parser('to-igf', help='convert PNG → IGF')
    p_toigf.add_argument('input', type=Path, help='PNG file or directory of .png files')
    p_toigf.add_argument('output', type=Path, nargs='?', help='IGF file or output directory')
    p_toigf.add_argument('-w', '--workers', type=int, default=0,
                         help='parallel workers (0=auto, 1=serial)')
    p_toigf.add_argument('--raw', action='store_true',
                         help='store uncompressed (default: LZSS compressed)')
    p_toigf.add_argument('--compress-level', type=int, choices=(0, 1, 2), default=1,
                         help='LZSS compression level: 0 fast, 1 balanced, 2 best')
    p_toigf.add_argument('-q', '--quiet', action='store_true')

    args = ap.parse_args(argv)

    # ── info ──
    if args.cmd == 'info':
        print_info(args.file)
        return 0

    # ── to-png ──
    if args.cmd == 'to-png':
        inp: Path = args.input
        if inp.is_dir():
            out_dir = args.output or (inp.parent / (inp.name + '_png'))
            batch_igf2png(inp, out_dir, workers=args.workers, quiet=args.quiet)
        else:
            out = args.output or inp.with_suffix('.png')
            igf_to_png(inp, out, quiet=args.quiet)
        return 0

    # ── to-igf ──
    if args.cmd == 'to-igf':
        inp: Path = args.input
        if inp.is_dir():
            out_dir = args.output or (inp.parent / (inp.name + '_igf'))
            batch_png2igf(inp, out_dir,
                          compress=not args.raw,
                          compress_level=args.compress_level,
                          workers=args.workers,
                          quiet=args.quiet)
        else:
            out = args.output or inp.with_suffix('.igf')
            png_to_igf(inp, out, compress=not args.raw,
                       compress_level=args.compress_level, quiet=args.quiet)
        return 0

    return 1


if __name__ == '__main__':
    # Show accelerator status
    if have_c_accel():
        print(f"[C accelerator loaded: {_dll_path.name}]")
    else:
        print("[C accelerator NOT found — using Python fallback (slower)]")

    raise SystemExit(main())
