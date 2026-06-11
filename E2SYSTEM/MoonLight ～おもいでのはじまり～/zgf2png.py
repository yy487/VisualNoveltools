# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import binascii
import ctypes
import os
import platform
import shutil
import struct
import subprocess
import sys
import zlib
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

MAGIC = b"ZGF\x1A"
HEADER_SIZE = 14
RGB_XOR_SEED = 0x007F7F7F


@dataclass(frozen=True)
class ZgfHeader:
    max_block_size: int
    flags: int
    bpp: int
    width: int
    height: int

    @property
    def pixel_count(self) -> int:
        return self.width * self.height

    @property
    def plane_count(self) -> int:
        if self.bpp == 24:
            return 3
        if self.bpp == 32:
            return 4
        raise ValueError(f"unsupported ZGF bpp={self.bpp}, expected 24 or 32")


def read_u32le(data: bytes, off: int) -> int:
    if off + 4 > len(data):
        raise ValueError(f"truncated u32 at 0x{off:X}")
    return struct.unpack_from("<I", data, off)[0]


def parse_header(data: bytes) -> ZgfHeader:
    if len(data) < HEADER_SIZE:
        raise ValueError("file too small for ZGF header")
    if data[:4] != MAGIC:
        raise ValueError(f"bad magic: {data[:4]!r}, expected {MAGIC!r}")
    max_block_size = read_u32le(data, 4)
    flags = data[8]
    bpp = data[9]
    width, height = struct.unpack_from("<HH", data, 10)
    if width <= 0 or height <= 0:
        raise ValueError(f"bad image size {width}x{height}")
    return ZgfHeader(max_block_size=max_block_size, flags=flags, bpp=bpp, width=width, height=height)


def parse_blocks(data: bytes, header: ZgfHeader) -> list[bytes]:
    pos = HEADER_SIZE
    planes: list[bytes] = []
    expected = header.pixel_count
    max_seen = 0

    for i in range(header.plane_count):
        if pos + 4 > len(data):
            raise ValueError(f"missing compressed block #{i} length at 0x{pos:X}")
        block_size = read_u32le(data, pos)
        pos += 4
        max_seen = max(max_seen, block_size)
        if block_size < 5:
            raise ValueError(f"invalid compressed block #{i} size={block_size}")
        if pos + block_size > len(data):
            raise ValueError(
                f"truncated compressed block #{i}: pos=0x{pos:X} size={block_size} file_size={len(data)}"
            )
        block = data[pos:pos + block_size]
        pos += block_size

        # First 4 bytes are the engine's stored checksum. sub_405E4E inflates from block+4.
        try:
            plane = zlib.decompress(block[4:])
        except zlib.error as e:
            raise ValueError(f"zlib inflate failed in block #{i}: {e}") from e
        if len(plane) != expected:
            raise ValueError(f"block #{i} inflated to {len(plane)} bytes, expected {expected}")
        planes.append(plane)

    if pos != len(data):
        # Most files should end exactly here. Treat trailing bytes as an error, because otherwise
        # a damaged or mis-parsed ZGF can silently produce a wrong image.
        raise ValueError(f"trailing data after ZGF blocks: 0x{pos:X}..0x{len(data):X}")
    if header.max_block_size and max_seen > header.max_block_size:
        raise ValueError(f"header max_block_size={header.max_block_size} smaller than actual block={max_seen}")
    return planes


def _platform_lib_name() -> str:
    if os.name == "nt":
        return "zgf_fast.dll"
    if sys.platform == "darwin":
        return "libzgf_fast.dylib"
    return "libzgf_fast.so"


def _compile_c_accel(src: Path, lib: Path) -> bool:
    if not src.exists():
        return False
    commands: list[list[str]] = []
    if os.name == "nt":
        # Prefer MinGW/MSYS2 gcc because the user environment already uses it frequently.
        commands.append(["gcc", "-O3", "-shared", "-o", str(lib), str(src)])
        commands.append(["cc", "-O3", "-shared", "-o", str(lib), str(src)])
        # cl.exe fallback. /LD builds a DLL.
        commands.append(["cl", "/O2", "/LD", str(src), f"/Fe:{lib}"])
    else:
        if sys.platform == "darwin":
            commands.append(["cc", "-O3", "-fPIC", "-dynamiclib", "-o", str(lib), str(src)])
            commands.append(["gcc", "-O3", "-fPIC", "-dynamiclib", "-o", str(lib), str(src)])
        else:
            commands.append(["cc", "-O3", "-fPIC", "-shared", "-o", str(lib), str(src)])
            commands.append(["gcc", "-O3", "-fPIC", "-shared", "-o", str(lib), str(src)])

    for cmd in commands:
        if shutil.which(cmd[0]) is None:
            continue
        try:
            r = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        except OSError:
            continue
        if r.returncode == 0 and lib.exists():
            return True
    return False


