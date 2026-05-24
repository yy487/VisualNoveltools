# -*- coding: utf-8 -*-
"""
cbm2png.py - decoder for KEYINSE / 刻音色 .CBM image files.

Format recovered from keyinse.exe.c:
  FUN_00401da0  : image loader, strips .bmp and appends .cbm
  FUN_00413cc0  : reads header: u16 channels, u16 width, u16 height, u32 packed_size
  FUN_00413de0  : dispatches channels==3 / channels==4 decoder
  FUN_00413e90  : 24bpp channel-plane packet decoder
  FUN_00414010  : 32bpp channel-plane packet decoder

This version can use cbm_fast.dll / libcbm_fast.so / libcbm_fast.dylib from the
same directory as a ctypes C accelerator. If the native library is absent, it
falls back to the pure Python decoder.
"""
from __future__ import annotations

import argparse
import ctypes
import json
import os
import struct
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

try:
    from PIL import Image
except Exception as exc:  # pragma: no cover
    raise SystemExit(
        "Pillow is required. Install it with: python -m pip install pillow"
    ) from exc

VERSION = "2026-05-24-cbm2png-fast-v2"


@dataclass
class CbmInfo:
    path: str
    channels: int
    width: int
    height: int
    packed_size: int
    actual_payload_size: int
    row_stride: int
    consumed_payload_size: int = 0
    backend: str = "python"
    warning: str | None = None


class CbmDecodeError(RuntimeError):
    pass


def u16le(data: bytes, off: int) -> int:
    return struct.unpack_from("<H", data, off)[0]


