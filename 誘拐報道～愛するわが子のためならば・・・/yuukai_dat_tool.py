#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
yuukai_dat_tool.py

Unpack / repack system.dat-like Adobe AIR resource bundle used by this sample.

Observed format:
  [4-byte big-endian catalog compressed-stream offset]
  [raw-deflate stream 0]
  [raw-deflate stream 1]
  ...
  [raw-deflate catalog stream]

The last stream decompresses to an AMF3 dynamic object:
  Object(dynamic) { filename: [compressed_offset, compressed_size], ... }

Offsets/sizes in the catalog refer to compressed stream positions inside the .dat.
File payloads are the decompressed stream contents. Zero-size catalog entries are
represented as empty files and do not consume a compressed stream.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
import zlib
from dataclasses import dataclass, asdict
from pathlib import Path, PurePosixPath
from typing import Any, Iterable

PREFIX_SIZE = 4
RAW_WBITS = -15


def encode_catalog_offset(offset: int) -> bytes:
    return int(offset).to_bytes(PREFIX_SIZE, "big")


def decode_catalog_offset(prefix: bytes) -> int:
    if len(prefix) != PREFIX_SIZE:
        raise DatFormatError(f"catalog offset prefix must be {PREFIX_SIZE} bytes")
    return int.from_bytes(prefix, "big")
DEFAULT_LEVEL = 9
MANIFEST_NAME = "_manifest.json"
CATALOG_DEBUG_NAME = "_catalog.amf3.bin"
ORPHAN_DIR_NAME = "_orphan_streams"


class DatFormatError(RuntimeError):
    pass


@dataclass
class StreamInfo:
    index: int
    offset: int
    comp_size: int
    unpacked_size: int
    sha256: str
    data: bytes | None = None


@dataclass
class CatalogEntry:
    name: str
    offset: int
    comp_size: int
    order: int
    stream_index: int | None = None
    unpacked_size: int = 0
    sha256: str = ""
    zero_size: bool = False


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def raw_deflate_decompress_stream(data: bytes, offset: int) -> tuple[bytes, int]:
    obj = zlib.decompressobj(RAW_WBITS)
    try:
        out = obj.decompress(data[offset:]) + obj.flush()
    except zlib.error as e:
        raise DatFormatError(f"raw-deflate decompress failed at 0x{offset:X}: {e}") from e
    consumed = len(data[offset:]) - len(obj.unused_data)
    if not obj.eof or consumed <= 0:
        raise DatFormatError(f"unterminated raw-deflate stream at 0x{offset:X}")
    return out, consumed


def raw_deflate_compress(data: bytes, level: int = DEFAULT_LEVEL) -> bytes:
    co = zlib.compressobj(level=level, wbits=RAW_WBITS)
    return co.compress(data) + co.flush()


def parse_streams(dat_bytes: bytes, keep_data: bool = True) -> tuple[bytes, list[StreamInfo]]:
    if len(dat_bytes) < PREFIX_SIZE:
        raise DatFormatError("file is too small")
    prefix = dat_bytes[:PREFIX_SIZE]
    pos = PREFIX_SIZE
    streams: list[StreamInfo] = []
    index = 0
    while pos < len(dat_bytes):
        out, comp_size = raw_deflate_decompress_stream(dat_bytes, pos)
        streams.append(
            StreamInfo(
                index=index,
                offset=pos,
                comp_size=comp_size,
                unpacked_size=len(out),
                sha256=sha256_hex(out),
                data=out if keep_data else None,
            )
        )
        pos += comp_size
        index += 1
    if pos != len(dat_bytes):
        raise DatFormatError(f"stream parse ended at 0x{pos:X}, file size=0x{len(dat_bytes):X}")
    if not streams:
        raise DatFormatError("no deflate streams found")
    return prefix, streams


# ---- AMF3 subset: dynamic object { name: [int, int], ... } ----

