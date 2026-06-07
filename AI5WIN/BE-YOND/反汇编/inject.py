# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

from common import DEFAULT_ENCODING, assemble_with_replacements, text_entries_for_file, load_json, bracket_name


def iter_mes_files(path: Path):
    if path.is_file():
        yield path
    else:
        yield from sorted(p for p in path.rglob("*") if p.is_file() and p.suffix.upper() == ".MES")


def collect_json_entries(json_path: Path) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    if json_path.is_file():
        data = load_json(json_path)
        if not isinstance(data, list):
            raise ValueError(f"JSON root must be list: {json_path}")
        for e in data:
            if not isinstance(e, dict):
                continue
            f = e.get("_file")
            if not isinstance(f, str):
                raise ValueError(f"missing _file in {json_path}: {e}")
            grouped[f].append(e)
    else:
        for jp in sorted(json_path.rglob("*.json")):
            data = load_json(jp)
            if not isinstance(data, list):
                raise ValueError(f"JSON root must be list: {jp}")
            for e in data:
                if not isinstance(e, dict):
                    continue
                f = e.get("_file")
                if not isinstance(f, str):
                    # fallback from per-file JSON name
                    f = jp.name[:-5].replace("__", "/")
                    e["_file"] = f
                grouped[f].append(e)
    return grouped


def inject_one(src: Path, entries: list[dict[str, Any]], out: Path, root: Path | None, encoding: str, strict: bool, include_non_scenario: bool = False) -> tuple[int, int, list[str]]:
    current = text_entries_for_file(src, root, encoding, include_system=False, include_non_scenario=include_non_scenario)
    by_index = {e["_index"]: e for e in current}
    replacements: dict[int, str] = {}  # _inst_offset -> message
    failed = 0
    warnings: list[str] = []
    for e in entries:
        idx = e.get("_index")
        scr = e.get("scr_msg")
        msg = e.get("message")
        if not isinstance(idx, int) or not isinstance(scr, str) or not isinstance(msg, str):
            failed += 1
            warnings.append(f"bad json entry in {src.name}: {e}")
            continue
        cur = by_index.get(idx)
        if cur is None:
            failed += 1
            warnings.append(f"index not found {src.name} #{idx}: {scr}")
            continue
        if cur["scr_msg"] != scr:
            failed += 1
            warnings.append(f"scr_msg mismatch {src.name} #{idx}: json={scr!r} file={cur['scr_msg']!r}")
            continue
        # Body text replacement.
        if msg != scr:
            inst_off = e.get("_inst_offset")
            if not isinstance(inst_off, int):
                failed += 1
                warnings.append(f"missing _inst_offset {src.name} #{idx}: {scr}")
                continue
            replacements[inst_off] = msg

        # Optional speaker-name replacement.  The JSON name field stores the
        # inner name only, while the MES visible name slot is 【name】.
        json_name = e.get("name")
        cur_name = cur.get("name")
        if isinstance(json_name, str) and isinstance(cur_name, str) and json_name != cur_name:
            name_inst = e.get("_name_inst_offset")
            if not isinstance(name_inst, int):
                failed += 1
                warnings.append(f"missing _name_inst_offset {src.name} #{idx}: {scr}")
                continue
            replacements[name_inst] = bracket_name(json_name)
    if failed and strict:
        raise RuntimeError("; ".join(warnings[:5]))
    rebuilt, w = assemble_with_replacements(src.read_bytes(), replacements, encoding)
    warnings.extend(w)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(rebuilt)
    return len(replacements), failed, warnings


def main() -> None:
    ap = argparse.ArgumentParser(description="Inject project JSON text into AI5WIN decompressed MES with relocation")
    ap.add_argument("input", help="source MES file or directory")
    ap.add_argument("json", help="translated JSON file or directory")
    ap.add_argument("output", help="output MES file or directory")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--non-strict", action="store_true")
    ap.add_argument("--all-files", action="store_true", help="allow injection into non-scenario/resource MES files")
    ap.add_argument("--report", help="write JSON report")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    grouped = collect_json_entries(Path(args.json))
    patched_total = 0
    failed_total = 0
    warn_all: list[str] = []
    files_done = 0

    if inp.is_file():
        # one file mode may use _file from JSON, but source path is authoritative.
        entries = next(iter(grouped.values())) if grouped else []
        p, f, w = inject_one(inp, entries, out, None, args.encoding, strict=not args.non_strict, include_non_scenario=args.all_files)
        patched_total += p; failed_total += f; warn_all += w; files_done = 1
    else:
        for src in iter_mes_files(inp):
            rel = src.relative_to(inp).as_posix()
            dst = out / rel
            entries = grouped.get(rel, [])
            if not entries:
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes(src.read_bytes())
                continue
            p, f, w = inject_one(src, entries, dst, inp, args.encoding, strict=not args.non_strict, include_non_scenario=args.all_files)
            patched_total += p; failed_total += f; warn_all += w; files_done += 1
    report = {"files_done": files_done, "patched": patched_total, "failed": failed_total, "warnings": warn_all[:200]}
    if args.report:
        Path(args.report).write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[inject] files={files_done} patched={patched_total} failed={failed_total} warnings={len(warn_all)} output={out}")

if __name__ == "__main__":
    main()
