# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import unicodedata
from pathlib import Path

from rp_font_common import read_tbl


def collect_json_chars(path: Path, fields: list[str]) -> set[str]:
    chars: set[str] = set()
    files = [path] if path.is_file() else sorted(path.rglob("*.json"))
    for p in files:
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        stack = [data]
        while stack:
            cur = stack.pop()
            if isinstance(cur, dict):
                for k, v in cur.items():
                    if k in fields and isinstance(v, str):
                        chars.update(v)
                    elif isinstance(v, (dict, list)):
                        stack.append(v)
            elif isinstance(cur, list):
                stack.extend(cur)
    return chars


def cp932_two_byte_fullwidth_chars() -> list[str]:
    out = []
    first_list = list(range(0x81, 0xA0)) + list(range(0xE0, 0xEB)) + list(range(0xFA, 0xFD))
    second_list = list(range(0x40, 0x7F)) + list(range(0x80, 0x100))
    for hi in first_list:
        for lo in second_list:
            bs = bytes([hi, lo])
            try:
                ch = bs.decode("cp932")
            except UnicodeDecodeError:
                continue
            if len(ch) != 1:
                continue
            if unicodedata.east_asian_width(ch) in ("F", "W", "A"):
                out.append(ch)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Create char_list.txt for FONT building.")
    ap.add_argument("out_txt")
    ap.add_argument("--base-tbl", default="", help="Existing FONT.TBL to keep original order first")
    ap.add_argument("--json", default="", help="Optional translated JSON file/dir; chars from message fields are appended")
    ap.add_argument("--fields", default="message", help="Comma-separated JSON fields to scan, default message")
    ap.add_argument("--append-cp932-fullwidth", action="store_true", help="Append all CP932 full-width-ish two-byte chars")
    ap.add_argument("--columns", type=int, default=80)
    args = ap.parse_args()
    chars: list[str] = []
    seen: set[str] = set()
    def add(ch: str):
        if ch in "\r\n" or ch in seen:
            return
        try:
            if len(ch.encode("cp932")) != 2:
                return
        except UnicodeEncodeError:
            return
        seen.add(ch)
        chars.append(ch)
    if args.base_tbl:
        for c in read_tbl(Path(args.base_tbl)):
            add(c)
    if args.json:
        fields = [x.strip() for x in args.fields.split(",") if x.strip()]
        for c in sorted(collect_json_chars(Path(args.json), fields)):
            add(c)
    if args.append_cp932_fullwidth:
        for c in cp932_two_byte_fullwidth_chars():
            add(c)
    lines = ["".join(chars[i:i + args.columns]) for i in range(0, len(chars), args.columns)]
    Path(args.out_txt).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"chars={len(chars)} -> {args.out_txt}")


if __name__ == "__main__":
    main()