def read_u29(buf: bytes, pos: int) -> tuple[int, int]:
    value = 0
    for i in range(4):
        if pos >= len(buf):
            raise DatFormatError("unexpected EOF while reading AMF3 U29")
        b = buf[pos]
        pos += 1
        if i < 3:
            value = (value << 7) | (b & 0x7F)
            if (b & 0x80) == 0:
                return value, pos
        else:
            value = (value << 8) | b
            return value, pos
    return value, pos


def write_u29(value: int) -> bytes:
    if not (0 <= value <= 0x1FFFFFFF):
        raise DatFormatError(f"AMF3 U29 out of range: {value}")
    if value < 0x80:
        return bytes([value])
    if value < 0x4000:
        return bytes([((value >> 7) & 0x7F) | 0x80, value & 0x7F])
    if value < 0x200000:
        return bytes([((value >> 14) & 0x7F) | 0x80, ((value >> 7) & 0x7F) | 0x80, value & 0x7F])
    return bytes([
        ((value >> 22) & 0x7F) | 0x80,
        ((value >> 15) & 0x7F) | 0x80,
        ((value >> 8) & 0x7F) | 0x80,
        value & 0xFF,
    ])


def read_amf3_inline_string(buf: bytes, pos: int) -> tuple[str, int]:
    header, pos = read_u29(buf, pos)
    if (header & 1) == 0:
        raise DatFormatError(f"AMF3 string reference is not supported in catalog at 0x{pos:X}")
    size = header >> 1
    end = pos + size
    if end > len(buf):
        raise DatFormatError("AMF3 string exceeds catalog size")
    try:
        text = buf[pos:end].decode("utf-8")
    except UnicodeDecodeError as e:
        raise DatFormatError(f"catalog filename is not valid UTF-8 at 0x{pos:X}: {e}") from e
    return text, end


def write_amf3_inline_string(text: str) -> bytes:
    raw = text.encode("utf-8")
    return write_u29((len(raw) << 1) | 1) + raw


def parse_catalog(catalog: bytes) -> list[CatalogEntry]:
    pos = 0
    if pos >= len(catalog) or catalog[pos] != 0x0A:
        raise DatFormatError("catalog is not an AMF3 object: missing marker 0x0A")
    pos += 1
    trait, pos = read_u29(catalog, pos)
    # Observed: 0x0B = inline dynamic anonymous object, no sealed properties.
    if trait != 0x0B:
        raise DatFormatError(f"unsupported AMF3 object trait 0x{trait:X}; expected 0x0B")
    class_name, pos = read_amf3_inline_string(catalog, pos)
    if class_name != "":
        raise DatFormatError(f"unsupported catalog class name: {class_name!r}")

    entries: list[CatalogEntry] = []
    order = 0
    while pos < len(catalog):
        name, pos = read_amf3_inline_string(catalog, pos)
        if name == "":
            if pos != len(catalog):
                raise DatFormatError(f"trailing bytes after AMF3 dynamic-object terminator: {len(catalog) - pos}")
            return entries

        if pos >= len(catalog) or catalog[pos] != 0x09:
            raise DatFormatError(f"catalog value for {name!r} is not AMF3 array at 0x{pos:X}")
        pos += 1
        arr_header, pos = read_u29(catalog, pos)
        if (arr_header & 1) == 0:
            raise DatFormatError(f"array reference is not supported for {name!r}")
        dense_count = arr_header >> 1
        if dense_count != 2:
            raise DatFormatError(f"catalog value for {name!r} has {dense_count} items, expected 2")

        assoc_end, pos = read_amf3_inline_string(catalog, pos)
        if assoc_end != "":
            raise DatFormatError(f"associative array part is not empty for {name!r}")

        vals: list[int] = []
        for _ in range(2):
            if pos >= len(catalog) or catalog[pos] != 0x04:
                raise DatFormatError(f"catalog value for {name!r} is not AMF3 integer at 0x{pos:X}")
            pos += 1
            value, pos = read_u29(catalog, pos)
            vals.append(value)

        entries.append(CatalogEntry(name=name, offset=vals[0], comp_size=vals[1], order=order, zero_size=(vals[1] == 0)))
        order += 1

    raise DatFormatError("catalog dynamic object is missing terminator")


