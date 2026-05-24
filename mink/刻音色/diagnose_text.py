# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path

from keyinse_common import (
    DEFAULT_ENCODING,
    TOOL_VERSION,
    TEXT_OP,
    CHOICE_OP,
    apply_page_marks_from_scr_msg,
    decode_choice_instruction,
    decode_text_instruction,
    entries_for_file,
    parse_instructions,
    read_json,
    renderer_layout_report,
    strip_page_marks,
)


def collect_original_texts(data: bytes, encoding: str):
    out = []
    text_index = 0
    for inst in parse_instructions(data):
        if inst.op == TEXT_OP:
            old = decode_text_instruction(inst, encoding)
            out.append((text_index, "text", inst.offset, old, inst.b3))
            text_index += 1
        elif inst.op == CHOICE_OP:
            old = decode_choice_instruction(inst, encoding)
            out.append((text_index, "choice", inst.offset, old, inst.b2))
            text_index += 1
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Diagnose translated text layout around a script text index")
    ap.add_argument("script", help="original .s file")
    ap.add_argument("json", help="translation json for this .s file")
    ap.add_argument("--index", type=int, required=True, help="center text index")
    ap.add_argument("--radius", type=int, default=5, help="number of entries before/after to show")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--page-mark-mode", choices=["auto-fit", "proportional", "byte-offset", "manual", "none"], default="auto-fit")
    ap.add_argument("--max-columns", type=int, default=23)
    ap.add_argument("--max-rows", type=int, default=4)
    args = ap.parse_args()

    script = Path(args.script)
    entries = read_json(args.json)
    entry_map = entries_for_file(entries, script.name)
    originals = collect_original_texts(script.read_bytes(), args.encoding)

    start = max(0, args.index - args.radius)
    end = args.index + args.radius
    rows = []
    for idx, kind, off, old_text, old_size in originals:
        if not (start <= idx <= end):
            continue
        e = entry_map.get(idx)
        if e is None:
            message = old_text
            inject_text = old_text
            note = "NO_JSON_ENTRY"
        else:
            message = e.get("message", "")
            scr_msg = e.get("scr_msg", "")
            note = ""
            if scr_msg != old_text:
                note = "SCR_MSG_MISMATCH"
                inject_text = message
            elif kind == "text":
                if strip_page_marks(message) == strip_page_marks(scr_msg):
                    inject_text = old_text
                    note = "UNCHANGED_ROUNDTRIP"
                else:
                    inject_text = apply_page_marks_from_scr_msg(scr_msg, message, args.encoding, args.page_mark_mode)
            else:
                inject_text = message
        try:
            enc = inject_text.encode(args.encoding)
            enc_hex = enc.hex(" ")
            enc_len = len(enc)
        except Exception as ex:
            enc_hex = f"<encode error: {ex}>"
            enc_len = -1
        if kind == "text":
            report = renderer_layout_report(inject_text, args.encoding, args.max_columns, args.max_rows)
        else:
            report = {"ok": True, "rows": 0, "segment_units": [], "warnings": []}
        rows.append({
            "index": idx,
            "kind": kind,
            "inst_offset": f"0x{off:X}",
            "old_size": old_size,
            "old_text": old_text,
            "json_message": message,
            "inject_text": inject_text,
            "encoded_len": enc_len,
            "encoded_hex_head": enc_hex[:240],
            "layout_ok": report.get("ok"),
            "rows": report.get("rows"),
            "segment_units": report.get("segment_units"),
            "segment_rows": report.get("segment_rows"),
            "max_segment_rows": report.get("max_segment_rows"),
            "layout_warnings": report.get("warnings"),
            "note": note,
        })

    print(json.dumps({
        "tool_version": TOOL_VERSION,
        "script": script.name,
        "center_index": args.index,
        "page_mark_mode": args.page_mark_mode,
        "max_columns": args.max_columns,
        "max_rows": args.max_rows,
        "entries": rows,
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