class CAccel:
    def __init__(self, root: Path, auto_build: bool = True):
        self.root = root
        self.lib_path = root / _platform_lib_name()
        self.enabled = False
        self.lib = None
        if not self.lib_path.exists() and auto_build:
            _compile_c_accel(root / "zgf_fast.c", self.lib_path)
        if self.lib_path.exists():
            try:
                self.lib = ctypes.CDLL(str(self.lib_path))
                fn = self.lib.zgf_planes_to_pngbuf
                fn.argtypes = [
                    ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int,
                ]
                fn.restype = ctypes.c_int
                self.enabled = True
            except Exception:
                self.enabled = False
                self.lib = None

    def combine(self, planes: list[bytes], pixel_count: int, channels: int) -> bytes:
        out = bytearray(pixel_count * channels)
        if not self.enabled or self.lib is None:
            return combine_planes_python(planes, pixel_count, channels)

        plane_a = planes[3] if len(planes) >= 4 else None
        ret = self.lib.zgf_planes_to_pngbuf(
            ctypes.c_char_p(planes[0]),
            ctypes.c_char_p(planes[1]),
            ctypes.c_char_p(planes[2]),
            ctypes.c_char_p(plane_a) if plane_a is not None else None,
            (ctypes.c_char * len(out)).from_buffer(out),
            ctypes.c_size_t(pixel_count),
            ctypes.c_int(channels),
        )
        if ret != 0:
            raise RuntimeError(f"C accelerator returned {ret}")
        return bytes(out)


def combine_planes_python(planes: list[bytes], pixel_count: int, channels: int) -> bytes:
    if channels not in (3, 4):
        raise ValueError("channels must be 3 or 4")
    b, g, r = planes[0], planes[1], planes[2]
    a = planes[3] if len(planes) >= 4 else None
    out = bytearray(pixel_count * channels)
    prev = RGB_XOR_SEED
    if channels == 3:
        j = 0
        for i in range(pixel_count):
            v = (b[i] | (g[i] << 8) | (r[i] << 16)) ^ prev
            prev = v & 0xFFFFFFFF
            out[j + 0] = (v >> 16) & 0xFF
            out[j + 1] = (v >> 8) & 0xFF
            out[j + 2] = v & 0xFF
            j += 3
    else:
        j = 0
        for i in range(pixel_count):
            v = (b[i] | (g[i] << 8) | (r[i] << 16)) ^ prev
            prev = v & 0xFFFFFFFF
            out[j + 0] = (v >> 16) & 0xFF
            out[j + 1] = (v >> 8) & 0xFF
            out[j + 2] = v & 0xFF
            out[j + 3] = a[i] if a is not None else 255
            j += 4
    return bytes(out)


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", binascii.crc32(kind + payload) & 0xFFFFFFFF)
    )


def write_png(path: Path, width: int, height: int, pixels: bytes, channels: int, compress_level: int = 6) -> None:
    if channels == 3:
        color_type = 2
    elif channels == 4:
        color_type = 6
    else:
        raise ValueError("PNG writer supports RGB/RGBA only")
    row_bytes = width * channels
    if len(pixels) != row_bytes * height:
        raise ValueError(f"bad pixel buffer length={len(pixels)}, expected={row_bytes * height}")

    # PNG scanlines: one filter byte per row. Filter 0 is fastest and lossless.
    raw = bytearray((row_bytes + 1) * height)
    src = memoryview(pixels)
    dst_pos = 0
    src_pos = 0
    for _ in range(height):
        raw[dst_pos] = 0
        raw[dst_pos + 1:dst_pos + 1 + row_bytes] = src[src_pos:src_pos + row_bytes]
        dst_pos += row_bytes + 1
        src_pos += row_bytes

    ihdr = struct.pack(">IIBBBBB", width, height, 8, color_type, 0, 0, 0)
    compressed = zlib.compress(bytes(raw), compress_level)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", compressed)
        + png_chunk(b"IEND", b"")
    )


