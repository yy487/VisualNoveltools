# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import struct
import zlib
from pathlib import Path

ARCHIVE_KEY = 0xADD1F4AA
GRAPHIC_STREAM_KEY = 0x4AF3D7A3
XOR_MUL = 0x9E370001
HEADER_SIZE = 0x30
ENTRY_SIZE = 0x30


def crypt_chunk(data: bytes | bytearray, key: int, file_pos: int) -> bytes:
    """TVLost fread-layer XOR. file_pos is the real ftell() position before fread.

    The engine reinitializes the XOR state for every fread using file_pos & 0x3f.
    """
    prod = ((key & 0xFFFFFFFF) * XOR_MUL) & 0xFFFFFFFFFFFFFFFF
    lo = prod & 0xFFFFFFFF
    hi = (prod >> 32) & 0xFFFFFFFF
    if file_pos & 0x20:
        lo, hi = hi, lo
    cnt = file_pos & 0x1F
    if cnt:
        old_lo, old_hi = lo, hi
        lo = ((old_lo << cnt) & 0xFFFFFFFF) | (old_hi >> (32 - cnt))
        hi = ((old_hi << cnt) & 0xFFFFFFFF) | (old_lo >> (32 - cnt))
    state = ((hi << 32) | lo) & 0xFFFFFFFFFFFFFFFF
    out = bytearray(data)
    for i in range(len(out)):
        out[i] ^= state & 0xFF
        state = ((state << 1) | (state >> 63)) & 0xFFFFFFFFFFFFFFFF
    return bytes(out)


def _read_cstr(raw: bytes) -> str:
    return raw.split(b"\x00", 1)[0].decode("cp932", errors="replace")


def parse_arc(data: bytes, archive_key: int = ARCHIVE_KEY) -> tuple[dict, list[dict]]:
    header = crypt_chunk(data[:HEADER_SIZE], archive_key, 0)
    magic, version, count, table_comp_size = struct.unpack_from("<4sIII", header, 0)
    if magic != b"ARC\x00":
        raise ValueError(f"bad ARC magic after decrypt: {magic!r}")
    default_ext = _read_cstr(header[0x10:0x30])
    table_start = HEADER_SIZE
    table_end = table_start + table_comp_size
    table_blob = crypt_chunk(data[table_start:table_end], archive_key, table_start)
    stored_adler, = struct.unpack_from("<I", table_blob, 0)
    calc_adler = zlib.adler32(table_blob[4:]) & 0xFFFFFFFF
    if stored_adler != calc_adler:
        raise ValueError(f"ARC table adler mismatch: stored=0x{stored_adler:08x}, calc=0x{calc_adler:08x}")
    table = zlib.decompress(table_blob[4:])
    if len(table) != count * ENTRY_SIZE:
        raise ValueError(f"ARC table size mismatch: got={len(table)}, expected={count * ENTRY_SIZE}")
    data_base = table_end
    entries: list[dict] = []
    for index in range(count):
        ent = table[index * ENTRY_SIZE:(index + 1) * ENTRY_SIZE]
        rel_off, size, name_hash, unknown = struct.unpack_from("<IIII", ent, 0)
        name = _read_cstr(ent[0x10:0x30])
        offset = data_base + rel_off
        if offset + size > len(data):
            raise ValueError(f"entry out of range: {name} off=0x{offset:x} size=0x{size:x}")
        entries.append({
            "index": index,
            "name": name,
            "offset": offset,
            "relative_offset": rel_off,
            "size": size,
            "hash": name_hash,
            "unknown": unknown,
        })
    info = {
        "magic": "ARC",
        "version_or_flags": version,
        "count": count,
        "table_compressed_size": table_comp_size,
        "table_uncompressed_size": len(table),
        "default_ext": default_ext,
        "data_base": data_base,
        "archive_key": f"0x{archive_key:08X}",
    }
    return info, entries


def save_manifest(path: Path, info: dict, entries: list[dict]) -> None:
    path.write_text(json.dumps({"info": info, "entries": entries}, ensure_ascii=False, indent=2), encoding="utf-8")
