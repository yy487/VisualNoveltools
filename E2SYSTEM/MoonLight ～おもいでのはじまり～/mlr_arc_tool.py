#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
MLR ARC archive unpack/repack tool.

Format confirmed from sound.arc and mlr.exe decompile:
  header:  magic u32('ARC\x1A'), version u32, file_count u32, default_ext[32]
  entry :  offset u32, size u32, name[24]
  data  :  raw file bytes, usually contiguous after the table

The tool keeps a manifest on unpack so repacking can preserve entry order and
header metadata exactly. Packed archives are rebuilt with contiguous data.
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
from typing import Any, Iterable

MAGIC = b"ARC\x1A"
HEADER_STRUCT = struct.Struct("<4sII32s")
ENTRY_STRUCT = struct.Struct("<II24s")
HEADER_SIZE = HEADER_STRUCT.size  # 44 / 0x2C
ENTRY_SIZE = ENTRY_STRUCT.size    # 32 / 0x20
MANIFEST_NAME = "_arc_manifest.json"
DEFAULT_ENCODING = "cp932"


class ArcError(Exception):
    pass


@dataclass
class ArcEntry:
    index: int
    name: str
    offset: int
    size: int
    sha256: str | None = None

    @property
    def end(self) -> int:
        return self.offset + self.size


@dataclass
class ArcArchive:
    path: Path
    version: int
    default_ext: str
    entries: list[ArcEntry]
    data: bytes


