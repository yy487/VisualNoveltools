# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from .iga_common import MANIFEST_NAME, parse_archive, safe_join, write_manifest, xor_entry_data


def unpack_archive(archive_path: Path, output_dir: Path, manifest_name: str = MANIFEST_NAME) -> None:
    data = archive_path.read_bytes()
    header, entries, _names_start, _names_length, data_offset = parse_archive(data)
    output_dir.mkdir(parents=True, exist_ok=True)

    for entry in entries:
        abs_off = data_offset + entry.offset
        stored = data[abs_off:abs_off + entry.size]
        clear = xor_entry_data(entry.name, stored)
        out_path = safe_join(output_dir, entry.name)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(clear)

    write_manifest(output_dir / manifest_name, archive_path, header, entries, data_offset)
    print(f"[noesis unpack] archive={archive_path}")
    print(f"[noesis unpack] files={len(entries)} output={output_dir}")
    print(f"[noesis unpack] manifest={output_dir / manifest_name}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Unpack Noesis IGA0 archive and decrypt entries")
    ap.add_argument("archive", help="input .iga archive, e.g. script.iga")
    ap.add_argument("output_dir", help="output directory")
    ap.add_argument("--manifest-name", default=MANIFEST_NAME, help="manifest filename written inside output_dir")
    args = ap.parse_args()
    unpack_archive(Path(args.archive), Path(args.output_dir), args.manifest_name)


if __name__ == "__main__":
    main()