def decode_zgf_to_png(src: Path, dst: Path, accel: CAccel, force_rgba: bool = False, compress_level: int = 6) -> tuple[int, int, int]:
    data = src.read_bytes()
    header = parse_header(data)
    planes = parse_blocks(data, header)
    channels = 4 if force_rgba or header.bpp == 32 else 3
    pixels = accel.combine(planes, header.pixel_count, channels)
    write_png(dst, header.width, header.height, pixels, channels, compress_level=compress_level)
    return header.width, header.height, header.bpp


def iter_zgf_files(path: Path, recursive: bool = True) -> Iterable[Path]:
    if path.is_file():
        if path.suffix.lower() == ".zgf":
            yield path
        else:
            raise ValueError(f"input file is not .zgf: {path}")
    elif path.is_dir():
        pat = "**/*.zgf" if recursive else "*.zgf"
        yield from sorted(p for p in path.glob(pat) if p.is_file())
    else:
        raise FileNotFoundError(path)


def output_path_for(src: Path, input_root: Path, output_root: Path, keep_dirs: bool) -> Path:
    if input_root.is_file():
        if output_root.suffix.lower() == ".png":
            return output_root
        return output_root / (src.stem + ".png")
    if keep_dirs:
        rel = src.relative_to(input_root)
        return output_root / rel.with_suffix(".png")
    return output_root / (src.stem + ".png")


def main() -> None:
    ap = argparse.ArgumentParser(description="Batch convert MoonLight Renewal .zgf images to .png with a C accelerator.")
    ap.add_argument("input", help="input .zgf file or directory")
    ap.add_argument("output", help="output .png file or directory")
    ap.add_argument("--no-recursive", action="store_true", help="do not scan subdirectories when input is a directory")
    ap.add_argument("--flat", action="store_true", help="write all png files directly under output directory")
    ap.add_argument("--jobs", "-j", type=int, default=max(1, (os.cpu_count() or 4) // 2), help="parallel worker count")
    ap.add_argument("--force", "-f", action="store_true", help="overwrite existing .png files")
    ap.add_argument("--rgba", action="store_true", help="force RGBA PNG even for 24bpp ZGF")
    ap.add_argument("--png-level", type=int, default=6, choices=range(0, 10), metavar="0..9", help="zlib compression level for PNG")
    ap.add_argument("--no-c-build", action="store_true", help="do not auto-build the C accelerator if the library is missing")
    args = ap.parse_args()

    input_root = Path(args.input)
    output_root = Path(args.output)
    root = Path(__file__).resolve().parent
    accel = CAccel(root, auto_build=not args.no_c_build)

    files = list(iter_zgf_files(input_root, recursive=not args.no_recursive))
    if not files:
        print("[zgf2png] no .zgf files found")
        return

    keep_dirs = not args.flat
    tasks: list[tuple[Path, Path]] = []
    for src in files:
        dst = output_path_for(src, input_root, output_root, keep_dirs=keep_dirs)
        if dst.exists() and not args.force:
            print(f"[skip] exists: {dst}")
            continue
        tasks.append((src, dst))

    print(f"[zgf2png] files={len(files)} pending={len(tasks)} jobs={max(1,args.jobs)} c_accel={'yes' if accel.enabled else 'no'}")
    if not tasks:
        return

    ok = 0
    fail = 0

    def worker(pair: tuple[Path, Path]) -> tuple[Path, Path, int, int, int, str | None]:
        src, dst = pair
        try:
            w, h, bpp = decode_zgf_to_png(src, dst, accel, force_rgba=args.rgba, compress_level=args.png_level)
            return src, dst, w, h, bpp, None
        except Exception as e:
            return src, dst, 0, 0, 0, str(e)

    with ThreadPoolExecutor(max_workers=max(1, args.jobs)) as ex:
        futures = [ex.submit(worker, p) for p in tasks]
        for fut in as_completed(futures):
            src, dst, w, h, bpp, err = fut.result()
            if err:
                fail += 1
                print(f"[fail] {src}: {err}")
            else:
                ok += 1
                print(f"[ok] {src} -> {dst} ({w}x{h}, {bpp}bpp)")

    print(f"[zgf2png] done ok={ok} fail={fail} output={output_root}")


if __name__ == "__main__":
    main()
