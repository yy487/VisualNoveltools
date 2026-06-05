# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

# Load the local required file named opcode.py without relying on Python's
# normal import resolution, because stdlib also has an opcode module.
def _load_local_opcode():
    here = Path(__file__).resolve().parent
    spec = importlib.util.spec_from_file_location("baigui_local_opcode", here / "opcode.py")
    mod = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(mod)
    return mod

_op = _load_local_opcode()
DEFAULT_ENCODING = _op.DEFAULT_ENCODING
TEXT_BLOCK_CTRL = _op.TEXT_BLOCK_CTRL
TEXT_STRING_CTRL = _op.TEXT_STRING_CTRL
decode_cp = _op.decode_cp
escape_string_bytes = _op.escape_string_bytes
is_choice_label = _op.is_choice_label
is_display_string = _op.is_display_string
is_probably_resource = _op.is_probably_resource
load_json = _op.load_json
lzss_decompress = _op.lzss_decompress
quote_asm_string = _op.quote_asm_string
save_json = _op.save_json
sha256_bytes = _op.sha256_bytes

MAX_CSTRING = 512


def try_decode_cstring(data: bytes, start: int, encoding: str):
    """Return (end_zero_offset, text, raw) if data[start] is a 0x01 cstring."""
    if start >= len(data) or data[start] != TEXT_STRING_CTRL:
        return None
    end = data.find(b"\x00", start + 1, min(len(data), start + 1 + MAX_CSTRING))
    if end < 0:
        return None
    raw = data[start + 1:end]
    if not raw:
        return None
    # Avoid obvious binary blobs.
    if any(b < 0x09 for b in raw):
        return None
    # Render with the same semantic placeholder logic used by asm strings.
    # This prevents orphan SJIS lead bytes from turning into bogus glyphs.
    try:
        text = escape_string_bytes(raw, encoding)
    except Exception:
        return None
    if not text or any(ord(ch) < 0x20 for ch in text):
        return None
    return end, text, raw


def scan_cstrings(data: bytes, encoding: str):
    records = []
    i = 0
    while i < len(data):
        hit = try_decode_cstring(data, i, encoding)
        if hit is None:
            i += 1
            continue
        end, text, raw = hit
        typ = "text"
        if is_probably_resource(text):
            typ = "resource"
        elif is_choice_label(text):
            typ = "choice_label"
        records.append({
            "ctrl_offset": i,
            "offset": i + 1,
            "end_offset": end,
            "next_offset": end + 1,
            "raw": raw,
            "text": text,
            "kind": typ,
        })
        i = end + 1
    return records


def block_starts(data: bytes):
    return [i for i, b in enumerate(data) if b == TEXT_BLOCK_CTRL]


def _has_japanese_punct(text: str) -> bool:
    return any(ch in text for ch in "「」『』。！？…")


def _is_quoted_message(text: str) -> bool:
    return text.startswith("「") or text.startswith("『")


def _is_prose_message(text: str) -> bool:
    # Conservative: long/prose strings with Japanese sentence punctuation.
    if _is_quoted_message(text):
        return True
    if any(ch in text for ch in "。！？…"):
        return True
    # Some narration lines use Japanese comma but no period; require enough length.
    if len(text) >= 18 and "、" in text:
        return True
    return False


def _is_speaker_candidate(text: str) -> bool:
    if not text or len(text) > 12:
        return False
    if _has_japanese_punct(text):
        return False
    # Avoid map hotspots / object labels that often include location/object suffixes.
    object_words = ("床", "棚", "窓", "天井", "机", "椅子", "壁", "箱", "板", "ゴミ", "蛍光灯", "ストーブ")
    if text in object_words or any(text.endswith(w) and len(text) > 2 for w in object_words):
        return False
    return True


def _is_short_choice_text(text: str) -> bool:
    if not text or len(text) > 24:
        return False
    if _is_quoted_message(text):
        return False
    if any(ch in text for ch in "。！？…"):
        return False
    return True


