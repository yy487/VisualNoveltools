# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from common import DEFAULT_ENCODING, dump_json, text_entries_for_file


def iter_mes_files(path: Path):
    if path.is_file():
        yield path
    else:
        yield from sorted(p for p in path.rglob("*") if p.is_file() and p.suffix.upper() == ".MES")


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract AI5WIN decompressed MES text to project JSON")
    ap.add_argument("input", help="MES file or directory")
    ap.add_argument("output", help="output JSON file or directory")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--include-system", action="store_true", help="also export SYSTEM_TEXT control strings")
    ap.add_argument("--all-files", action="store_true", help="do not skip known non-scenario/resource MES files")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    files = list(iter_mes_files(inp))
    total = 0
    choices = 0
    if inp.is_file():
        entries = text_entries_for_file(inp, None, args.encoding, args.include_system, include_non_scenario=args.all_files)
        dump_json(out, entries)
        total = len(entries)
        choices = sum(1 for e in entries if e.get("_type") == "choice")
    else:
        out.mkdir(parents=True, exist_ok=True)
        for f in files:
            entries = text_entries_for_file(f, inp, args.encoding, args.include_system, include_non_scenario=args.all_files)
            if not entries:
                continue
            rel = f.relative_to(inp).as_posix().replace("/", "__")
            dump_json(out / f"{rel}.json", entries)
            total += len(entries)
            choices += sum(1 for e in entries if e.get("_type") == "choice")
    print(f"[extract] files={len(files)} entries={total} choices={choices} output={out}")

if __name__ == "__main__":
    main()
