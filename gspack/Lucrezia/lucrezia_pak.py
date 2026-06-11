# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import os
import struct
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable

MAGIC = b"DataPack5"
HEADER_SIZE = 0x48
DATA_BASE_DEFAULT = 0x800
ENTRY_SIZE_V5 = 0x68
NAME_SIZE = 0x40
DEFAULT_ENCODING = "cp932"


@dataclass
class PakHeader:
    magic_raw: bytes
    title_raw: bytes
    version: int
    toc_packed_size: int
    toc_flags: int
    file_count: int
    data_base: int
    toc_offset: int

    @property
    def magic(self) -> str:
        return self.magic_raw.split(b"\0", 1)[0].decode("ascii", "replace")

    @property
    def title(self) -> str:
        return self.title_raw.split(b"\0", 1)[0].decode(DEFAULT_ENCODING, "replace")


@dataclass
class PakEntry:
    index: int
    name: str
    offset: int
    size: int
    flags0: int
    flags1: int
    meta: list[int]

    def to_table_bytes(self) -> bytes:
        name_b = self.name.encode(DEFAULT_ENCODING)
        if len(name_b) >= NAME_SIZE:
            raise ValueError(f"entry name too long for 0x40-byte field: {self.name!r}")
        name_field = name_b + b"\0" * (NAME_SIZE - len(name_b))
        nums = [self.offset, self.size, self.flags0, self.flags1] + list(self.meta)
        if len(nums) != 10:
            raise ValueError(f"internal error: entry {self.name!r} must contain 10 DWORD fields")
        return name_field + struct.pack("<10I", *nums)


def align_up(value: int, align: int) -> int:
    return (value + align - 1) // align * align


def lzss_decompress(src: bytes, out_size: int | None = None) -> bytes:
    """Engine-compatible CGsLZSS decompressor, from Lucrezia.exe sub_448890."""
    ring = bytearray(0x1000)
    ring_pos = 4078
    flags = 0
    pos = 0
    out = bytearray()

    while pos < len(src):
        flags >>= 1
        if (flags & 0x100) == 0:
            flags = src[pos] | 0xFF00
            pos += 1

        if flags & 1:
            if pos >= len(src):
                break
            c = src[pos]
            pos += 1
            out.append(c)
            ring[ring_pos] = c
            ring_pos = (ring_pos + 1) & 0xFFF
        else:
            if pos >= len(src):
                break
            b0 = src[pos]
            pos += 1
            if pos >= len(src):
                break
            b1 = src[pos]
            pos += 1
            ref_pos = ((b1 & 0xF0) << 4) | b0
            length = (b1 & 0x0F) + 3
            for i in range(length):
                c = ring[(ref_pos + i) & 0xFFF]
                out.append(c)
                ring[ring_pos] = c
                ring_pos = (ring_pos + 1) & 0xFFF

        if out_size is not None and len(out) >= out_size:
            return bytes(out[:out_size])

    return bytes(out)


def xor_by_index(buf: bytearray) -> None:
    # C code is byte ^= i, so only the low 8 bits take effect.
    for i in range(len(buf)):
        buf[i] ^= i & 0xFF


def read_header(data: bytes) -> PakHeader:
    if len(data) < HEADER_SIZE:
        raise ValueError("file is smaller than DataPack5 header")
    magic_raw = data[0x00:0x10]
    if MAGIC not in magic_raw:
        raise ValueError(f"unsupported magic: {magic_raw!r}")
    return PakHeader(
        magic_raw=magic_raw,
        title_raw=data[0x10:0x30],
        version=struct.unpack_from("<I", data, 0x30)[0],
        toc_packed_size=struct.unpack_from("<I", data, 0x34)[0],
        toc_flags=struct.unpack_from("<I", data, 0x38)[0],
        file_count=struct.unpack_from("<I", data, 0x3C)[0],
        data_base=struct.unpack_from("<I", data, 0x40)[0],
        toc_offset=struct.unpack_from("<I", data, 0x44)[0],
    )


