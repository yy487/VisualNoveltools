# -*- coding: utf-8 -*-
"""Silky ARC 通用读写模块。

支持两类常见 Silky ARC：
1. silky-lzss：用户本次 Script.arc 使用的格式。头部为 header_size，文件名变换，数据可 LZSS 压缩。
2. garbro-fixed：GARbro ArcARC.cs 中的简易格式。头部为 count，固定 0x20 文件名，offset/size 小端，无压缩。

注意：两者都使用 .arc 扩展名，但结构完全不同，不能混用。
"""
from __future__ import annotations

from dataclasses import dataclass, asdict
from pathlib import Path, PurePosixPath
import json
import os
import struct
from typing import Iterable, Literal

from silky_lzss import compress as lzss_compress, decompress as lzss_decompress

FormatName = Literal["silky-lzss", "garbro-fixed"]
MANIFEST_NAME = ".silky_arc_manifest.json"


@dataclass
class ArcEntry:
    name: str
    offset: int
    size: int
    unpacked_size: int
    packed: bool = False
    index: int = 0

    def to_json(self) -> dict:
        return asdict(self)


@dataclass
class ArcManifest:
    format: FormatName
    encoding: str
    entries: list[ArcEntry]

    def to_json(self) -> dict:
        return {
            "format": self.format,
            "encoding": self.encoding,
            "entries": [e.to_json() for e in self.entries],
        }

    @staticmethod
    def from_json(obj: dict) -> "ArcManifest":
        return ArcManifest(
            format=obj["format"],
            encoding=obj.get("encoding", "cp932"),
            entries=[ArcEntry(**e) for e in obj.get("entries", [])],
        )


def _safe_join(root: Path, arc_name: str) -> Path:
    # ARC 内部一般是平面文件名；这里仍做路径净化，防止 ../ 写出目录。
    name = arc_name.replace("\\", "/")
    pure = PurePosixPath(name)
    if pure.is_absolute() or ".." in pure.parts:
        raise ValueError(f"unsafe archive path: {arc_name!r}")
    return root.joinpath(*pure.parts)


def decrypt_name(data: bytes, encoding: str = "cp932") -> str:
    # 引擎逻辑：从尾到头，每个字节加递增 k。
    buf = bytearray(data)
    k = 0
    for i in range(len(buf) - 1, -1, -1):
        k += 1
        buf[i] = (buf[i] + k) & 0xff
    return bytes(buf).decode(encoding)


def encrypt_name(name: str, encoding: str = "cp932") -> bytes:
    raw = bytearray(name.encode(encoding))
    k = 0
    for i in range(len(raw) - 1, -1, -1):
        k += 1
        raw[i] = (raw[i] - k) & 0xff
    if len(raw) > 255:
        raise ValueError(f"file name too long for silky-lzss ARC: {name}")
    return bytes(raw)


def read_manifest(path: Path) -> ArcManifest:
    return ArcManifest.from_json(json.loads(path.read_text("utf-8")))


def write_manifest(manifest: ArcManifest, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / MANIFEST_NAME).write_text(
        json.dumps(manifest.to_json(), ensure_ascii=False, indent=2),
        "utf-8",
    )


def parse_silky_lzss(arc_path: Path, encoding: str = "cp932") -> ArcManifest:
    data = arc_path.read_bytes()
    if len(data) < 4:
        raise ValueError("file too small")
    header_size = struct.unpack_from("<I", data, 0)[0]
    if header_size <= 0 or header_size + 4 > len(data):
        raise ValueError("invalid silky-lzss header size")
    pos = 4
    end = 4 + header_size
    entries: list[ArcEntry] = []
    seen_offsets: set[int] = set()
    while pos < end:
        name_len = data[pos]
        pos += 1
        if name_len == 0 or pos + name_len + 12 > end:
            raise ValueError("invalid silky-lzss name/record length")
        name = decrypt_name(data[pos:pos + name_len], encoding)
        pos += name_len
        size, unpacked_size, offset = struct.unpack_from(">III", data, pos)
        pos += 12
        if offset < end or offset + size > len(data):
            raise ValueError(f"invalid placement for {name}: off={offset}, size={size}")
        if offset in seen_offsets:
            raise ValueError(f"duplicated offset: {offset}")
        seen_offsets.add(offset)
        entries.append(ArcEntry(name, offset, size, unpacked_size, size != unpacked_size, len(entries)))
    if pos != end:
        raise ValueError("silky-lzss header parse did not end exactly")
    return ArcManifest("silky-lzss", encoding, entries)


