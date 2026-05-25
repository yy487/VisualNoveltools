# -*- coding: utf-8 -*-
"""Common helpers for Noesis IGA0 archives used by Love es M script.iga."""
from __future__ import annotations

import json
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import BinaryIO, Iterable

MAGIC = b"IGA0"
HEADER_SIZE = 0x10
DEFAULT_ENCODING = "cp932"
MANIFEST_NAME = "_noesis_iga_manifest.json"


class IgaFormatError(ValueError):
    pass


@dataclass
class IgaEntry:
    name: str
    offset: int          # offset relative to data section in archive-stored form
    size: int            # stored size, equal to decrypted size
    name_offset: int = 0 # offset within packed name table
    index: int = 0

    def to_manifest(self) -> dict:
        return asdict(self)


def read_packed_uint_from(data: bytes | bytearray, pos: int) -> tuple[int, int, bytes]:
    """Read Noesis packed uint.

    Decoder mirrors GARbro's Noesis/ArcIGA.cs:
        val = 0
        while ((val & 1) == 0): val = val << 7 | next_byte
        return val >> 1
    """
    val = 0
    start = pos
    n = len(data)
    while (val & 1) == 0:
        if pos >= n:
            raise IgaFormatError(f"truncated packed uint at 0x{start:X}")
        val = (val << 7) | data[pos]
        pos += 1
    return val >> 1, pos, bytes(data[start:pos])


def read_packed_uint(stream: BinaryIO) -> int:
    val = 0
    while (val & 1) == 0:
        b = stream.read(1)
        if not b:
            raise IgaFormatError("truncated packed uint")
        val = (val << 7) | b[0]
    return val >> 1


def write_packed_uint(value: int) -> bytes:
    """Encode integer with the same overlapping 8/7-bit layout used by IGA0.

    The low byte of each step is kept as a full byte, then the remaining value is
    shifted by 7.  This reproduces the original archive's packed integer bytes
    for zero-modification repacks.
    """
    if value < 0:
        raise ValueError("packed uint cannot be negative")
    val = (value << 1) | 1
    parts: list[int] = []
    while val > 0xFF:
        low = val & 0xFF
        parts.append(low)
        val = (val - low) >> 7
    parts.append(val)
    return bytes(reversed(parts))


def xor_entry_data(name: str, data: bytes | bytearray) -> bytes:
    """Encrypt/decrypt one entry payload.

    .s scripts use key 0xFF. Other entries use key 0.  The operation is
    symmetric, so the same routine is used for unpack and pack.
    """
    key = 0xFF if name.lower().endswith(".s") else 0x00
    out = bytearray(data)
    for i in range(len(out)):
        out[i] ^= ((i + 2) ^ key) & 0xFF
    return bytes(out)


