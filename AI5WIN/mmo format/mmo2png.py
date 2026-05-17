#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""MMO -> PNG 转换命令行工具。"""

from __future__ import annotations

import argparse
from pathlib import Path

from mmo_common import decode_mmo_file, iter_mmo_files, parse_header, mmo_fast_status


def build_argparser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description="Decode Doukyuusei/AI5WIN MMO image files to PNG.")
    ap.add_argument("inputs", nargs="+", help="输入 MMO 文件或目录；目录默认递归扫描")
    ap.add_argument("-o", "--out", default=None, help="输出 PNG 文件或输出目录。多输入时必须是目录。")
    ap.add_argument("--no-recursive", action="store_true", help="目录输入只扫描当前层，不递归子目录")
    ap.add_argument("--no-flip", action="store_true", help="不做垂直翻转，保留游戏内 bottom-up 内存方向")
    ap.add_argument("--list", action="store_true", help="只列出 MMO header，不导出 PNG")
    ap.add_argument("--no-fast", action="store_true", help="禁用 C 加速，强制使用纯 Python 解码")
    ap.add_argument("--fast-info", action="store_true", help="显示 C 加速加载状态")
    return ap


def main() -> int:
    args = build_argparser().parse_args()
    if args.fast_info:
        print(f"mmo_fast: {mmo_fast_status()}")

    files = list(iter_mmo_files(args.inputs, recursive=not args.no_recursive))
    if not files:
        raise SystemExit("没有找到 MMO 文件")

    if args.list:
        for p in files:
            h = parse_header(p.read_bytes())
            print(f"{p}\t{h.width}x{h.height}\talpha_offset=0x{h.alpha_offset:X}")
        return 0

    out_arg = Path(args.out) if args.out else None
    multi = len(files) > 1 or any(Path(x).is_dir() for x in args.inputs)
    if multi:
        out_dir = out_arg if out_arg else Path("png_out")
        out_dir.mkdir(parents=True, exist_ok=True)
        for src in files:
            dst = out_dir / (src.stem + ".png")
            decode_mmo_file(src, dst, flip_y=not args.no_flip, use_fast=not args.no_fast)
            print(f"[OK] {src} -> {dst}")
    else:
        src = files[0]
        dst = out_arg if out_arg else src.with_suffix(".png")
        decode_mmo_file(src, dst, flip_y=not args.no_flip, use_fast=not args.no_fast)
        print(f"[OK] {src} -> {dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
