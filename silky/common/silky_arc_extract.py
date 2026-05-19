# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from silky_arc_common import extract_archive


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract Silky ARC archive")
    ap.add_argument("arc", type=Path, help="输入 .arc")
    ap.add_argument("out_dir", type=Path, help="输出目录")
    ap.add_argument("--format", default="auto", choices=["auto", "silky-lzss", "garbro-fixed"], help="格式检测/指定")
    ap.add_argument("--encoding", default="cp932", help="文件名编码，默认 cp932")
    ap.add_argument("--no-manifest", action="store_true", help="不写 .silky_arc_manifest.json")
    args = ap.parse_args()

    manifest = extract_archive(args.arc, args.out_dir, args.format, args.encoding, not args.no_manifest)
    print(f"[OK] extracted {len(manifest.entries)} files")
    print(f"     format={manifest.format}")
    print(f"     out={args.out_dir}")


if __name__ == "__main__":
    main()