def parse_archive(data: bytes, encoding: str = DEFAULT_ENCODING) -> tuple[bytes, list[IgaEntry], int, int, int]:
    """Return (header, entries, names_start, names_length, data_offset)."""
    if len(data) < HEADER_SIZE or data[:4] != MAGIC:
        raise IgaFormatError("not an IGA0 archive")

    pos = HEADER_SIZE
    index_length, pos, _ = read_packed_uint_from(data, pos)
    index_start = pos
    index_end = index_start + index_length
    if index_end > len(data):
        raise IgaFormatError(f"index table exceeds file size: end=0x{index_end:X}")

    raw_entries: list[tuple[int, int, int]] = []
    while pos < index_end:
        name_offset, pos, _ = read_packed_uint_from(data, pos)
        offset, pos, _ = read_packed_uint_from(data, pos)
        size, pos, _ = read_packed_uint_from(data, pos)
        raw_entries.append((name_offset, offset, size))
    if pos != index_end:
        raise IgaFormatError("index table did not end on packed uint boundary")

    names_length, pos, _ = read_packed_uint_from(data, pos)
    names_start = pos
    data_offset = names_start + names_length
    if data_offset > len(data):
        raise IgaFormatError(f"name table exceeds file size: data_offset=0x{data_offset:X}")

    entries: list[IgaEntry] = []
    for i, (name_offset, rel_offset, size) in enumerate(raw_entries):
        next_name_offset = raw_entries[i + 1][0] if i + 1 < len(raw_entries) else names_length
        name_len = next_name_offset - name_offset
        if name_len < 0:
            raise IgaFormatError(f"negative name length at entry {i}")
        name_pos = names_start + name_offset
        name_bytes = bytearray()
        p = name_pos
        for _ in range(name_len):
            v, p, _ = read_packed_uint_from(data, p)
            if not 0 <= v <= 0xFF:
                raise IgaFormatError(f"filename byte out of range at entry {i}: {v}")
            name_bytes.append(v)
        try:
            name = name_bytes.decode(encoding)
        except UnicodeDecodeError as e:
            raise IgaFormatError(f"filename decode failed at entry {i}: {e}") from e

        abs_offset = data_offset + rel_offset
        if abs_offset < data_offset or abs_offset + size > len(data):
            raise IgaFormatError(
                f"entry placement invalid at {i}: {name!r}, offset=0x{abs_offset:X}, size=0x{size:X}"
            )
        entries.append(IgaEntry(name=name, offset=rel_offset, size=size, name_offset=name_offset, index=i))

    return data[:HEADER_SIZE], entries, names_start, names_length, data_offset


def build_index_and_names(entries: list[IgaEntry], encoding: str = DEFAULT_ENCODING) -> tuple[bytes, bytes, list[IgaEntry]]:
    names_blob = bytearray()
    rebuilt: list[IgaEntry] = []
    for i, entry in enumerate(entries):
        name_bytes = entry.name.encode(encoding)
        # The original Noesis archives use plain ASCII filenames; GARbro's reader
        # also effectively assumes one packed byte per filename byte for offsets.
        # Reject uncommon names instead of building an archive the engine may not read.
        bad = [b for b in name_bytes if len(write_packed_uint(b)) != 1]
        if bad:
            raise IgaFormatError(
                f"filename must be single-byte packed ASCII/low cp932 for IGA0 compatibility: {entry.name!r}"
            )
        name_offset = len(names_blob)
        for b in name_bytes:
            names_blob += write_packed_uint(b)
        rebuilt.append(IgaEntry(entry.name, entry.offset, entry.size, name_offset, i))

    index_body = bytearray()
    for entry in rebuilt:
        index_body += write_packed_uint(entry.name_offset)
        index_body += write_packed_uint(entry.offset)
        index_body += write_packed_uint(entry.size)
    return bytes(index_body), bytes(names_blob), rebuilt


def read_manifest(path: str | Path) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, dict) or "entries" not in data:
        raise IgaFormatError(f"invalid manifest: {path}")
    return data


def write_manifest(path: str | Path, archive_path: str | Path, header: bytes, entries: list[IgaEntry], data_offset: int) -> None:
    obj = {
        "format": "Noesis IGA0",
        "engine": "noesis",
        "archive": Path(archive_path).name,
        "encoding": DEFAULT_ENCODING,
        "header_hex": header.hex(),
        "data_offset": data_offset,
        "entries": [e.to_manifest() for e in entries],
        "notes": [
            "Entry files are stored decrypted in this directory.",
            "During pack, .s files are XOR-encrypted with key 0xFF and other files with key 0.",
            "Entry order is significant and is preserved from this manifest."
        ],
    }
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2)


def safe_join(root: Path, name: str) -> Path:
    # IGA0 used here is flat, but guard against crafted names.
    p = root / name.replace("\\", "/")
    resolved_root = root.resolve()
    resolved_p = p.resolve()
    if resolved_root != resolved_p and resolved_root not in resolved_p.parents:
        raise IgaFormatError(f"unsafe entry path: {name!r}")
    return p
