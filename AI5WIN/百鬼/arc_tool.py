# -*- coding: utf-8 -*-
"""Baigui / AI5WIN ARC archive unpacker and repacker.

Format confirmed from ai5win.exe archive-class functions and mes.arc sample:
    u32le file_count
    repeated file_count times:
        name[20]  : each byte XOR 0x03, zero padding encoded as 0x03
        size_enc  : u32le = packed_size ^ 0x56428101
        off_enc   : u32le = data_offset ^ 0x32388531
    data blobs concatenated at offset 4 + file_count * 28

The data blob for .MES entries is the original compressed .MES stream. This tool
extracts/rebuilds the ARC container; it does not LZSS-decompress individual MES
files unless --plain-manifest inspection is added externally.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import struct
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Iterable

ENTRY_SIZE = 28
NAME_SIZE = 20
NAME_XOR = 0x03
SIZE_XOR = 0x56428101
OFFSET_XOR = 0x32388531
MANIFEST_NAME = "arc_manifest.json"


@dataclass
class ArcEntry:
    index: int
    name: str
    offset: int
    size: int
    sha256: str = ""


def read_u32le(data: bytes, off: int) -> int:
    if off + 4 > len(data):
        raise ValueError(f"unexpected EOF reading u32 at 0x{off:X}")
    return struct.unpack_from("<I", data, off)[0]


def encode_name(name: str) -> bytes:
    raw = name.encode("ascii")
    if len(raw) >= NAME_SIZE:
        raise ValueError(f"archive name too long, max {NAME_SIZE - 1} bytes: {name!r}")
    raw = raw + b"\x00" * (NAME_SIZE - len(raw))
    return bytes(b ^ NAME_XOR for b in raw)


def decode_name(raw: bytes) -> str:
    if len(raw) != NAME_SIZE:
        raise ValueError("bad encoded name size")
    dec = bytes(b ^ NAME_XOR for b in raw)
    name = dec.split(b"\x00", 1)[0]
    return name.decode("ascii")


def parse_arc(path: str | Path) -> tuple[bytes, list[ArcEntry]]:
    path = Path(path)
    data = path.read_bytes()
    if len(data) < 4:
        raise ValueError(f"too small archive: {path}")
    count = read_u32le(data, 0)
    header_size = 4 + count * ENTRY_SIZE
    if header_size > len(data):
        raise ValueError(f"invalid header: count={count}, header_size=0x{header_size:X}, file_size=0x{len(data):X}")

    entries: list[ArcEntry] = []
    seen: set[str] = set()
    for i in range(count):
        pos = 4 + i * ENTRY_SIZE
        name = decode_name(data[pos:pos + NAME_SIZE])
        size = read_u32le(data, pos + NAME_SIZE) ^ SIZE_XOR
        offset = read_u32le(data, pos + NAME_SIZE + 4) ^ OFFSET_XOR
        if not name:
            raise ValueError(f"empty name at entry {i}")
        if name in seen:
            raise ValueError(f"duplicate name at entry {i}: {name}")
        seen.add(name)
        if offset < header_size or offset + size > len(data):
            raise ValueError(
                f"entry out of range index={i} name={name} off=0x{offset:X} size=0x{size:X} file_size=0x{len(data):X}"
            )
        blob = data[offset:offset + size]
        entries.append(ArcEntry(i, name, offset, size, hashlib.sha256(blob).hexdigest()))

    # The official archives normally store data blobs sequentially in table order.
    # We do not require it for extraction, but warn during verification/listing.
    return data, entries


def safe_output_path(root: Path, name: str) -> Path:
    # Archive names are ASCII basenames in the sample. Still protect against traversal.
    p = root / name.replace("\\", "/")
    rp = p.resolve()
    rr = root.resolve()
    if rr not in rp.parents and rp != rr:
        raise ValueError(f"unsafe archive path: {name!r}")
    return p


def unpack_arc(arc_path: str | Path, out_dir: str | Path, overwrite: bool = False) -> None:
    arc_path = Path(arc_path)
    out_dir = Path(out_dir)
    data, entries = parse_arc(arc_path)
    out_dir.mkdir(parents=True, exist_ok=True)

    for e in entries:
        out_path = safe_output_path(out_dir, e.name)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        if out_path.exists() and not overwrite:
            raise FileExistsError(f"refuse to overwrite existing file: {out_path}; use --overwrite")
        out_path.write_bytes(data[e.offset:e.offset + e.size])

    manifest = {
        "format": "baigui_ai5win_arc",
        "source": arc_path.name,
        "entry_size": ENTRY_SIZE,
        "name_size": NAME_SIZE,
        "name_xor": NAME_XOR,
        "size_xor": SIZE_XOR,
        "offset_xor": OFFSET_XOR,
        "count": len(entries),
        "entries": [asdict(e) for e in entries],
    }
    (out_dir / MANIFEST_NAME).write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[unpack] {arc_path} -> {out_dir}")
    print(f"[unpack] files={len(entries)} manifest={out_dir / MANIFEST_NAME}")


def load_manifest(path: str | Path) -> list[ArcEntry]:
    path = Path(path)
    obj = json.loads(path.read_text(encoding="utf-8"))
    if obj.get("format") != "baigui_ai5win_arc":
        raise ValueError(f"not a baigui arc manifest: {path}")
    entries = []
    for item in obj.get("entries", []):
        entries.append(ArcEntry(
            index=int(item["index"]),
            name=str(item["name"]),
            offset=int(item.get("offset", 0)),
            size=int(item.get("size", 0)),
            sha256=str(item.get("sha256", "")),
        ))
    entries.sort(key=lambda e: e.index)
    return entries


def discover_entries(input_dir: Path) -> list[ArcEntry]:
    files = []
    for p in input_dir.iterdir():
        if p.is_file() and p.name != MANIFEST_NAME:
            files.append(p.name)
    files.sort(key=lambda s: s.lower())
    return [ArcEntry(i, name, 0, 0, "") for i, name in enumerate(files)]


def pack_arc(input_dir: str | Path, out_arc: str | Path, manifest_path: str | Path | None = None) -> None:
    input_dir = Path(input_dir)
    out_arc = Path(out_arc)
    if not input_dir.is_dir():
        raise NotADirectoryError(input_dir)
    if manifest_path is None:
        default_manifest = input_dir / MANIFEST_NAME
        entries = load_manifest(default_manifest) if default_manifest.exists() else discover_entries(input_dir)
    else:
        entries = load_manifest(manifest_path)

    count = len(entries)
    header_size = 4 + count * ENTRY_SIZE
    offset = header_size
    blobs: list[bytes] = []
    packed_entries: list[ArcEntry] = []

    for i, e in enumerate(entries):
        in_path = safe_output_path(input_dir, e.name)
        if not in_path.is_file():
            raise FileNotFoundError(f"missing entry file: {in_path}")
        blob = in_path.read_bytes()
        blobs.append(blob)
        packed_entries.append(ArcEntry(i, e.name, offset, len(blob), hashlib.sha256(blob).hexdigest()))
        offset += len(blob)

    out = bytearray()
    out += struct.pack("<I", count)
    for e in packed_entries:
        out += encode_name(e.name)
        out += struct.pack("<I", e.size ^ SIZE_XOR)
        out += struct.pack("<I", e.offset ^ OFFSET_XOR)
    for blob in blobs:
        out += blob

    out_arc.parent.mkdir(parents=True, exist_ok=True)
    out_arc.write_bytes(out)
    print(f"[pack] {input_dir} -> {out_arc}")
    print(f"[pack] files={count} size={len(out)}")


def list_arc(arc_path: str | Path, json_out: bool = False) -> None:
    data, entries = parse_arc(arc_path)
    if json_out:
        print(json.dumps([asdict(e) for e in entries], ensure_ascii=False, indent=2))
        return
    print(f"archive: {arc_path}")
    print(f"files  : {len(entries)}")
    print(f"size   : {len(data)}")
    prev_end = 4 + len(entries) * ENTRY_SIZE
    sequential = True
    for e in entries:
        if e.offset != prev_end:
            sequential = False
        prev_end = e.offset + e.size
    print(f"layout : {'sequential' if sequential else 'non-sequential'}")
    for e in entries[:20]:
        print(f"{e.index:04d} off=0x{e.offset:08X} size=0x{e.size:08X} {e.name}")
    if len(entries) > 20:
        print(f"... {len(entries)-20} more")


def verify_roundtrip(arc_path: str | Path) -> None:
    import tempfile
    import filecmp
    arc_path = Path(arc_path)
    original = arc_path.read_bytes()
    with tempfile.TemporaryDirectory(prefix="baigui_arc_rt_") as td:
        td = Path(td)
        unpack_arc(arc_path, td / "unpack", overwrite=True)
        pack_arc(td / "unpack", td / "rebuild.arc")
        rebuilt = (td / "rebuild.arc").read_bytes()
    ok = original == rebuilt
    print(f"[verify] byte_exact={ok}")
    print(f"[verify] original_sha256={hashlib.sha256(original).hexdigest()}")
    print(f"[verify] rebuild_sha256 ={hashlib.sha256(rebuilt).hexdigest()}")
    if not ok:
        raise SystemExit(1)


def build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Baigui / AI5WIN ARC unpacker and repacker")
    sub = p.add_subparsers(dest="cmd", required=True)

    pu = sub.add_parser("unpack", help="extract archive files")
    pu.add_argument("arc", help="input .arc")
    pu.add_argument("out_dir", help="output directory")
    pu.add_argument("--overwrite", action="store_true", help="overwrite existing output files")

    pp = sub.add_parser("pack", help="build archive from directory")
    pp.add_argument("input_dir", help="directory containing extracted files")
    pp.add_argument("out_arc", help="output .arc")
    pp.add_argument("--manifest", help="manifest path; default input_dir/arc_manifest.json")

    pl = sub.add_parser("list", help="list archive entries")
    pl.add_argument("arc", help="input .arc")
    pl.add_argument("--json", action="store_true", help="print JSON entry list")

    pv = sub.add_parser("verify", help="unpack and repack, then compare byte-for-byte")
    pv.add_argument("arc", help="input .arc")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_arg_parser().parse_args(argv)
    if args.cmd == "unpack":
        unpack_arc(args.arc, args.out_dir, args.overwrite)
    elif args.cmd == "pack":
        pack_arc(args.input_dir, args.out_arc, args.manifest)
    elif args.cmd == "list":
        list_arc(args.arc, args.json)
    elif args.cmd == "verify":
        verify_roundtrip(args.arc)
    else:
        raise AssertionError(args.cmd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
