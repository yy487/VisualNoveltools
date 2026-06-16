# -*- coding: utf-8 -*-
from __future__ import annotations
import argparse
from pathlib import Path
from common import *


def parse_args():
    ap = argparse.ArgumentParser(description='Extract RealLive Seen.txt messages to project JSON from decoded VM stream')
    ap.add_argument('seen_txt')
    ap.add_argument('arg2', help='json_out, or legacy ida_export_dir when arg3 is also supplied')
    ap.add_argument('arg3', nargs='?', help='legacy json_out when ida_export_dir is supplied')
    ap.add_argument('--ida-export', help='optional IDA export dir; normally not needed because crypt_template.py is bundled')
    ap.add_argument('--seen', type=int, nargs='*')
    ap.add_argument('--clean', action='store_true', help='delete existing Seen*.json in json_out before extraction; use this when rerunning after parser fixes')
    args = ap.parse_args()
    if args.arg3 is None:
        args.ida_export_dir = args.ida_export
        args.json_out = args.arg2
    else:
        args.ida_export_dir = args.ida_export or args.arg2
        args.json_out = args.arg3
    return args


def main() -> None:
    args = parse_args()
    seen_data = Path(args.seen_txt).read_bytes()
    key = load_xor_key_from_export(args.ida_export_dir)
    out = Path(args.json_out); out.mkdir(parents=True, exist_ok=True)
    if args.clean:
        removed = 0
        for old in out.glob('Seen*.json'):
            old.unlink()
            removed += 1
        if removed:
            print(f'[extract] clean removed={removed} old json files')
    else:
        stale = list(out.glob('Seen*.json'))
        if stale:
            print(f'[extract][warn] json_out already contains {len(stale)} Seen*.json files; stale files from older extractor may remain. Use --clean when re-extracting.')
    want = set(args.seen or [])
    total = 0; files = 0
    for entry in iter_seen_entries(seen_data):
        if want and entry.seen_no not in want:
            continue
        dec = decode_seen_chunk(entry, seen_data, key)
        entries = iter_text_entries(dec)
        if entries:
            save_json(out / f'Seen{entry.seen_no:04d}.json', entries)
            total += len(entries); files += 1
    src = 'static crypt_template.py' if not args.ida_export_dir else str(args.ida_export_dir)
    print(f'[extract] files={files} entries={total} out={out} key={src}')

if __name__ == '__main__':
    main()
