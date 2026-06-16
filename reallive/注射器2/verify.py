# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse, json
from collections import defaultdict
from pathlib import Path
from typing import Any

from common import (
    DEFAULT_ENCODING, compose_entry_text, decode_seen_chunk, encode_text, entry_text_range,
    iter_seen_entries, iter_text_entries, load_xor_key_from_export, patch_inline_jump_targets,
    replace_ranges,
)
from inject import load_json_files, apply_map, load_char_map, validate_entry

SUSPICIOUS_EXACT = {
    '祐','崖','皛','芫','弖',')就',')ア',')曚',')宗',')擢','))擢',')櫓','()弖','()皛','()芫'
}


def basic_verify(seen_path: Path, ida_export: str | None = None) -> tuple[bool, dict[str, Any]]:
    key = load_xor_key_from_export(ida_export)
    data = seen_path.read_bytes()
    files = entries = 0
    decode_errors: list[str] = []
    suspicious: list[dict[str, Any]] = []
    missing_fields: list[dict[str, Any]] = []
    replacement_char: list[dict[str, Any]] = []

    for ent in iter_seen_entries(data):
        try:
            dec = decode_seen_chunk(ent, data, key)
            arr = iter_text_entries(dec)
        except Exception as ex:
            decode_errors.append(f'Seen{ent.seen_no:04d}: {ex}')
            continue
        if arr:
            files += 1
        entries += len(arr)
        for e in arr:
            obj = e.to_json()
            msg = obj.get('message', '')
            if not all(k in obj for k in ('scr_msg','message','_file','_index','_seen_no','_type')):
                missing_fields.append({'file': obj.get('_file'), 'index': obj.get('_index'), 'message': msg})
            if msg in SUSPICIOUS_EXACT or (isinstance(msg, str) and msg.startswith(')') and len(msg) <= 4):
                suspicious.append({'file': obj.get('_file'), 'index': obj.get('_index'), 'line': obj.get('_line'), 'message': msg})
            if isinstance(msg, str) and '\ufffd' in msg:
                replacement_char.append({'file': obj.get('_file'), 'index': obj.get('_index'), 'line': obj.get('_line'), 'message': msg})

    ok = not decode_errors and not suspicious and not missing_fields and not replacement_char
    return ok, {
        'files': files,
        'entries': entries,
        'decode_errors': decode_errors[:50],
        'suspicious': suspicious[:50],
        'missing_fields': missing_fields[:50],
        'replacement_char': replacement_char[:50],
        'decode_error_count': len(decode_errors),
        'suspicious_count': len(suspicious),
        'missing_field_count': len(missing_fields),
        'replacement_char_count': len(replacement_char),
    }


