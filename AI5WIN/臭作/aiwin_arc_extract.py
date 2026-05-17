#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""AIWIN/旧 ELF 系 ARC 解包器。"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from aiwin_arc_common import ArcFormatError, iter_arc_files, parse_arc, safe_output_path


def cmd_list(args: argparse.Namespace) -> None:
    data = Path(args.arc).read_bytes()
    entries = parse_arc(data)
    rows = []
    for e in entries:
        rows.append(
            {
                "index": e.index,
                "name": e.name,
                "offset": e.offset,
                "packed_size": e.packed_size,
                "auto_mode": "decompress" if e.should_decompress_by_name else "raw",
            }
        )
    if args.json:
        print(json.dumps(rows, ensure_ascii=False, indent=2))
    else:
        print(f"entries: {len(entries)}")
        for e in entries:
            auto = "decompress" if e.should_decompress_by_name else "raw"
            print(
                f"[{e.index:03d}] {e.name:<16} offset=0x{e.offset:08X} "
                f"size=0x{e.packed_size:08X} ({e.packed_size}) auto={auto}"
            )


def cmd_extract(args: argparse.Namespace) -> None:
    arc_path = Path(args.arc)
    out_dir = Path(args.out_dir)
    data = arc_path.read_bytes()

    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = []
    total_out = 0
    mode_count = {"raw": 0, "decompressed": 0}

    for entry, payload, actual_mode in iter_arc_files(data, mode=args.mode):
        out_path = safe_output_path(out_dir, entry.name)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(payload)
        total_out += len(payload)
        mode_count[actual_mode] = mode_count.get(actual_mode, 0) + 1
        manifest.append(
            {
                "index": entry.index,
                "name": entry.name,
                "offset": entry.offset,
                "entry_size": entry.packed_size,
                "output_size": len(payload),
                "mode": actual_mode,
            }
        )

    manifest_name = args.manifest
    manifest_path_text = None
    if manifest_name:
        manifest_path = out_dir / manifest_name
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
        manifest_path_text = str(manifest_path)

    print(json.dumps({
        "arc": str(arc_path),
        "out_dir": str(out_dir),
        "files": len(manifest),
        "requested_mode": args.mode,
        "mode_count": mode_count,
        "total_output_bytes": total_out,
        "manifest": manifest_path_text,
    }, ensure_ascii=False, indent=2))


def build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="AIWIN/旧 ELF 系 ARC 解包器")
    sub = p.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="列出 ARC 目录")
    p_list.add_argument("arc", help="输入 .arc 文件")
    p_list.add_argument("--json", action="store_true", help="以 JSON 输出目录")
    p_list.set_defaults(func=cmd_list)

    p_ext = sub.add_parser("extract", help="解包 ARC")
    p_ext.add_argument("arc", help="输入 .arc 文件")
    p_ext.add_argument("out_dir", help="输出目录")
    p_ext.add_argument(
        "--mode",
        choices=("auto", "raw", "decompress"),
        default="auto",
        help="解包模式：auto=按扩展名判断，raw=直接切出，decompress=强制LZ解压；默认 auto",
    )
    # 兼容上一版命令：--raw 等价于 --mode raw
    p_ext.add_argument("--raw", action="store_true", help=argparse.SUPPRESS)
    p_ext.add_argument("--manifest", default="manifest.json", help="输出清单文件名；传空字符串则不输出")
    p_ext.set_defaults(func=cmd_extract)
    return p


def main() -> int:
    parser = build_argparser()
    args = parser.parse_args()
    if getattr(args, "raw", False):
        args.mode = "raw"
    try:
        args.func(args)
        return 0
    except ArcFormatError as e:
        parser.error(str(e))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