def build_catalog(entries: Iterable[CatalogEntry]) -> bytes:
    out = bytearray()
    out.append(0x0A)              # AMF3 object marker
    out += write_u29(0x0B)        # inline dynamic anonymous object, no sealed fields
    out += write_amf3_inline_string("")
    for e in entries:
        out += write_amf3_inline_string(e.name)
        out.append(0x09)          # AMF3 array marker
        out += write_u29((2 << 1) | 1)
        out += write_amf3_inline_string("")  # end associative part
        out.append(0x04)          # integer marker
        out += write_u29(e.offset)
        out.append(0x04)
        out += write_u29(e.comp_size)
    out += write_amf3_inline_string("")      # dynamic object terminator
    return bytes(out)


def safe_output_path(base: Path, name: str) -> Path:
    # Treat catalog names as relative POSIX-like paths if a future sample uses folders.
    rel = PurePosixPath(name)
    if rel.is_absolute() or any(part in ("", ".", "..") for part in rel.parts):
        raise DatFormatError(f"unsafe catalog filename: {name!r}")
    return base.joinpath(*rel.parts)


def load_manifest(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict) or "entries" not in data:
        raise DatFormatError(f"invalid manifest: {path}")
    return data


def write_manifest(path: Path, manifest: dict[str, Any]) -> None:
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8", newline="\n")


def attach_catalog_to_streams(streams: list[StreamInfo], entries: list[CatalogEntry]) -> None:
    by_offset: dict[tuple[int, int], StreamInfo] = {(s.offset, s.comp_size): s for s in streams[:-1]}
    by_offset_any: dict[int, StreamInfo] = {s.offset: s for s in streams[:-1]}
    missing: list[str] = []
    for e in entries:
        if e.comp_size == 0:
            e.zero_size = True
            e.stream_index = None
            e.unpacked_size = 0
            e.sha256 = sha256_hex(b"")
            continue
        s = by_offset.get((e.offset, e.comp_size))
        if s is None:
            near = by_offset_any.get(e.offset)
            if near is not None:
                missing.append(f"{e.name}: offset 0x{e.offset:X} exists but comp_size catalog={e.comp_size}, stream={near.comp_size}")
            else:
                missing.append(f"{e.name}: missing stream offset=0x{e.offset:X}, comp_size={e.comp_size}")
            continue
        e.stream_index = s.index
        e.unpacked_size = s.unpacked_size
        e.sha256 = s.sha256
    if missing:
        joined = "\n  ".join(missing[:20])
        more = "" if len(missing) <= 20 else f"\n  ... {len(missing) - 20} more"
        raise DatFormatError(f"catalog/stream mismatch:\n  {joined}{more}")


