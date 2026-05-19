# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from silky_arc_common import list_archive


def main() -> None:
    ap = argparse.ArgumentParser(description="List Silky ARC archive")
    ap.add_argument("arc", type=Path)
    ap.add_argument("--format", default="auto", choices=["auto", "silky-lzss", "garbro-fixed"])
    ap.add_argument("--encoding", default="cp932")
    args = ap.parse_args()
    manifest = list_archive(args.arc, args.format, args.encoding)
    print(f"format={manifest.format} entries={len(manifest.entries)} encoding={manifest.encoding}")
    for e in manifest.entries:
        flag = "lzss" if e.packed else "raw"
        print(f"{e.index:04d}  off={e.offset:08x}  size={e.size:8d}  out={e.unpacked_size:8d}  {flag:4s}  {e.name}")


if __name__ == "__main__":
    main()