def parse_entries(data: bytes, header: PakHeader) -> list[PakEntry]:
    table_size = header.file_count * ENTRY_SIZE_V5
    if header.toc_packed_size:
        blob = bytearray(data[header.toc_offset:header.toc_offset + header.toc_packed_size])
        if len(blob) != header.toc_packed_size:
            raise ValueError("truncated packed table")
        if header.toc_flags & 1:
            xor_by_index(blob)
        table = lzss_decompress(bytes(blob), table_size)
    else:
        table = data[header.toc_offset:header.toc_offset + table_size]

    if len(table) < table_size:
        raise ValueError(f"table decompressed/read too short: {len(table)} < {table_size}")

    entries: list[PakEntry] = []
    for index in range(header.file_count):
        ent = table[index * ENTRY_SIZE_V5:(index + 1) * ENTRY_SIZE_V5]
        raw_name = ent[:NAME_SIZE].split(b"\0", 1)[0]
        name = raw_name.decode(DEFAULT_ENCODING, "replace")
        nums = list(struct.unpack_from("<10I", ent, NAME_SIZE))
        entries.append(PakEntry(index, name, nums[0], nums[1], nums[2], nums[3], nums[4:]))
    return entries


def safe_output_name(name: str, index: int, add_ext: str | None = None) -> str:
    # Keep engine name but prevent accidental path traversal on unpack.
    cleaned = name.replace("\\", "_").replace("/", "_").replace(":", "_")
    if not cleaned:
        cleaned = f"entry_{index:04d}"
    if add_ext and not Path(cleaned).suffix:
        cleaned += add_ext
    return cleaned


def load_pak(path: Path) -> tuple[bytes, PakHeader, list[PakEntry]]:
    data = path.read_bytes()
    header = read_header(data)
    entries = parse_entries(data, header)
    return data, header, entries


def command_list(args: argparse.Namespace) -> None:
    data, header, entries = load_pak(Path(args.pak))
    print(f"magic={header.magic!r} title={header.title!r} version=0x{header.version:08X}")
    print(f"file_count={header.file_count} data_base=0x{header.data_base:X} toc_offset=0x{header.toc_offset:X} toc_packed_size=0x{header.toc_packed_size:X} toc_flags=0x{header.toc_flags:X}")
    for e in entries:
        sig = data[header.data_base + e.offset:header.data_base + e.offset + 8]
        print(f"{e.index:03d} {e.name:<32} off=0x{e.offset:06X} size={e.size:6d} flags=({e.flags0},{e.flags1}) sig={sig.hex(' ')}")


def command_unpack(args: argparse.Namespace) -> None:
    pak_path = Path(args.pak)
    out_dir = Path(args.out)
    data, header, entries = load_pak(pak_path)
    out_dir.mkdir(parents=True, exist_ok=True)

    manifest = {
        "format": "DataPack5",
        "source": pak_path.name,
        "header": asdict(header) | {
            "magic": header.magic,
            "title": header.title,
            "magic_raw_hex": header.magic_raw.hex(),
            "title_raw_hex": header.title_raw.hex(),
        },
        "entry_size": ENTRY_SIZE_V5,
        "encoding": DEFAULT_ENCODING,
        "files": [],
    }
    # bytes fields are not JSON-serializable; remove raw byte keys produced by asdict.
    manifest["header"].pop("magic_raw", None)
    manifest["header"].pop("title_raw", None)

    used_names: dict[str, int] = {}
    for e in entries:
        out_name = safe_output_name(e.name, e.index, args.add_ext)
        if out_name in used_names:
            used_names[out_name] += 1
            stem = Path(out_name).stem
            suffix = Path(out_name).suffix
            out_name = f"{stem}_{used_names[out_name]:02d}{suffix}"
        else:
            used_names[out_name] = 0
        start = header.data_base + e.offset
        end = start + e.size
        if start < 0 or end > len(data):
            raise ValueError(f"entry out of range: index={e.index} name={e.name!r} start=0x{start:X} end=0x{end:X}")
        (out_dir / out_name).write_bytes(data[start:end])
        manifest["files"].append({
            "index": e.index,
            "name": e.name,
            "output_name": out_name,
            "offset": e.offset,
            "size": e.size,
            "flags0": e.flags0,
            "flags1": e.flags1,
            "meta": e.meta,
        })

    manifest_path = out_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[unpack] files={len(entries)} output={out_dir}")
    print(f"[unpack] manifest={manifest_path}")


