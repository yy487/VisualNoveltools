# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
import tempfile
import shutil

# inject.py deliberately calls disassembler.py first and assembler.py second.
# This keeps injection anchored to the same IR used for extraction: original MES
# -> disassembly IR -> JSON patch -> full plain rebuild -> LZSS recompress.
import assembler
import disassembler


def _json_for_file(json_root: Path, input_root: Path, file_path: Path) -> Path | None:
    """Match the naming used by disassembler.process_dir."""
    if json_root.is_file():
        return json_root
    rel = file_path.relative_to(input_root).as_posix().replace("/", "__")
    jp = json_root / (rel + ".json")
    if jp.exists():
        return jp
    # Convenience fallbacks for hand-organized JSON directories.
    candidates = [
        json_root / (file_path.name + ".json"),
        json_root / (file_path.stem + ".json"),
    ]
    for c in candidates:
        if c.exists():
            return c
    return None


def inject_one(input_path: Path, json_path: Path, output_path: Path, encoding: str | None, plain: bool, keep_asm: Path | None) -> None:
    if keep_asm:
        asm_path = keep_asm
        asm_path.parent.mkdir(parents=True, exist_ok=True)
        disassembler.process_one(input_path, asm_path, None, encoding or disassembler.DEFAULT_ENCODING, plain)
    else:
        with tempfile.TemporaryDirectory(prefix="baigui_inject_") as td:
            asm_path = Path(td) / (input_path.name + ".asm.txt")
            disassembler.process_one(input_path, asm_path, None, encoding or disassembler.DEFAULT_ENCODING, plain)
            assembler.assemble_one(asm_path, output_path, encoding, [json_path], plain)
            return
    assembler.assemble_one(asm_path, output_path, encoding, [json_path], plain)


def inject_dir(input_dir: Path, json_dir: Path, output_dir: Path, encoding: str | None, plain: bool, keep_asm_dir: Path | None) -> None:
    files = [p for p in sorted(input_dir.rglob("*")) if p.is_file() and p.suffix.lower() == ".mes"]
    patched = 0
    skipped = 0
    for p in files:
        jp = _json_for_file(json_dir, input_dir, p)
        if jp is None:
            skipped += 1
            print(f"[inject][skip] no JSON for {p}")
            continue
        rel = p.relative_to(input_dir)
        out_path = output_dir / rel
        keep_asm = None
        if keep_asm_dir:
            keep_asm = keep_asm_dir / rel.with_suffix(rel.suffix + ".asm.txt")
        inject_one(p, jp, out_path, encoding, plain, keep_asm)
        patched += 1
    print(f"[batch-inject] files={len(files)} patched={patched} skipped={skipped} output={output_dir}")


def main(argv=None) -> None:
    ap = argparse.ArgumentParser(description="Baigui MES JSON injector backed by disassembler.py + assembler.py")
    ap.add_argument("input", help="original .MES file or directory")
    ap.add_argument("json", help="translated JSON file or directory")
    ap.add_argument("output", help="output .MES file or directory")
    ap.add_argument("--asm-out", help="optional path to keep generated asm IR file or directory")
    ap.add_argument("--encoding", default=None, help="override text encoding; defaults to asm header/cp932")
    ap.add_argument("--plain", action="store_true", help="write rebuilt plain stream instead of LZSS-compressed MES")
    args = ap.parse_args(argv)

    inp = Path(args.input)
    js = Path(args.json)
    out = Path(args.output)
    asm_out = Path(args.asm_out) if args.asm_out else None

    if inp.is_dir():
        inject_dir(inp, js, out, args.encoding, args.plain, asm_out)
    else:
        inject_one(inp, js, out, args.encoding, args.plain, asm_out)


if __name__ == "__main__":
    main()
