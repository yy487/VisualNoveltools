# -*- coding: utf-8 -*-
"""MES.ARC reader/writer for Refrain Blue / AI5WIN-like archive."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct
from typing import Iterable

ENTRY_SIZE = 0x1C
NAME_SIZE = 20
NAME_XOR = 0x55
SIZE_XOR = 0xAA55AA55
OFFSET_XOR = 0x55AA55AA


@dataclass
class ArcEntry:
    name: str
    offset: int
    size: int
    data: bytes


def _decode_name(raw: bytes) -> str:
    dec = bytes(b ^ NAME_XOR for b in raw[:NAME_SIZE])
    return dec.split(b"\x00", 1)[0].decode("ascii")


def _encode_name(name: str) -> bytes:
    b = name.encode("ascii")
    if len(b) >= NAME_SIZE:
        raise ValueError(f"archive name too long: {name!r}")
    raw = b + b"\x00" * (NAME_SIZE - len(b))
    return bytes(x ^ NAME_XOR for x in raw)


def read_arc(path: str | Path) -> list[ArcEntry]:
    path = Path(path)
    buf = path.read_bytes()
    if len(buf) < 4:
        raise ValueError(f"too small archive: {path}")
    count = struct.unpack_from("<I", buf, 0)[0]
    table_end = 4 + count * ENTRY_SIZE
    if table_end > len(buf):
        raise ValueError(f"invalid archive table: count={count}, size={len(buf)}")

    entries: list[ArcEntry] = []
    for i in range(count):
        off = 4 + i * ENTRY_SIZE
        item = buf[off:off + ENTRY_SIZE]
        name = _decode_name(item[:NAME_SIZE])
        size = struct.unpack_from("<I", item, 20)[0] ^ SIZE_XOR
        data_offset = struct.unpack_from("<I", item, 24)[0] ^ OFFSET_XOR
        if data_offset < table_end or data_offset + size > len(buf):
            raise ValueError(
                f"invalid entry[{i}] {name}: offset=0x{data_offset:X}, size=0x{size:X}, archive=0x{len(buf):X}"
            )
        entries.append(ArcEntry(name=name, offset=data_offset, size=size, data=buf[data_offset:data_offset + size]))
    return entries


def build_arc(entries: Iterable[tuple[str, bytes] | ArcEntry]) -> bytes:
    normalized: list[tuple[str, bytes]] = []
    for e in entries:
        if isinstance(e, ArcEntry):
            normalized.append((e.name, e.data))
        else:
            name, data = e
            normalized.append((name, bytes(data)))

    count = len(normalized)
    table_size = 4 + count * ENTRY_SIZE
    offset = table_size
    table = bytearray()
    table += struct.pack("<I", count)
    data_blob = bytearray()

    for name, data in normalized:
        table += _encode_name(name)
        table += struct.pack("<I", len(data) ^ SIZE_XOR)
        table += struct.pack("<I", offset ^ OFFSET_XOR)
        data_blob += data
        offset += len(data)

    return bytes(table + data_blob)


def write_arc(path: str | Path, entries: Iterable[tuple[str, bytes] | ArcEntry]) -> None:
    Path(path).write_bytes(build_arc(entries))


def unpack_arc(arc_path: str | Path, out_dir: str | Path) -> list[ArcEntry]:
    entries = read_arc(arc_path)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    for e in entries:
        (out / e.name).write_bytes(e.data)
    return entries


def load_dir_entries(input_dir: str | Path, order: list[str] | None = None) -> list[tuple[str, bytes]]:
    d = Path(input_dir)
    if order is None:
        names = sorted(p.name for p in d.iterdir() if p.is_file())
    else:
        names = order
    result: list[tuple[str, bytes]] = []
    for name in names:
        p = d / name
        if not p.exists():
            raise FileNotFoundError(f"missing file for archive pack: {p}")
        result.append((name, p.read_bytes()))
    return result
