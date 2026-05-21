# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path
from rp_arc import read_arc
from rp_mes_common import scan_mes_text, records_to_json, save_json


def iter_inputs(path: Path):
    if path.is_file() and path.suffix.lower() == ".arc":
        for e in read_arc(path):
            if e.name.lower().endswith(".mes"):
                yield e.name, e.data
    elif path.is_file():
        yield path.name, path.read_bytes()
    else:
        for p in sorted(path.rglob("*")):
            if p.is_file() and p.suffix.lower() == ".mes":
                yield p.relative_to(path).as_posix(), p.read_bytes()


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract plain text from Refrain Blue MES/MES.ARC")
    ap.add_argument("input", help="MES.ARC, MES file, or directory containing MES files")
    ap.add_argument("out_dir", help="output JSON directory")
    ap.add_argument("--include-ruby", action="store_true", help="also extract 0B 15 FF 01 ruby helper strings")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)

    reports = []
    total = 0
    for name, data in iter_inputs(inp):
        records = scan_mes_text(data, name, include_ruby=args.include_ruby)
        entries = records_to_json(records)
        json_path = out / (Path(name).stem + ".json")
        save_json(json_path, entries)
        total += len(entries)
        reports.append({"file": name, "json": str(json_path), "entries": len(entries)})

    print(json.dumps({"files": len(reports), "entries": total, "reports": reports}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
