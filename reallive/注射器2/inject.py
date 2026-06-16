# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse, json, shutil
from collections import defaultdict
from pathlib import Path
from typing import Any

from common import (
    SEEN_INDEX_SIZE, DEFAULT_ENCODING, compose_entry_text, decode_seen_chunk, encode_text,
    entry_text_range, iter_seen_entries, load_xor_key_from_export,
    rebuild_seen_chunk_with_code, replace_ranges, write_u32,
)


def load_json_files(json_path: Path) -> list[dict[str, Any]]:
    files = sorted(json_path.glob('*.json')) if json_path.is_dir() else [json_path]
    out: list[dict[str, Any]] = []
    for f in files:
        data = json.loads(f.read_text(encoding='utf-8'))
        if not isinstance(data, list):
            raise ValueError(f'JSON root must be list: {f}')
        for e in data:
            if isinstance(e, dict):
                e['_json_file'] = str(f)
                out.append(e)
    return out


def apply_map(text: str, cmap: dict[str, str] | None) -> str:
    if not cmap:
        return text
    return ''.join(cmap.get(ch, ch) for ch in text)


def load_char_map(path: str | None) -> dict[str, str] | None:
    if not path:
        return None
    data = json.loads(Path(path).read_text(encoding='utf-8'))
    if not isinstance(data, dict):
        raise ValueError('map json must be an object')
    return {str(k): str(v) for k, v in data.items()}


def validate_entry(decoded, entry: dict[str, Any], start: int, old_len: int) -> bool:
    scr = entry.get('scr_msg')
    enc = entry.get('_encoding', DEFAULT_ENCODING)
    if not isinstance(scr, str):
        return False
    old_raw = decoded.code[start:start+old_len]
    try:
        old_text = old_raw.decode(enc)
    except Exception:
        return False

    expected = compose_entry_text(entry, scr, source_field='scr_msg')
    if old_text == expected:
        return True

    # Compatibility for JSON extracted before `_scr_name` was added, or JSON
    # where translators already edited `name`.  In bracket-prefix entries, the
    # source body is the stable verification key; the current editable `name`
    # must not make validation fail.
    if entry.get('_name_source') == 'bracket_prefix':
        from common import split_bracket_name
        old_name, old_body, old_src = split_bracket_name(old_text)
        return old_src == 'bracket_prefix' and old_body == scr

    return False


def inject_seen_txt(seen_path: Path, export_dir: Path | None, json_path: Path, out_path: Path,
                    only_seen: set[int] | None = None, map_json: str | None = None,
                    strict: bool = True) -> tuple[int, int, int]:
    cmap = load_char_map(map_json)
    seen_data = bytearray(seen_path.read_bytes())
    key = load_xor_key_from_export(export_dir)

    by_seen: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for e in load_json_files(json_path):
        no = e.get('_seen_no')
        if isinstance(no, int) and (only_seen is None or no in only_seen):
            by_seen[no].append(e)

    entries = {e.seen_no: e for e in iter_seen_entries(seen_data)}
    rebuilt_chunks: dict[int, bytes] = {}
    patched = failed = skipped = 0

    for seen_no, edits in sorted(by_seen.items()):
        if seen_no not in entries:
            failed += len(edits)
            print(f'[inject][warn] Seen{seen_no:04d} not present')
            continue
        decoded = decode_seen_chunk(entries[seen_no], seen_data, key)
        replacements: list[tuple[int, int, bytes]] = []
        used_ranges: set[tuple[int, int]] = set()

        for e in sorted(edits, key=lambda x: int(x.get('_index', 0))):
            msg = e.get('message')
            scr = e.get('scr_msg')
            if not isinstance(msg, str) or not isinstance(scr, str):
                failed += 1
                print(f'[inject][warn] bad json entry Seen{seen_no:04d} index={e.get("_index")}')
                continue
            old_composed = compose_entry_text(e, scr, source_field='scr_msg')
            new_body = apply_map(msg, cmap)
            new_composed = compose_entry_text(e, new_body, source_field='message')
            if new_composed == old_composed:
                skipped += 1
                continue
            try:
                start, old_len = entry_text_range(e)
            except Exception as ex:
                failed += 1
                print(f'[inject][warn] locate failed Seen{seen_no:04d} index={e.get("_index")}: {ex}')
                continue
            if (start, old_len) in used_ranges:
                skipped += 1
                continue
            if strict and not validate_entry(decoded, e, start, old_len):
                failed += 1
                got = decoded.code[start:start+old_len].decode(e.get('_encoding', DEFAULT_ENCODING), errors='replace')
                print(f'[inject][warn] scr_msg mismatch Seen{seen_no:04d} index={e.get("_index")} line={e.get("_line")}')
                print(f'  json: {old_composed}')
                print(f'  file: {got}')
                continue
            try:
                new_raw = encode_text(new_composed, e.get('_encoding', DEFAULT_ENCODING))
            except UnicodeEncodeError as ex:
                failed += 1
                print(f'[inject][warn] encode failed Seen{seen_no:04d} index={e.get("_index")}: {ex}')
                continue
            replacements.append((start, old_len, new_raw))
            used_ranges.add((start, old_len))
            patched += 1

        if replacements:
            new_code, delta_recs = replace_ranges(decoded.code, replacements)
            rebuilt_chunks[seen_no] = rebuild_seen_chunk_with_code(decoded, new_code, key, delta_recs)

    # Rebuild Seen.txt index and chunk area. Empty/unmodified entries are copied byte-exact.
    index = bytearray(SEEN_INDEX_SIZE)
    body = bytearray()
    cur = SEEN_INDEX_SIZE
    for e in iter_seen_entries(seen_data):
        chunk = rebuilt_chunks.get(e.seen_no, bytes(seen_data[e.offset:e.offset+e.size]))
        write_u32(index, e.seen_no * 8, cur)
        write_u32(index, e.seen_no * 8 + 4, len(chunk))
        body.extend(chunk)
        cur += len(chunk)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(bytes(index) + bytes(body))
    return patched, skipped, failed


def main() -> None:
    ap = argparse.ArgumentParser(description='RealLive Seen.txt variable-length injector')
    ap.add_argument('seen_txt', help='original Seen.txt')
    ap.add_argument('arg2', help='json path, or legacy exe_export when arg4 is supplied')
    ap.add_argument('arg3', help='output Seen.txt, or legacy json path when arg4 is supplied')
    ap.add_argument('arg4', nargs='?', help='legacy output Seen.txt when exe_export is supplied')
    ap.add_argument('--ida-export', help='optional IDA export dir; normally not needed because crypt_template.py is bundled')
    ap.add_argument('--seen', type=int, nargs='*', help='only inject selected SEEN numbers')
    ap.add_argument('--map-json', help='optional one-char substitution map before cp932 encoding')
    ap.add_argument('--no-strict', action='store_true', help='disable scr_msg byte-range validation')
    args = ap.parse_args()

    if args.arg4 is None:
        export_dir = Path(args.ida_export) if args.ida_export else None
        json_path = Path(args.arg2)
        out_path = Path(args.arg3)
    else:
        export_dir = Path(args.ida_export or args.arg2)
        json_path = Path(args.arg3)
        out_path = Path(args.arg4)

    patched, skipped, failed = inject_seen_txt(
        Path(args.seen_txt), export_dir, json_path, out_path,
        set(args.seen) if args.seen else None, args.map_json, strict=not args.no_strict,
    )
    src = 'static crypt_template.py' if export_dir is None else str(export_dir)
    print(f'[inject] patched={patched} skipped={skipped} failed={failed} output={out_path} key={src}')


if __name__ == '__main__':
    main()
