# -*- coding: utf-8 -*-
"""
YDG2PNG batch converter for YU-RIS YDG images.

Current supported YDG variant:
  - magic: b"YDG\0"
  - signature: b"YU-RIS\0\0"
  - embedded RIFF/WEBP chunks, usually vertical strips

The actual WebP decoding and PNG writing are handled by Pillow's C backends
(libwebp/libpng/zlib). This script deliberately avoids Python per-pixel loops.
"""
from __future__ import annotations

import argparse
import concurrent.futures as futures
import dataclasses
import io
import os
import sys
import traceback
from pathlib import Path
from typing import Iterable, Sequence

try:
    from PIL import Image, features
except Exception as exc:  # pragma: no cover - user environment diagnostic
    Image = None  # type: ignore[assignment]
    features = None  # type: ignore[assignment]
    _PIL_IMPORT_ERROR = exc
else:
    _PIL_IMPORT_ERROR = None


YDG_MAGIC = b"YDG\x00"
YURIS_SIG = b"YU-RIS\x00\x00"
RIFF_MAGIC = b"RIFF"
WEBP_MAGIC = b"WEBP"


class YdgError(RuntimeError):
    pass


@dataclasses.dataclass(frozen=True)
class YdgChunk:
    index: int
    offset: int
    size: int
    kind: int
    strip_height: int
    flags0: int
    flags1: int


@dataclasses.dataclass(frozen=True)
class YdgHeader:
    version_or_type: int
    table_offset: int
    file_size: int
    width: int
    height: int
    chunks: tuple[YdgChunk, ...]


def _u16le(data: bytes, off: int) -> int:
    if off < 0 or off + 2 > len(data):
        raise YdgError(f"u16 out of range at 0x{off:X}")
    return int.from_bytes(data[off:off + 2], "little")


def _u32le(data: bytes, off: int) -> int:
    if off < 0 or off + 4 > len(data):
        raise YdgError(f"u32 out of range at 0x{off:X}")
    return int.from_bytes(data[off:off + 4], "little")


def parse_ydg_header(data: bytes) -> YdgHeader:
    """Parse the YDG container header and chunk table."""
    if len(data) < 0x40:
        raise YdgError("file too small for YDG header")
    if data[:4] != YDG_MAGIC:
        raise YdgError("bad magic: expected YDG\\0")
    if data[4:12] != YURIS_SIG:
        raise YdgError("unsupported YDG signature: expected YU-RIS")

    version_or_type = _u32le(data, 0x0C)
    table_offset = _u32le(data, 0x10)
    file_size = _u32le(data, 0x14)
    # The observed YU-RIS YDG stores width/height as u16 at 0x20/0x22.
    width = _u16le(data, 0x20)
    height = _u16le(data, 0x22)

    if file_size not in (0, len(data)):
        # Keep parsing, but make the mismatch visible as an error because offsets
        # usually depend on this value. Some future variants may set it to 0.
        raise YdgError(f"file size mismatch: header=0x{file_size:X}, actual=0x{len(data):X}")
    if width <= 0 or height <= 0:
        raise YdgError(f"invalid canvas size: {width}x{height}")
    if table_offset <= 0 or table_offset + 4 > len(data):
        raise YdgError(f"invalid table offset: 0x{table_offset:X}")

    chunk_count = _u32le(data, table_offset)
    if chunk_count <= 0 or chunk_count > 4096:
        raise YdgError(f"invalid chunk count: {chunk_count}")

    entry_base = table_offset + 4
    table_size = 4 + chunk_count * 16
    if entry_base + chunk_count * 16 > len(data):
        raise YdgError("chunk table exceeds file size")

    chunks: list[YdgChunk] = []
    last_end = entry_base + chunk_count * 16
    for i in range(chunk_count):
        off = entry_base + i * 16
        chunk_offset = _u32le(data, off + 0)
        chunk_size = _u32le(data, off + 4)
        kind = _u16le(data, off + 8)
        strip_height = _u16le(data, off + 10)
        flags0 = _u16le(data, off + 12)
        flags1 = _u16le(data, off + 14)

        if chunk_offset < table_offset + table_size:
            raise YdgError(
                f"chunk #{i} offset overlaps header/table: offset=0x{chunk_offset:X}"
            )
        if chunk_size <= 12 or chunk_offset + chunk_size > len(data):
            raise YdgError(
                f"chunk #{i} range invalid: offset=0x{chunk_offset:X}, size=0x{chunk_size:X}"
            )
        if data[chunk_offset:chunk_offset + 4] != RIFF_MAGIC:
            raise YdgError(f"chunk #{i} is not RIFF at 0x{chunk_offset:X}")
        # RIFF size excludes the first 8 bytes.
        riff_size = _u32le(data, chunk_offset + 4) + 8
        if riff_size != chunk_size:
            raise YdgError(
                f"chunk #{i} RIFF size mismatch: table=0x{chunk_size:X}, riff=0x{riff_size:X}"
            )
        if data[chunk_offset + 8:chunk_offset + 12] != WEBP_MAGIC:
            raise YdgError(f"chunk #{i} is RIFF but not WEBP at 0x{chunk_offset:X}")
        if chunk_offset < last_end:
            raise YdgError(f"chunk #{i} is out of order or overlaps previous chunk")
        last_end = chunk_offset + chunk_size

        chunks.append(
            YdgChunk(
                index=i,
                offset=chunk_offset,
                size=chunk_size,
                kind=kind,
                strip_height=strip_height,
                flags0=flags0,
                flags1=flags1,
            )
        )

    return YdgHeader(
        version_or_type=version_or_type,
        table_offset=table_offset,
        file_size=file_size or len(data),
        width=width,
        height=height,
        chunks=tuple(chunks),
    )


