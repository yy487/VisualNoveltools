# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from silky_arc_common import pack_archive, write_manifest


def main() -> None:
    ap = argparse.ArgumentParser(description="Pack Silky ARC archive")
    ap.add_argument("input_dir", type=Path, help="待封包目录")
    ap.add_argument("out_arc", type=Path, help="输出 .arc")
    ap.add_argument("--format", default="auto", choices=["auto", "silky-lzss", "garbro-fixed"], help="格式；auto 优先读 manifest")
    ap.add_argument("--encoding", default="cp932", help="文件名编码，默认 cp932")
    ap.add_argument("--manifest", type=Path, default=None, help="指定原始 manifest；默认读取 input_dir/.silky_arc_manifest.json")
    ap.add_argument("--no-compress", action="store_true", help="silky-lzss 新增文件不压缩；默认仍保持原有条目的压缩状态")
    ap.add_argument("--store-all", action="store_true", help="silky-lzss 所有文件都不压缩，速度最快，体积较大")
    ap.add_argument("--write-manifest", action="store_true", help="同时在输出目录旁写一份新 manifest")
    args = ap.parse_args()

    manifest = pack_archive(
        args.input_dir,
        args.out_arc,
        fmt=args.format,
        encoding=args.encoding,
        compress=not args.no_compress,
        manifest_path=args.manifest,
        preserve_packed=not args.store_all,
    )
    if args.write_manifest:
        write_manifest(manifest, args.out_arc.parent)
    print(f"[OK] packed {len(manifest.entries)} files")
    print(f"     format={manifest.format}")
    print(f"     out={args.out_arc}")


if __name__ == "__main__":
    main()
