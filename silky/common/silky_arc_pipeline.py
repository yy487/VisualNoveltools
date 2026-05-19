# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from silky_arc_common import extract_archive, pack_archive


def main() -> None:
    ap = argparse.ArgumentParser(description="Silky ARC batch pipeline")
    sub = ap.add_subparsers(dest="cmd", required=True)

    u = sub.add_parser("unpack", help="批量解包目录下的 .arc")
    u.add_argument("arc_dir", type=Path)
    u.add_argument("out_root", type=Path)
    u.add_argument("--format", default="auto", choices=["auto", "silky-lzss", "garbro-fixed"])
    u.add_argument("--encoding", default="cp932")

    p = sub.add_parser("pack", help="批量封包，每个子目录打成同名 .arc")
    p.add_argument("input_root", type=Path)
    p.add_argument("out_dir", type=Path)
    p.add_argument("--format", default="auto", choices=["auto", "silky-lzss", "garbro-fixed"])
    p.add_argument("--encoding", default="cp932")
    p.add_argument("--no-compress", action="store_true")
    p.add_argument("--store-all", action="store_true", help="所有文件都不压缩，适合快速测试")

    args = ap.parse_args()
    if args.cmd == "unpack":
        args.out_root.mkdir(parents=True, exist_ok=True)
        arcs = sorted(args.arc_dir.glob("*.arc"))
        for arc in arcs:
            out = args.out_root / arc.stem
            mf = extract_archive(arc, out, args.format, args.encoding, True)
            print(f"[OK] {arc.name}: {len(mf.entries)} files -> {out}")
    elif args.cmd == "pack":
        args.out_dir.mkdir(parents=True, exist_ok=True)
        dirs = sorted(p for p in args.input_root.iterdir() if p.is_dir())
        for d in dirs:
            out = args.out_dir / f"{d.name}.arc"
            mf = pack_archive(d, out, args.format, args.encoding, not args.no_compress, preserve_packed=not args.store_all)
            print(f"[OK] {d.name}: {len(mf.entries)} files -> {out}")


if __name__ == "__main__":
    main()