def decode_fixed_string(raw: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    raw = raw.split(b"\x00", 1)[0]
    return raw.decode(encoding)


def encode_fixed_string(text: str, size: int, field_name: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    try:
        raw = text.encode(encoding)
    except UnicodeEncodeError as exc:
        raise ArcError(f"{field_name} cannot be encoded as {encoding}: {text!r}") from exc
    if len(raw) >= size:
        # The game uses lstrcmpA/lstrcpyA-style C strings, so reserve one NUL byte.
        raise ArcError(f"{field_name} is too long: {text!r} encodes to {len(raw)} bytes, max is {size - 1}")
    return raw + b"\x00" * (size - len(raw))


def safe_output_path(root: Path, name: str) -> Path:
    # ARC names observed here are flat filenames. This keeps the tool safe even if
    # a malformed archive contains '../' or absolute paths.
    normalized = name.replace("\\", "/")
    if normalized.startswith("/") or any(part in ("", ".", "..") for part in normalized.split("/")):
        raise ArcError(f"unsafe entry name: {name!r}")
    out = (root / normalized).resolve()
    root_resolved = root.resolve()
    try:
        out.relative_to(root_resolved)
    except ValueError as exc:
        raise ArcError(f"unsafe entry path escapes output directory: {name!r}") from exc
    return out


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def read_arc(path: Path, encoding: str = DEFAULT_ENCODING, strict: bool = True) -> ArcArchive:
    data = path.read_bytes()
    if len(data) < HEADER_SIZE:
        raise ArcError(f"file too small for ARC header: {path}")

    magic, version, count, default_ext_raw = HEADER_STRUCT.unpack_from(data, 0)
    if magic != MAGIC:
        raise ArcError(f"bad magic: expected {MAGIC!r}, got {magic!r}")

    table_end = HEADER_SIZE + count * ENTRY_SIZE
    if table_end > len(data):
        raise ArcError(f"file table exceeds archive size: table_end=0x{table_end:X}, size=0x{len(data):X}")

    default_ext = decode_fixed_string(default_ext_raw, encoding)
    entries: list[ArcEntry] = []
    occupied: list[tuple[int, int, int, str]] = []
    for i in range(count):
        entry_off = HEADER_SIZE + i * ENTRY_SIZE
        offset, size, name_raw = ENTRY_STRUCT.unpack_from(data, entry_off)
        name = decode_fixed_string(name_raw, encoding)
        if not name:
            raise ArcError(f"empty filename at entry {i}")
        end = offset + size
        if offset < table_end:
            # The sample starts exactly at table_end. Let non-strict mode inspect odd archives,
            # but strict mode rejects table/data overlap.
            msg = f"entry {i} {name!r} overlaps header/table: offset=0x{offset:X}, table_end=0x{table_end:X}"
            if strict:
                raise ArcError(msg)
            print(f"[warn] {msg}", file=sys.stderr)
        if end > len(data) or end < offset:
            raise ArcError(f"entry {i} {name!r} outside archive: off=0x{offset:X}, size=0x{size:X}, archive=0x{len(data):X}")
        entries.append(ArcEntry(i, name, offset, size, sha256_bytes(data[offset:end])))
        occupied.append((offset, end, i, name))

    # Warn about overlapping data regions. Some archive variants might allow holes,
    # but overlap would make extraction ambiguous.
    occupied.sort()
    prev_end = table_end
    for offset, end, i, name in occupied:
        if offset < prev_end and strict:
            # table overlap is already checked above; this catches entry-entry overlap.
            raise ArcError(f"entry data overlap around entry {i} {name!r}: offset=0x{offset:X}, previous_end=0x{prev_end:X}")
        prev_end = max(prev_end, end)

    return ArcArchive(path=path, version=version, default_ext=default_ext, entries=entries, data=data)


def make_manifest(arc: ArcArchive) -> dict[str, Any]:
    return {
        "format": "MLR_ARC",
        "magic": "ARC\\x1A",
        "version": arc.version,
        "default_ext": arc.default_ext,
        "encoding": DEFAULT_ENCODING,
        "entry_struct": "<u32 offset><u32 size><char name[24]>",
        "entries": [asdict(e) for e in arc.entries],
    }


def write_manifest(path: Path, manifest: dict[str, Any]) -> None:
    path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8", newline="\n")


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        obj = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ArcError(f"manifest not found: {path}") from exc
    if not isinstance(obj, dict) or obj.get("format") != "MLR_ARC":
        raise ArcError(f"not an MLR_ARC manifest: {path}")
    entries = obj.get("entries")
    if not isinstance(entries, list):
        raise ArcError(f"manifest entries must be a list: {path}")
    return obj


def unpack_arc(arc_path: Path, out_dir: Path, encoding: str = DEFAULT_ENCODING, strict: bool = True, overwrite: bool = False) -> None:
    arc = read_arc(arc_path, encoding=encoding, strict=strict)
    out_dir.mkdir(parents=True, exist_ok=True)

    extracted = 0
    for entry in arc.entries:
        out_path = safe_output_path(out_dir, entry.name)
        if out_path.exists() and not overwrite:
            raise ArcError(f"output exists, use --overwrite to replace: {out_path}")
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(arc.data[entry.offset:entry.end])
        extracted += 1
        print(f"[unpack] {entry.index:04d} off=0x{entry.offset:08X} size=0x{entry.size:08X} {entry.name}")

    manifest = make_manifest(arc)
    manifest["source_arc"] = arc_path.name
    manifest["archive_size"] = len(arc.data)
    manifest["archive_sha256"] = sha256_bytes(arc.data)
    write_manifest(out_dir / MANIFEST_NAME, manifest)
    print(f"[unpack] files={extracted} output={out_dir}")
    print(f"[unpack] manifest={out_dir / MANIFEST_NAME}")


def iter_input_files_without_manifest(in_dir: Path) -> list[Path]:
    files: list[Path] = []
    for p in in_dir.rglob("*"):
        if not p.is_file():
            continue
        if p.name == MANIFEST_NAME:
            continue
        files.append(p)
    return sorted(files, key=lambda p: p.relative_to(in_dir).as_posix().lower())


def entries_from_manifest(manifest: dict[str, Any]) -> list[str]:
    result: list[str] = []
    for i, item in enumerate(manifest["entries"]):
        if not isinstance(item, dict) or not isinstance(item.get("name"), str):
            raise ArcError(f"bad manifest entry at index {i}")
        result.append(item["name"])
    return result


def guess_default_ext(names: Iterable[str]) -> str:
    suffixes = [Path(name).suffix for name in names if Path(name).suffix]
    if suffixes and all(s.lower() == suffixes[0].lower() for s in suffixes):
        return suffixes[0]
    return ""


def pack_arc(
    in_dir: Path,
    arc_path: Path,
    encoding: str = DEFAULT_ENCODING,
    manifest_path: Path | None = None,
    version: int | None = None,
    default_ext: str | None = None,
    overwrite: bool = False,
    verify_hash: bool = False,
) -> None:
    if not in_dir.is_dir():
        raise ArcError(f"input directory not found: {in_dir}")
    if arc_path.exists() and not overwrite:
        raise ArcError(f"output exists, use --overwrite to replace: {arc_path}")

    if manifest_path is None:
        candidate = in_dir / MANIFEST_NAME
        manifest_path = candidate if candidate.exists() else None

    manifest: dict[str, Any] | None = None
    if manifest_path is not None:
        manifest = load_manifest(manifest_path)
        names = entries_from_manifest(manifest)
        if version is None:
            version = int(manifest.get("version", 1))
        if default_ext is None:
            default_ext = str(manifest.get("default_ext", ""))
    else:
        files = iter_input_files_without_manifest(in_dir)
        names = [p.relative_to(in_dir).as_posix() for p in files]
        if version is None:
            version = 1
        if default_ext is None:
            default_ext = guess_default_ext(names)

    if len(names) >= 0x100000000:
        raise ArcError("too many entries for u32 count")

    table_end = HEADER_SIZE + len(names) * ENTRY_SIZE
    offset = table_end
    table = bytearray()
    data_chunks: list[bytes] = []
    new_entries: list[ArcEntry] = []

    old_by_name: dict[str, dict[str, Any]] = {}
    if manifest:
        old_by_name = {e["name"]: e for e in manifest["entries"] if isinstance(e, dict) and isinstance(e.get("name"), str)}

    for index, name in enumerate(names):
        path = safe_output_path(in_dir, name)
        if not path.is_file():
            raise ArcError(f"missing file for entry {index}: {name}")
        payload = path.read_bytes()
        if verify_hash and manifest:
            old_sha = old_by_name.get(name, {}).get("sha256")
            if isinstance(old_sha, str) and old_sha != sha256_bytes(payload):
                print(f"[pack][warn] content changed from manifest: {name}", file=sys.stderr)
        size = len(payload)
        if offset + size >= 0x100000000:
            raise ArcError(f"archive too large for u32 offsets at entry {index}: {name}")
        name_raw = encode_fixed_string(name, 24, "entry name", encoding)
        table += ENTRY_STRUCT.pack(offset, size, name_raw)
        data_chunks.append(payload)
        new_entries.append(ArcEntry(index, name, offset, size, sha256_bytes(payload)))
        print(f"[pack] {index:04d} off=0x{offset:08X} size=0x{size:08X} {name}")
        offset += size

    header = HEADER_STRUCT.pack(MAGIC, int(version), len(names), encode_fixed_string(default_ext or "", 32, "default_ext", encoding))
    out_data = header + bytes(table) + b"".join(data_chunks)
    arc_path.parent.mkdir(parents=True, exist_ok=True)
    arc_path.write_bytes(out_data)

    out_manifest = {
        "format": "MLR_ARC",
        "magic": "ARC\\x1A",
        "version": int(version),
        "default_ext": default_ext or "",
        "encoding": encoding,
        "entry_struct": "<u32 offset><u32 size><char name[24]>",
        "entries": [asdict(e) for e in new_entries],
        "archive_size": len(out_data),
        "archive_sha256": sha256_bytes(out_data),
    }
    write_manifest(arc_path.with_suffix(arc_path.suffix + ".manifest.json"), out_manifest)
    print(f"[pack] files={len(names)} output={arc_path} size=0x{len(out_data):X}")


def list_arc(arc_path: Path, encoding: str = DEFAULT_ENCODING, strict: bool = True, json_out: bool = False) -> None:
    arc = read_arc(arc_path, encoding=encoding, strict=strict)
    manifest = make_manifest(arc)
    manifest["archive_size"] = len(arc.data)
    manifest["archive_sha256"] = sha256_bytes(arc.data)
    if json_out:
        print(json.dumps(manifest, ensure_ascii=False, indent=2))
        return
    print(f"path={arc.path}")
    print(f"magic=ARC\\x1A version={arc.version} count={len(arc.entries)} default_ext={arc.default_ext!r}")
    print(f"table_end=0x{HEADER_SIZE + len(arc.entries) * ENTRY_SIZE:X} archive_size=0x{len(arc.data):X}")
    for e in arc.entries:
        print(f"{e.index:04d} off=0x{e.offset:08X} size=0x{e.size:08X} end=0x{e.end:08X} sha256={e.sha256[:16]} {e.name}")


def verify_arc(arc_path: Path, in_dir: Path, encoding: str = DEFAULT_ENCODING, strict: bool = True) -> int:
    arc = read_arc(arc_path, encoding=encoding, strict=strict)
    failed = 0
    for entry in arc.entries:
        path = safe_output_path(in_dir, entry.name)
        if not path.is_file():
            print(f"[verify][missing] {entry.name}")
            failed += 1
            continue
        file_data = path.read_bytes()
        arc_data = arc.data[entry.offset:entry.end]
        if file_data != arc_data:
            print(f"[verify][diff] {entry.name} arc_size={len(arc_data)} file_size={len(file_data)}")
            failed += 1
        else:
            print(f"[verify][ok] {entry.name}")
    print(f"[verify] files={len(arc.entries)} failed={failed}")
    return failed


def build_argparser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="MLR ARC unpack/repack tool")
    parser.add_argument("--encoding", default=DEFAULT_ENCODING, help="filename encoding, default: cp932")
    parser.add_argument("--non-strict", action="store_true", help="allow unusual table/data layout and print warnings")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="list archive entries")
    p_list.add_argument("arc", type=Path)
    p_list.add_argument("--json", action="store_true", help="print manifest-like JSON")

    p_unpack = sub.add_parser("unpack", help="extract archive files")
    p_unpack.add_argument("arc", type=Path)
    p_unpack.add_argument("out_dir", type=Path)
    p_unpack.add_argument("--overwrite", action="store_true")

    p_pack = sub.add_parser("pack", help="rebuild an archive from a directory")
    p_pack.add_argument("in_dir", type=Path)
    p_pack.add_argument("arc", type=Path)
    p_pack.add_argument("--manifest", type=Path, default=None, help=f"manifest path, default: in_dir/{MANIFEST_NAME} if present")
    p_pack.add_argument("--version", type=int, default=None, help="archive version, default: manifest version or 1")
    p_pack.add_argument("--ext", dest="default_ext", default=None, help="default extension in header, default: manifest value or guessed suffix")
    p_pack.add_argument("--overwrite", action="store_true")
    p_pack.add_argument("--verify-hash", action="store_true", help="warn when files differ from manifest sha256")

    p_verify = sub.add_parser("verify", help="compare archive entries with files in a directory")
    p_verify.add_argument("arc", type=Path)
    p_verify.add_argument("in_dir", type=Path)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_argparser()
    args = parser.parse_args(argv)
    strict = not args.non_strict
    try:
        if args.cmd == "list":
            list_arc(args.arc, encoding=args.encoding, strict=strict, json_out=args.json)
        elif args.cmd == "unpack":
            unpack_arc(args.arc, args.out_dir, encoding=args.encoding, strict=strict, overwrite=args.overwrite)
        elif args.cmd == "pack":
            pack_arc(
                args.in_dir,
                args.arc,
                encoding=args.encoding,
                manifest_path=args.manifest,
                version=args.version,
                default_ext=args.default_ext,
                overwrite=args.overwrite,
                verify_hash=args.verify_hash,
            )
        elif args.cmd == "verify":
            return 1 if verify_arc(args.arc, args.in_dir, encoding=args.encoding, strict=strict) else 0
        else:
            parser.error("unknown command")
    except ArcError as exc:
        print(f"[error] {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