def load_webp_strip(data: bytes, chunk: YdgChunk):
    assert Image is not None
    blob = data[chunk.offset:chunk.offset + chunk.size]
    im = Image.open(io.BytesIO(blob))
    # Force decoding while BytesIO is alive.
    im.load()
    return im


def convert_ydg_to_png(input_path: Path, output_path: Path, *, overwrite: bool = False) -> YdgHeader:
    if Image is None:
        raise YdgError(
            "Pillow is not installed or failed to import. Install with: python -m pip install pillow. "
            f"Original import error: {_PIL_IMPORT_ERROR!r}"
        )
    if output_path.exists() and not overwrite:
        raise YdgError(f"output exists, use --overwrite: {output_path}")

    data = input_path.read_bytes()
    header = parse_ydg_header(data)

    strips = [load_webp_strip(data, chunk) for chunk in header.chunks]
    if not strips:
        raise YdgError("no WEBP strips found")

    # Prefer the declared canvas size. If a strange variant has a different total
    # strip height, use the decoded strip sum but keep a warning in verbose mode.
    strip_widths = {im.width for im in strips}
    if len(strip_widths) != 1:
        raise YdgError(f"mixed strip widths are not supported: {sorted(strip_widths)}")
    decoded_width = strips[0].width
    decoded_height = sum(im.height for im in strips)
    if decoded_width != header.width:
        raise YdgError(f"width mismatch: header={header.width}, decoded={decoded_width}")

    out_height = header.height if header.height == decoded_height else decoded_height
    has_alpha = any("A" in im.getbands() for im in strips)
    out_mode = "RGBA" if has_alpha else "RGB"
    out = Image.new(out_mode, (decoded_width, out_height))

    y = 0
    for im in strips:
        if im.mode != out_mode:
            im = im.convert(out_mode)
        out.paste(im, (0, y))
        y += im.height

    output_path.parent.mkdir(parents=True, exist_ok=True)
    out.save(output_path, format="PNG")
    return header


def export_raw_webp(input_path: Path, output_dir: Path, *, overwrite: bool = False) -> int:
    data = input_path.read_bytes()
    header = parse_ydg_header(data)
    stem = input_path.stem
    target_dir = output_dir / f"{stem}_webp"
    target_dir.mkdir(parents=True, exist_ok=True)
    for chunk in header.chunks:
        out = target_dir / f"{stem}_{chunk.index:02d}.webp"
        if out.exists() and not overwrite:
            raise YdgError(f"output exists, use --overwrite: {out}")
        out.write_bytes(data[chunk.offset:chunk.offset + chunk.size])
    return len(header.chunks)


def iter_ydg_files(input_path: Path, *, recursive: bool = True) -> list[Path]:
    if input_path.is_file():
        if input_path.suffix.lower() != ".ydg":
            raise YdgError(f"input file is not .ydg: {input_path}")
        return [input_path]
    if not input_path.is_dir():
        raise YdgError(f"input path not found: {input_path}")
    pattern = "**/*.ydg" if recursive else "*.ydg"
    return sorted(p for p in input_path.glob(pattern) if p.is_file())