def command_pack(args: argparse.Namespace) -> None:
    """Rebuild a DataPack5 archive with an uncompressed TOC.

    Lucrezia.exe explicitly supports toc_packed_size == 0 and then reads the
    0x68-byte entries directly, so this avoids implementing the compressor while
    keeping the engine-readable structure.
    """
    src_dir = Path(args.dir)
    manifest_path = src_dir / "manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(f"missing manifest: {manifest_path}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    files = manifest["files"]

    data_base = int(args.data_base, 0) if args.data_base else DATA_BASE_DEFAULT
    body = bytearray()
    entries: list[PakEntry] = []
    for item in files:
        file_path = src_dir / item["output_name"]
        blob = file_path.read_bytes()
        # Original files are 4-byte aligned in SCR.pak; preserve that convention.
        while len(body) % 4:
            body.append(0)
        off = len(body)
        body.extend(blob)
        e = PakEntry(
            index=int(item["index"]),
            name=str(item["name"]),
            offset=off,
            size=len(blob),
            flags0=int(item.get("flags0", 1)),
            flags1=int(item.get("flags1", 1)),
            meta=[int(x) for x in item.get("meta", [0, 0, 0, 0, 0, 0])],
        )
        entries.append(e)

    while len(body) % 4:
        body.append(0)
    table = b"".join(e.to_table_bytes() for e in entries)
    toc_offset = data_base + len(body)

    header_info = manifest.get("header", {})
    magic_raw = bytes.fromhex(header_info.get("magic_raw_hex", MAGIC.hex())).ljust(0x10, b"\0")[:0x10]
    title_raw = bytes.fromhex(header_info.get("title_raw_hex", b"".hex())).ljust(0x20, b"\0")[:0x20]
    if not title_raw.strip(b"\0"):
        title = str(header_info.get("title", "Lucrezia")).encode(DEFAULT_ENCODING, "replace")[:0x1F]
        title_raw = title + b"\0" * (0x20 - len(title))
    version = int(header_info.get("version", 0x00050001))

    header = bytearray(HEADER_SIZE)
    header[0x00:0x10] = magic_raw
    header[0x10:0x30] = title_raw
    struct.pack_into("<I", header, 0x30, version)
    struct.pack_into("<I", header, 0x34, 0)          # raw table mode
    struct.pack_into("<I", header, 0x38, 0)
    struct.pack_into("<I", header, 0x3C, len(entries))
    struct.pack_into("<I", header, 0x40, data_base)
    struct.pack_into("<I", header, 0x44, toc_offset)

    out = bytearray(header)
    if len(out) < data_base:
        out.extend(b"\0" * (data_base - len(out)))
    out.extend(body)
    out.extend(table)
    Path(args.out).write_bytes(out)
    print(f"[pack] files={len(entries)} output={args.out} data_base=0x{data_base:X} toc_offset=0x{toc_offset:X} raw_toc_size=0x{len(table):X}")


def build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Lucrezia DataPack5 PAK unpack/list/pack tool")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("list", help="list archive entries")
    s.add_argument("pak")
    s.set_defaults(func=command_list)

    s = sub.add_parser("unpack", help="unpack archive files and write manifest.json")
    s.add_argument("pak")
    s.add_argument("out")
    s.add_argument("--add-ext", default=None, help="append an extension such as .scw when archive names have no suffix")
    s.set_defaults(func=command_unpack)

    s = sub.add_parser("pack", help="rebuild archive from an unpack directory; writes an uncompressed TOC")
    s.add_argument("dir")
    s.add_argument("out")
    s.add_argument("--data-base", default=None, help="override data base offset, default 0x800")
    s.set_defaults(func=command_pack)
    return p


def main() -> None:
    parser = build_argparser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
