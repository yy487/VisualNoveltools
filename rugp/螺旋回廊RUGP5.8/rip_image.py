"""Convert extracted rUGP CRip, CRip007, and CS5i objects to PNG."""

from __future__ import annotations

import argparse
import colorsys
import json
import struct
import zlib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


class RipFormatError(ValueError):
    """Raised when an extracted image object is malformed or unsupported."""


@dataclass(frozen=True)
class DecodedImage:
    width: int
    height: int
    bgra: bytes
    kind: str
    metadata: dict[str, int] = field(default_factory=dict)


class BitReader:
    def __init__(self, data: bytes, *, lsb_first: bool) -> None:
        self.data = data
        self.lsb_first = lsb_first
        self.bit_pos = 0

    def read_bit(self) -> int:
        if self.bit_pos >= len(self.data) * 8:
            raise RipFormatError("compressed bitstream ended early")
        byte = self.data[self.bit_pos >> 3]
        bit = self.bit_pos & 7
        self.bit_pos += 1
        shift = bit if self.lsb_first else 7 - bit
        return (byte >> shift) & 1

    def read_uint(self) -> int:
        value = 1
        pairs = 0
        while self.read_bit():
            value = (value << 1) | self.read_bit()
            pairs += 1
            if pairs > 30:
                raise RipFormatError("integer code is too long")
        return value

    def read_signed(self) -> int:
        if not self.read_bit():
            return 0
        negative = self.read_bit()
        value = self.read_uint()
        return -value if negative else value


QUANT_TABLES = (
    bytes(range(128)),
    bytes(
        [
            0x00, 0x01, 0x02, 0x04, 0x06, 0x09, 0x0C, 0x0F,
            0x13, 0x16, 0x19, 0x1C, 0x1F, 0x23, 0x27, 0x2B,
            0x30, 0x34, 0x38, 0x3C, 0x40, 0x44, 0x48, 0x4C,
            0x50, 0x54, 0x58, 0x5C, 0x60, 0x64, 0x68, 0x6C,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
            0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
        ]
    ),
    bytes(
        [
            0x00, 0x01, 0x02, 0x04, 0x08, 0x0C, 0x10, 0x14,
            0x18, 0x1B, 0x1E, 0x22, 0x26, 0x2A, 0x2E, 0x32,
            0x36, 0x3A, 0x3E, 0x42, 0x46, 0x4B, 0x50, 0x55,
            0x5A, 0x5F, 0x64, 0x69, 0x6E, 0x73, 0x78, 0x7D,
        ]
    ),
    bytes(
        [
            0x00, 0x01, 0x03, 0x07, 0x0C, 0x10, 0x15, 0x1A,
            0x20, 0x25, 0x2A, 0x30, 0x36, 0x3C, 0x42, 0x48,
            0x50, 0x54, 0x58, 0x5C, 0x60, 0x64, 0x68, 0x6C,
            0x70, 0x74, 0x78, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
        ]
    ),
    bytes(
        [
            0x00, 0x01, 0x03, 0x07, 0x0D, 0x13, 0x1A, 0x21,
            0x28, 0x2F, 0x36, 0x3E, 0x46, 0x4E, 0x56, 0x5E,
            0x68, 0x6A, 0x6C, 0x6E, 0x70, 0x72, 0x74, 0x76,
            0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
        ]
    ),
    bytes(
        [
            0x00, 0x01, 0x04, 0x0A, 0x11, 0x18, 0x20, 0x28,
            0x32, 0x3C, 0x46, 0x50, 0x5A, 0x64, 0x6E, 0x78,
        ]
    ),
)


def _u16(data: bytes, offset: int) -> int:
    return struct.unpack_from("<H", data, offset)[0]


def _u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def _validate_dimensions(width: int, height: int) -> None:
    if not (0 < width <= 0x7FFF and 0 < height <= 0x7FFF):
        raise RipFormatError(f"invalid image dimensions {width}x{height}")
    if width * height > 100_000_000:
        raise RipFormatError("image dimensions are unreasonably large")


def _quantize(table_index: int, value: int) -> int:
    if table_index >= len(QUANT_TABLES):
        raise RipFormatError(f"unsupported CRip007 quantizer {table_index}")
    table = QUANT_TABLES[table_index]
    return table[value] if value < len(table) else 0