def assign_blocks_and_types(data: bytes, records: list[dict]):
    """Assign semantic entry types using the current VM/text-stream model.

    This is intentionally conservative.  The previous version treated every
    decodable 0x01 cstring as dialogue, which pulled in resources, map object
    labels, and script identifiers such as bg26_1.a6.  Here we only export:
      * choice strings after an explicit 選択肢 label,
      * quoted dialogue, optionally paired with a nearby short speaker name,
      * prose/narration strings with sentence punctuation.
    Other 0x01 cstrings remain visible in asm but are not exported to JSON.
    """
    starts = block_starts(data)
    si = 0
    for r in records:
        while si + 1 < len(starts) and starts[si + 1] <= r["ctrl_offset"]:
            si += 1
        r["block_offset"] = starts[si] if starts and starts[si] <= r["ctrl_offset"] else -1
        r["entry_type"] = None
        r["choice_label"] = None

    # Choice groups.  The engine commonly stores a non-displayed label such as
    # "選択肢：..." followed by the visible menu strings.
    active_label = None
    choice_index = 0
    label_start = 0
    for r in records:
        text = r["text"]
        if r["kind"] == "choice_label":
            active_label = text
            choice_index = 0
            label_start = r["ctrl_offset"]
            r["entry_type"] = "label"
            continue
        if r["kind"] == "resource" or not is_display_string(text):
            continue
        if active_label:
            # Keep the choice window local; stop when normal prose/dialogue appears
            # or when too much unrelated data has passed.
            if r["ctrl_offset"] - label_start > 0x900:
                active_label = None
            elif _is_short_choice_text(text):
                r["entry_type"] = "choice"
                r["choice_label"] = active_label
                r["choice_index"] = choice_index
                choice_index += 1
                # Most menus have 2-6 items.  Do not force-close immediately;
                # a prose string below will close it.
                continue
            elif _is_prose_message(text):
                active_label = None

    # Dialogue and monologue.  Speaker names are not exported independently;
    # they are only attached to the following quoted message when sufficiently
    # close in the stream.
    pending_speaker = None
    for r in records:
        if r.get("entry_type") or r["kind"] != "text":
            continue
        text = r["text"]
        if not is_display_string(text):
            continue
        if _is_speaker_candidate(text):
            pending_speaker = r
            continue
        if _is_quoted_message(text):
            if pending_speaker and 0 <= r["ctrl_offset"] - pending_speaker["ctrl_offset"] <= 0x280:
                pending_speaker["entry_type"] = "name_part"
                r["entry_type"] = "dialogue"
                r["name_record"] = pending_speaker
            else:
                r["entry_type"] = "monologue"
            pending_speaker = None
            continue
        if _is_prose_message(text):
            r["entry_type"] = "monologue"
            pending_speaker = None
            continue
        # Otherwise keep it in asm only.  This covers map object labels,
        # script markers, and other UI/internal names.

    return records

def build_json_entries(records: list[dict], file_name: str, encoding: str):
    entries = []
    index = 0
    used_message_offsets = set()
    for r in records:
        et = r.get("entry_type")
        if et == "choice":
            entries.append({
                "scr_msg": r["text"],
                "message": r["text"],
                "_file": file_name,
                "_index": index,
                "_type": "choice",
                "_offset": r["offset"],
                "_inst_offset": r["ctrl_offset"],
                "_size": len(r["raw"]),
                "_cstr_size": len(r["raw"]) + 1,
                "_choice_index": r.get("choice_index", 0),
                "_label": r.get("choice_label"),
                "_encoding": encoding,
                "_policy": "relocate",
            })
            index += 1
            used_message_offsets.add(r["offset"])
        elif et == "dialogue":
            name_rec = r.get("name_record")
            obj = {
                "scr_msg": r["text"],
                "message": r["text"],
                "_file": file_name,
                "_index": index,
                "_type": "dialogue",
                "_offset": r["offset"],
                "_inst_offset": r["ctrl_offset"],
                "_size": len(r["raw"]),
                "_cstr_size": len(r["raw"]) + 1,
                "_block_offset": r.get("block_offset"),
                "_encoding": encoding,
                "_policy": "relocate",
            }
            if name_rec:
                obj = {"name": name_rec["text"], **obj}
                obj["_name_offset"] = name_rec["offset"]
                obj["_name_inst_offset"] = name_rec["ctrl_offset"]
                obj["_name_size"] = len(name_rec["raw"])
            entries.append(obj)
            index += 1
            used_message_offsets.add(r["offset"])
        elif et == "monologue":
            if r["offset"] in used_message_offsets:
                continue
            entries.append({
                "scr_msg": r["text"],
                "message": r["text"],
                "_file": file_name,
                "_index": index,
                "_type": "monologue",
                "_offset": r["offset"],
                "_inst_offset": r["ctrl_offset"],
                "_size": len(r["raw"]),
                "_cstr_size": len(r["raw"]) + 1,
                "_block_offset": r.get("block_offset"),
                "_encoding": encoding,
                "_policy": "relocate",
            })
            index += 1
    return entries