def unpack_dat(dat_path: Path, out_dir: Path, *, write_catalog_debug: bool = True) -> None:
    data = dat_path.read_bytes()
    prefix, streams = parse_streams(data, keep_data=True)
    catalog_stream = streams[-1]
    assert catalog_stream.data is not None
    entries = parse_catalog(catalog_stream.data)
    attach_catalog_to_streams(streams, entries)

    out_dir.mkdir(parents=True, exist_ok=True)
    files_dir = out_dir / "files"
    files_dir.mkdir(parents=True, exist_ok=True)

    by_stream_index = {s.index: s for s in streams[:-1]}
    for e in entries:
        dst = safe_output_path(files_dir, e.name)
        dst.parent.mkdir(parents=True, exist_ok=True)
        payload = b""
        if e.stream_index is not None:
            stream = by_stream_index[e.stream_index]
            if stream.data is None:
                raise AssertionError("stream data was not kept")
            payload = stream.data
        dst.write_bytes(payload)

    if write_catalog_debug:
        (out_dir / CATALOG_DEBUG_NAME).write_bytes(catalog_stream.data)

    used_offsets = {e.offset for e in entries if e.comp_size != 0}
    orphan_streams = [s for s in streams[:-1] if s.offset not in used_offsets]
    orphan_dir = out_dir / ORPHAN_DIR_NAME
    if orphan_streams:
        orphan_dir.mkdir(parents=True, exist_ok=True)
        for s in orphan_streams:
            if s.data is None:
                raise AssertionError("orphan stream data was not kept")
            (orphan_dir / f"stream_{s.index:03d}.bin").write_bytes(s.data)

    manifest = {
        "format": "yuukai-system-dat-raw-deflate-amf3-catalog-v1",
        "source": str(dat_path),
        "prefix_hex": prefix.hex(),
        "catalog_offset_prefix": decode_catalog_offset(prefix),
        "stream_count_total": len(streams),
        "file_count": len(entries),
        "catalog_stream": {
            "index": catalog_stream.index,
            "offset": catalog_stream.offset,
            "comp_size": catalog_stream.comp_size,
            "unpacked_size": catalog_stream.unpacked_size,
            "sha256": catalog_stream.sha256,
        },
        "orphan_streams": [
            {
                "index": s.index,
                "offset": s.offset,
                "comp_size": s.comp_size,
                "unpacked_size": s.unpacked_size,
                "sha256": s.sha256,
                "path": f"{ORPHAN_DIR_NAME}/stream_{s.index:03d}.bin",
            }
            for s in orphan_streams
        ],
        "entries": [asdict(e) for e in entries],
    }
    write_manifest(out_dir / MANIFEST_NAME, manifest)

    print(f"[unpack] input={dat_path}")
    print(f"[unpack] streams={len(streams)} files={len(entries)} zero_size={sum(1 for e in entries if e.zero_size)}")
    print(f"[unpack] prefix_catalog_offset=0x{decode_catalog_offset(prefix):X} catalog_offset=0x{catalog_stream.offset:X} catalog_comp={catalog_stream.comp_size}")
    print(f"[unpack] output={out_dir}")


def compute_zero_offsets(entries: list[CatalogEntry], new_offset_by_order: dict[int, int], catalog_offset: int) -> None:
    # For an empty entry, original sample points it to the next real stream offset.
    # Preserve that behavior after rebuilding: choose the first non-zero entry whose
    # old offset is >= the zero entry's old offset; fallback to catalog offset.
    nonzero = sorted((e.offset, e.order) for e in entries if e.comp_size != 0)
    for e in entries:
        if not e.zero_size:
            continue
        chosen = catalog_offset
        for old_off, order in nonzero:
            if old_off >= e.offset:
                chosen = new_offset_by_order[order]
                break
        e.offset = chosen
        e.comp_size = 0
        e.stream_index = None
        e.unpacked_size = 0
        e.sha256 = sha256_hex(b"")