def u32le(data: bytes, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def parse_cbm_header(data: bytes, path: str = "<memory>") -> CbmInfo:
    if len(data) < 10:
        raise CbmDecodeError(f"{path}: file too small for CBM header: {len(data)} bytes")

    channels = u16le(data, 0)
    width = u16le(data, 2)
    height = u16le(data, 4)
    packed_size = u32le(data, 6)

    if channels not in (3, 4):
        raise CbmDecodeError(
            f"{path}: unsupported channels/bytes-per-pixel={channels}; expected 3 or 4"
        )
    if width <= 0 or height <= 0:
        raise CbmDecodeError(f"{path}: invalid image size {width}x{height}")

    actual_payload_size = max(0, len(data) - 10)
    warning = None
    if packed_size > actual_payload_size:
        warning = (
            f"header packed_size={packed_size}, but file only contains "
            f"{actual_payload_size} payload bytes; using available bytes"
        )
        packed_size = actual_payload_size
    elif packed_size < actual_payload_size:
        warning = (
            f"file has {actual_payload_size - packed_size} trailing bytes after packed payload"
        )

    row_stride = ((width * channels + 3) // 4) * 4
    return CbmInfo(
        path=path,
        channels=channels,
        width=width,
        height=height,
        packed_size=packed_size,
        actual_payload_size=actual_payload_size,
        row_stride=row_stride,
        warning=warning,
    )


_FAST_LIB: ctypes.CDLL | None | bool = None
_FAST_LIB_ERROR: str | None = None


def _candidate_fast_libs() -> list[Path]:
    base = Path(__file__).resolve().parent
    names = ["cbm_fast.dll", "libcbm_fast.so", "libcbm_fast.dylib"]
    return [base / name for name in names]


def load_fast_lib() -> ctypes.CDLL | None:
    """Load native accelerator from this script directory, if available."""
    global _FAST_LIB, _FAST_LIB_ERROR
    if isinstance(_FAST_LIB, ctypes.CDLL):
        return _FAST_LIB
    if _FAST_LIB is False:
        return None

    last_error: str | None = None
    for lib_path in _candidate_fast_libs():
        if not lib_path.exists():
            continue
        try:
            if os.name == "nt" and hasattr(os, "add_dll_directory"):
                os.add_dll_directory(str(lib_path.parent))
            lib = ctypes.CDLL(str(lib_path))
            lib.cbm_decode_to_rgba.argtypes = [
                ctypes.c_void_p,            # data
                ctypes.c_size_t,           # data_size
                ctypes.c_int,              # flip_y
                ctypes.POINTER(ctypes.c_void_p),  # out_pixels
                ctypes.POINTER(ctypes.c_int),     # width
                ctypes.POINTER(ctypes.c_int),     # height
                ctypes.POINTER(ctypes.c_int),     # channels
                ctypes.POINTER(ctypes.c_uint32),  # packed_size
                ctypes.POINTER(ctypes.c_uint32),  # consumed_size
                ctypes.c_char_p,           # errbuf
                ctypes.c_size_t,           # errbuf_size
            ]
            lib.cbm_decode_to_rgba.restype = ctypes.c_int
            lib.cbm_free.argtypes = [ctypes.c_void_p]
            lib.cbm_free.restype = None
            _FAST_LIB = lib
            _FAST_LIB_ERROR = None
            return lib
        except Exception as exc:  # pragma: no cover - platform dependent
            last_error = f"{lib_path}: {exc}"

    _FAST_LIB = False
    _FAST_LIB_ERROR = last_error or "native library not found"
    return None


def fast_available() -> bool:
    return load_fast_lib() is not None


def _append_warning(info: CbmInfo, msg: str) -> None:
    info.warning = f"{info.warning}; {msg}" if info.warning else msg


def decode_cbm_bytes_fast(data: bytes, path: str = "<memory>", *, strict: bool = False, flip_y: bool = True) -> tuple[Image.Image, CbmInfo]:
    lib = load_fast_lib()
    if lib is None:
        raise CbmDecodeError(f"native accelerator unavailable: {_FAST_LIB_ERROR}")

    info = parse_cbm_header(data, path)
    info.backend = "fast"

    out_ptr = ctypes.c_void_p()
    width = ctypes.c_int()
    height = ctypes.c_int()
    channels = ctypes.c_int()
    packed_size = ctypes.c_uint32()
    consumed_size = ctypes.c_uint32()
    errbuf = ctypes.create_string_buffer(1024)

    # ctypes.c_char_p keeps the bytes object alive during the call; length is passed separately.
    data_ptr = ctypes.c_char_p(data)
    ret = lib.cbm_decode_to_rgba(
        data_ptr,
        ctypes.c_size_t(len(data)),
        ctypes.c_int(1 if flip_y else 0),
        ctypes.byref(out_ptr),
        ctypes.byref(width),
        ctypes.byref(height),
        ctypes.byref(channels),
        ctypes.byref(packed_size),
        ctypes.byref(consumed_size),
        errbuf,
        ctypes.c_size_t(len(errbuf)),
    )
    if ret != 0:
        err = errbuf.value.decode("utf-8", errors="replace") or f"native decoder error {ret}"
        raise CbmDecodeError(f"{path}: {err}")
    if not out_ptr.value:
        raise CbmDecodeError(f"{path}: native decoder returned null output")

    try:
        w = int(width.value)
        h = int(height.value)
        ch = int(channels.value)
        size = w * h * ch
        pixels = ctypes.string_at(out_ptr, size)
    finally:
        lib.cbm_free(out_ptr)

    info.width = w
    info.height = h
    info.channels = ch
    info.packed_size = int(packed_size.value)
    info.consumed_payload_size = int(consumed_size.value)

    if info.consumed_payload_size != info.packed_size:
        msg = f"decoder consumed {info.consumed_payload_size} payload bytes, header payload size is {info.packed_size}"
        _append_warning(info, msg)
        if strict:
            raise CbmDecodeError(f"{path}: {msg}")

    mode = "RGB" if ch == 3 else "RGBA"
    return Image.frombytes(mode, (w, h), pixels), info


def _put_channel(buf: bytearray, row_stride: int, channels: int, x: int, y: int, ch: int, value: int) -> None:
    buf[y * row_stride + x * channels + ch] = value & 0xFF


def decode_cbm_bytes_python(data: bytes, path: str = "<memory>", *, strict: bool = False, flip_y: bool = True) -> tuple[Image.Image, CbmInfo]:
    info = parse_cbm_header(data, path)
    info.backend = "python"
    payload = memoryview(data)[10:10 + info.packed_size]
    channels = info.channels
    width = info.width
    height = info.height
    row_stride = info.row_stride

    dib = bytearray(row_stride * height)

    # Match FUN_00414010's 32bpp default initialization. Normally every channel is
    # completely covered by packets, but this keeps truncated/non-standard files closer
    # to engine behavior.
    if channels == 4:
        for y in range(height):
            base = y * row_stride + 2
            for x in range(width):
                dib[base + x * 4] = 0xFF

    pos = 0
    for ch in range(channels):
        for y in range(height):
            x = 0
            while x < width:
                if pos + 2 > len(payload):
                    raise CbmDecodeError(
                        f"{path}: truncated packet at channel={ch}, row={y}, x={x}, pos={pos}"
                    )
                base = payload[pos]
                ctrl = payload[pos + 1]
                pos += 2

                _put_channel(dib, row_stride, channels, x, y, ch, base)
                x += 1

                if ctrl & 0x80:
                    count = ctrl & 0x7F
                    if x + count > width:
                        raise CbmDecodeError(
                            f"{path}: repeat packet overruns row at channel={ch}, row={y}, "
                            f"x={x}, count={count}, width={width}"
                        )
                    for _ in range(count):
                        _put_channel(dib, row_stride, channels, x, y, ch, base)
                        x += 1
                else:
                    count = ctrl
                    packed_nibbles = (count + 1) // 2
                    if pos + packed_nibbles > len(payload):
                        raise CbmDecodeError(
                            f"{path}: truncated nibble data at channel={ch}, row={y}, "
                            f"x={x}, count={count}, pos={pos}"
                        )
                    if x + count > width:
                        raise CbmDecodeError(
                            f"{path}: nibble packet overruns row at channel={ch}, row={y}, "
                            f"x={x}, count={count}, width={width}"
                        )
                    high = base & 0xF0
                    for k in range(count):
                        packed = payload[pos + (k // 2)]
                        low = (packed >> 4) if (k % 2 == 0) else (packed & 0x0F)
                        _put_channel(dib, row_stride, channels, x, y, ch, high | low)
                        x += 1
                    pos += packed_nibbles

    info.consumed_payload_size = pos
    if pos != info.packed_size:
        msg = f"decoder consumed {pos} payload bytes, header payload size is {info.packed_size}"
        _append_warning(info, msg)
        if strict:
            raise CbmDecodeError(f"{path}: {msg}")

    # Convert from DIB memory order BGR/BGRA to PNG RGB/RGBA.
    out = bytearray(width * height * channels)
    dst = 0
    if flip_y:
        rows: Iterable[int] = range(height - 1, -1, -1)  # positive-height DIBSection = bottom-up
    else:
        rows = range(height)
    for y in rows:
        row = dib[y * row_stride:y * row_stride + width * channels]
        if channels == 3:
            for i in range(0, len(row), 3):
                b, g, r = row[i], row[i + 1], row[i + 2]
                out[dst:dst + 3] = bytes((r, g, b))
                dst += 3
        else:
            for i in range(0, len(row), 4):
                b, g, r, a = row[i], row[i + 1], row[i + 2], row[i + 3]
                out[dst:dst + 4] = bytes((r, g, b, a))
                dst += 4

    mode = "RGB" if channels == 3 else "RGBA"
    img = Image.frombytes(mode, (width, height), bytes(out))
    return img, info


def decode_cbm_bytes(
    data: bytes,
    path: str = "<memory>",
    *,
    strict: bool = False,
    flip_y: bool = True,
    use_fast: bool = True,
    require_fast: bool = False,
) -> tuple[Image.Image, CbmInfo]:
    if use_fast:
        try:
            return decode_cbm_bytes_fast(data, path, strict=strict, flip_y=flip_y)
        except Exception:
            if require_fast:
                raise
            # Fall through to pure Python if the DLL is missing or failed to load.
    return decode_cbm_bytes_python(data, path, strict=strict, flip_y=flip_y)


def decode_cbm_file(
    input_path: Path,
    output_path: Path,
    *,
    strict: bool = False,
    write_info: bool = False,
    flip_y: bool = True,
    use_fast: bool = True,
    require_fast: bool = False,
) -> CbmInfo:
    data = input_path.read_bytes()
    img, info = decode_cbm_bytes(
        data,
        input_path.as_posix(),
        strict=strict,
        flip_y=flip_y,
        use_fast=use_fast,
        require_fast=require_fast,
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    img.save(output_path)
    if write_info:
        info_path = output_path.with_suffix(output_path.suffix + ".json")
        info_path.write_text(json.dumps(asdict(info), ensure_ascii=False, indent=2), encoding="utf-8")
    return info


def iter_cbm_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.rglob("*") if p.is_file() and p.suffix.lower() == ".cbm")


def output_for(input_root: Path, in_file: Path, output_root: Path) -> Path:
    if input_root.is_file():
        if output_root.suffix.lower() == ".png":
            return output_root
        return output_root / (input_root.stem + ".png")
    rel = in_file.relative_to(input_root)
    return (output_root / rel).with_suffix(".png")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Convert KEYINSE .CBM images to PNG")
    parser.add_argument("input", nargs="?", help="input .cbm file or directory")
    parser.add_argument("output", nargs="?", help="output .png file or directory")
    parser.add_argument("--strict", action="store_true", help="fail on trailing/unused payload bytes")
    parser.add_argument("--info", action="store_true", help="write sidecar .json with decoded header/stat info")
    parser.add_argument("--no-flip", action="store_true", help="do not vertically flip decoded DIB rows")
    parser.add_argument("--no-fast", action="store_true", help="disable native C accelerator and use pure Python")
    parser.add_argument("--require-fast", action="store_true", help="fail if native C accelerator cannot be loaded")
    parser.add_argument("--check-fast", action="store_true", help="print native accelerator status and exit")
    parser.add_argument("--version", action="version", version=VERSION)
    args = parser.parse_args(argv)

    if args.check_fast:
        lib = load_fast_lib()
        if lib is None:
            print(f"[fast] unavailable: {_FAST_LIB_ERROR}")
            return 1
        print("[fast] available")
        return 0

    if not args.input or not args.output:
        parser.error("input and output are required unless --check-fast is used")

    input_path = Path(args.input)
    output_path = Path(args.output)
    files = iter_cbm_files(input_path)
    if not files:
        print(f"[cbm2png] no .cbm files found: {input_path}", file=sys.stderr)
        return 1

    ok = 0
    failed = 0
    use_fast = not args.no_fast
    for in_file in files:
        out_file = output_for(input_path, in_file, output_path)
        try:
            info = decode_cbm_file(
                in_file,
                out_file,
                strict=args.strict,
                write_info=args.info,
                flip_y=not args.no_flip,
                use_fast=use_fast,
                require_fast=args.require_fast,
            )
            ok += 1
            warn = f" warning={info.warning}" if info.warning else ""
            print(
                f"[ok][{info.backend}] {in_file} -> {out_file} "
                f"{info.width}x{info.height} {info.channels * 8}bpp packed={info.packed_size}{warn}"
            )
        except Exception as exc:
            failed += 1
            print(f"[fail] {in_file}: {exc}", file=sys.stderr)

    print(f"[cbm2png] converted={ok} failed={failed}")
    return 0 if failed == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