def _label_color(index: int) -> tuple[int, int, int, int]:
    if index == 0:
        return 0, 0, 0, 0
    hue = (index * 0.618033988749895) % 1.0
    red, green, blue = colorsys.hsv_to_rgb(hue, 0.72, 1.0)
    return round(blue * 255), round(green * 255), round(red * 255), 255


def _render_mask(indices: bytes, mode: str) -> bytes:
    output = bytearray(len(indices) * 4)
    if mode == "labels":
        palette = [_label_color(i) for i in range(256)]
        for pos, value in enumerate(indices):
            output[pos * 4 : pos * 4 + 4] = bytes(palette[value])
        return bytes(output)

    if mode == "raw":
        for pos, value in enumerate(indices):
            output[pos * 4 : pos * 4 + 4] = bytes((value, value, value, 255))
        return bytes(output)

    if mode == "gray":
        maximum = max(indices, default=0)
        scale = 255 / maximum if maximum else 0
        for pos, value in enumerate(indices):
            gray = round(value * scale)
            output[pos * 4 : pos * 4 + 4] = bytes((gray, gray, gray, 255))
        return bytes(output)

    raise RipFormatError(f"unknown CRip mask mode {mode!r}")


def decode_crip(data: bytes, *, mask_mode: str = "labels") -> DecodedImage:
    """Decode the compact row RLE used by the base CRip class."""
    if len(data) < 31:
        raise RipFormatError("CRip object is shorter than its 31-byte header")

    width, height, region_width, region_height = struct.unpack_from(
        "<HHHH", data, 11
    )
    codec, planes = struct.unpack_from("<HH", data, 19)
    payload_size = _u32(data, 23)
    reserved = _u32(data, 27)
    _validate_dimensions(width, height)

    if codec != 1 or planes != 1:
        raise RipFormatError(
            f"unsupported CRip codec tuple ({codec}, {planes})"
        )
    if payload_size != len(data) - 31:
        raise RipFormatError(
            f"CRip payload size mismatch: header={payload_size}, "
            f"available={len(data) - 31}"
        )

    payload = data[31:]
    indices = bytearray(width * height)
    src = 0
    for y in range(height):
        x = 0
        value = 0
        while x < width:
            if src >= len(payload):
                raise RipFormatError(f"CRip RLE ended on row {y}")
            count = payload[src]
            src += 1
            if count:
                end = min(width, x + count)
                indices[y * width + x : y * width + end] = bytes([value]) * (
                    end - x
                )
                x += count
                if x > width:
                    raise RipFormatError(f"CRip RLE overruns row {y}")
            if x < width:
                if src >= len(payload):
                    raise RipFormatError(f"CRip RLE misses a value on row {y}")
                value = payload[src]
                src += 1

    if src != len(payload):
        raise RipFormatError(
            f"CRip RLE left {len(payload) - src} trailing payload bytes"
        )

    return DecodedImage(
        width,
        height,
        _render_mask(bytes(indices), mask_mode),
        "CRip",
        {
            "region_width": region_width,
            "region_height": region_height,
            "codec": codec,
            "planes": planes,
            "reserved": reserved,
            "region_count": max(indices, default=0),
        },
    )


def _decode_color_delta(
    bits: BitReader,
    pixel_format: int,
    quantizer: int,
    color: list[int],
    luma_state: list[int],
) -> None:
    red_bits = (pixel_format >> 8) & 0xF
    green_bits = (pixel_format >> 16) & 0xF
    blue_bits = (pixel_format >> 24) & 0xF

    if pixel_format & 4:
        prior = max(-color[1], min(luma_state[0], (1 << green_bits) - 1 - color[1]))
        luma_state[0] = prior + bits.read_signed()
    else:
        luma_state[0] = bits.read_signed()

    blue_chroma = bits.read_signed()
    red_chroma = bits.read_signed()
    if not (pixel_format & 2):
        blue_chroma = (
            -_quantize(quantizer, -blue_chroma)
            if blue_chroma < 0
            else _quantize(quantizer, blue_chroma)
        )
        red_chroma = (
            -_quantize(quantizer, -red_chroma)
            if red_chroma < 0
            else _quantize(quantizer, red_chroma)
        )

    green_delta = luma_state[0]
    base_delta = green_delta >> (green_bits - red_bits)

    if pixel_format & 1:
        red_delta = (
            max(-color[0], min(base_delta, (1 << red_bits) - 1 - color[0]))
            + red_chroma
        )
        blue_delta = (
            max(-color[2], min(base_delta, (1 << blue_bits) - 1 - color[2]))
            + blue_chroma
        )
    else:
        red_delta = base_delta + red_chroma
        blue_delta = base_delta + blue_chroma

    color[0] += red_delta
    color[1] += green_delta
    color[2] += blue_delta