def pack_dat(in_dir: Path, out_dat: Path, *, level: int = DEFAULT_LEVEL, keep_manifest_prefix: bool = True) -> None:
    manifest_path = in_dir / MANIFEST_NAME
    if not manifest_path.exists():
        raise DatFormatError(f"missing manifest: {manifest_path}; run unpack first")
    manifest = load_manifest(manifest_path)
    files_dir = in_dir / "files"
    if not files_dir.is_dir():
        raise DatFormatError(f"missing files directory: {files_dir}")

    # The first four bytes are not a key/random salt: they are the catalog stream
    # offset encoded as big-endian u32.  Recompute it on pack because edited files
    # change compressed stream sizes and therefore move the catalog.
    prefix_hex = manifest.get("prefix_hex", "")
    if prefix_hex:
        try:
            old_prefix = bytes.fromhex(prefix_hex)
        except ValueError as e:
            raise DatFormatError("manifest prefix_hex is invalid") from e
        if len(old_prefix) != PREFIX_SIZE:
            raise DatFormatError(f"manifest prefix must be {PREFIX_SIZE} bytes")

    entries: list[CatalogEntry] = []
    for raw in manifest["entries"]:
        e = CatalogEntry(
            name=raw["name"],
            offset=int(raw["offset"]),
            comp_size=int(raw["comp_size"]),
            order=int(raw["order"]),
            stream_index=raw.get("stream_index"),
            unpacked_size=int(raw.get("unpacked_size", 0)),
            sha256=str(raw.get("sha256", "")),
            zero_size=bool(raw.get("zero_size", False)) or int(raw.get("comp_size", 0)) == 0,
        )
        entries.append(e)
    entries.sort(key=lambda x: x.order)

    out = bytearray(b"\x00" * PREFIX_SIZE)
    new_offset_by_order: dict[int, int] = {}

    # Rebuild every physical stream before the catalog.  Most streams are named
    # by catalog entries, but the sample also contains one orphan duplicate mp3
    # stream.  Keeping orphan streams is necessary for byte-exact roundtrip.
    stream_items: list[tuple[int, int, str, CatalogEntry | dict[str, Any], Path]] = []
    for e in entries:
        if e.zero_size:
            continue
        stream_items.append((e.offset, e.order, "entry", e, safe_output_path(files_dir, e.name)))

    for raw in manifest.get("orphan_streams", []):
        old_off = int(raw["offset"])
        old_index = int(raw.get("index", 0))
        rel = str(raw.get("path", f"{ORPHAN_DIR_NAME}/stream_{old_index:03d}.bin"))
        src = in_dir / rel
        stream_items.append((old_off, 10_000_000 + old_index, "orphan", raw, src))

    stream_items.sort(key=lambda x: (x[0], x[1]))
    for _old_off, _ord, kind, obj, src in stream_items:
        if not src.exists():
            raise DatFormatError(f"missing payload file: {src}")
        payload = src.read_bytes()
        comp = raw_deflate_compress(payload, level=level)
        new_off = len(out)
        out += comp
        if kind == "entry":
            e = obj  # type: ignore[assignment]
            assert isinstance(e, CatalogEntry)
            e.offset = new_off
            e.comp_size = len(comp)
            e.unpacked_size = len(payload)
            e.sha256 = sha256_hex(payload)
            new_offset_by_order[e.order] = e.offset

    catalog_offset = len(out)
    compute_zero_offsets(entries, new_offset_by_order, catalog_offset)

    # Restore catalog order before encoding.
    entries.sort(key=lambda x: x.order)
    catalog = build_catalog(entries)
    out[:PREFIX_SIZE] = encode_catalog_offset(catalog_offset)
    out += raw_deflate_compress(catalog, level=level)

    out_dat.parent.mkdir(parents=True, exist_ok=True)
    out_dat.write_bytes(bytes(out))

    print(f"[pack] input={in_dir}")
    print(f"[pack] files={len(entries)} zero_size={sum(1 for e in entries if e.zero_size)} level={level}")
    print(f"[pack] catalog_offset=0x{catalog_offset:X} catalog_unpacked={len(catalog)}")
    print(f"[pack] output={out_dat} size={len(out)}")


def list_dat(dat_path: Path, *, limit: int | None = None) -> None:
    data = dat_path.read_bytes()
    prefix, streams = parse_streams(data, keep_data=True)
    catalog = streams[-1].data
    assert catalog is not None
    entries = parse_catalog(catalog)
    attach_catalog_to_streams(streams, entries)
    prefix_catalog_offset = decode_catalog_offset(prefix)
    marker = "ok" if prefix_catalog_offset == streams[-1].offset else "mismatch"
    print(f"prefix={prefix.hex()} prefix_catalog_offset=0x{prefix_catalog_offset:X}({marker}) streams={len(streams)} files={len(entries)} catalog_offset=0x{streams[-1].offset:X}")
    rows = entries if limit is None else entries[:limit]
    for e in rows:
        print(f"{e.order:03d} off=0x{e.offset:08X} comp={e.comp_size:8d} unpacked={e.unpacked_size:8d} {e.name}")
    if limit is not None and len(entries) > limit:
        print(f"... {len(entries) - limit} more")