def full_verify(original_path: Path, patched_path: Path, json_path: Path,
                ida_export: str | None = None, map_json: str | None = None) -> tuple[bool, dict[str, Any]]:
    key = load_xor_key_from_export(ida_export)
    cmap = load_char_map(map_json)
    orig_data = original_path.read_bytes()
    patched_data = patched_path.read_bytes()
    orig_entries = {e.seen_no: e for e in iter_seen_entries(orig_data)}
    patched_entries = {e.seen_no: e for e in iter_seen_entries(patched_data)}

    by_seen: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for e in load_json_files(json_path):
        no = e.get('_seen_no')
        if isinstance(no, int):
            by_seen[no].append(e)

    mismatch: list[dict[str, Any]] = []
    bad_json: list[dict[str, Any]] = []
    total_replacements = 0
    total_jump_patches = 0
    checked_seen = 0

    for seen_no, edits in sorted(by_seen.items()):
        if seen_no not in orig_entries:
            bad_json.append({'seen': seen_no, 'error': 'not present in original'})
            continue
        if seen_no not in patched_entries:
            mismatch.append({'seen': seen_no, 'error': 'not present in patched'})
            continue
        orig_dec = decode_seen_chunk(orig_entries[seen_no], orig_data, key)
        patched_dec = decode_seen_chunk(patched_entries[seen_no], patched_data, key)
        replacements: list[tuple[int, int, bytes]] = []
        used: set[tuple[int, int]] = set()
        for e in sorted(edits, key=lambda x: int(x.get('_index', 0))):
            msg = e.get('message')
            scr = e.get('scr_msg')
            if not isinstance(msg, str) or not isinstance(scr, str):
                bad_json.append({'seen': seen_no, 'index': e.get('_index'), 'error': 'missing message/scr_msg'})
                continue
            old_composed = compose_entry_text(e, scr, source_field='scr_msg')
            new_composed = compose_entry_text(e, apply_map(msg, cmap), source_field='message')
            if old_composed == new_composed:
                continue
            try:
                start, old_len = entry_text_range(e)
            except Exception as ex:
                bad_json.append({'seen': seen_no, 'index': e.get('_index'), 'error': f'locate failed: {ex}'})
                continue
            if (start, old_len) in used:
                continue
            if not validate_entry(orig_dec, e, start, old_len):
                got = orig_dec.code[start:start+old_len].decode(e.get('_encoding', DEFAULT_ENCODING), errors='replace')
                bad_json.append({'seen': seen_no, 'index': e.get('_index'), 'error': 'scr_msg mismatch', 'file': got[:120]})
                continue
            try:
                new_raw = encode_text(new_composed, e.get('_encoding', DEFAULT_ENCODING))
            except Exception as ex:
                bad_json.append({'seen': seen_no, 'index': e.get('_index'), 'error': f'encode failed: {ex}'})
                continue
            replacements.append((start, old_len, new_raw))
            used.add((start, old_len))
        if not replacements:
            if patched_dec.code != orig_dec.code:
                mismatch.append({'seen': seen_no, 'error': 'changed although no replacements expected'})
            continue
        expected, delta_recs = replace_ranges(orig_dec.code, replacements)
        expected_buf = bytearray(expected)
        jump_patches = patch_inline_jump_targets(orig_dec.code, expected_buf, delta_recs)
        expected = bytes(expected_buf)
        total_replacements += len(replacements)
        total_jump_patches += jump_patches
        checked_seen += 1
        if expected != patched_dec.code:
            # Find first mismatch for diagnosis.
            first = None
            for i, (a, b) in enumerate(zip(expected, patched_dec.code)):
                if a != b:
                    first = i
                    break
            if first is None and len(expected) != len(patched_dec.code):
                first = min(len(expected), len(patched_dec.code))
            mismatch.append({
                'seen': seen_no,
                'error': 'decoded code differs from expected relocation result',
                'expected_len': len(expected),
                'patched_len': len(patched_dec.code),
                'first_diff': first,
            })

    ok_basic, basic = basic_verify(patched_path, ida_export)
    ok = not mismatch and not bad_json and ok_basic
    return ok, {
        'checked_seen': checked_seen,
        'total_replacements': total_replacements,
        'total_jump_patches': total_jump_patches,
        'mismatch_count': len(mismatch),
        'bad_json_count': len(bad_json),
        'mismatch': mismatch[:50],
        'bad_json': bad_json[:50],
        'patched_basic': basic,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description='Verify RealLive Seen.txt extraction/injection structure')
    ap.add_argument('seen_txt', help='patched or original Seen.txt to verify')
    ap.add_argument('--original', help='original Seen.txt; enables relocation verification with --json')
    ap.add_argument('--json', help='translation json directory/file; used with --original')
    ap.add_argument('--ida-export', help='optional IDA export dir; normally not needed')
    ap.add_argument('--map-json', help='same char map used during injection')
    args = ap.parse_args()

    if args.original and args.json:
        ok, report = full_verify(Path(args.original), Path(args.seen_txt), Path(args.json), args.ida_export, args.map_json)
    else:
        ok, report = basic_verify(Path(args.seen_txt), args.ida_export)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if not ok:
        raise SystemExit(1)

if __name__ == '__main__':
    main()
