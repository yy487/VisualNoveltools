"""Convert this game's PC98 GPC images and GPA frames to lossless indexed PNG.

Input is an extracted game directory, not an FDI file. No Rust tool changes.
Codec evidence: ADVBIOS 12B94/1300E (sparse blocks), 12B57/1305C
(horizontal XOR), 12AF4/130E0 (previous row), 12C8A (GRB palette).
"""

from __future__ import annotations

import argparse
import json
import os
import struct
import tempfile
from dataclasses import dataclass
from pathlib import Path

from PIL import Image


class FormatError(ValueError):
    pass


def u16(data: bytes, offset: int) -> int:
    if offset < 0 or offset + 2 > len(data):
        raise FormatError(f"Truncated u16 at 0x{offset:X}")
    return struct.unpack_from("<H", data, offset)[0]


def u32(data: bytes, offset: int) -> int:
    if offset < 0 or offset + 4 > len(data):
        raise FormatError(f"Truncated u32 at 0x{offset:X}")
    return struct.unpack_from("<I", data, offset)[0]


class Sparse:
    def __init__(self, data: bytes, start: int, limit: int):
        if not 0 <= start <= limit <= len(data):
            raise FormatError("Compressed source range is outside the resource")
        self.data, self.pos, self.limit = data, start, limit
        self.pending = bytearray()

    def byte(self) -> int:
        if self.pos >= self.limit:
            raise FormatError(f"Truncated sparse stream at 0x{self.pos:X}")
        value = self.data[self.pos]
        self.pos += 1
        return value

    def row(self, size: int) -> bytearray:
        while len(self.pending) < size:
            outer = self.byte()
            for bit in range(7, -1, -1):
                if outer & (1 << bit):
                    inner = self.byte()
                    for sub in range(7, -1, -1):
                        self.pending.append(self.byte() if inner & (1 << sub) else 0)
                else:
                    self.pending.extend(b"\0" * 8)
        result = self.pending[:size]
        del self.pending[:size]
        return result


def horizontal_xor(row: bytearray) -> bytearray:
    stride = row[0]
    pixels = row[1:]
    if stride:
        value = pixels[0]
        # The accumulator carries between residue chains; it is NOT reset
        # to the first byte of each chain (12B72..12B8E).
        for residue in range(stride):
            start = stride if residue == 0 else residue
            for p in range(start, len(pixels), stride):
                pixels[p] ^= value
                value = pixels[p]
    return pixels


def decode_pixels(data: bytes, start: int, limit: int, width: int,
                  height: int, interleave: int) -> tuple[bytes, int]:
    if not 1 <= width <= 640 or not 1 <= height <= 400:
        raise FormatError(f"Unsupported dimensions {width}x{height}")
    if not 1 <= interleave <= height:
        raise FormatError(f"Invalid row interleave {interleave}")
    plane_width = (width + 7) // 8
    row_size = plane_width * 4
    stream = Sparse(data, start, limit)
    previous = None
    indices = bytearray(width * height)
    for phase in range(interleave):
        for y in range(phase, height, interleave):
            row = horizontal_xor(stream.row(row_size + 1))
            if previous is not None:
                for x in range(row_size):
                    row[x] ^= previous[x]
            previous = row
            for x in range(width):
                byte, bit = x // 8, 7 - x % 8
                indices[y * width + x] = sum(
                    ((row[plane * plane_width + byte] >> bit) & 1) << plane
                    for plane in range(4)
                )
    return bytes(indices), stream.pos - start


def palette_rgb(word: int) -> tuple[int, int, int]:
    # PC98 ports AA/AC/AE receive G/R/B. Source word is 0x0GRB.
    return ((word >> 4 & 15) * 17, (word >> 8 & 15) * 17, (word & 15) * 17)


