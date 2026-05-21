# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path
from rp_arc import read_arc


def iter_inputs(path: Path):
    if path.is_file() and path.suffix.lower() == ".arc":
        for e in read_arc(path):
            yield e.name, e.data
    elif path.is_file():
        yield path.name, path.read_bytes()
    else:
        for p in sorted(path.rglob("*")):
            if p.is_file():
                yield p.name, p.read_bytes()


def main() -> None:
    ap = argparse.ArgumentParser(description="Find a CP932 string in MES/MES.ARC")
    ap.add_argument("input")
    ap.add_argument("text")
    args = ap.parse_args()
    needle = args.text.encode("cp932")
    hits = []
    for name, data in iter_inputs(Path(args.input)):
        start = 0
        while True:
            pos = data.find(needle, start)
            if pos < 0:
                break
            hits.append({"file": name, "offset": pos, "offset_hex": f"0x{pos:X}"})
            start = pos + 1
    print(json.dumps({"text": args.text, "hits": hits}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
