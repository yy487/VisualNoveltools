"""
Convert extracted rUGP CRip+COptimizedObs resources to viewable images.

CRip files contain image metadata (dimensions + palette) but NOT pixel data.
COptimizedObs files contain compressed/encoded pixel data.
CBmp files are standard Windows BMPs that can be viewed directly.

Usage:
  python rip2png.py unpacked/螺旋_11.rio/          # convert all in directory
  python rip2png.py CRip_*.rip COptimizedObs_*.bin # specific files
"""
import struct, math, os, sys
from pathlib import Path
from collections import defaultdict
import zlib


def read_crip(path: str) -> dict | None:
    """Parse CRip header, extract palette."""
    with open(path, 'rb') as f:
        data = f.read()
    if len(data) < 32:
        return None

    magic = struct.unpack_from('<I', data, 0)[0]
    # Dimensions at offset 11,13 (WORDs)
    w = struct.unpack_from('<H', data, 11)[0] if len(data) > 13 else 0
    h = struct.unpack_from('<H', data, 13)[0] if len(data) > 15 else 0

    if w == 0 or h == 0 or w > 4096 or h > 4096:
        # Try offset 12,14 or other positions
        w = struct.unpack_from('<H', data, 12)[0] if len(data) > 14 else 0
        h = struct.unpack_from('<H', data, 14)[0] if len(data) > 16 else 0

    if w == 0 or h == 0 or w > 4096 or h > 4096:
        return None

    # Try to find palette
    palette = []
    # Look for the repeating RGBA pattern that starts at offset 0x20
    pal_start = 0x20
    pal_end = min(len(data), 0x420)  # up to 256*4 palette entries

    # Count unique bytes in 64-byte windows to find palette boundary
    for offset in range(0x20, min(len(data) - 64, 0x800), 0x10):
        chunk = data[offset:offset+64]
        unique = len(set(chunk))
        if unique > 20:
            pal_end = offset
            break

    pal_size = pal_end - pal_start
    num_colors = pal_size // 4
    if num_colors > 0:
        for i in range(num_colors):
            r = data[pal_start + i*4 + 2]  # BGRA → RGB
            g = data[pal_start + i*4 + 1]
            b = data[pal_start + i*4 + 0]
            palette.append((r, g, b))

    return {
        'width': w, 'height': h,
        'magic': magic,
        'palette': palette,
        'num_colors': num_colors,
        'pixel_offset': pal_end,
        'pixel_bytes': len(data) - pal_end,
    }


def read_cbmp(path: str) -> bytes | None:
    """Read a standard BMP file (CBmp resources are standard BMPs)."""
    with open(path, 'rb') as f:
        data = f.read()
    if data[:2] == b'BM':
        return data
    return None


def write_png(path: str, width: int, height: int, pixels: bytes, palette: list | None = None):
    """Write a PNG file from raw pixel data."""
    # Build PNG chunks
    def chunk(ctype: bytes, data: bytes) -> bytes:
        c = ctype + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)

    # IHDR
    color_type = 3 if palette else 2  # indexed or RGB
    bit_depth = 8
    ihdr = struct.pack('>IIBBBBB', width, height, bit_depth, color_type, 0, 0, 0)

    png = b'\x89PNG\r\n\x1a\n'
    png += chunk(b'IHDR', ihdr)

    # PLTE
    if palette:
        plte = b''.join(bytes(c) for c in palette)
        png += chunk(b'PLTE', plte)

    # IDAT - raw pixel data, filtered
    raw = b''
    row_size = width if palette else width * 3
    for y in range(height):
        raw += b'\x00'  # filter none
        row_start = y * row_size
        raw += pixels[row_start:row_start + row_size]

    png += chunk(b'IDAT', zlib.compress(raw))
    png += chunk(b'IEND', b'')

    with open(path, 'wb') as f:
        f.write(png)


def make_palette_png(crip_path: str, output_path: str):
    """Visualize CRip palette as a PNG swatch."""
    info = read_crip(crip_path)
    if not info or not info['palette']:
        print(f"  skip: no palette in {os.path.basename(crip_path)}")
        return False

    num = info['num_colors']
    cols = min(16, num)
    rows = (num + cols - 1) // cols
    swatch_w = cols * 20
    swatch_h = rows * 20
    pixels = bytearray()
    for r_idx in range(rows):
        for _ in range(20):  # height of each swatch
            for c_idx in range(cols):
                color_idx = r_idx * cols + c_idx
                for _ in range(20):  # width of each swatch
                    if color_idx < num:
                        pixels.append(color_idx)
                    else:
                        pixels.append(0)

    write_png(output_path, swatch_w, swatch_h, bytes(pixels), info['palette'])
    print(f"  -> {output_path} ({num} colors, {info['width']}x{info['height']})")
    return True


