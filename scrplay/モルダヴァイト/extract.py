# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from scr0034_common import (
    STR_CODE_CONFIG,
    Scr0034,
    decode_text,
    normalize_for_export,
    save_json,
    split_display_text,
)


def iter_scr_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    return sorted(p for p in path.rglob("*.scr") if p.is_file())


def extract_one(path: Path, base: Path, fix_orig: bool = False, strict: bool = True) -> list[dict]:
    scr = Scr0034.read(path.read_bytes(), strict=strict)
    rel = path.relative_to(base).as_posix() if base.is_dir() else path.name
    entries: list[dict] = []
    index = 0
    for cmd in scr.commands:
        cfg = STR_CODE_CONFIG.get(cmd.code)
        if not cfg:
            continue
        for param_i, flag in enumerate(cfg[:len(cmd.params)]):
            if flag != 1:
                continue
            if param_i not in cmd.ref_indices:
                continue
            slot_i = cmd.ref_indices[param_i]
            raw = scr.str_list[slot_i]
            try:
                slot_text = decode_text(raw)
            except UnicodeDecodeError as e:
                raise UnicodeDecodeError(e.encoding, e.object, e.start, e.end, f"{path}: slot={slot_i}: {e.reason}")
            name, scr_msg, prefix, suffix, typ = split_display_text(slot_text, cmd.code)
            out_name = normalize_for_export(name, fix_orig) if name else None
            out_msg = normalize_for_export(scr_msg, fix_orig)
            entry = {
                "scr_msg": out_msg,
                "message": out_msg,
                "_file": rel,
                "_index": index,
                "_type": typ,
                "_opcode": f"0x{cmd.code:02X}",
                "_cmd_offset": cmd.offset,
                "_param_index": param_i,
                "_slot_index": slot_i,
                "_slot_offset": scr.addr_list[slot_i],
                "_slot_size": len(raw),
                "_prefix": normalize_for_export(prefix, fix_orig),
                "_suffix": suffix,
                "_encoding": "cp932",
                "_policy": "relocate",
            }
            if out_name:
                # name 放在 scr_msg/message 前，符合你前面要求的条目顺序。
                entry = {"name": out_name, **entry}
            entries.append(entry)
            index += 1
    return entries


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract SCR:0034 ScrPlayer text to JSON")
    ap.add_argument("input", help="input .scr file or directory")
    ap.add_argument("output", help="output .json file or directory")
    ap.add_argument("--fix-orig", action="store_true", help="export readable full-width punctuation/kana like old fixOrig")
    ap.add_argument("--non-strict", action="store_true", help="ignore unresolved resource string refs")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    files = iter_scr_files(inp)
    if not files:
        raise SystemExit("no .scr files found")

    total = 0
    if inp.is_file():
        entries = extract_one(inp, inp, fix_orig=args.fix_orig, strict=not args.non_strict)
        save_json(out, entries)
        total = len(entries)
    else:
        for file in files:
            entries = extract_one(file, inp, fix_orig=args.fix_orig, strict=not args.non_strict)
            if not entries:
                continue
            rel = file.relative_to(inp).as_posix().replace("/", "__")
            save_json(out / f"{rel}.json", entries)
            total += len(entries)
    print(f"[extract] files={len(files)} entries={total} output={out}")


if __name__ == "__main__":
    main()
