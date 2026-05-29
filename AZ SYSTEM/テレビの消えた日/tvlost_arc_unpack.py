# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import struct
import zlib
from pathlib import Path

DEFAULT_KEY = 0xADD1F4AA
XOR_MUL = 0x9E370001
HEADER_SIZE = 0x30
ENTRY_SIZE = 0x30


def _crypt_chunk(data: bytes | bytearray, key: int, file_pos: int) -> bytes:
    prod = (key & 0xFFFFFFFF) * XOR_MUL & 0xFFFFFFFFFFFFFFFF
    eax = prod & 0xFFFFFFFF
    edx = (prod >> 32) & 0xFFFFFFFF
    if file_pos & 0x20:
        eax, edx = edx, eax
    cnt = file_pos & 0x1F
    if cnt:
        old_eax, old_edx = eax, edx
        eax = ((old_eax << cnt) & 0xFFFFFFFF) | (old_edx >> (32 - cnt))
        edx = ((old_edx << cnt) & 0xFFFFFFFF) | (old_eax >> (32 - cnt))
    state = ((edx << 32) | eax) & 0xFFFFFFFFFFFFFFFF
    out = bytearray(data)
    for i in range(len(out)):
        out[i] ^= state & 0xFF
        state = ((state << 1) | (state >> 63)) & 0xFFFFFFFFFFFFFFFF
    return bytes(out)


def _read_cstr32(raw: bytes) -> str:
    raw = raw.split(b"\x00", 1)[0]
    return raw.decode("cp932", errors="replace")


def parse_arc(data: bytes, key: int = DEFAULT_KEY) -> tuple[dict, list[dict]]:
    header = _crypt_chunk(data[:HEADER_SIZE], key, 0)
    magic, version, count, table_comp_size = struct.unpack_from("<4sIII", header, 0)
    if magic != b"ARC\x00":
        raise ValueError(f"bad ARC magic after decrypt: {magic!r}; wrong key or unsupported archive")
    ext = _read_cstr32(header[0x10:0x30])
    table_start = HEADER_SIZE
    table_end = table_start + table_comp_size
    if table_end > len(data):
        raise ValueError(f"compressed table extends beyond file: end=0x{table_end:x}, file=0x{len(data):x}")
    table_blob = _crypt_chunk(data[table_start:table_end], key, table_start)
    stored_adler, = struct.unpack_from("<I", table_blob, 0)
    calc_adler = zlib.adler32(table_blob[4:]) & 0xFFFFFFFF
    if stored_adler != calc_adler:
        raise ValueError(f"table adler mismatch: stored=0x{stored_adler:08x}, calc=0x{calc_adler:08x}")
    table = zlib.decompress(table_blob[4:])
    expected = count * ENTRY_SIZE
    if len(table) != expected:
        raise ValueError(f"table size mismatch: got={len(table)}, expected={expected}")
    data_base = table_end
    entries: list[dict] = []
    for idx in range(count):
        ent = table[idx * ENTRY_SIZE:(idx + 1) * ENTRY_SIZE]
        rel_off, size, name_hash, unk = struct.unpack_from("<IIII", ent, 0)
        name = _read_cstr32(ent[0x10:0x30])
        abs_off = data_base + rel_off
        if abs_off + size > len(data):
            raise ValueError(f"entry out of range index={idx} name={name} off=0x{abs_off:x} size=0x{size:x}")
        entries.append({
            "index": idx,
            "name": name,
            "offset": abs_off,
            "relative_offset": rel_off,
            "size": size,
            "hash": name_hash,
            "unknown": unk,
        })
    info = {
        "magic": magic.decode("ascii", errors="replace").rstrip("\x00"),
        "version_or_flags": version,
        "count": count,
        "table_compressed_size": table_comp_size,
        "table_uncompressed_size": len(table),
        "entry_size": ENTRY_SIZE,
        "default_ext": ext,
        "data_base": data_base,
        "key": f"0x{key:08x}",
    }
    return info, entries


def unpack_arc(input_path: Path, output_dir: Path, key: int = DEFAULT_KEY) -> dict:
    data = input_path.read_bytes()
    info, entries = parse_arc(data, key)
    output_dir.mkdir(parents=True, exist_ok=True)
    for ent in entries:
        raw = data[ent["offset"]:ent["offset"] + ent["size"]]
        plain = _crypt_chunk(raw, key, ent["offset"])
        out_path = output_dir / ent["name"]
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(plain)
    manifest = {"archive": input_path.name, "info": info, "entries": entries}
    (output_dir / "_manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    return manifest


def main() -> None:
    ap = argparse.ArgumentParser(description="Unpack TVLost-style encrypted ARC archives")
    ap.add_argument("input", help="input .arc")
    ap.add_argument("output", help="output directory")
    ap.add_argument("--key", type=lambda s: int(s, 0), default=DEFAULT_KEY, help="XOR key, default: 0xADD1F4AA")
    args = ap.parse_args()
    manifest = unpack_arc(Path(args.input), Path(args.output), args.key)
    info = manifest["info"]
    print(f"[unpack] archive={args.input}")
    print(f"[unpack] key={info['key']} entries={info['count']} ext={info['default_ext']} table={info['table_compressed_size']}->{info['table_uncompressed_size']}")
    print(f"[unpack] output={args.output}")


if __name__ == "__main__":
    main()