def verify_dat(dat_path: Path) -> None:
    data = dat_path.read_bytes()
    prefix, streams = parse_streams(data, keep_data=True)
    catalog = streams[-1].data
    assert catalog is not None
    entries = parse_catalog(catalog)
    attach_catalog_to_streams(streams, entries)
    named_offsets = {e.offset for e in entries if e.comp_size != 0}
    orphan_streams = [s for s in streams[:-1] if s.offset not in named_offsets]
    print(f"[verify] ok input={dat_path}")
    prefix_catalog_offset = decode_catalog_offset(prefix)
    print(f"[verify] prefix={prefix.hex()} prefix_catalog_offset=0x{prefix_catalog_offset:X} streams={len(streams)} files={len(entries)} zero_size={sum(e.zero_size for e in entries)}")
    print(f"[verify] orphan_streams={len(orphan_streams)} catalog_offset=0x{streams[-1].offset:X}")
    if prefix_catalog_offset != streams[-1].offset:
        raise DatFormatError(f"catalog offset prefix mismatch: prefix=0x{prefix_catalog_offset:X}, actual=0x{streams[-1].offset:X}")


def roundtrip_test(dat_path: Path, work_dir: Path) -> None:
    import shutil
    if work_dir.exists():
        shutil.rmtree(work_dir)
    unpack_dir = work_dir / "unpacked"
    rebuilt = work_dir / "rebuilt.dat"
    unpack_dat(dat_path, unpack_dir, write_catalog_debug=True)
    pack_dat(unpack_dir, rebuilt, level=DEFAULT_LEVEL)
    a = dat_path.read_bytes()
    b = rebuilt.read_bytes()
    print(f"[roundtrip] original_sha256={sha256_hex(a)}")
    print(f"[roundtrip] rebuilt_sha256 ={sha256_hex(b)}")
    print(f"[roundtrip] byte_exact={a == b}")
    if a != b:
        first = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), None)
        print(f"[roundtrip] first_diff={first} original_size={len(a)} rebuilt_size={len(b)}")
        raise SystemExit(1)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Unpack/repack Yuukai system.dat raw-deflate AMF3 resource bundle")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_unpack = sub.add_parser("unpack", help="unpack .dat to directory")
    p_unpack.add_argument("dat", type=Path)
    p_unpack.add_argument("out_dir", type=Path)
    p_unpack.add_argument("--no-catalog-debug", action="store_true")

    p_pack = sub.add_parser("pack", help="repack directory created by unpack")
    p_pack.add_argument("in_dir", type=Path)
    p_pack.add_argument("out_dat", type=Path)
    p_pack.add_argument("--level", type=int, default=DEFAULT_LEVEL, choices=range(0, 10), metavar="0..9")

    p_list = sub.add_parser("list", help="list catalog entries")
    p_list.add_argument("dat", type=Path)
    p_list.add_argument("--limit", type=int, default=None)

    p_verify = sub.add_parser("verify", help="validate streams and catalog")
    p_verify.add_argument("dat", type=Path)

    p_rt = sub.add_parser("roundtrip", help="unpack+pack and require byte-exact output")
    p_rt.add_argument("dat", type=Path)
    p_rt.add_argument("work_dir", type=Path)

    args = parser.parse_args(argv)
    try:
        if args.cmd == "unpack":
            unpack_dat(args.dat, args.out_dir, write_catalog_debug=not args.no_catalog_debug)
        elif args.cmd == "pack":
            pack_dat(args.in_dir, args.out_dat, level=args.level)
        elif args.cmd == "list":
            list_dat(args.dat, limit=args.limit)
        elif args.cmd == "verify":
            verify_dat(args.dat)
        elif args.cmd == "roundtrip":
            roundtrip_test(args.dat, args.work_dir)
        else:
            parser.error("unknown command")
        return 0
    except DatFormatError as e:
        print(f"[error] {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
