# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from common import extract_text_entries, iter_source, safe_name, save_json


def command_extract(args: argparse.Namespace) -> None:
    out = Path(args.output)
    out.mkdir(parents=True, exist_ok=True)
    all_entries: list[dict[str, Any]] = []
    manifest: list[dict[str, Any]] = []
    files = 0
    for name, data in iter_source(Path(args.input)):
        files += 1
        entries, meta = extract_text_entries(name, data, include_system=args.include_system)
        if entries:
            save_json(out / f"{safe_name(name)}.json", entries)
            all_entries.extend(entries)
        manifest.append(meta)
        print(f"[extract] {name}: entries={len(entries)}")
    save_json(out / "manifest.json", manifest)
    save_json(out / "all_text.json", all_entries)
    print(f"[extract] files={files} entries={len(all_entries)} output={out}")


def command_verify(args: argparse.Namespace) -> None:
    ok = 0
    fail = 0
    total_entries = 0
    for name, data in iter_source(Path(args.input)):
        try:
            entries, _meta = extract_text_entries(name, data, include_system=args.include_system)
            ok += 1
            total_entries += len(entries)
        except Exception as ex:
            fail += 1
            print(f"[verify][fail] {name}: {ex}")
    print(f"[verify] files_ok={ok} files_failed={fail} entries={total_entries}")


def build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Lucrezia Scw5.x structural text extractor")
    p.add_argument("input", help="input .scw file, directory, or zip")
    p.add_argument("output", nargs="?", help="output JSON directory; omit when --verify is used")
    p.add_argument("--include-system", action="store_true", help="also export opcode 0x01C9 system/UI title strings")
    p.add_argument("--verify", action="store_true", help="only verify scripts can be decoded/extracted")
    return p


def main() -> None:
    args = build_argparser().parse_args()
    if args.verify:
        command_verify(args)
    else:
        if not args.output:
            raise SystemExit("extract mode requires output directory")
        command_extract(args)


if __name__ == "__main__":
    main()