def _decode_rip007_pixels(
    payload: bytes,
    width: int,
    height: int,
    image_flags: int,
    pixel_format: int,
    quantizer: int,
) -> bytes:
    has_alpha = bool(image_flags & 1)
    lsb_first = bool(pixel_format & 8)
    bits = BitReader(payload, lsb_first=lsb_first)

    red_bits = (pixel_format >> 8) & 0xF
    green_bits = (pixel_format >> 16) & 0xF
    blue_bits = (pixel_format >> 24) & 0xF
    if min(red_bits, green_bits, blue_bits) <= 0:
        raise RipFormatError(f"invalid CRip007 pixel format 0x{pixel_format:08X}")
    if green_bits < red_bits:
        raise RipFormatError(
            f"unsupported CRip007 channel widths in 0x{pixel_format:08X}"
        )

    shifts = (8 - red_bits, 8 - green_bits, 8 - blue_bits)
    output = bytearray(width * height * 4)
    previous_line = [[0, 0, 0] for _ in range(width)]

    for y in range(height):
        color = [0, 0, 0]
        alpha_level = 0 if has_alpha else 32
        alpha_run = 0
        color_run = 0
        decode_color = False
        luma_state = [0]
        x = 0

        while x < width:
            if has_alpha and alpha_run == 0:
                alpha_level += bits.read_signed()

            if alpha_level:
                opaque = alpha_level > 31 or (
                    not (pixel_format & 2) and alpha_level == 31
                )
                if has_alpha and opaque:
                    if alpha_run == 0:
                        alpha_run = bits.read_uint()
                    alpha_run -= 1

                if color_run == 0:
                    color_run = bits.read_uint()
                    decode_color = not decode_color
                    luma_state[0] = 0

                if decode_color:
                    if bits.read_bit():
                        color[:] = previous_line[x]
                        luma_state[0] = 0
                    else:
                        _decode_color_delta(
                            bits, pixel_format, quantizer, color, luma_state
                        )

                previous_line[x] = color.copy()
                offset = (y * width + x) * 4
                # The predictor state is RGB; normalize every decoder to BGRA.
                output[offset] = (color[2] << shifts[2]) & 0xFF
                output[offset + 1] = (color[1] << shifts[1]) & 0xFF
                output[offset + 2] = (color[0] << shifts[0]) & 0xFF
                output[offset + 3] = 255 if opaque else (alpha_level * 8) & 0xFF
                color_run -= 1
                x += 1
            else:
                count = bits.read_uint()
                if x + count > width:
                    raise RipFormatError(f"CRip007 transparent run overruns row {y}")
                if pixel_format & 8:
                    for clear_x in range(x, x + count):
                        previous_line[clear_x] = [0, 0, 0]
                x += count

    return bytes(output)


def decode_crip007(data: bytes) -> DecodedImage:
    """Decode the CRip007 bitstream used by the second game."""
    if len(data) < 41:
        raise RipFormatError("CRip007 object is shorter than its 41-byte header")

    width, height = struct.unpack_from("<HH", data, 15)
    image_flags = _u32(data, 19)
    quantizer = data[23]
    reserved_word = _u16(data, 24)
    pixel_format = _u32(data, 26)
    object_marker = data[30]
    object_value = _u16(data, 31)
    payload_size = _u32(data, 33)
    reserved_dword = _u32(data, 37)
    _validate_dimensions(width, height)

    if image_flags & 0xFE != 2:
        raise RipFormatError(
            f"invalid CRip007 image flags 0x{image_flags:08X}"
        )
    if object_marker not in (4, 5) or object_value != 0:
        raise RipFormatError(
            "unsupported CRip007 auxiliary object reference encoding"
        )
    if payload_size != len(data) - 41:
        raise RipFormatError(
            f"CRip007 payload size mismatch: header={payload_size}, "
            f"available={len(data) - 41}"
        )

    pixels = _decode_rip007_pixels(
        data[41:],
        width,
        height,
        image_flags,
        pixel_format,
        quantizer,
    )
    return DecodedImage(
        width,
        height,
        pixels,
        "CRip007",
        {
            "image_flags": image_flags,
            "quantizer": quantizer,
            "reserved_word": reserved_word,
            "pixel_format": pixel_format,
            "object_marker": object_marker,
            "reserved_dword": reserved_dword,
        },
    )