def parse_garbro_fixed(arc_path: Path, encoding: str = "cp932") -> ArcManifest:
    data = arc_path.read_bytes()
    if len(data) < 4:
        raise ValueError("file too small")
    count = struct.unpack_from("<I", data, 0)[0]
    if count <= 0 or count > 200000:
        raise ValueError("invalid garbro-fixed entry count")
    record_size = 0x20 + 8
    index_end = 4 + count * record_size
    if index_end > len(data):
        raise ValueError("garbro-fixed index exceeds file size")
    entries: list[ArcEntry] = []
    seen_offsets: set[int] = set()
    pos = 4
    for i in range(count):
        raw_name = data[pos:pos + 0x20]
        pos += 0x20
        name = raw_name.split(b"\x00", 1)[0].decode(encoding)
        if not name:
            raise ValueError("empty garbro-fixed name")
        offset, size = struct.unpack_from("<II", data, pos)
        pos += 8
        if offset < index_end or offset + size > len(data):
            raise ValueError(f"invalid placement for {name}: off={offset}, size={size}")
        if offset in seen_offsets:
            raise ValueError(f"duplicated offset: {offset}")
        seen_offsets.add(offset)
        entries.append(ArcEntry(name, offset, size, size, False, i))
    return ArcManifest("garbro-fixed", encoding, entries)


def detect_format(arc_path: Path, encoding: str = "cp932") -> ArcManifest:
    errors = []
    for parser in (parse_silky_lzss, parse_garbro_fixed):
        try:
            return parser(arc_path, encoding)
        except Exception as exc:  # noqa: BLE001 - 收集检测失败原因，方便调试。
            errors.append(f"{parser.__name__}: {exc}")
    raise ValueError("unsupported ARC format:\n" + "\n".join(errors))


def parse_archive(arc_path: Path, fmt: str = "auto", encoding: str = "cp932") -> ArcManifest:
    if fmt == "auto":
        return detect_format(arc_path, encoding)
    if fmt == "silky-lzss":
        return parse_silky_lzss(arc_path, encoding)
    if fmt == "garbro-fixed":
        return parse_garbro_fixed(arc_path, encoding)
    raise ValueError(f"unknown format: {fmt}")


def extract_archive(arc_path: Path, out_dir: Path, fmt: str = "auto", encoding: str = "cp932",
                    write_meta: bool = True) -> ArcManifest:
    manifest = parse_archive(arc_path, fmt, encoding)
    data = arc_path.read_bytes()
    out_dir.mkdir(parents=True, exist_ok=True)
    for entry in manifest.entries:
        blob = data[entry.offset:entry.offset + entry.size]
        if manifest.format == "silky-lzss" and entry.packed:
            blob = lzss_decompress(blob)
            if len(blob) != entry.unpacked_size:
                raise ValueError(f"LZSS size mismatch for {entry.name}: {len(blob)} != {entry.unpacked_size}")
        out_path = _safe_join(out_dir, entry.name)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(blob)
    if write_meta:
        write_manifest(manifest, out_dir)
    return manifest


def _iter_files_by_manifest(input_dir: Path, manifest: ArcManifest) -> Iterable[tuple[str, Path, ArcEntry | None]]:
    used: set[Path] = set()
    for entry in manifest.entries:
        path = _safe_join(input_dir, entry.name)
        if not path.is_file():
            raise FileNotFoundError(f"missing file for manifest entry: {entry.name}")
        used.add(path.resolve())
        yield entry.name, path, entry

    # 允许追加新文件：按路径排序追加到末尾。
    for path in sorted(p for p in input_dir.rglob("*") if p.is_file() and p.name != MANIFEST_NAME):
        if path.resolve() in used:
            continue
        rel = path.relative_to(input_dir).as_posix()
        yield rel, path, None


def _iter_files_sorted(input_dir: Path) -> Iterable[tuple[str, Path, ArcEntry | None]]:
    for path in sorted(p for p in input_dir.rglob("*") if p.is_file() and p.name != MANIFEST_NAME):
        yield path.relative_to(input_dir).as_posix(), path, None


