# -*- coding: utf-8 -*-
"""Recover translation JSON from a previously patched Refrain Blue MES.ARC.

This is for the old buggy injector output that used shorter in-place strings.
It compares original MES.ARC and patched MES.ARC and writes JSON files whose
scr_msg comes from the original archive and message comes from the patched one.
"""
from __future__ import annotations

import argparse
from pathlib import Path
from rp_arc import read_arc
from rp_mes_common import scan_mes_text, save_json


def read_new_text_at(new_data: bytes, off: int) -> str | None:
    if off >= len(new_data):
        return None
    op = new_data[off]
    if op == 0x01:
        end = new_data.find(b"\x00", off + 1)
        if end < 0:
            return None
        return new_data[off + 1:end].decode("cp932", "replace")
    if op == 0x0A and off + 5 <= len(new_data):
        target = int.from_bytes(new_data[off + 1:off + 5], "little")
        if 0 <= target < len(new_data) and new_data[target] == 0x01:
            end = new_data.find(b"\x00", target + 1)
            if end < 0:
                return None
            return new_data[target + 1:end].decode("cp932", "replace")
    return None


def main() -> None:
    ap = argparse.ArgumentParser(description="Recover JSON translations from old patched MES.ARC")
    ap.add_argument("original_arc")
    ap.add_argument("patched_arc")
    ap.add_argument("json_out")
    args = ap.parse_args()

    orig = {e.name: e.data for e in read_arc(args.original_arc)}
    new = {e.name: e.data for e in read_arc(args.patched_arc)}
    out_dir = Path(args.json_out)
    out_dir.mkdir(parents=True, exist_ok=True)

    total = changed = 0
    for name, orig_data in orig.items():
        if not name.lower().endswith(".mes") or name not in new:
            continue
        new_data = new[name]
        entries = []
        for r in scan_mes_text(orig_data, name, include_ruby=True):
            msg = read_new_text_at(new_data, r._offset)
            obj = r.to_json_obj()
            if msg is not None:
                obj["message"] = msg
                if msg != r.scr_msg:
                    changed += 1
            entries.append(obj)
            total += 1
        save_json(out_dir / (Path(name).stem + ".json"), entries)

    print(f"[recover] entries={total} changed={changed} out={out_dir}")


if __name__ == "__main__":
    main()