def make_bmp_png(bmp_path: str, output_path: str):
    """Convert standard BMP to PNG by extracting pixel data."""
    bmp_data = read_cbmp(bmp_path)
    if not bmp_data:
        return False

    # Parse BMP header
    pixel_offset = struct.unpack_from('<I', bmp_data, 10)[0]
    width = struct.unpack_from('<i', bmp_data, 18)[0]
    height = abs(struct.unpack_from('<i', bmp_data, 22)[0])
    bpp = struct.unpack_from('<H', bmp_data, 28)[0]
    compression = struct.unpack_from('<I', bmp_data, 30)[0]

    if compression != 0:  # only uncompressed
        return False

    # Extract palette for 8-bit
    palette = None
    if bpp == 8:
        num_colors = struct.unpack_from('<I', bmp_data, 46)[0]
        if num_colors == 0:
            num_colors = 256
        palette = []
        for i in range(num_colors):
            off = 54 + i * 4
            b = bmp_data[off]
            g = bmp_data[off + 1]
            r = bmp_data[off + 2]
            palette.append((r, g, b))

    # Extract pixels (BMP is bottom-up, flip it)
    row_size = ((width * bpp + 31) // 32) * 4
    pixels = bytearray()
    for y in range(height - 1, -1, -1):
        row_start = pixel_offset + y * row_size
        if bpp == 8:
            pixels.extend(bmp_data[row_start:row_start + width])
        elif bpp == 24:
            row = bmp_data[row_start:row_start + width * 3]
            # BGR → RGB
            for i in range(0, len(row), 3):
                pixels.extend([row[i+2], row[i+1], row[i]])
        elif bpp == 32:
            row = bmp_data[row_start:row_start + width * 4]
            # BGRA → RGBA
            for i in range(0, len(row), 4):
                pixels.extend([row[i+2], row[i+1], row[i]])

    write_png(output_path, width, height, bytes(pixels), palette)
    print(f"  -> {output_path} ({width}x{height}, {bpp}bpp)")
    return True


def make_raw_dump_png(path: str, output_path: str, width: int = 640, height: int = 480):
    """Dump raw bytes as BGRA32 image - for quick visual check of COptimizedObs."""
    with open(path, 'rb') as f:
        data = f.read()

    # Skip MFC header (8 bytes) if present
    tag = struct.unpack_from('<H', data, 0)[0]
    schema = struct.unpack_from('<H', data, 2)[0]
    offset = 8 if (tag < 0x8000 and schema < 100) else 0
    pixels = data[offset:offset + width * height * 4]

    if len(pixels) < width * height * 4:
        pixels = pixels + b'\x00' * (width * height * 4 - len(pixels))

    # BGRA → RGBA
    rgb = bytearray()
    for i in range(0, len(pixels), 4):
        if i + 3 < len(pixels):
            rgb.extend([pixels[i+2], pixels[i+1], pixels[i], pixels[i+3]])

    write_png(output_path, width, height, bytes(rgb))
    print(f"  -> {output_path} ({width}x{height} raw dump)")
    return True


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Convert rUGP resources to PNG")
    parser.add_argument("paths", nargs="*", help="Files or directories to convert")
    parser.add_argument("-o", "--output", default="png_output", help="Output directory")
    args = parser.parse_args()

    out = Path(args.output)
    out.mkdir(parents=True, exist_ok=True)

    targets = []
    for p in args.paths:
        pp = Path(p)
        if pp.is_dir():
            targets.extend(pp.rglob("*"))
        else:
            targets.append(pp)

    if not targets:
        # Default: scan current dir
        targets = list(Path(".").rglob("*.rip")) + list(Path(".").rglob("*.bin")) + list(Path(".").rglob("*.bmp"))

    stats = {'palette': 0, 'bmp': 0, 'raw': 0}

    for f in sorted(targets):
        if not f.is_file():
            continue
        name = f.stem
        ext = f.suffix.lower()

        if 'rip' in ext or 'CRip' in str(f):
            png_path = out / f"{name}_palette.png"
            if make_palette_png(str(f), str(png_path)):
                stats['palette'] += 1

        elif 'bmp' in ext or 'CBmp' in str(f) or 'Bmp' in str(f):
            png_path = out / f"{name}.png"
            if make_bmp_png(str(f), str(png_path)):
                stats['bmp'] += 1

        elif 'optimizedobs' in str(f).lower() or 'COptimizedObs' in str(f):
            png_path = out / f"{name}_raw.png"
            if make_raw_dump_png(str(f), str(png_path)):
                stats['raw'] += 1

    print(f"\nDone: {stats['palette']} palettes, {stats['bmp']} BMPs, {stats['raw']} raw dumps")


if __name__ == "__main__":
    main()