def decode_cs5i(data: bytes) -> DecodedImage:
    """Decode CS5i's bottom-up BGRA32 DIB payload."""
    if len(data) < 27:
        raise RipFormatError("CS5i object is shorter than its 27-byte header")

    width, height, region_width, region_height = struct.unpack_from(
        "<HHHH", data, 11
    )
    stride = _u32(data, 19)
    payload_size = _u32(data, 23)
    _validate_dimensions(width, height)

    if stride < width * 4 or stride % 4:
        raise RipFormatError(f"invalid CS5i row stride {stride}")
    if payload_size != stride * height:
        raise RipFormatError(
            f"CS5i payload is not stride x height: {payload_size} != "
            f"{stride} x {height}"
        )
    if payload_size != len(data) - 27:
        raise RipFormatError(
            f"CS5i payload size mismatch: header={payload_size}, "
            f"available={len(data) - 27}"
        )

    payload = data[27:]
    output = bytearray(width * height * 4)
    row_size = width * 4
    for y in range(height):
        source_y = height - 1 - y
        source = source_y * stride
        target = y * row_size
        output[target : target + row_size] = payload[source : source + row_size]

    return DecodedImage(
        width,
        height,
        bytes(output),
        "CS5i",
        {
            "region_width": region_width,
            "region_height": region_height,
            "stride": stride,
        },
    )


def identify_image_kind(data: bytes) -> str | None:
    """Identify supported image payloads using strict structural checks."""
    if data.startswith(b"BM"):
        return "BMP"
    if data.startswith(b"\x00\x00\x01\x00"):
        return "ICO"

    if len(data) >= 31:
        width, height = struct.unpack_from("<HH", data, 11)
        codec, planes = struct.unpack_from("<HH", data, 19)
        if (
            0 < width <= 0x7FFF
            and 0 < height <= 0x7FFF
            and width * height <= 100_000_000
            and codec == 1
            and planes == 1
            and _u32(data, 23) == len(data) - 31
        ):
            return "CRip"

    if len(data) >= 41:
        width, height = struct.unpack_from("<HH", data, 15)
        image_flags = _u32(data, 19)
        pixel_format = _u32(data, 26)
        channel_bits = (
            (pixel_format >> 8) & 0xF,
            (pixel_format >> 16) & 0xF,
            (pixel_format >> 24) & 0xF,
        )
        if (
            0 < width <= 0x7FFF
            and 0 < height <= 0x7FFF
            and width * height <= 100_000_000
            and image_flags & 0xFE == 2
            and min(channel_bits) > 0
            and data[30] in (4, 5)
            and _u16(data, 31) == 0
            and _u32(data, 33) == len(data) - 41
        ):
            return "CRip007"

    if len(data) >= 27:
        width, height = struct.unpack_from("<HH", data, 11)
        stride = _u32(data, 19)
        payload_size = _u32(data, 23)
        if (
            0 < width <= 0x7FFF
            and 0 < height <= 0x7FFF
            and width * height <= 100_000_000
            and stride >= width * 4
            and stride % 4 == 0
            and payload_size == stride * height
            and payload_size == len(data) - 27
        ):
            return "CS5i"
    return None


def _pillow_to_decoded(image: object, kind: str, **metadata: int) -> DecodedImage:
    try:
        rgba_image = image.convert("RGBA")
        width, height = rgba_image.size
        rgba = rgba_image.tobytes()
    except (AttributeError, OSError, ValueError) as error:
        raise RipFormatError(f"cannot decode {kind}: {error}") from error

    bgra = bytearray(rgba)
    for offset in range(0, len(bgra), 4):
        bgra[offset], bgra[offset + 2] = bgra[offset + 2], bgra[offset]
    return DecodedImage(width, height, bytes(bgra), kind, metadata)


