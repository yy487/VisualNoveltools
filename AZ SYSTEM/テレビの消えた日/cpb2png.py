# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import ctypes
import json
import os
import struct
import sys
import zlib
from pathlib import Path

from tvlost_arc_common import ARCHIVE_KEY, GRAPHIC_STREAM_KEY, crypt_chunk, parse_arc, save_manifest

# TYP1 24bpp stores three compressed channel blocks in this order.
# Header has four size dwords at 0x0A/0x0E/0x12/0x16; for 24bpp the engine reads
# sizes[3], sizes[2], sizes[1]. sizes[0] is duplicated/metadata in this path.
TYP1_24_STORED_ORDER = (3, 2, 1)  # B, G, R after inflate


def _load_interleave_lib() -> object | None:
    here = Path(__file__).resolve().parent
    names = []
    if os.name == "nt":
        names.append(here / "typ1_interleave.dll")
    else:
        names.append(here / "typ1_interleave.so")
    for p in names:
        if not p.exists():
            continue
        try:
            lib = ctypes.CDLL(str(p))
            lib.typ1_bgr_to_rgb24.argtypes = [
                ctypes.POINTER(ctypes.c_uint8), ctypes.POINTER(ctypes.c_uint8), ctypes.POINTER(ctypes.c_uint8),
                ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ]
            lib.typ1_bgr_to_rgb24.restype = None
            return lib
        except OSError:
            pass
    return None


def _inflate_game_block(block: bytes, expected_size: int) -> bytes:
    if len(block) < 5:
        raise ValueError("compressed block too small")
    stored_adler, = struct.unpack_from("<I", block, 0)
    zstream = block[4:]

    # TYP1 block prefix is NOT adler32(inflated_pixels).
    # It is adler32(the compressed zlib stream bytes after this dword).
    # The old build compared it with the inflated channel and produced false
    # failures such as bg015a.cpb / ev0007c.cpb / trueend.cpb.
    calc_payload_adler = zlib.adler32(zstream) & 0xFFFFFFFF
    if stored_adler != calc_payload_adler:
        raise ValueError(
            f"compressed payload adler mismatch: "
            f"stored=0x{stored_adler:08X}, calc=0x{calc_payload_adler:08X}"
        )

    out = zlib.decompress(zstream)
    if len(out) != expected_size:
        raise ValueError(f"inflate size mismatch: got={len(out)}, expected={expected_size}")
    return out


def decode_typ1_24_from_arc_stream(raw: bytes, base_file_pos: int, *, stream_key: int = GRAPHIC_STREAM_KEY,
                                   use_c: bool = True) -> tuple[bytes, int, int, dict]:
    """Decode one physical ARC entry stream to RGB bytes.

    raw must be the bytes read directly from graphic.arc at the entry offset, before applying
    the fread XOR. base_file_pos must be the real absolute file offset of that entry.
    """
    if len(raw) < 0x1E:
        raise ValueError("TYP1 stream too small")
    header = crypt_chunk(raw[:0x1E], stream_key, base_file_pos)
    if header[:4] != b"TYP1":
        raise ValueError(f"bad TYP1 magic after stream decrypt: {header[:8].hex()}")
    bpp = header[4]
    flag = header[5]
    width, height = struct.unpack_from("<HH", header, 6)
    sizes = struct.unpack_from("<IIII", header, 0x0A)
    if bpp != 24:
        raise NotImplementedError(f"this converter currently supports TYP1 24bpp only; got bpp={bpp}")
    pixels = width * height
    if pixels <= 0:
        raise ValueError(f"invalid image size: {width}x{height}")

    offset = 0x1E
    channels: list[bytes] = []
    block_info = []
    for size_index in TYP1_24_STORED_ORDER:
        comp_size = sizes[size_index]
        if comp_size <= 4 or offset + comp_size > len(raw):
            raise ValueError(f"invalid block size index={size_index} size={comp_size} offset=0x{offset:X}")
        enc_block = raw[offset:offset + comp_size]
        block = crypt_chunk(enc_block, stream_key, base_file_pos + offset)
        inflated = _inflate_game_block(block, pixels)
        channels.append(inflated)
        block_info.append({"size_index": size_index, "offset": offset, "compressed_size": comp_size})
        offset += comp_size

    b, g, r = channels
    rgb = bytearray(pixels * 3)
    lib = _load_interleave_lib() if use_c else None
    if lib is not None:
        arr_t = ctypes.c_uint8 * pixels
        rgb_t = ctypes.c_uint8 * (pixels * 3)
        b_arr = arr_t.from_buffer_copy(b)
        g_arr = arr_t.from_buffer_copy(g)
        r_arr = arr_t.from_buffer_copy(r)
        rgb_arr = rgb_t.from_buffer(rgb)
        lib.typ1_bgr_to_rgb24(b_arr, g_arr, r_arr, rgb_arr, pixels)
        accel = "c"
    else:
        for i in range(pixels):
            j = i * 3
            rgb[j + 0] = r[i]
            rgb[j + 1] = g[i]
            rgb[j + 2] = b[i]
        accel = "python"

    meta = {
        "magic": "TYP1",
        "bpp": bpp,
        "flag": flag,
        "width": width,
        "height": height,
        "sizes": list(sizes),
        "stored_order": list(TYP1_24_STORED_ORDER),
        "consumed_size": offset,
        "remaining_size": len(raw) - offset,
        "stream_key": f"0x{stream_key:08X}",
        "accelerator": accel,
        "blocks": block_info,
    }
    return bytes(rgb), width, height, meta


