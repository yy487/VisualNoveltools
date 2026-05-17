#!/usr/bin/env python3
# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from d2_arc_common import extract_arc, list_entries


def main() -> int:
    ap = argparse.ArgumentParser(description="Extract Doukyuusei2/SilAI ARC archives.")
    ap.add_argument("input", nargs="+", help="input .ARC file(s)")
    ap.add_argument("output", help="output directory")
    ap.add_argument("--encoding", default="cp932", help="filename encoding after XOR decode, default: cp932")
    ap.add_argument("--list", action="store_true", help="only list entries; do not extract")
    ap.add_argument("--no-overwrite", action="store_true", help="fail if output file exists")
    args = ap.parse_args()

    out_root = Path(args.output)
    multi = len(args.input) > 1
    for src in args.input:
        src_path = Path(src)
        if args.list:
            entries = list_entries(src_path, encoding=args.encoding)
            print(f"{src_path}: {len(entries)} entries")
            for e in entries:
                print(f"  {e.index:04d}  off=0x{e.offset:08X}  size=0x{e.size:08X}  {e.name}")
            continue

        out_dir = out_root / src_path.stem if multi else out_root
        entries = extract_arc(src_path, out_dir, encoding=args.encoding, overwrite=not args.no_overwrite)
        total = sum(e.size for e in entries)
        print(f"extracted {len(entries)} files, {total} bytes: {src_path} -> {out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
