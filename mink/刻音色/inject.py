# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path

from keyinse_common import (
    DEFAULT_ENCODING,
    TOOL_VERSION,
    entries_for_file,
    iter_script_files,
    json_name_for_script,
    patch_script,
    read_json,
)


def _load_entries_for_script(script: Path, json_root: Path) -> list[dict]:
    if json_root.is_file():
        return read_json(json_root)
    candidates = [json_root / json_name_for_script(script), json_root / f"{script.name}.json"]
    # Directory extraction with relative path flattening may use subdir__file.s.json.
    for c in candidates:
        if c.exists():
            return read_json(c)
    raise FileNotFoundError(f"JSON not found for {script.name}: tried {', '.join(str(c) for c in candidates)}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Inject JSON translations back into 刻音色 *.s scripts")
    ap.add_argument("--version", action="version", version=f"%(prog)s {TOOL_VERSION}")
    ap.add_argument("input", help="original .s file or directory")
    ap.add_argument("json", help="translation .json file or json directory")
    ap.add_argument("output", help="output .s file or directory")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING, help="script encoding, default: cp932")
    ap.add_argument("--mode", choices=["relocate", "in-place"], default="relocate",
                    help="relocate can grow/shrink records and fixes known absolute script offsets; in-place keeps original byte sizes")
    ap.add_argument("--strict", action="store_true", help="abort on first mismatch/encoding/length error")
    ap.add_argument("--stats-json", help="optional path to write machine-readable injection stats")
    args = ap.parse_args()

    inp = Path(args.input)
    js = Path(args.json)
    out = Path(args.output)
    all_stats = []

    if inp.is_file():
        entries = _load_entries_for_script(inp, js)
        entry_map = entries_for_file(entries, inp.name)
        patched, stats = patch_script(inp.read_bytes(), inp.name, entry_map, encoding=args.encoding,
                                      mode=args.mode, strict=args.strict)
        if out.exists() and out.is_dir():
            out_path = out / inp.name
        elif out.suffix:
            out_path = out
        else:
            out.mkdir(parents=True, exist_ok=True)
            out_path = out / inp.name
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(patched)
        all_stats.append(stats)
        print(f"[inject] file={inp.name} patched={stats['patched']} failed={stats['failed']} size={stats['old_size']}->{stats['new_size']} output={out_path}")
        for w in stats["warnings"][:20]:
            print(f"[inject][warn] {w}")
    else:
        out.mkdir(parents=True, exist_ok=True)
        scripts = list(iter_script_files(inp))
        for script in scripts:
            try:
                entries = _load_entries_for_script(script, js)
            except FileNotFoundError:
                # Preserve untranslated scripts unchanged.
                rel = script.relative_to(inp)
                dst = out / rel
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes(script.read_bytes())
                all_stats.append({"file": script.name, "patched": 0, "failed": 0, "warnings": ["json missing; copied unchanged"]})
                continue
            entry_map = entries_for_file(entries, script.name)
            patched, stats = patch_script(script.read_bytes(), script.name, entry_map, encoding=args.encoding,
                                          mode=args.mode, strict=args.strict)
            rel = script.relative_to(inp)
            dst = out / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(patched)
            all_stats.append(stats)
        print(
            f"[inject] files={len(scripts)} patched={sum(s.get('patched',0) for s in all_stats)} "
            f"failed={sum(s.get('failed',0) for s in all_stats)} output={out}"
        )
        warn_count = sum(len(s.get("warnings", [])) for s in all_stats)
        if warn_count:
            print(f"[inject] warnings={warn_count}; use --stats-json for full details")

    if args.stats_json:
        stats_path = Path(args.stats_json)
        stats_path.parent.mkdir(parents=True, exist_ok=True)
        with open(stats_path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(all_stats, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
