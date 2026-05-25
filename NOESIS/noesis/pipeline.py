# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

from .iga_common import MANIFEST_NAME
from .iga_pack import pack_archive
from .iga_unpack import unpack_archive
from .script_extract import extract_scripts
from .script_inject import inject_scripts


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> None:
    ap = argparse.ArgumentParser(description="Noesis IGA/script localization pipeline")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p_unpack = sub.add_parser("unpack", help="unpack .iga to directory")
    p_unpack.add_argument("archive")
    p_unpack.add_argument("output_dir")

    p_pack = sub.add_parser("pack", help="pack directory back to .iga")
    p_pack.add_argument("input_dir")
    p_pack.add_argument("output_archive")
    p_pack.add_argument("--manifest", default=None)

    p_extract = sub.add_parser("extract", help="extract .s text to JSON")
    p_extract.add_argument("input")
    p_extract.add_argument("output")
    p_extract.add_argument("--keep-ruby", action="store_true")
    p_extract.add_argument("--export-names", action="store_true", default=False, help="also export separate editable name entries (not recommended for normal translation)")
    p_extract.add_argument("--no-export-names", dest="export_names", action="store_false", help="legacy compatibility; this is already the default")
    p_extract.add_argument("--no-name-dict", dest="write_name_dict", action="store_false", help="do not write _noesis_name_dict.json")
    p_extract.add_argument("--name-dict", default=None, help="path to write the name dictionary")

    p_inject = sub.add_parser("inject", help="inject JSON text into .s scripts")
    p_inject.add_argument("input")
    p_inject.add_argument("json")
    p_inject.add_argument("output")
    p_inject.add_argument("--output-encoding", "--encoding", default=None,
                          help="encoding used to encode message/name fields when injecting; use gbk for GBK hook mode")
    p_inject.add_argument("--name-dict", default=None,
                          help="optional original-name -> translated-name JSON object; default auto-loads _noesis_name_dict.json")

    p_round = sub.add_parser("roundtrip", help="unpack/repack and compare SHA-256")
    p_round.add_argument("archive")
    p_round.add_argument("work_dir")
    p_round.add_argument("rebuilt_archive")

    args = ap.parse_args()
    if args.cmd == "unpack":
        unpack_archive(Path(args.archive), Path(args.output_dir))
    elif args.cmd == "pack":
        inp = Path(args.input_dir)
        manifest = Path(args.manifest) if args.manifest else inp / MANIFEST_NAME
        pack_archive(inp, Path(args.output_archive), manifest)
    elif args.cmd == "extract":
        extract_scripts(
            Path(args.input),
            Path(args.output),
            strip_ruby=not args.keep_ruby,
            export_names=args.export_names,
            write_name_dict=args.write_name_dict,
            name_dict_path=Path(args.name_dict) if args.name_dict else None,
        )
    elif args.cmd == "inject":
        inject_scripts(
            Path(args.input),
            Path(args.json),
            Path(args.output),
            output_encoding=args.output_encoding,
            name_dict_path=Path(args.name_dict) if args.name_dict else None,
        )
    elif args.cmd == "roundtrip":
        src = Path(args.archive)
        work = Path(args.work_dir)
        rebuilt = Path(args.rebuilt_archive)
        unpack_archive(src, work)
        pack_archive(work, rebuilt, work / MANIFEST_NAME)
        same = src.read_bytes() == rebuilt.read_bytes()
        print(f"[noesis roundtrip] byte_exact={same}")
        print(f"[noesis roundtrip] original_sha256={sha256(src)}")
        print(f"[noesis roundtrip] rebuilt_sha256 ={sha256(rebuilt)}")
        if not same:
            raise SystemExit(1)


if __name__ == "__main__":
    main()