@dataclass
class Picture:
    width: int
    height: int
    x: int
    y: int
    interleave: int
    pixels: bytes
    palette: list[tuple[int, int, int]]
    used: int
    available: int

    def save(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        image = Image.frombytes("P", (self.width, self.height), self.pixels)
        image.putpalette([component for rgb in self.palette for component in rgb]
                         + [0] * (768 - 16 * 3))
        image.save(path, format="PNG", bits=4)

    def info(self) -> dict:
        return {
            "width": self.width, "height": self.height,
            "source_x": self.x, "source_y": self.y,
            "row_interleave": self.interleave,
            "compressed_bytes_used": self.used,
            "compressed_bytes_available": self.available,
        }


def gpc(data: bytes) -> Picture:
    if data[:16] != b"PC98)GPCFILE   \0":
        raise FormatError("Not this game's GPC signature")
    if u32(data, 0x20) != len(data):
        raise FormatError("GPC declared file length does not match")
    palette_offset, image_offset = u32(data, 0x14), u32(data, 0x18)
    if u16(data, palette_offset) != 16 or u16(data, palette_offset + 2) != 2:
        raise FormatError("Unsupported GPC palette layout")
    palette = [palette_rgb(u16(data, palette_offset + 4 + i * 2)) for i in range(16)]
    width, height = u16(data, image_offset), u16(data, image_offset + 2)
    x, y = u16(data, image_offset + 10), u16(data, image_offset + 12)
    interleave = u16(data, 0x10)
    start = image_offset + 16
    pixels, used = decode_pixels(data, start, len(data), width, height, interleave)
    return Picture(width, height, x, y, interleave, pixels, palette, used, len(data) - start)


def gpa(data: bytes, palette: list[tuple[int, int, int]]) -> list[Picture]:
    if data[:16] != b"PC98)GPAFILE   \0":
        raise FormatError("Not this game's GPA signature")
    count = u16(data, 0x10)
    if not 1 <= count <= 256:
        raise FormatError(f"Invalid GPA frame count {count}")
    offsets = [u32(data, 0x18 + i * 4) for i in range(count)]
    if offsets != sorted(set(offsets)) or offsets[0] < 0x18 + 4 * count:
        raise FormatError("GPA frame offsets overlap or are out of order")
    result = []
    for i, offset in enumerate(offsets):
        end = offsets[i + 1] if i + 1 < count else len(data)
        if not offset + 10 <= end <= len(data):
            raise FormatError(f"GPA frame {i} is outside the file")
        interleave, x, y, width, height = struct.unpack_from("<5H", data, offset)
        pixels, used = decode_pixels(data, offset + 10, end, width, height, interleave)
        result.append(Picture(width, height, x, y, interleave, pixels,
                              palette, used, end - offset - 10))
    return result


def convert(source: Path, output: Path) -> dict:
    source = source.resolve(strict=True)
    if not source.is_dir():
        raise FormatError("Input must be an extracted game directory")
    output = output.absolute()
    parent = output.parent.resolve(strict=True)
    output = parent / output.name
    if output.exists():
        raise FormatError(f"Output already exists; choose a new directory: {output}")
    resources = sorted(p for p in source.rglob("*")
                       if p.is_file() and p.suffix.lower() in (".gpc", ".gpa"))
    if not resources:
        raise FormatError("No GPC/GPA resources found")
    pictures: dict[str, tuple[Path, Picture]] = {}
    for path in resources:
        if path.suffix.lower() == ".gpc":
            key = path.stem.casefold()
            if key in pictures:
                raise FormatError(f"Duplicate GPC stem is ambiguous: {path.stem}")
            pictures[key] = (path, gpc(path.read_bytes()))
    manifest = {
        "format": "kerakera-pc98-png-v1",
        "palette": "16-color 0x0GRB, native full brightness",
        "transparency": "opaque; GPA frames overwrite their source rectangle",
        "pixel_scale": "native stored dimensions; no resampling",
        "gpc": [], "gpa": [],
    }
    with tempfile.TemporaryDirectory(prefix=f".{output.name}-", dir=parent) as temporary:
        stage = Path(temporary)
        for path, picture in pictures.values():
            relative = Path("GPC") / f"{path.stem}.png"
            picture.save(stage / relative)
            manifest["gpc"].append({"source": path.relative_to(source).as_posix(),
                                    "png": relative.as_posix(), **picture.info()})
        for path in resources:
            if path.suffix.lower() != ".gpa":
                continue
            paired = pictures.get(path.stem.casefold())
            if paired is None:
                raise FormatError(f"No same-named GPC palette for {path.name}")
            frames = gpa(path.read_bytes(), paired[1].palette)
            records = []
            for i, picture in enumerate(frames):
                relative = Path("GPA") / path.stem / f"frame_{i:03d}.png"
                picture.save(stage / relative)
                records.append({"index": i, "png": relative.as_posix(), **picture.info()})
            manifest["gpa"].append({"source": path.relative_to(source).as_posix(),
                                    "palette_source": paired[0].relative_to(source).as_posix(),
                                    "frames": records})
        manifest["gpc_count"] = len(manifest["gpc"])
        manifest["gpa_count"] = len(manifest["gpa"])
        manifest["gpa_frame_count"] = sum(len(a["frames"]) for a in manifest["gpa"])
        manifest["png_count"] = manifest["gpc_count"] + manifest["gpa_frame_count"]
        (stage / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2)
                                             + "\n", encoding="utf-8")
        if output.exists():
            raise FormatError(f"Output appeared during conversion: {output}")
        os.rename(stage, output)
    return manifest


def self_check() -> None:
    # A sparse block with one literal, followed by an all-zero block.
    stream = Sparse(bytes((0x80, 0x80, 0x5a, 0)), 0, 4)
    assert stream.row(65) == bytes((0x5a,)) + bytes(64)
    assert horizontal_xor(bytearray((1, 1, 3, 7, 15))) == bytes((1, 2, 5, 10))
    assert horizontal_xor(bytearray((2, 1, 2, 4, 8))) == bytes((1, 7, 5, 15))
    assert palette_rgb(0x0a5f) == (85, 170, 255)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, nargs="?", help="Extracted game directory containing GPC/GPA")
    parser.add_argument("output", type=Path, nargs="?", help="New output directory (parent must exist)")
    parser.add_argument("--self-check", action="store_true", help="Check codec primitives without reading game data")
    args = parser.parse_args()
    if args.self_check:
        self_check()
        print("Codec primitive checks passed.")
        return
    if args.source is None or args.output is None:
        parser.error("source and output are required")
    try:
        manifest = convert(args.source, args.output)
    except (FormatError, OSError) as error:
        parser.exit(1, f"Conversion failed: {error}\n")
    print(f"Converted {manifest['gpc_count']} GPC images and "
          f"{manifest['gpa_frame_count']} frames from {manifest['gpa_count']} GPA files "
          f"to {manifest['png_count']} PNGs: {args.output.resolve()}")


if __name__ == "__main__":
    main()
