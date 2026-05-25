# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
from pathlib import Path

from .script_common import iter_script_files, json_name_for_script, parse_script_records, save_json

DEFAULT_NAME_DICT_FILE = "_noesis_name_dict.json"


def extract_one_file(path: Path, file_name: str, *, strip_ruby: bool, export_names: bool) -> list[dict]:
    data = path.read_bytes()
    records = parse_script_records(data, export_names=export_names)
    entries = []
    for index, rec in enumerate(records):
        entries.append(rec.to_entry(file_name, index, message_uses_clean=strip_ruby))
    return entries


def collect_name_dict_from_entries(entries: list[dict]) -> dict[str, str]:
    """Build an editable original-name -> translated-name dictionary.

    The normal JSON keeps `name` only as dialogue context.  This auxiliary
    dictionary lets translators edit each speaker name once without exporting a
    separate `_type=name` row for every line.
    """
    names: dict[str, str] = {}
    for entry in entries:
        name = entry.get("name")
        if isinstance(name, str) and name:
            names.setdefault(name, name)
    return names


def save_name_dict(path: Path, name_dict: dict[str, str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as f:
        json.dump(dict(sorted(name_dict.items())), f, ensure_ascii=False, indent=2)


def extract_scripts(
    input_path: Path,
    output_path: Path,
    *,
    strip_ruby: bool = True,
    export_names: bool = False,
    write_name_dict: bool = True,
    name_dict_path: Path | None = None,
) -> None:
    all_names: dict[str, str] = {}

    if input_path.is_file():
        entries = extract_one_file(input_path, input_path.name, strip_ruby=strip_ruby, export_names=export_names)
        save_json(output_path, entries)
        if write_name_dict:
            all_names.update(collect_name_dict_from_entries(entries))
            nd_path = name_dict_path or output_path.with_name(DEFAULT_NAME_DICT_FILE)
            save_name_dict(nd_path, all_names)
            print(f"[noesis extract] name_dict={nd_path} names={len(all_names)}")
        print(f"[noesis extract] files=1 entries={len(entries)} output={output_path}")
        return

    total_files = 0
    total_entries = 0
    output_path.mkdir(parents=True, exist_ok=True)
    for file in iter_script_files(input_path):
        rel = file.relative_to(input_path).as_posix()
        entries = extract_one_file(file, rel, strip_ruby=strip_ruby, export_names=export_names)
        if not entries:
            continue
        save_json(output_path / json_name_for_script(rel), entries)
        all_names.update(collect_name_dict_from_entries(entries))
        total_files += 1
        total_entries += len(entries)

    if write_name_dict:
        nd_path = name_dict_path or (output_path / DEFAULT_NAME_DICT_FILE)
        save_name_dict(nd_path, all_names)
        print(f"[noesis extract] name_dict={nd_path} names={len(all_names)}")
    print(f"[noesis extract] files={total_files} entries={total_entries} output={output_path}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract Noesis .s script text to JSON")
    ap.add_argument("input", help="input .s file or directory")
    ap.add_argument("output", help="output JSON file or directory")
    ap.add_argument("--keep-ruby", action="store_true", help="keep ruby markup in initial message field")
    ap.add_argument("--export-names", action="store_true", default=False,
                    help="also export speaker marker strings like ＃あやか as separate _type=name entries (not recommended for normal translation)")
    ap.add_argument("--no-export-names", dest="export_names", action="store_false",
                    help="legacy compatibility; this is already the default")
    ap.add_argument("--no-name-dict", dest="write_name_dict", action="store_false",
                    help="do not write _noesis_name_dict.json")
    ap.add_argument("--name-dict", default=None,
                    help="path to write the name dictionary; default is _noesis_name_dict.json beside/in the JSON output")
    args = ap.parse_args()
    extract_scripts(
        Path(args.input),
        Path(args.output),
        strip_ruby=not args.keep_ruby,
        export_names=args.export_names,
        write_name_dict=args.write_name_dict,
        name_dict_path=Path(args.name_dict) if args.name_dict else None,
    )


if __name__ == "__main__":
    main()
