# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from rp_font_common import read_tbl, write_tbl, split_char_list_text


def main() -> None:
    ap = argparse.ArgumentParser(description="Decode/encode Refrain Blue FONT.TBL reversed CP932 table.")
    sub = ap.add_subparsers(dest="cmd", required=True)
    d = sub.add_parser("decode", help="FONT.TBL -> UTF-8 txt")
    d.add_argument("tbl")
    d.add_argument("txt")
    d.add_argument("--encoding", default="cp932")
    d.add_argument("--columns", type=int, default=80)
    e = sub.add_parser("encode", help="UTF-8 txt -> FONT.TBL")
    e.add_argument("txt")
    e.add_argument("tbl")
    e.add_argument("--encoding", default="cp932")
    args = ap.parse_args()
    if args.cmd == "decode":
        chars = read_tbl(Path(args.tbl), args.encoding)
        lines = ["".join(chars[i:i + args.columns]) for i in range(0, len(chars), args.columns)]
        Path(args.txt).write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"decoded chars={len(chars)} -> {args.txt}")
    else:
        chars = split_char_list_text(Path(args.txt).read_text(encoding="utf-8"))
        write_tbl(Path(args.tbl), chars, args.encoding)
        print(f"encoded chars={len(chars)} -> {args.tbl}")


if __name__ == "__main__":
    main()
