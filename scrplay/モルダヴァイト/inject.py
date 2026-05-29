# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

from scr0034_common import (
    Scr0034,
    denormalize_for_import,
    encode_text,
    ensure_json_list,
    normalize_for_export,
    split_display_text,
)


def iter_scr_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.rglob("*.scr") if p.is_file())


def load_json_map(json_path: Path) -> dict[str, list[dict[str, Any]]]:
    mapping: dict[str, list[dict[str, Any]]] = {}
    if json_path.is_file():
        data = ensure_json_list(json_path)
        if data:
            file_name = data[0].get("_file")
            if isinstance(file_name, str):
                mapping[file_name] = data
            else:
                mapping["__single__"] = data
        return mapping
    for jp in sorted(json_path.rglob("*.json")):
        data = ensure_json_list(jp)
        if not data:
            continue
        file_name = data[0].get("_file")
        if not isinstance(file_name, str):
            # fallback: adv_01.scr.json -> adv_01.scr
            name = jp.name[:-5]
            file_name = name.replace("__", "/")
        mapping.setdefault(file_name, []).extend(data)
    return mapping


def build_slot_text(entry: dict[str, Any], old_slot_text: str, opcode: int, fix_orig: bool) -> tuple[str, str]:
    old_name, old_msg, old_prefix, old_suffix, _typ = split_display_text(old_slot_text, opcode)
    json_scr = entry.get("scr_msg")
    json_msg = entry.get("message", json_scr)
    if not isinstance(json_scr, str) or not isinstance(json_msg, str):
        raise ValueError("entry missing scr_msg/message")

    cmp_old_msg = normalize_for_export(old_msg, fix_orig)
    if cmp_old_msg != json_scr:
        raise ValueError(f"scr_msg mismatch: file='{cmp_old_msg}' json='{json_scr}'")

    prefix = entry.get("_prefix")
    suffix = entry.get("_suffix")
    if not isinstance(prefix, str):
        prefix = normalize_for_export(old_prefix, fix_orig)
    if not isinstance(suffix, str):
        suffix = old_suffix

    # 默认不改 name。若你手动改 JSON 的 name，则同步写回 prefix。
    if old_name is not None and isinstance(entry.get("name"), str):
        name_text = denormalize_for_import(entry["name"], fix_orig)
        prefix_raw = name_text + "\n"
    else:
        prefix_raw = denormalize_for_import(prefix, fix_orig)

    msg_raw = denormalize_for_import(json_msg, fix_orig)
    suffix_raw = suffix
    return prefix_raw + msg_raw + suffix_raw, json_msg


def inject_one(src: Path, out: Path, entries: list[dict[str, Any]], fix_orig: bool, strict: bool = True) -> dict[str, Any]:
    scr = Scr0034.read(src.read_bytes(), strict=strict)
    patched = 0
    unchanged = 0
    failed = 0
    warnings: list[str] = []

    # 优先按 _slot_index 定位；同一个 slot 多次出现时以后出现的条目为准。
    for entry in entries:
        try:
            slot_i = entry.get("_slot_index")
            opcode_s = entry.get("_opcode")
            if not isinstance(slot_i, int):
                raise ValueError("missing _slot_index")
            if slot_i < 0 or slot_i >= len(scr.str_list):
                raise ValueError(f"_slot_index out of range: {slot_i}")
            opcode = int(opcode_s, 16) if isinstance(opcode_s, str) else 0
            old_slot_text = scr.str_list[slot_i].decode("cp932", errors="strict")
            new_slot_text, json_msg = build_slot_text(entry, old_slot_text, opcode, fix_orig=fix_orig)
            new_raw = encode_text(new_slot_text)
            if new_raw == scr.str_list[slot_i]:
                unchanged += 1
            else:
                scr.str_list[slot_i] = new_raw
                patched += 1
        except Exception as e:
            failed += 1
            ident = f"{entry.get('_file')} index={entry.get('_index')} slot={entry.get('_slot_index')}"
            warnings.append(f"{ident}: {e}")

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(scr.to_bytes(encrypt=True))
    return {
        "file": src.name,
        "entries": len(entries),
        "patched": patched,
        "unchanged": unchanged,
        "failed": failed,
        "warnings": warnings,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Inject JSON text into SCR:0034 ScrPlayer scripts")
    ap.add_argument("input", help="original .scr file or directory")
    ap.add_argument("json", help="translated .json file or json directory")
    ap.add_argument("output", help="output .scr file or directory")
    ap.add_argument("--fix-orig", action="store_true", help="JSON was extracted with --fix-orig; convert back before encoding")
    ap.add_argument("--copy-missing", action="store_true", help="when input is dir, copy files without matching json")
    ap.add_argument("--non-strict", action="store_true", help="ignore unresolved resource string refs")
    ap.add_argument("--stats-json", help="write detailed stats json")
    args = ap.parse_args()

    inp = Path(args.input)
    json_path = Path(args.json)
    out = Path(args.output)
    jmap = load_json_map(json_path)
    files = iter_scr_files(inp)
    if not files:
        raise SystemExit("no .scr files found")

    stats: list[dict[str, Any]] = []
    total_patched = total_failed = total_entries = 0

    if inp.is_file():
        entries = jmap.get(inp.name) or jmap.get("__single__")
        if entries is None:
            raise SystemExit(f"no json entries for {inp.name}")
        st = inject_one(inp, out, entries, fix_orig=args.fix_orig, strict=not args.non_strict)
        stats.append(st)
    else:
        for file in files:
            rel = file.relative_to(inp).as_posix()
            entries = jmap.get(rel)
            dst = out / rel
            if entries is None:
                if args.copy_missing:
                    dst.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(file, dst)
                continue
            st = inject_one(file, dst, entries, fix_orig=args.fix_orig, strict=not args.non_strict)
            stats.append(st)

    for st in stats:
        total_entries += st["entries"]
        total_patched += st["patched"]
        total_failed += st["failed"]
        for w in st["warnings"][:10]:
            print(f"[inject][warn] {w}")
        if len(st["warnings"]) > 10:
            print(f"[inject][warn] ... {len(st['warnings']) - 10} more in {st['file']}")

    if args.stats_json:
        p = Path(args.stats_json)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps(stats, ensure_ascii=False, indent=2), encoding="utf-8")

    print(f"[inject] files={len(stats)} entries={total_entries} patched={total_patched} failed={total_failed} output={out}")


if __name__ == "__main__":
    main()
