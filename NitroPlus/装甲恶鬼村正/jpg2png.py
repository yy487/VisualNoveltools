#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Muramasa / Nitro+ NPA resource JPG -> PNG converter.

This tool follows the image loading path visible in the supplied disassembly:

  sub_464B50  generic resource image load dispatcher
  sub_466D60  JPEG loader branch
  sub_467040  JPEG/JFIF signature check
  sub_467000  builds a memory input stream over the resource bytes

The .jpg resources observed in this game are zlib-wrapped JPEG streams:
resource bytes start with a zlib header, and after inflate they start with
FF D8 FF E0 00 10 'JFIF'.  The game then decodes JPEG scanlines and expands
RGB/grayscale into a 32-bit surface with alpha = 0xFF.  This script mirrors
that result by inflating when needed, decoding JPEG, and saving RGBA PNG.
"""
from __future__ import annotations

import argparse
import io
import sys
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

try:
    from PIL import Image, ImageOps
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Pillow is required. Install with: python -m pip install pillow"
    ) from exc

JFIF_PREFIX = b"\xFF\xD8\xFF\xE0\x00\x10JFIF"  # sub_467040 checks first 11 bytes
JPEG_SOI = b"\xFF\xD8"


@dataclass
class ConvertResult:
    input_path: Path
    output_path: Path | None
    ok: bool
    inflated: bool = False
    jpeg_size: int = 0
    width: int = 0
    height: int = 0
    mode: str = ""
    error: str = ""


def is_probably_zlib(data: bytes) -> bool:
    """Cheap zlib stream test. 0x78 0x01/0x5E/0x9C/0xDA are common deflate levels."""
    if len(data) < 2:
        return False
    cmf, flg = data[0], data[1]
    return (cmf & 0x0F) == 8 and ((cmf << 8) + flg) % 31 == 0


def unwrap_game_jpg(data: bytes, *, force_inflate: bool = False) -> tuple[bytes, bool]:
    """
    Return the actual JPEG byte stream.

    For this sample the disk file is zlib-compressed and inflates to a normal
    JFIF JPEG.  If the input is already a JPEG, return it unchanged.
    """
    if data.startswith(JPEG_SOI) and not force_inflate:
        return data, False

    if force_inflate or is_probably_zlib(data):
        try:
            inflated = zlib.decompress(data)
        except zlib.error as exc:
            raise ValueError(f"zlib inflate failed: {exc}") from exc
        return inflated, True

    # Last chance: some archives may store raw JPEG with junk-free SOI but not JFIF.
    if data.startswith(JPEG_SOI):
        return data, False

    raise ValueError(
        "not a recognized game JPG: input is neither JPEG SOI nor zlib-wrapped JPEG"
    )


def check_jfif(jpeg: bytes, *, strict_jfif: bool = False) -> None:
    """
    Validate the JPEG header.

    The game's sub_467040 compares the first 11 bytes against:
      FF D8 FF E0 00 10 4A 46 49 46 00
    In non-strict mode we accept any normal JPEG SOI, because some files may be
    Exif/APP1-first JPEGs even though this game's common path is JFIF.
    """
    if strict_jfif:
        if not jpeg.startswith(JFIF_PREFIX):
            got = jpeg[:16].hex(" ")
            raise ValueError(f"strict JFIF check failed; first bytes: {got}")
        return

    if not jpeg.startswith(JPEG_SOI):
        got = jpeg[:16].hex(" ")
        raise ValueError(f"inflated data is not JPEG; first bytes: {got}")


def convert_bytes_to_png(jpeg: bytes, out_path: Path) -> tuple[int, int, str]:
    """
    Decode JPEG and save RGBA PNG.

    Disassembly behavior:
      - 3 components: RGB -> 32-bit surface with alpha FF
      - 1 component : gray -> RGB replicated, alpha FF
    RGBA PNG preserves that 32-bit opaque result.
    """
    with Image.open(io.BytesIO(jpeg)) as im:
        im = ImageOps.exif_transpose(im)
        original_mode = im.mode
        if im.mode not in ("RGB", "RGBA", "L"):
            im = im.convert("RGB")
        if im.mode == "L":
            im = im.convert("RGBA")
        elif im.mode == "RGB":
            im = im.convert("RGBA")
        elif im.mode == "RGBA":
            pass
        else:
            im = im.convert("RGBA")

        out_path.parent.mkdir(parents=True, exist_ok=True)
        im.save(out_path, "PNG", optimize=False)
        return im.width, im.height, original_mode


def output_path_for(input_path: Path, input_root: Path, output_root: Path, *, keep_tree: bool) -> Path:
    if keep_tree:
        try:
            rel = input_path.relative_to(input_root)
        except ValueError:
            rel = Path(input_path.name)
        return (output_root / rel).with_suffix(".png")
    return output_root / (input_path.stem + ".png")


def iter_inputs(path: Path, recursive: bool) -> Iterable[Path]:
    if path.is_file():
        yield path
        return
    pattern = "**/*" if recursive else "*"
    for p in sorted(path.glob(pattern)):
        if p.is_file() and p.suffix.lower() in {".jpg", ".jpeg"}:
            yield p


def convert_one(
    input_path: Path,
    output_path: Path,
    *,
    strict_jfif: bool = False,
    force_inflate: bool = False,
    overwrite: bool = True,
) -> ConvertResult:
    result = ConvertResult(input_path=input_path, output_path=output_path, ok=False)
    try:
        if output_path.exists() and not overwrite:
            raise FileExistsError(f"output exists: {output_path}")
        raw = input_path.read_bytes()
        jpeg, inflated = unwrap_game_jpg(raw, force_inflate=force_inflate)
        check_jfif(jpeg, strict_jfif=strict_jfif)
        width, height, mode = convert_bytes_to_png(jpeg, output_path)
        result.ok = True
        result.inflated = inflated
        result.jpeg_size = len(jpeg)
        result.width = width
        result.height = height
        result.mode = mode
        return result
    except Exception as exc:
        result.error = str(exc)
        return result


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Convert Muramasa/Nitro+ zlib-wrapped JPG resources to PNG."
    )
    ap.add_argument("input", help="input .jpg/.jpeg file or directory")
    ap.add_argument("output", help="output .png file or directory")
    ap.add_argument("--recursive", "-r", action="store_true", help="scan input directory recursively")
    ap.add_argument("--keep-tree", action="store_true", help="preserve relative directory layout in batch mode")
    ap.add_argument("--strict-jfif", action="store_true", help="require the exact JFIF prefix checked by sub_467040")
    ap.add_argument("--force-inflate", action="store_true", help="force zlib inflate before JPEG decode")
    ap.add_argument("--no-overwrite", action="store_true", help="do not overwrite existing PNG files")
    args = ap.parse_args(argv)

    in_path = Path(args.input)
    out_path = Path(args.output)

    if in_path.is_file():
        final_out = out_path if out_path.suffix.lower() == ".png" else out_path / (in_path.stem + ".png")
        inputs = [(in_path, final_out)]
        input_root = in_path.parent
    else:
        input_root = in_path
        inputs = [
            (p, output_path_for(p, input_root, out_path, keep_tree=args.keep_tree))
            for p in iter_inputs(in_path, args.recursive)
        ]

    if not inputs:
        print(f"[jpg2png] no jpg/jpeg files found: {in_path}")
        return 1

    ok = 0
    failed = 0
    for src, dst in inputs:
        res = convert_one(
            src,
            dst,
            strict_jfif=args.strict_jfif,
            force_inflate=args.force_inflate,
            overwrite=not args.no_overwrite,
        )
        if res.ok:
            ok += 1
            flag = "inflate+decode" if res.inflated else "decode"
            print(
                f"[jpg2png] {flag}: {src} -> {dst} "
                f"{res.width}x{res.height} mode={res.mode} jpeg_size={res.jpeg_size}"
            )
        else:
            failed += 1
            print(f"[jpg2png][warn] failed: {src}: {res.error}", file=sys.stderr)

    print(f"[jpg2png] converted={ok} failed={failed}")
    return 0 if failed == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
