# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from keyinse_common import (
    DEFAULT_ENCODING,
    TOOL_VERSION,
    build_entries,
    iter_script_files,
    json_name_for_script,
    load_name_map_object,
    write_json,
)


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract 刻音色 *.s script text/choices to JSON")
    ap.add_argument("--version", action="version", version=f"%(prog)s {TOOL_VERSION}")
    ap.add_argument("input", help="input .s file or directory")
    ap.add_argument("output", help="output .json file for single input, or output directory for directory input")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING, help="script encoding, default: cp932")
    ap.add_argument("--name-map", help="optional JSON object mapping voice prefix to display name, e.g. {\"ka\": \"カスミ\"}")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    name_map = load_name_map_object(args.name_map)

    if inp.is_file():
        entries = build_entries(inp, encoding=args.encoding, name_map=name_map)
        if out.suffix.lower() != ".json":
            out.mkdir(parents=True, exist_ok=True)
            out_path = out / json_name_for_script(inp)
        else:
            out_path = out
        write_json(out_path, entries)
        print(f"[extract] file={inp.name} entries={len(entries)} output={out_path}")
        return

    out.mkdir(parents=True, exist_ok=True)
    files = list(iter_script_files(inp))
    total = 0
    for script in files:
        entries = build_entries(script, encoding=args.encoding, name_map=name_map)
        if not entries:
            continue
        rel = script.relative_to(inp).as_posix().replace("/", "__")
        out_path = out / f"{rel}.json"
        write_json(out_path, entries)
        total += len(entries)
    print(f"[extract] files={len(files)} entries={total} output={out}")


if __name__ == "__main__":
    main()
