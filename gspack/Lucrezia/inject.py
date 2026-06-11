# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from common import copy_tree_manifest, load_translation_map, normalize_file_key, rebuild_scw_with_entries, save_json


def inject_directory(src: Path, json_input: Path, out: Path, clean: bool = False) -> None:
    tr_by_file = load_translation_map(json_input)
    copy_tree_manifest(src, out, clean=clean)
    files = sorted(p for p in src.rglob("*") if p.is_file() and p.name != "manifest.json")
    reports: list[dict] = []
    total_patched = 0
    total_failed = 0
    for p in files:
        rel = p.relative_to(src)
        outp = out / rel
        outp.parent.mkdir(parents=True, exist_ok=True)
        key = normalize_file_key(rel.as_posix())
        entries = tr_by_file.get(key, [])
        if not entries:
            shutil.copy2(p, outp)
            continue
        try:
            rebuilt, rep = rebuild_scw_with_entries(rel.as_posix(), p.read_bytes(), entries)
            outp.write_bytes(rebuilt)
            reports.append(rep)
            total_patched += rep["patched_entries"]
            total_failed += rep["failed"]
            print(
                f"[inject] {rel}: entries={rep['entries_seen']} "
                f"patched={rep['patched_entries']} strings={rep['unique_strings_replaced']} "
                f"body {rep['old_body_size']}->{rep['new_body_size']}"
            )
        except Exception as ex:
            total_failed += len(entries)
            reports.append({"file": rel.as_posix(), "entries_seen": len(entries), "patched_entries": 0, "failed": len(entries), "error": str(ex)})
            shutil.copy2(p, outp)
            print(f"[inject][fail] {rel}: {ex}")
    save_json(out / "inject_report.json", reports)
    print(f"[inject] files={len(files)} patched_entries={total_patched} failed={total_failed} output={out}")


def inject_file(src: Path, json_input: Path, out: Path) -> None:
    tr_by_file = load_translation_map(json_input)
    key = normalize_file_key(src.name)
    entries = tr_by_file.get(key, [])
    rebuilt, rep = rebuild_scw_with_entries(src.name, src.read_bytes(), entries)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(rebuilt)
    print(json.dumps(rep, ensure_ascii=False, indent=2))


def roundtrip_directory(src: Path, out: Path, clean: bool = False) -> None:
    from common import decode_scw, iter_source, safe_name

    if out.exists() and clean:
        shutil.rmtree(out)
    out.mkdir(parents=True, exist_ok=True)
    reports = []
    ok = 0
    fail = 0
    for name, data in iter_source(src):
        try:
            rebuilt, rep = rebuild_scw_with_entries(name, data, [])
            old_body, _ = decode_scw(data, name)
            new_body, _ = decode_scw(rebuilt, name)
            same_body = old_body == new_body
            ok += 1 if same_body else 0
            fail += 0 if same_body else 1
            out_name = safe_name(name)
            if not Path(out_name).suffix:
                out_name += ".scw"
            (out / out_name).write_bytes(rebuilt)
            rep["decoded_body_equal"] = same_body
            reports.append(rep)
            print(f"[roundtrip] {name}: decoded_body_equal={same_body} size {len(data)}->{len(rebuilt)}")
        except Exception as ex:
            fail += 1
            reports.append({"file": name, "error": str(ex), "decoded_body_equal": False})
            print(f"[roundtrip][fail] {name}: {ex}")
    save_json(out / "roundtrip_report.json", reports)
    print(f"[roundtrip] ok={ok} fail={fail} output={out}")


def build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Lucrezia Scw5.x blockB/tableB rebuild injector")
    p.add_argument("input", help="original .scw file or unpacked script directory")
    p.add_argument("json", nargs="?", help="translated JSON file or directory")
    p.add_argument("output", help="output .scw file or directory")
    p.add_argument("--clean", action="store_true", help="remove output directory before injecting")
    p.add_argument("--roundtrip", action="store_true", help="zero-translation rebuild test; json argument is ignored")
    return p


def main() -> None:
    args = build_argparser().parse_args()
    src = Path(args.input)
    out = Path(args.output)
    if args.roundtrip:
        roundtrip_directory(src, out, clean=args.clean)
        return
    if not args.json:
        raise SystemExit("inject mode requires JSON file or directory")
    json_input = Path(args.json)
    if src.is_dir():
        inject_directory(src, json_input, out, clean=args.clean)
    else:
        inject_file(src, json_input, out)


if __name__ == "__main__":
    main()