def decode_images(
    path: Path,
    *,
    mask_mode: str = "labels",
) -> list[tuple[str, DecodedImage]]:
    """Decode every image frame stored in one extracted resource."""
    data = path.read_bytes()
    kind = identify_image_kind(data)
    if kind is None:
        return []
    if kind == "CRip":
        return [("", decode_crip(data, mask_mode=mask_mode))]
    if kind == "CRip007":
        return [("", decode_crip007(data))]
    if kind == "CS5i":
        return [("", decode_cs5i(data))]

    try:
        from PIL import Image
    except ImportError as error:
        raise RipFormatError(
            f"Pillow is required to decode standard {kind} resources"
        ) from error

    try:
        image = Image.open(path)
        if kind == "ICO":
            frames = []
            for index, entry in enumerate(image.ico.entry):
                frame = image.ico.frame(index)
                label = (
                    f"_ico_{index:02d}_{entry.width}x{entry.height}"
                    f"_{entry.color_depth}bpp"
                )
                frames.append(
                    (
                        label,
                        _pillow_to_decoded(
                            frame,
                            kind,
                            frame=index,
                            color_depth=entry.color_depth,
                        ),
                    )
                )
            return frames
        return [("", _pillow_to_decoded(image, kind))]
    except (OSError, ValueError, AttributeError) as error:
        raise RipFormatError(f"cannot decode {kind}: {error}") from error


def decode_file(path: Path, *, mask_mode: str = "labels") -> DecodedImage:
    images = decode_images(path, mask_mode=mask_mode)
    if not images:
        raise RipFormatError("resource is not a supported image")
    return images[0][1]


def write_png(path: Path, image: DecodedImage) -> None:
    def chunk(kind: bytes, payload: bytes) -> bytes:
        body = kind + payload
        return (
            struct.pack(">I", len(payload))
            + body
            + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)
        )

    rgba = bytearray(len(image.bgra))
    for offset in range(0, len(image.bgra), 4):
        rgba[offset] = image.bgra[offset + 2]
        rgba[offset + 1] = image.bgra[offset + 1]
        rgba[offset + 2] = image.bgra[offset]
        rgba[offset + 3] = image.bgra[offset + 3]

    raw = bytearray()
    stride = image.width * 4
    for y in range(image.height):
        raw.append(0)
        raw.extend(rgba[y * stride : (y + 1) * stride])

    header = struct.pack(
        ">IIBBBBB", image.width, image.height, 8, 6, 0, 0, 0
    )
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def _iter_inputs(paths: Iterable[Path]) -> Iterable[tuple[Path, Path]]:
    for path in paths:
        if path.is_dir():
            for child in sorted(path.rglob("*")):
                if child.is_file() and child.suffix.lower() != ".json":
                    yield child, child.relative_to(path)
        elif path.is_file():
            yield path, Path(path.name)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Convert extracted rUGP CRip/CRip007/CS5i objects to PNG."
    )
    parser.add_argument("paths", nargs="+", type=Path)
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=Path("rip_png"),
        help="output directory (default: rip_png)",
    )
    parser.add_argument(
        "--mask-mode",
        choices=("labels", "gray", "raw"),
        default="labels",
        help="CRip region-map rendering mode (default: labels)",
    )
    args = parser.parse_args()

    converted = 0
    failed = 0
    skipped = 0
    manifest_records = []
    for source, relative_source in _iter_inputs(args.paths):
        try:
            images = decode_images(source, mask_mode=args.mask_mode)
            if not images:
                skipped += 1
                continue
            for label, image in images:
                destination = (
                    args.output
                    / relative_source.parent
                    / f"{source.stem}{label}.png"
                )
                write_png(destination, image)
                details = ", ".join(
                    f"{key}={value}"
                    for key, value in image.metadata.items()
                )
                print(
                    f"OK {source} -> {destination} "
                    f"({image.kind} {image.width}x{image.height}"
                    f"{', ' + details if details else ''})"
                )
                manifest_records.append(
                    {
                        "source": relative_source.as_posix(),
                        "output": destination.relative_to(
                            args.output
                        ).as_posix(),
                        "kind": image.kind,
                        "width": image.width,
                        "height": image.height,
                        "metadata": image.metadata,
                    }
                )
                converted += 1
        except (OSError, RipFormatError) as error:
            print(f"ERROR {source}: {error}")
            failed += 1

    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "manifest.json").write_text(
        json.dumps(
            {
                "converted_image_count": converted,
                "failed_resource_count": failed,
                "skipped_non_image_count": skipped,
                "records": manifest_records,
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    print(
        f"Converted {converted} image(s); {skipped} non-image resources "
        f"skipped; {failed} failed."
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