def pack_silky_lzss(input_dir: Path, out_arc: Path, manifest: ArcManifest | None = None,
                    encoding: str = "cp932", compress: bool = True, preserve_packed: bool = True) -> ArcManifest:
    records: list[tuple[str, bytes, bytes, int]] = []
    iterator = _iter_files_by_manifest(input_dir, manifest) if manifest else _iter_files_sorted(input_dir)
    for name, path, old_entry in iterator:
        raw = path.read_bytes()
        # 默认 preserve_packed=True：原来压缩的条目继续压缩；新增文件由 compress 控制。
        # 若 preserve_packed=False 且 compress=False，则所有文件直接存储，速度最快，体积较大。
        should_pack = (old_entry.packed if old_entry else compress) if preserve_packed else compress
        stored = lzss_compress(raw) if should_pack else raw
        records.append((name, encrypt_name(name, encoding), stored, len(raw)))
    header_size = sum(1 + len(enc_name) + 12 for _, enc_name, _, _ in records)
    offset = 4 + header_size
    entries: list[ArcEntry] = []
    for idx, (name, _, stored, raw_size) in enumerate(records):
        entries.append(ArcEntry(name, offset, len(stored), raw_size, len(stored) != raw_size, idx))
        offset += len(stored)
    out_arc.parent.mkdir(parents=True, exist_ok=True)
    with out_arc.open("wb") as f:
        f.write(struct.pack("<I", header_size))
        for entry, (_, enc_name, _, _) in zip(entries, records):
            f.write(struct.pack("B", len(enc_name)))
            f.write(enc_name)
            f.write(struct.pack(">III", entry.size, entry.unpacked_size, entry.offset))
        for _, _, stored, _ in records:
            f.write(stored)
    return ArcManifest("silky-lzss", encoding, entries)


def pack_garbro_fixed(input_dir: Path, out_arc: Path, manifest: ArcManifest | None = None,
                      encoding: str = "cp932") -> ArcManifest:
    records: list[tuple[str, bytes]] = []
    iterator = _iter_files_by_manifest(input_dir, manifest) if manifest else _iter_files_sorted(input_dir)
    for name, path, _old_entry in iterator:
        name_bytes = name.encode(encoding)
        if len(name_bytes) > 0x1f:
            raise ValueError(f"name too long for garbro-fixed ARC: {name}")
        records.append((name, path.read_bytes()))
    count = len(records)
    offset = 4 + count * (0x20 + 8)
    entries: list[ArcEntry] = []
    for idx, (name, blob) in enumerate(records):
        entries.append(ArcEntry(name, offset, len(blob), len(blob), False, idx))
        offset += len(blob)
    out_arc.parent.mkdir(parents=True, exist_ok=True)
    with out_arc.open("wb") as f:
        f.write(struct.pack("<I", count))
        for entry in entries:
            raw = entry.name.encode(encoding)
            f.write(raw + b"\x00" * (0x20 - len(raw)))
            f.write(struct.pack("<II", entry.offset, entry.size))
        for _, blob in records:
            f.write(blob)
    return ArcManifest("garbro-fixed", encoding, entries)


def pack_archive(input_dir: Path, out_arc: Path, fmt: str = "auto", encoding: str = "cp932",
                 compress: bool = True, manifest_path: Path | None = None, preserve_packed: bool = True) -> ArcManifest:
    manifest = None
    if manifest_path is None and (input_dir / MANIFEST_NAME).is_file():
        manifest_path = input_dir / MANIFEST_NAME
    if manifest_path and manifest_path.is_file():
        manifest = read_manifest(manifest_path)
        if fmt == "auto":
            fmt = manifest.format
        encoding = manifest.encoding or encoding
    if fmt == "auto":
        fmt = "silky-lzss"
    if fmt == "silky-lzss":
        return pack_silky_lzss(input_dir, out_arc, manifest, encoding, compress, preserve_packed)
    if fmt == "garbro-fixed":
        return pack_garbro_fixed(input_dir, out_arc, manifest, encoding)
    raise ValueError(f"unknown format: {fmt}")


def list_archive(arc_path: Path, fmt: str = "auto", encoding: str = "cp932") -> ArcManifest:
    return parse_archive(arc_path, fmt, encoding)
