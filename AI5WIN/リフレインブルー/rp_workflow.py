# -*- coding: utf-8 -*-
"""Recommended unpack -> per-MES JSON -> inject -> pack workflow.

This wrapper deliberately treats MES.ARC as an archive, not as one text pool.
It is a thin orchestrator over rp_arc.py / rp_mes_common.py.
"""
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from rp_arc import read_arc, unpack_arc, load_dir_entries, write_arc
from rp_mes_common import scan_mes_text, records_to_json, save_json, load_json_entries, patch_mes_non_equal


def iter_mes_files(root: Path):
    for p in sorted(root.rglob("*")):
        if p.is_file() and p.suffix.lower() == ".mes":
            yield p


def cmd_unpack(args) -> None:
    entries = unpack_arc(args.arc, args.out_dir)
    print(json.dumps({"arc": args.arc, "out_dir": args.out_dir, "files": len(entries)}, ensure_ascii=False, indent=2))


def cmd_extract(args) -> None:
    in_dir = Path(args.mes_dir)
    out_dir = Path(args.json_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    reports = []
    total = 0
    for mes in iter_mes_files(in_dir):
        rel = mes.relative_to(in_dir)
        data = mes.read_bytes()
        records = scan_mes_text(data, rel.as_posix(), include_ruby=args.include_ruby)
        entries = records_to_json(records)
        json_path = out_dir / rel.with_suffix(".json")
        save_json(json_path, entries)
        reports.append({"file": rel.as_posix(), "json": str(json_path), "entries": len(entries)})
        total += len(entries)
    print(json.dumps({"mes_dir": str(in_dir), "json_dir": str(out_dir), "files": len(reports), "entries": total, "reports": reports}, ensure_ascii=False, indent=2))



def inject_dir(mes_dir: Path, json_dir: Path, out_dir: Path, *, force_jump: bool = False, clean: bool = False) -> list[dict]:
    if out_dir.exists() and clean:
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    reports = []
    for src in sorted(p for p in mes_dir.rglob("*") if p.is_file()):
        rel = src.relative_to(mes_dir)
        dst = out_dir / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        if src.suffix.lower() != ".mes":
            shutil.copyfile(src, dst)
            continue
        json_path = json_dir / rel.with_suffix(".json")
        if not json_path.exists():
            shutil.copyfile(src, dst)
            reports.append({"file": rel.as_posix(), "patched": 0, "warnings": [f"missing JSON: {json_path}"]})
            continue
        entries = load_json_entries(json_path)
        new_data, rep = patch_mes_non_equal(src.read_bytes(), entries, force_jump=force_jump)
        dst.write_bytes(new_data)
        rep["file"] = rel.as_posix()
        rep["json"] = str(json_path)
        reports.append(rep)
    return reports

def cmd_inject(args) -> None:
    in_dir = Path(args.mes_dir)
    json_dir = Path(args.json_dir)
    out_dir = Path(args.out_mes_dir)
    reports = inject_dir(in_dir, json_dir, out_dir, force_jump=args.force_jump, clean=args.clean)
    print(json.dumps({"mes_dir": str(in_dir), "json_dir": str(json_dir), "out_mes_dir": str(out_dir), "files": len(reports), "reports": reports}, ensure_ascii=False, indent=2))

def cmd_pack(args) -> None:
    order = [e.name for e in read_arc(args.base_arc)] if args.base_arc else None
    entries = load_dir_entries(args.mes_dir, order)
    write_arc(args.out_arc, entries)
    print(json.dumps({"mes_dir": args.mes_dir, "out_arc": args.out_arc, "base_arc": args.base_arc, "files": len(entries)}, ensure_ascii=False, indent=2))


def cmd_unpack_extract(args) -> None:
    work = Path(args.work_dir)
    mes_dir = work / "mes_raw"
    json_dir = work / "json"
    unpack_arc(args.arc, mes_dir)
    class A: pass
    a = A(); a.mes_dir = str(mes_dir); a.json_dir = str(json_dir); a.include_ruby = args.include_ruby
    cmd_extract(a)


def cmd_inject_pack(args) -> None:
    work = Path(args.work_dir)
    mes_raw = work / "mes_raw"
    json_dir = work / "json"
    mes_new = work / "mes_new"
    reports = inject_dir(mes_raw, json_dir, mes_new, force_jump=args.force_jump, clean=args.clean)
    entries = load_dir_entries(mes_new, [e.name for e in read_arc(args.base_arc)])
    write_arc(args.out_arc, entries)
    print(json.dumps({
        "base_arc": args.base_arc,
        "json_dir": str(json_dir),
        "patched_mes_dir": str(mes_new),
        "out_arc": args.out_arc,
        "files": len(reports),
        "reports": reports,
    }, ensure_ascii=False, indent=2))

def main() -> None:
    ap = argparse.ArgumentParser(description="Refrain Blue MES.ARC unpack/per-file JSON workflow")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("unpack", help="MES.ARC -> scattered MES files")
    p.add_argument("arc")
    p.add_argument("out_dir")
    p.set_defaults(func=cmd_unpack)

    p = sub.add_parser("extract", help="scattered MES directory -> one JSON per MES")
    p.add_argument("mes_dir")
    p.add_argument("json_dir")
    p.add_argument("--include-ruby", action="store_true")
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser("inject", help="scattered MES directory + JSON directory -> patched MES directory")
    p.add_argument("mes_dir")
    p.add_argument("json_dir")
    p.add_argument("out_mes_dir")
    p.add_argument("--force-jump", action="store_true")
    p.add_argument("--clean", action="store_true", help="delete output directory before writing")
    p.set_defaults(func=cmd_inject)

    p = sub.add_parser("pack", help="patched MES directory -> MES.ARC")
    p.add_argument("mes_dir")
    p.add_argument("out_arc")
    p.add_argument("--base-arc", default="", help="original MES.ARC, used to preserve entry order")
    p.set_defaults(func=cmd_pack)

    p = sub.add_parser("unpack-extract", help="one command: MES.ARC -> work/mes_raw + work/json")
    p.add_argument("arc")
    p.add_argument("work_dir")
    p.add_argument("--include-ruby", action="store_true")
    p.set_defaults(func=cmd_unpack_extract)

    p = sub.add_parser("inject-pack", help="one command: work/json + work/mes_raw -> patched MES.ARC")
    p.add_argument("base_arc")
    p.add_argument("work_dir")
    p.add_argument("out_arc")
    p.add_argument("--force-jump", action="store_true")
    p.add_argument("--clean", action="store_true")
    p.set_defaults(func=cmd_inject_pack)

    args = ap.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
