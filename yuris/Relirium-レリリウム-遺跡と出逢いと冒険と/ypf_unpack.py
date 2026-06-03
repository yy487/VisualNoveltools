# -*- coding: utf-8 -*-
"""
YU-RIS YPF v500 unpacker for Relirium / similar archives.

Current scope verified with sc.ypf from Relirium:
- header magic: b"YPF\0"
- version: 500
- entry table starts at 0x20
- file names are XOR-obfuscated with 0xC9
- encoded file-name length is decoded through the YU-RIS runtime length table
- payload type 1 is normal zlib stream

Usage:
    python ypf_unpack.py list sc.ypf
    python ypf_unpack.py unpack sc.ypf out_dir
    python ypf_unpack.py unpack sc.ypf out_dir --raw
    python ypf_unpack.py index sc.ypf manifest.json
"""
from __future__ import annotations

import argparse
import json
import os
import struct
import sys
import zlib
from dataclasses import asdict, dataclass
from pathlib import Path, PureWindowsPath
from typing import Iterable


HEADER_SIZE = 0x20
NAME_XOR = 0xC9
DEFAULT_ENCODING = "cp932"


@dataclass
class YpfEntry:
    index: int
    name: str
    name_hash: int
    flag: int
    compress_type: int
    raw_size: int
    packed_size: int
    offset: int
    checksum: int
    table_offset: int


class YpfError(RuntimeError):
    pass


def read_u32(data: bytes, offset: int) -> int:
    if offset + 4 > len(data):
        raise YpfError(f"unexpected EOF while reading u32 at 0x{offset:X}")
    return struct.unpack_from("<I", data, offset)[0]


def build_length_table() -> list[int]:
    """Rebuilds the 256-byte table initialized around FUN_00498b24.

    The game uses (&DAT_0067ffdf)[-encoded_len], i.e. table[255 - encoded_len].
    The table is identity with several title/runtime swaps.
    """
    table = list(range(256))
    swaps = [
        (0x03, 0x0A),
        (0x06, 0x35),
        (0x09, 0x0B),
        (0x0C, 0x10),
        (0x0D, 0x13),
        (0x11, 0x18),
        (0x15, 0x1B),
        (0x1C, 0x1E),
        (0x20, 0x23),
        (0x26, 0x29),
        (0x2C, 0x2F),
        (0x2E, 0x14),
    ]
    for a, b in swaps:
        table[a], table[b] = table[b], table[a]
    return table


LENGTH_TABLE = build_length_table()


def decode_name_length(encoded_len: int) -> int:
    return LENGTH_TABLE[0xFF - encoded_len]


