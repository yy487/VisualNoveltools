#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Doukyuusei2 / SilAI ARC archive common routines.

ARC layout observed from Doukyuusei2.EXE.c:
  u32le count
  count * 0x14 encrypted directory entries
  raw file data concatenated at stored offsets

Directory entry after decryption:
  char name[12]      # NUL padded, usually ASCII 8.3
  u32le size
  u32le offset

Encryption in game code:
  name bytes xor 0x55
  size      xor 0xAA55AA55
  offset    xor 0x55AA55AA
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json
import os
import struct
from typing import Iterable, List

ENTRY_SIZE = 0x14
NAME_SIZE = 12
NAME_XOR = 0x55
SIZE_XOR = 0xAA55AA55
OFFSET_XOR = 0x55AA55AA


@dataclass
class ArcEntry:
    index: int
    name: str
    size: int
    offset: int
    # 解密后的 12 字节 name 字段原貌。样本中 NUL 之后并不全为 0，
    # 例如 b"MAIN.MES\0\x055\0"；保留它可做到未修改文件的 bit-exact 重封包。
    name_field: bytes | None = None

    @property
    def end(self) -> int:
        return self.offset + self.size


def _decode_name(raw: bytes, encoding: str = "cp932") -> str:
    dec = bytes(b ^ NAME_XOR for b in raw[:NAME_SIZE])
    dec = dec.split(b"\x00", 1)[0]
    # 实测样本为 ASCII 8.3；保留 cp932 以兼容日文名的可能性。
    return dec.decode(encoding, errors="replace")


def _encode_name(name: str, encoding: str = "cp932", name_field: bytes | None = None) -> bytes:
    if name_field is not None:
        if len(name_field) != NAME_SIZE:
            raise ValueError(f"name_field must be exactly {NAME_SIZE} bytes")
        # name_field 是“解密后的 12 字节原始字段”，这里重新加密写回。
        return bytes(b ^ NAME_XOR for b in name_field)
    raw = name.encode(encoding)
    if len(raw) > NAME_SIZE:
        raise ValueError(f"ARC file name too long for 12-byte field: {name!r} ({len(raw)} bytes)")
    raw = raw.ljust(NAME_SIZE, b"\x00")
    return bytes(b ^ NAME_XOR for b in raw)


def parse_arc(data: bytes, *, encoding: str = "cp932", strict: bool = True) -> List[ArcEntry]:
    if len(data) < 4:
        raise ValueError("file too small: missing ARC count")
    count = struct.unpack_from("<I", data, 0)[0]
    dir_end = 4 + count * ENTRY_SIZE
    if dir_end > len(data):
        raise ValueError(f"directory exceeds file size: count={count}, dir_end={dir_end}, size={len(data)}")

    entries: List[ArcEntry] = []
    for i in range(count):
        base = 4 + i * ENTRY_SIZE
        raw_name = data[base : base + NAME_SIZE]
        enc_size, enc_offset = struct.unpack_from("<II", data, base + NAME_SIZE)
        name_field = bytes(b ^ NAME_XOR for b in raw_name[:NAME_SIZE])
        name = name_field.split(b"\x00", 1)[0].decode(encoding, errors="replace")
        size = enc_size ^ SIZE_XOR
        offset = enc_offset ^ OFFSET_XOR
        ent = ArcEntry(i, name, size, offset, name_field)
        if strict:
            if offset < dir_end:
                raise ValueError(f"entry {i} {name}: offset {offset} overlaps directory ending at {dir_end}")
            if ent.end > len(data):
                raise ValueError(f"entry {i} {name}: end {ent.end} exceeds file size {len(data)}")
        entries.append(ent)
    return entries


def read_arc(path: os.PathLike[str] | str, *, encoding: str = "cp932", strict: bool = True) -> tuple[bytes, List[ArcEntry]]:
    data = Path(path).read_bytes()
    return data, parse_arc(data, encoding=encoding, strict=strict)


def safe_output_path(root: Path, arc_name: str) -> Path:
    # ARC 内一般是 8.3 文件名。这里仍然防止路径穿越。
    name = arc_name.replace("\\", "/").split("/")[-1]
    if name in ("", ".", ".."):
        raise ValueError(f"unsafe output name: {arc_name!r}")
    return root / name


def extract_arc(arc_path: os.PathLike[str] | str, out_dir: os.PathLike[str] | str, *, encoding: str = "cp932", overwrite: bool = True) -> List[ArcEntry]:
    arc_path = Path(arc_path)
    out_dir = Path(out_dir)
    data, entries = read_arc(arc_path, encoding=encoding, strict=True)
    out_dir.mkdir(parents=True, exist_ok=True)

    manifest = []
    for ent in entries:
        dst = safe_output_path(out_dir, ent.name)
        if dst.exists() and not overwrite:
            raise FileExistsError(dst)
        dst.write_bytes(data[ent.offset : ent.end])
        manifest.append({
            "index": ent.index,
            "name": ent.name,
            "size": ent.size,
            "offset": ent.offset,
            "name_field_hex": ent.name_field.hex() if ent.name_field is not None else None,
        })

    (out_dir / "_arc_manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    return entries


def build_arc_from_manifest(src_dir: os.PathLike[str] | str, out_arc: os.PathLike[str] | str, *, encoding: str = "cp932") -> List[ArcEntry]:
    """Pack files according to _arc_manifest.json order.

    This is intended for round-trip rebuilding after extracted files are edited.
    It does not compress data and does not align file bodies, matching the sample ARC.
    """
    src_dir = Path(src_dir)
    manifest_path = src_dir / "_arc_manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(f"missing manifest: {manifest_path}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, list):
        raise ValueError("manifest must be a list")

    count = len(manifest)
    offset = 4 + count * ENTRY_SIZE
    entries: List[ArcEntry] = []
    bodies: list[bytes] = []

    for i, item in enumerate(manifest):
        name = item["name"]
        path = safe_output_path(src_dir, name)
        body = path.read_bytes()
        name_field_hex = item.get("name_field_hex")
        name_field = bytes.fromhex(name_field_hex) if name_field_hex else None
        # 如果用户改了 manifest 中的 name，则不复用旧 name_field，避免字段名和实体文件名不一致。
        if name_field is not None:
            old_name = name_field.split(b"\x00", 1)[0].decode(encoding, errors="replace")
            if old_name != name:
                name_field = None
        entries.append(ArcEntry(i, name, len(body), offset, name_field))
        bodies.append(body)
        offset += len(body)

    header = bytearray()
    header += struct.pack("<I", count)
    for ent in entries:
        header += _encode_name(ent.name, encoding=encoding, name_field=ent.name_field)
        header += struct.pack("<I", ent.size ^ SIZE_XOR)
        header += struct.pack("<I", ent.offset ^ OFFSET_XOR)

    Path(out_arc).write_bytes(bytes(header) + b"".join(bodies))
    return entries


def list_entries(path: os.PathLike[str] | str, *, encoding: str = "cp932") -> List[ArcEntry]:
    _, entries = read_arc(path, encoding=encoding, strict=True)
    return entries
