# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
import tempfile
import shutil

# extract.py is a thin workflow wrapper.  The actual parser and JSON exporter
# live in disassembler.py so extraction cannot drift away from the disassembly IR.
import disassembler


def _is_json_file(path: Path) -> bool:
    return path.suffix.lower() == ".json"


def extract_one(input_path: Path, json_output: Path, asm_output: Path | None, encoding: str, plain: bool) -> None:
    if asm_output is None:
        # Keep the required disassembly IR beside the JSON by default.  This is
        # intentionally not a separate parser: it is the disassembler output that
        # the injector later consumes for full rebuild.
        if _is_json_file(json_output):
            asm_output = json_output.with_suffix(".asm.txt")
        else:
            asm_output = json_output / (input_path.name + ".asm.txt")
    if _is_json_file(json_output):
        json_path = json_output
    else:
        json_path = json_output / (input_path.name + ".json")
    disassembler.process_one(input_path, asm_output, json_path, encoding, plain)


def extract_dir(input_dir: Path, json_dir: Path, asm_dir: Path | None, encoding: str, plain: bool) -> None:
    if asm_dir is None:
        asm_dir = json_dir.with_name(json_dir.name + "_asm")
    disassembler.process_dir(input_dir, asm_dir, json_dir, encoding, plain)


def main(argv=None) -> None:
    ap = argparse.ArgumentParser(description="Baigui MES JSON extractor backed by disassembler.py")
    ap.add_argument("input", help="input .MES file or directory")
    ap.add_argument("output", help="output JSON file or directory")
    ap.add_argument("--asm-out", help="optional asm IR output file or directory; default is output_json_asm")
    ap.add_argument("--encoding", default=disassembler.DEFAULT_ENCODING)
    ap.add_argument("--plain", action="store_true", help="input is already decompressed plain MES")
    args = ap.parse_args(argv)

    inp = Path(args.input)
    out = Path(args.output)
    asm_out = Path(args.asm_out) if args.asm_out else None

    if inp.is_dir():
        extract_dir(inp, out, asm_out, args.encoding, args.plain)
    else:
        extract_one(inp, out, asm_out, args.encoding, args.plain)


if __name__ == "__main__":
    main()