def output_path_for(src: Path, input_root: Path, output_root: Path, suffix: str) -> Path:
    if input_root.is_file():
        rel = Path(src.stem + suffix + ".png")
    else:
        rel0 = src.relative_to(input_root)
        rel = rel0.with_name(rel0.stem + suffix + ".png")
    return output_root / rel


def convert_one(args: tuple[Path, Path, Path, str, bool, bool]) -> tuple[str, bool, str]:
    src, input_root, output_root, suffix, overwrite, raw_webp = args
    try:
        dst = output_path_for(src, input_root, output_root, suffix)
        header = convert_ydg_to_png(src, dst, overwrite=overwrite)
        msg = f"{src} -> {dst} ({header.width}x{header.height}, chunks={len(header.chunks)})"
        if raw_webp:
            n = export_raw_webp(src, output_root / "_raw_webp", overwrite=overwrite)
            msg += f", raw_webp={n}"
        return (str(src), True, msg)
    except Exception as exc:
        return (str(src), False, f"{src}: {exc}")


def print_info(path: Path) -> None:
    data = path.read_bytes()
    h = parse_ydg_header(data)
    print(f"file        : {path}")
    print(f"version/type: {h.version_or_type}")
    print(f"table_offset: 0x{h.table_offset:X}")
    print(f"file_size   : 0x{h.file_size:X}")
    print(f"canvas      : {h.width}x{h.height}")
    print(f"chunks      : {len(h.chunks)}")
    for c in h.chunks:
        print(
            f"  #{c.index:02d} off=0x{c.offset:08X} size=0x{c.size:08X} "
            f"kind={c.kind} strip_h={c.strip_height} flags=0x{c.flags0:04X}/0x{c.flags1:04X}"
        )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Batch convert YU-RIS .ydg images to .png."
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_info = sub.add_parser("info", help="show YDG container info")
    p_info.add_argument("input", help="input .ydg file")

    p_conv = sub.add_parser("convert", help="convert one file or all .ydg files in a directory")
    p_conv.add_argument("input", help="input .ydg file or directory")
    p_conv.add_argument("output", help="output .png file or output directory")
    p_conv.add_argument("-j", "--jobs", type=int, default=max(1, (os.cpu_count() or 4) // 2), help="parallel workers")
    p_conv.add_argument("--no-recursive", action="store_true", help="only process files directly under input directory")
    p_conv.add_argument("--overwrite", action="store_true", help="overwrite existing PNG files")
    p_conv.add_argument("--suffix", default="", help="append suffix before .png, for example _chs")
    p_conv.add_argument("--raw-webp", action="store_true", help="also export embedded WEBP strips")
    p_conv.add_argument("--traceback", action="store_true", help="print Python traceback on top-level failure")

    args = parser.parse_args(argv)

    try:
        if args.cmd == "info":
            print_info(Path(args.input))
            return 0

        input_path = Path(args.input)
        output_path = Path(args.output)
        files = iter_ydg_files(input_path, recursive=not args.no_recursive)
        if not files:
            print(f"[ydg2png] no .ydg files found: {input_path}")
            return 1

        # If converting a single input file and output ends with .png, respect it.
        if input_path.is_file() and output_path.suffix.lower() == ".png":
            output_root = output_path.parent
            suffix = ""
            # convert directly without mirrored relative path.
            try:
                h = convert_ydg_to_png(input_path, output_path, overwrite=args.overwrite)
                print(f"[ok] {input_path} -> {output_path} ({h.width}x{h.height}, chunks={len(h.chunks)})")
                if args.raw_webp:
                    n = export_raw_webp(input_path, output_root / "_raw_webp", overwrite=args.overwrite)
                    print(f"[ok] raw_webp chunks={n}")
                return 0
            except Exception as exc:
                print(f"[fail] {input_path}: {exc}")
                return 2

        tasks = [(src, input_path, output_path, args.suffix, args.overwrite, args.raw_webp) for src in files]
        ok = 0
        fail = 0
        max_workers = max(1, args.jobs)
        with futures.ThreadPoolExecutor(max_workers=max_workers) as ex:
            for _src, success, msg in ex.map(convert_one, tasks):
                if success:
                    ok += 1
                    print(f"[ok] {msg}")
                else:
                    fail += 1
                    print(f"[fail] {msg}")
        print(f"[ydg2png] files={len(files)} ok={ok} failed={fail} output={output_path}")
        return 0 if fail == 0 else 2
    except Exception:
        if getattr(args, "traceback", False):
            traceback.print_exc()
        else:
            print(f"[error] {sys.exc_info()[1]}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
