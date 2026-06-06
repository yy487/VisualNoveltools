# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path

from keyinse_common import (
    DEFAULT_ENCODING,
    TEXT_OP,
    TARGET_FIELD_OFFSETS,
    TOOL_VERSION,
    _make_boundary_mapper,
    decode_text_instruction,
    iter_script_files,
    make_text_instruction,
    parse_instructions,
    put_u32le,
    renderer_layout_report,
    split_text_for_renderer,
    u32le,
)


def fix_one(data: bytes, file_name: str, encoding: str) -> tuple[bytes, dict]:
    instructions = parse_instructions(data)
    rebuilt: list[bytearray] = []
    old_offsets: list[int] = []
    old_lengths: list[int] = []
    new_offsets: list[int] = []
    new_pos = 0
    text_index = 0
    stats = {
        "file": file_name,
        "old_size": len(data),
        "split_entries": 0,
        "split_extra_instructions": 0,
        "fixed_targets": 0,
        "warnings": [],
    }

    for inst in instructions:
        raw_new = bytes(inst.raw)
        if inst.op == TEXT_OP:
            text = decode_text_instruction(inst, encoding)
            report = renderer_layout_report(text, encoding)
            if not report.get("ok", False):
                chunks = split_text_for_renderer(text, encoding)
                if len(chunks) > 1:
                    raw_new = b"".join(make_text_instruction(inst, chunk, encoding, keep_payload_size=None) for chunk in chunks)
                    stats["split_entries"] += 1
                    stats["split_extra_instructions"] += len(chunks) - 1
                    stats["warnings"].append({
                        "index": text_index,
                        "offset": f"0x{inst.offset:X}",
                        "old_rows": report.get("rows"),
                        "old_segment_rows": report.get("segment_rows"),
                        "chunks": len(chunks),
                        "chunk_reports": [renderer_layout_report(c, encoding) for c in chunks],
                        "text": text,
                    })
            text_index += 1
        elif inst.op == 0x1B:
            text_index += 1

        old_offsets.append(inst.offset)
        old_lengths.append(inst.length)
        new_offsets.append(new_pos)
        rebuilt.append(bytearray(raw_new))
        new_pos += len(raw_new)

    map_offset = _make_boundary_mapper(old_offsets, old_lengths, new_offsets)
    unresolved = []
    for inst, raw in zip(instructions, rebuilt):
        for field_off in TARGET_FIELD_OFFSETS.get(inst.op, ()):
            if field_off + 4 > len(inst.raw) or field_off + 4 > len(raw):
                continue
            old_target = u32le(inst.raw, field_off)
            mapped = map_offset(old_target)
            if mapped is None:
                unresolved.append(f"0x{inst.offset:X}:op=0x{inst.op:02X}:target=0x{old_target:X}")
                continue
            if mapped != old_target:
                put_u32le(raw, field_off, mapped)
                stats["fixed_targets"] += 1

    if unresolved:
        stats["unresolved_targets"] = unresolved[:50]
        if len(unresolved) > 50:
            stats["unresolved_targets_more"] = len(unresolved) - 50

    out = b"".join(bytes(x) for x in rebuilt)
    parse_instructions(out)
    stats["new_size"] = len(out)
    return out, stats


def main() -> None:
    ap = argparse.ArgumentParser(description="Split already-injected 刻音色 .s texts that exceed the 3-row renderer limit")
    ap.add_argument("input", help="input .s file or script directory")
    ap.add_argument("output", help="output .s file or script directory")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--stats-json", help="write machine-readable stats JSON")
    ap.add_argument("--version", action="version", version=f"%(prog)s {TOOL_VERSION}-fixed")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    all_stats = []

    if inp.is_file():
        patched, stats = fix_one(inp.read_bytes(), inp.name, args.encoding)
        out_path = out / inp.name if out.exists() and out.is_dir() else out
        if not out_path.suffix:
            out_path.mkdir(parents=True, exist_ok=True)
            out_path = out_path / inp.name
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(patched)
        all_stats.append(stats)
        print(f"[split] file={inp.name} split_entries={stats['split_entries']} fixed_targets={stats['fixed_targets']} size={stats['old_size']}->{stats['new_size']} output={out_path}")
    else:
        out.mkdir(parents=True, exist_ok=True)
        scripts = list(iter_script_files(inp))
        for script in scripts:
            rel = script.relative_to(inp)
            dst = out / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            patched, stats = fix_one(script.read_bytes(), script.name, args.encoding)
            dst.write_bytes(patched)
            all_stats.append(stats)
        print(f"[split] files={len(scripts)} split_entries={sum(s['split_entries'] for s in all_stats)} fixed_targets={sum(s['fixed_targets'] for s in all_stats)} output={out}")

    if args.stats_json:
        p = Path(args.stats_json)
        p.parent.mkdir(parents=True, exist_ok=True)
        with open(p, "w", encoding="utf-8", newline="\n") as f:
            json.dump(all_stats, f, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main()
