# -*- coding: utf-8 -*-
"""
Nitroplus NPA v1 unpacker, reconstructed from the supplied Muramasa_chs.exe
pseudo-C export.

Relevant decompiled routines:
  sub_401000 : finds NPA archives, checks magic/version, reads 41-byte header
  sub_401940 : reads/decrypts the file table
  sub_401860 : archive entry lookup by name hash
  sub_401E20 : decrypts file data while reading

Usage:
  python npa_unpack.py sound2.npa out_sound2
  python npa_unpack.py --list sound2.npa
"""
from __future__ import annotations

import argparse
import json
import struct
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

# byte_534830 from memory address 0x00534830.  sub_401000 builds byte_537CF8
# as the inverse table.  The file table name decoder uses arithmetic only;
# file body decoder uses byte_537CF8, matching sub_401E20.
BYTE_534830 = bytes([
    0x5B,0x8F,0x16,0xB8,0xC9,0xB2,0x18,0xA7,0x41,0x27,0x0E,0xA3,0xD8,0x7F,0x49,0xAA,
    0x68,0x03,0xF3,0x89,0xA0,0x24,0xF1,0xBD,0xEC,0x9F,0xD3,0xFC,0xC2,0x04,0x25,0xBE,
    0x94,0x47,0x77,0x08,0xB0,0x64,0x11,0xEB,0x5A,0xEE,0x51,0x73,0x69,0x9E,0x29,0xD9,
    0x22,0x87,0xAB,0x37,0x20,0xC8,0x5D,0xAD,0xAC,0x62,0x9A,0x6B,0x9C,0x75,0x78,0x53,
    0x21,0x0A,0xD0,0x2D,0x9D,0xA2,0x8A,0x96,0x00,0xBB,0x6E,0xF0,0x99,0xCE,0xF6,0xFE,
    0xD4,0x2F,0x80,0x6F,0x6A,0x17,0x3B,0xB1,0x05,0xBF,0xA1,0xC7,0x8E,0xFB,0x58,0x4A,
    0xFF,0x19,0x57,0x2A,0xCB,0x60,0xC6,0x3D,0xF2,0xB7,0x0D,0xED,0x86,0x55,0xD7,0xD5,
    0xE0,0x3E,0x7D,0xEA,0x6C,0xAF,0x1C,0x9B,0xC1,0xE1,0xC0,0x65,0x84,0xC5,0x28,0x66,
    0xE8,0x8D,0x52,0xCF,0x4E,0x82,0x0C,0xF9,0x81,0x26,0x59,0x2B,0x5F,0x7B,0x7C,0xF8,
    0x13,0x1F,0x15,0x33,0xDB,0xF5,0x1E,0xDD,0xE4,0x48,0xCA,0xDE,0xAE,0xCD,0x2E,0x39,
    0xE5,0x2C,0xDC,0xB9,0x95,0x67,0x23,0x50,0x56,0x61,0xCC,0x8B,0xEF,0xDA,0x12,0x1A,
    0x83,0xFD,0x32,0x0F,0x4C,0x30,0x7A,0x63,0x88,0x98,0x36,0xB4,0x3F,0x09,0xBA,0x14,
    0x0B,0xE2,0x91,0xD1,0x7E,0xDF,0x44,0xC4,0xC3,0x6D,0xB6,0x90,0x3C,0xB3,0x70,0xA8,
    0x85,0x35,0x79,0x02,0xF7,0x97,0x45,0x4F,0xA4,0x74,0xA9,0x4D,0x42,0xA5,0xD2,0x76,
    0x43,0x72,0x38,0xF4,0x5C,0x07,0xFA,0x34,0x01,0x10,0x06,0xE7,0x54,0x40,0xBC,0xE3,
    0x1D,0x1B,0x4B,0x8C,0xB5,0x92,0x3A,0xA6,0xE9,0xE6,0x31,0x93,0x46,0x5E,0x71,0xD6,
])

BYTE_537CF8 = bytearray(256)
for i, b in enumerate(BYTE_534830):
    BYTE_537CF8[b] = i
BYTE_537CF8 = bytes(BYTE_537CF8)

