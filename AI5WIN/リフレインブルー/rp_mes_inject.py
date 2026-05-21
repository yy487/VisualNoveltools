# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path
from rp_arc import read_arc, write_arc
from rp_mes_common import load_json_entries, patch_mes_non_equal


def inject_one(data: bytes, json_path: Path, *, force_jump: bool = False) -> tuple[bytes, dict]:
    entries = load_json_entries(json_path)
    return patch_mes_non_equal(data, entries, force_jump=force_jump)


def main() -> None:
    ap = argparse.ArgumentParser(description="Inject translated text into Refrain Blue MES/MES.ARC using EOF jump stubs")
    ap.add_argument("input", help="original MES.ARC, MES file, or MES directory")
    ap.add_argument("json_dir", help="JSON directory produced by rp_mes_extract.py")
    ap.add_argument("output", help="patched MES.ARC, MES file, or output directory")
    ap.add_argument("--force-jump", action="store_true", help="use EOF jump stub even when text fits in place")
    args = ap.parse_args()

    inp = Path(args.input)
    jout = Path(args.json_dir)
    out = Path(args.output)
    reports = []

    if inp.is_file() and inp.suffix.lower() == ".arc":
        new_entries = []
        for e in read_arc(inp):
            if e.name.lower().endswith(".mes"):
                jp = jout / (Path(e.name).stem + ".json")
                if jp.exists():
                    new_data, rep = inject_one(e.data, jp, force_jump=args.force_jump)
                    rep["file"] = e.name
                    rep["json"] = str(jp)
                    reports.append(rep)
                    new_entries.append((e.name, new_data))
                else:
                    reports.append({"file": e.name, "warnings": [f"missing JSON: {jp}"], "patched": 0})
                    new_entries.append((e.name, e.data))
            else:
                new_entries.append((e.name, e.data))
        write_arc(out, new_entries)
    elif inp.is_file():
        jp = jout if jout.is_file() else jout / (inp.stem + ".json")
        new_data, rep = inject_one(inp.read_bytes(), jp, force_jump=args.force_jump)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(new_data)
        rep["file"] = inp.name
        rep["json"] = str(jp)
        reports.append(rep)
    else:
        out.mkdir(parents=True, exist_ok=True)
        for p in sorted(x for x in inp.rglob("*") if x.is_file()):
            rel = p.relative_to(inp)
            op = out / rel
            op.parent.mkdir(parents=True, exist_ok=True)
            if p.suffix.lower() != ".mes":
                op.write_bytes(p.read_bytes())
                continue
            jp = jout / rel.with_suffix(".json")
            if not jp.exists():
                jp = jout / (p.stem + ".json")
            if jp.exists():
                new_data, rep = inject_one(p.read_bytes(), jp, force_jump=args.force_jump)
                op.write_bytes(new_data)
                rep["file"] = str(rel)
                rep["json"] = str(jp)
                reports.append(rep)
            else:
                op.write_bytes(p.read_bytes())
                reports.append({"file": str(rel), "warnings": [f"missing JSON: {jp}"], "patched": 0})

    print(json.dumps({"files": len(reports), "reports": reports}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
