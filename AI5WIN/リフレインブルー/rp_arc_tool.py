# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path
from rp_arc import read_arc, unpack_arc, write_arc, load_dir_entries


def main() -> None:
    ap = argparse.ArgumentParser(description="Unpack/pack Refrain Blue MES.ARC")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("list", help="list archive entries")
    p.add_argument("arc")

    p = sub.add_parser("unpack", help="unpack MES.ARC")
    p.add_argument("arc")
    p.add_argument("out_dir")

    p = sub.add_parser("pack", help="pack directory into MES.ARC")
    p.add_argument("input_dir")
    p.add_argument("out_arc")
    p.add_argument("--base", default="", help="optional original ARC to preserve entry order")

    args = ap.parse_args()
    if args.cmd == "list":
        entries = read_arc(args.arc)
        print(json.dumps([
            {"name": e.name, "offset": e.offset, "size": e.size}
            for e in entries
        ], ensure_ascii=False, indent=2))
    elif args.cmd == "unpack":
        entries = unpack_arc(args.arc, args.out_dir)
        print(f"[unpack] {args.arc} -> {args.out_dir}, files={len(entries)}")
    elif args.cmd == "pack":
        order = None
        if args.base:
            order = [e.name for e in read_arc(args.base)]
        entries = load_dir_entries(args.input_dir, order)
        write_arc(args.out_arc, entries)
        print(f"[pack] {args.input_dir} -> {args.out_arc}, files={len(entries)}")


if __name__ == "__main__":
    main()
