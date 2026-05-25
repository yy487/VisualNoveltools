# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from .iga_common import (
    HEADER_SIZE,
    MAGIC,
    MANIFEST_NAME,
    IgaEntry,
    build_index_and_names,
    read_manifest,
    safe_join,
    write_packed_uint,
    xor_entry_data,
)


def _header_from_manifest(manifest: dict) -> bytes:
    header_hex = manifest.get("header_hex")
    if isinstance(header_hex, str):
        header = bytes.fromhex(header_hex)
        if len(header) == HEADER_SIZE and header[:4] == MAGIC:
            return header
    return MAGIC + b"\x00" * (HEADER_SIZE - 4)


def pack_archive(input_dir: Path, output_archive: Path, manifest_path: Path) -> None:
    manifest = read_manifest(manifest_path)
    header = _header_from_manifest(manifest)

    entries_in = manifest.get("entries", [])
    entries: list[IgaEntry] = []
    data_blob = bytearray()

    for i, item in enumerate(entries_in):
        name = item.get("name")
        if not isinstance(name, str) or not name:
            raise ValueError(f"invalid entry name in manifest index {i}: {item!r}")
        file_path = safe_join(input_dir, name)
        if not file_path.is_file():
            raise FileNotFoundError(f"missing entry file: {file_path}")
        clear = file_path.read_bytes()
        stored = xor_entry_data(name, clear)
        rel_offset = len(data_blob)
        data_blob += stored
        entries.append(IgaEntry(name=name, offset=rel_offset, size=len(stored), index=i))

    index_body, names_blob, rebuilt_entries = build_index_and_names(entries)
    out = bytearray(header)
    out += write_packed_uint(len(index_body))
    out += index_body
    out += write_packed_uint(len(names_blob))
    out += names_blob
    out += data_blob

    output_archive.parent.mkdir(parents=True, exist_ok=True)
    output_archive.write_bytes(out)
    print(f"[noesis pack] input={input_dir}")
    print(f"[noesis pack] files={len(rebuilt_entries)} output={output_archive}")
    print(f"[noesis pack] size={len(out)}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Pack Noesis IGA0 archive and encrypt entries")
    ap.add_argument("input_dir", help="directory produced by unpack")
    ap.add_argument("output_archive", help="output .iga archive")
    ap.add_argument("--manifest", default=None, help=f"manifest path, default input_dir/{MANIFEST_NAME}")
    args = ap.parse_args()
    input_dir = Path(args.input_dir)
    manifest_path = Path(args.manifest) if args.manifest else input_dir / MANIFEST_NAME
    pack_archive(input_dir, Path(args.output_archive), manifest_path)


if __name__ == "__main__":
    main()