def decode_name(raw: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    return bytes((b ^ NAME_XOR) for b in raw).decode(encoding)


def parse_entries(data: bytes, *, encoding: str = DEFAULT_ENCODING) -> tuple[int, int, list[YpfEntry]]:
    if len(data) < HEADER_SIZE:
        raise YpfError("file too small for YPF header")
    magic, version, count, table_end = struct.unpack_from("<4sIII", data, 0)
    if magic != b"YPF\0":
        raise YpfError(f"bad magic: {magic!r}")
    if version != 500:
        raise YpfError(f"unsupported YPF version: {version}; this tool is verified for version 500")
    if table_end < HEADER_SIZE or table_end > len(data):
        raise YpfError(f"bad table/data-start offset: 0x{table_end:X}")

    entries: list[YpfEntry] = []
    pos = HEADER_SIZE
    for index in range(count):
        table_offset = pos
        if pos + 0x1B > table_end:
            raise YpfError(f"entry {index}: table overrun at 0x{pos:X}")
        name_hash = read_u32(data, pos)
        name_len = decode_name_length(data[pos + 4])
        name_start = pos + 5
        name_end = name_start + name_len
        meta_start = name_end
        meta_end = pos + name_len + 0x1B
        if meta_end > table_end:
            raise YpfError(f"entry {index}: metadata overrun at 0x{pos:X}")

        name = decode_name(data[name_start:name_end], encoding)
        meta = data[meta_start:meta_end]
        if len(meta) != 22:
            raise YpfError(f"entry {index}: bad metadata length {len(meta)}")
        flag = meta[0]
        compress_type = meta[1]
        raw_size, packed_size, off_low, off_high, checksum = struct.unpack_from("<IIIII", meta, 2)
        offset = off_low | (off_high << 32)
        if offset + packed_size > len(data):
            raise YpfError(
                f"entry {index} {name!r}: payload out of range, "
                f"offset=0x{offset:X}, packed_size=0x{packed_size:X}, file_size=0x{len(data):X}"
            )
        entries.append(
            YpfEntry(
                index=index,
                name=name,
                name_hash=name_hash,
                flag=flag,
                compress_type=compress_type,
                raw_size=raw_size,
                packed_size=packed_size,
                offset=offset,
                checksum=checksum,
                table_offset=table_offset,
            )
        )
        pos += name_len + 0x1B

    if pos != table_end:
        # This is a warning condition for some variants, not necessarily fatal.
        # Relirium sc.ypf should end exactly at table_end.
        pass
    return version, table_end, entries


def safe_output_path(root: Path, archive_name: str) -> Path:
    # YPF stores Windows separators. Convert cautiously and reject absolute/traversal paths.
    parts = PureWindowsPath(archive_name).parts
    clean: list[str] = []
    for part in parts:
        if part in ("", ".", "\\", "/"):
            continue
        if part == ".." or part.endswith(":"):
            raise YpfError(f"unsafe archive path: {archive_name!r}")
        clean.append(part)
    if not clean:
        raise YpfError(f"empty archive path for {archive_name!r}")
    return root.joinpath(*clean)


def read_payload(data: bytes, entry: YpfEntry, *, raw: bool = False) -> bytes:
    payload = data[entry.offset:entry.offset + entry.packed_size]
    if raw or entry.compress_type == 0:
        out = payload
    elif entry.compress_type == 1:
        out = zlib.decompress(payload)
    else:
        raise YpfError(
            f"entry {entry.index} {entry.name!r}: unsupported compression type {entry.compress_type}"
        )
    if not raw and len(out) != entry.raw_size:
        raise YpfError(
            f"entry {entry.index} {entry.name!r}: size mismatch, "
            f"expected {entry.raw_size}, got {len(out)}"
        )
    return out


def do_list(path: Path, entries: Iterable[YpfEntry]) -> None:
    for e in entries:
        print(
            f"{e.index:04d}  off=0x{e.offset:08X}  "
            f"packed={e.packed_size:8d}  raw={e.raw_size:8d}  "
            f"type={e.compress_type}  {e.name}"
        )


def do_index(out_path: Path, version: int, table_end: int, entries: list[YpfEntry]) -> None:
    obj = {
        "format": "YPF",
        "version": version,
        "table_end": table_end,
        "entry_count": len(entries),
        "entries": [asdict(e) for e in entries],
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(obj, ensure_ascii=False, indent=2), encoding="utf-8", newline="\n")


def do_unpack(path: Path, out_dir: Path, entries: list[YpfEntry], *, raw: bool) -> None:
    data = path.read_bytes()
    out_dir.mkdir(parents=True, exist_ok=True)
    ok = 0
    for e in entries:
        payload = read_payload(data, e, raw=raw)
        out_path = safe_output_path(out_dir, e.name)
        if raw and e.compress_type != 0:
            out_path = out_path.with_name(out_path.name + ".raw")
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(payload)
        ok += 1
    print(f"[unpack] archive={path} entries={len(entries)} extracted={ok} output={out_dir}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Unpack YU-RIS YPF v500 archives")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="list archive entries")
    p_list.add_argument("archive")
    p_list.add_argument("--encoding", default=DEFAULT_ENCODING)

    p_unpack = sub.add_parser("unpack", help="extract archive entries")
    p_unpack.add_argument("archive")
    p_unpack.add_argument("output")
    p_unpack.add_argument("--raw", action="store_true", help="dump compressed payloads without zlib decompression")
    p_unpack.add_argument("--encoding", default=DEFAULT_ENCODING)

    p_index = sub.add_parser("index", help="write a JSON manifest only")
    p_index.add_argument("archive")
    p_index.add_argument("output")
    p_index.add_argument("--encoding", default=DEFAULT_ENCODING)

    args = parser.parse_args(argv)
    archive = Path(args.archive)
    data = archive.read_bytes()
    version, table_end, entries = parse_entries(data, encoding=args.encoding)

    if args.cmd == "list":
        do_list(archive, entries)
    elif args.cmd == "unpack":
        do_unpack(archive, Path(args.output), entries, raw=args.raw)
    elif args.cmd == "index":
        do_index(Path(args.output), version, table_end, entries)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except YpfError as e:
        print(f"[ypf][error] {e}", file=sys.stderr)
        raise SystemExit(1)