def disassemble_to_asm(data: bytes, records: list[dict], out_path: Path, source_name: str, encoding: str, container: str, original_sha: str):
    rec_by_ctrl = {r["ctrl_offset"]: r for r in records}
    out = []
    out.append("; Baigui MES semantic assembly")
    out.append(f"; source: {source_name}")
    out.append(f"; encoding: {encoding}")
    out.append(f"; container: {container}")
    out.append(f"; original_sha256: {original_sha}")
    out.append(f"; plain_sha256: {sha256_bytes(data)}")
    out.append("; .cstring1 emits: 0x01 + encoded string bytes + 0x00")
    out.append("; .byte is used for opaque VM/control bytes that are preserved exactly")
    out.append("")

    i = 0
    pending = []

    def flush_pending():
        if not pending:
            return
        start = i - len(pending)
        out.append(f"loc_{start:08X}:")
        # line wrap .byte data
        for j in range(0, len(pending), 16):
            chunk = pending[j:j+16]
            out.append("    .byte " + ", ".join(f"0x{x:02X}" for x in chunk))
        pending.clear()
        out.append("")

    while i < len(data):
        r = rec_by_ctrl.get(i)
        if r is None:
            pending.append(data[i])
            i += 1
            continue
        flush_pending()
        out.append(f"str_{r['offset']:08X}:")
        comment_parts = [f"kind={r.get('kind','text')}"]
        if r.get("entry_type"):
            comment_parts.append(f"type={r['entry_type']}")
        if r.get("block_offset", -1) >= 0:
            comment_parts.append(f"block=0x{r['block_offset']:08X}")
        if r.get("choice_label"):
            comment_parts.append("choice_label=" + r["choice_label"])
        raw = r["raw"]
        out.append("    .cstring1 " + quote_asm_string(raw, encoding) + "    ; " + "; ".join(comment_parts))
        out.append("")
        i = r["next_offset"]
    flush_pending()

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(out).rstrip() + "\n", encoding="utf-8", newline="\n")


def read_input(path: Path, plain: bool):
    raw = path.read_bytes()
    if plain:
        return raw, "plain", sha256_bytes(raw)
    plain_data = lzss_decompress(raw)
    return plain_data, "lzss", sha256_bytes(raw)


def process_one(input_path: Path, output_asm: Path | None, output_json: Path | None, encoding: str, plain: bool):
    plain_data, container, original_sha = read_input(input_path, plain)
    records = assign_blocks_and_types(plain_data, scan_cstrings(plain_data, encoding))
    asm_path = output_asm or input_path.with_suffix(input_path.suffix + ".asm.txt")
    json_path = output_json
    disassemble_to_asm(plain_data, records, asm_path, input_path.name, encoding, container, original_sha)
    if json_path:
        entries = build_json_entries(records, input_path.name, encoding)
        save_json(json_path, entries)
        print(f"[json] {json_path} entries={len(entries)}")
    print(f"[disasm] {input_path} -> {asm_path}")
    print(f"[disasm] plain_size={len(plain_data)} cstrings={len(records)} container={container}")


def process_dir(input_dir: Path, out_dir: Path, json_dir: Path | None, encoding: str, plain: bool):
    files = [p for p in sorted(input_dir.rglob("*")) if p.is_file() and p.suffix.lower() == ".mes"]
    total_entries = 0
    for p in files:
        rel = p.relative_to(input_dir)
        asm_path = out_dir / rel.with_suffix(rel.suffix + ".asm.txt")
        json_path = None
        if json_dir:
            json_path = json_dir / (rel.as_posix().replace("/", "__") + ".json")
        plain_data, container, original_sha = read_input(p, plain)
        records = assign_blocks_and_types(plain_data, scan_cstrings(plain_data, encoding))
        disassemble_to_asm(plain_data, records, asm_path, p.name, encoding, container, original_sha)
        if json_path:
            entries = build_json_entries(records, p.name, encoding)
            save_json(json_path, entries)
            total_entries += len(entries)
    print(f"[batch-disasm] files={len(files)} asm_out={out_dir}")
    if json_dir:
        print(f"[batch-json] entries={total_entries} json_out={json_dir}")


def main(argv=None):
    ap = argparse.ArgumentParser(description="Baigui MES LZSS disassembler / JSON exporter")
    ap.add_argument("input", help="input .MES file or directory")
    ap.add_argument("-o", "--output", help="output asm file or directory")
    ap.add_argument("--json", help="optional JSON output path or directory")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--plain", action="store_true", help="input is already decompressed plain MES")
    args = ap.parse_args(argv)

    inp = Path(args.input)
    if inp.is_dir():
        out_dir = Path(args.output) if args.output else inp.with_name(inp.name + "_asm")
        json_dir = Path(args.json) if args.json else None
        process_dir(inp, out_dir, json_dir, args.encoding, args.plain)
    else:
        out_asm = Path(args.output) if args.output else None
        out_json = Path(args.json) if args.json else None
        process_one(inp, out_asm, out_json, args.encoding, args.plain)


if __name__ == "__main__":
    main()