def save_png(rgb: bytes, width: int, height: int, output: Path) -> None:
    try:
        from PIL import Image
    except ImportError as e:
        raise RuntimeError("Pillow is required: pip install pillow") from e
    output.parent.mkdir(parents=True, exist_ok=True)
    Image.frombytes("RGB", (width, height), rgb).save(output)


def convert_from_arc(arc_path: Path, output_dir: Path, names: list[str] | None, *, all_files: bool,
                     archive_key: int, stream_key: int, use_c: bool) -> dict:
    data = arc_path.read_bytes()
    info, entries = parse_arc(data, archive_key)
    output_dir.mkdir(parents=True, exist_ok=True)
    save_manifest(output_dir / "_arc_manifest.json", info, entries)

    selected: list[dict]
    if all_files:
        selected = entries
    else:
        if not names:
            raise SystemExit("provide --names or --all")
        lookup = {e["name"].lower(): e for e in entries}
        selected = []
        for name in names:
            key = name.lower()
            if not key.endswith(".cpb"):
                key += ".cpb"
            if key not in lookup:
                raise SystemExit(f"entry not found in ARC: {name}")
            selected.append(lookup[key])

    results = []
    ok = failed = skipped = 0
    for e in selected:
        raw = data[e["offset"]:e["offset"] + e["size"]]
        out_png = output_dir / (Path(e["name"]).stem + ".png")
        try:
            rgb, w, h, meta = decode_typ1_24_from_arc_stream(raw, e["offset"], stream_key=stream_key, use_c=use_c)
            save_png(rgb, w, h, out_png)
            ok += 1
            result = {"name": e["name"], "status": "ok", "output": out_png.name, **meta}
            print(f"[ok] {e['name']} -> {out_png.name} {w}x{h} bpp={meta['bpp']} accel={meta['accelerator']}")
        except NotImplementedError as ex:
            skipped += 1
            result = {"name": e["name"], "status": "skipped", "reason": str(ex)}
            print(f"[skip] {e['name']}: {ex}")
        except Exception as ex:
            failed += 1
            result = {"name": e["name"], "status": "failed", "reason": str(ex)}
            print(f"[fail] {e['name']}: {ex}")
        results.append(result)
    summary = {"ok": ok, "skipped": skipped, "failed": failed, "results": results}
    (output_dir / "_cpb2png_report.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[done] ok={ok} skipped={skipped} failed={failed} output={output_dir}")
    return summary


def main() -> None:
    ap = argparse.ArgumentParser(description="Convert TVLost graphic.arc TYP1 24bpp CPB images to PNG")
    ap.add_argument("arc", help="input graphic.arc")
    ap.add_argument("output", help="output PNG directory")
    ap.add_argument("--names", nargs="*", help="specific CPB names, e.g. bg002b.cpb bg005b.cpb")
    ap.add_argument("--all", action="store_true", help="try all entries; unsupported bpp is skipped")
    ap.add_argument("--archive-key", type=lambda s: int(s, 0), default=ARCHIVE_KEY, help="ARC table/header key")
    ap.add_argument("--stream-key", type=lambda s: int(s, 0), default=GRAPHIC_STREAM_KEY, help="post-start graphic stream key")
    ap.add_argument("--no-c", action="store_true", help="disable C interleave accelerator")
    args = ap.parse_args()
    convert_from_arc(Path(args.arc), Path(args.output), args.names, all_files=args.all,
                     archive_key=args.archive_key, stream_key=args.stream_key, use_c=not args.no_c)


if __name__ == "__main__":
    main()
