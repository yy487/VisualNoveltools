#!/usr/bin/env python3
# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from d2_arc_common import build_arc_from_manifest


def main() -> int:
    ap = argparse.ArgumentParser(description="Rebuild Doukyuusei2/SilAI ARC archives from extracted directory.")
    ap.add_argument("input_dir", help="directory containing files and _arc_manifest.json")
    ap.add_argument("output_arc", help="output .ARC path")
    ap.add_argument("--encoding", default="cp932", help="filename encoding before XOR encode, default: cp932")
    args = ap.parse_args()
    entries = build_arc_from_manifest(args.input_dir, args.output_arc, encoding=args.encoding)
    print(f"packed {len(entries)} files -> {args.output_arc}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