NPA_MAGIC = b"NPA"
NPA_VERSION = 1
HEADER_SIZE = 41


def u32(data: bytes, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def sum_u32_bytes(x: int) -> int:
    return ((x >> 24) & 0xFF) + ((x >> 16) & 0xFF) + ((x >> 8) & 0xFF) + (x & 0xFF)


def npa_name_hash(name_bytes: bytes) -> int:
    """sub_402010 / sub_401360 name hash."""
    v = -2023406815 & 0xFFFFFFFF
    for c in name_bytes:
        v = (v - c) & 0xFFFFFFFF
    return (v * len(name_bytes)) & 0xFFFFFFFF


def decode_name(enc: bytes, index: int, key1: int, key2: int) -> bytes:
    """Name decoder from sub_401940."""
    prod = (key1 * key2) & 0xFFFFFFFF
    # The decompiler shows low product byte as BYTE(key2) * BYTE(key1), then
    # char assignment truncates it, so it is equivalent to the low product byte.
    prod_sum = sum_u32_bytes(prod)
    idx_sum = sum_u32_bytes(index)
    out = bytearray(len(enc))
    for i, c in enumerate(enc):
        out[i] = (c - 4 * i - prod_sum - idx_sum) & 0xFF
    return bytes(out)


def decode_path(name_bytes: bytes) -> str:
    try:
        s = name_bytes.decode("cp932")
    except UnicodeDecodeError:
        s = "name_" + name_bytes.hex()
    # NPA stores backslashes.  Keep the internal path but make it safe enough
    # for normal extraction.
    s = s.replace("\\", "/")
    parts: list[str] = []
    for part in s.split("/"):
        if part in ("", ".", ".."):
            part = "_"
        # Windows-hostile characters; useful if files are extracted on Windows.
        part = "".join("_" if ch in '<>:"|?*' else ch for ch in part)
        parts.append(part)
    return "/".join(parts)


@dataclass
class NpaHeader:
    key1: int
    key2: int
    dir_flag: int
    crypt_flag: int
    entry_count: int
    unk24: int
    unk28: int
    unk32: int
    unk36: int
    index_size: int

    @property
    def data_base(self) -> int:
        return HEADER_SIZE + self.index_size


@dataclass
class NpaEntry:
    index: int
    name_bytes: bytes
    name: str
    entry_type: int
    field24: int
    offset: int
    packed_size: int
    unpacked_size: int
    data_offset: int
    crypt_limit: int
    crypt_key: int

    @property
    def is_file(self) -> bool:
        # Current sample: 1 = directory, 2 = file.
        return self.entry_type == 2 and self.packed_size > 0


def parse_header(data: bytes) -> NpaHeader:
    if len(data) < HEADER_SIZE:
        raise ValueError("file too small for NPA header")
    if data[0:3] != NPA_MAGIC or u32(data, 3) != NPA_VERSION:
        raise ValueError("not supported NPA v1 archive")
    return NpaHeader(
        key1=u32(data, 7),
        key2=u32(data, 11),
        dir_flag=data[15],
        crypt_flag=data[16],
        entry_count=u32(data, 17),
        unk24=u32(data, 21),
        unk28=u32(data, 25),
        unk32=u32(data, 29),
        unk36=u32(data, 33),
        index_size=u32(data, 37),
    )


def parse_entries(data: bytes, header: NpaHeader) -> list[NpaEntry]:
    pos = HEADER_SIZE
    entries: list[NpaEntry] = []
    prod = (header.key1 * header.key2) & 0xFFFFFFFF

    for idx in range(header.entry_count):
        if pos + 4 > len(data):
            raise ValueError(f"entry {idx}: truncated name length")
        name_len = u32(data, pos)
        pos += 4
        if name_len > 0x10000 or pos + name_len + 17 > len(data):
            raise ValueError(f"entry {idx}: invalid name length {name_len} at 0x{pos-4:X}")
        name_bytes = decode_name(data[pos:pos + name_len], idx, header.key1, header.key2)
        pos += name_len
        entry_type = data[pos]
        pos += 1
        field24, rel_off, packed_size, unpacked_size = struct.unpack_from("<IIII", data, pos)
        pos += 16

        h = npa_name_hash(name_bytes)
        crypt_key = (unpacked_size * ((h + prod) & 0xFFFFFFFF)) & 0xFFFFFFFF
        crypt_limit = min(packed_size, len(name_bytes) + 4096)
        entries.append(NpaEntry(
            index=idx,
            name_bytes=name_bytes,
            name=decode_path(name_bytes),
            entry_type=entry_type,
            field24=field24,
            offset=rel_off,
            packed_size=packed_size,
            unpacked_size=unpacked_size,
            data_offset=header.data_base + rel_off,
            crypt_limit=crypt_limit,
            crypt_key=crypt_key,
        ))

    if pos != header.data_base:
        # sub_401940 effectively uses index_size as data-base; mismatch means
        # either padding or a bad parse.  Do not fail hard, but report it.
        print(f"[npa][warn] parsed index ends at 0x{pos:X}, header data_base is 0x{header.data_base:X}")
    return entries


def decrypt_file_blob(blob: bytes, entry: NpaEntry, crypt_flag: int) -> bytes:
    if not crypt_flag:
        return blob
    out = bytearray(blob)
    # sub_401E20: byte_537CF8[src] - entry_crypt_key - current_file_pos - i.
    # When unpacking in one pass, current_file_pos starts at 0.
    for i in range(min(entry.crypt_limit, len(out))):
        out[i] = (BYTE_537CF8[out[i]] - entry.crypt_key - i) & 0xFF
    return bytes(out)


def iter_files(data: bytes, header: NpaHeader, entries: Iterable[NpaEntry]):
    for ent in entries:
        if not ent.is_file:
            continue
        end = ent.data_offset + ent.packed_size
        if ent.data_offset < 0 or end > len(data):
            raise ValueError(f"{ent.name}: data range out of archive: 0x{ent.data_offset:X}+0x{ent.packed_size:X}")
        yield ent, decrypt_file_blob(data[ent.data_offset:end], ent, header.crypt_flag)


def unpack_npa(npa_path: Path, out_dir: Path, list_only: bool = False, manifest_name: str = "npa_manifest.json") -> None:
    data = npa_path.read_bytes()
    header = parse_header(data)
    entries = parse_entries(data, header)

    files = [e for e in entries if e.is_file]
    dirs = [e for e in entries if not e.is_file]

    print(f"[npa] archive={npa_path}")
    print(f"[npa] entries={len(entries)} files={len(files)} non_files={len(dirs)} data_base=0x{header.data_base:X}")
    print(f"[npa] key1=0x{header.key1:08X} key2=0x{header.key2:08X} crypt_flag={header.crypt_flag} dir_flag={header.dir_flag}")

    if list_only:
        for e in entries:
            kind = "file" if e.is_file else "dir/entry"
            print(f"{e.index:05d} {kind:9s} type={e.entry_type} off=0x{e.data_offset:X} size={e.packed_size} name={e.name}")
        return

    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "archive": str(npa_path),
        "header": asdict(header),
        "entries": [
            {
                **asdict(e),
                "name_bytes_hex": e.name_bytes.hex(),
                "name_bytes": None,  # bytes are kept above as hex for JSON safety
            }
            for e in entries
        ],
    }
    # Remove raw bytes objects left by asdict.
    for e in manifest["entries"]:
        e.pop("name_bytes", None)

    written = 0
    for ent, blob in iter_files(data, header, entries):
        out_path = out_dir / ent.name
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(blob)
        written += 1
        sig = blob[:4]
        print(f"[npa] write {ent.name} size={len(blob)} sig={sig!r}")

    (out_dir / manifest_name).write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[npa] done written={written} manifest={out_dir / manifest_name}")


def main() -> int:
    ap = argparse.ArgumentParser(description="Unpack Nitroplus NPA v1 archives used by the supplied Muramasa executable.")
    ap.add_argument("input", help="input .npa")
    ap.add_argument("output", nargs="?", help="output directory")
    ap.add_argument("--list", action="store_true", help="only list entries")
    args = ap.parse_args()

    npa_path = Path(args.input)
    out_dir = Path(args.output) if args.output else npa_path.with_suffix("")
    unpack_npa(npa_path, out_dir, list_only=args.list)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
