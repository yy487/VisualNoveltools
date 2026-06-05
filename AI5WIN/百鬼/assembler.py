# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import importlib.util
import re
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
encode_cp = _op.encode_cp
load_json = _op.load_json
lzss_compress = _op.lzss_compress
lzss_decompress = _op.lzss_decompress
parse_placeholder_string = _op.parse_placeholder_string
sha256_bytes = _op.sha256_bytes
strip_comment = _op.strip_comment
unquote_asm_string = _op.unquote_asm_string

LABEL_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*$")
STR_LABEL_RE = re.compile(r"^str_([0-9A-Fa-f]{8})$")


def read_header(path: Path):
    meta = {}
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            if not line.startswith(";"):
                if line.strip():
                    break
                continue
            body = line[1:].strip()
            if ":" in body:
                k, v = body.split(":", 1)
                meta[k.strip().lower()] = v.strip()
    return meta


def build_patch_map(json_paths, encoding: str):
    """Return ctrl_offset -> replacement text bytes for cstring1 records."""
    patches = {}
    if not json_paths:
        return patches
    if isinstance(json_paths, (str, Path)):
        paths = [Path(json_paths)]
    else:
        paths = [Path(p) for p in json_paths]
    expanded = []
    for p in paths:
        if p.is_dir():
            expanded.extend(sorted(p.rglob("*.json")))
        else:
            expanded.append(p)
    for p in expanded:
        for e in load_json(p):
            enc = e.get("_encoding", encoding)
            # Message / choice / monologue body.
            if "_inst_offset" in e and isinstance(e.get("message"), str):
                try:
                    ctrl = int(e["_inst_offset"])
                except Exception:
                    ctrl = int(str(e["_inst_offset"]), 0)
                patches[ctrl] = parse_placeholder_string(e["message"], enc)
            # Optional speaker name replacement if name field was edited.
            if "_name_inst_offset" in e and isinstance(e.get("name"), str):
                try:
                    ctrl = int(e["_name_inst_offset"])
                except Exception:
                    ctrl = int(str(e["_name_inst_offset"]), 0)
                patches[ctrl] = parse_placeholder_string(e["name"], enc)
    return patches


def parse_byte_args(argstr: str):
    vals = []
    for part in argstr.split(','):
        part = part.strip()
        if not part:
            continue
        val = int(part, 0)
        if not 0 <= val <= 0xFF:
            raise ValueError(f".byte value out of range: {part}")
        vals.append(val)
    return bytes(vals)


def extract_quoted_arg(line: str):
    first = line.find('"')
    last = line.rfind('"')
    if first < 0 or last <= first:
        raise ValueError("missing quoted string")
    return line[first:last + 1]


def assemble_asm(path: Path, encoding: str, json_patch=None):
    patches = build_patch_map(json_patch, encoding)
    out = bytearray()
    current_label = None
    current_str_ctrl = None
    logical_offset = 0

    with open(path, "r", encoding="utf-8") as f:
        for lineno, raw_line in enumerate(f, 1):
            line = strip_comment(raw_line).strip()
            if not line:
                continue
            m = LABEL_RE.match(line)
            if m:
                current_label = m.group(1)
                sm = STR_LABEL_RE.match(current_label)
                current_str_ctrl = None
                if sm:
                    # str label stores content offset, control byte is one byte before.
                    current_str_ctrl = int(sm.group(1), 16) - 1
                continue
            if line.startswith(".byte") or line.startswith("    .byte"):
                argstr = line.split(".byte", 1)[1].strip()
                chunk = parse_byte_args(argstr)
                out.extend(chunk)
                logical_offset += len(chunk)
                continue
            if ".cstring1" in line:
                q = extract_quoted_arg(line)
                raw = unquote_asm_string(q, encoding)
                if current_str_ctrl is not None and current_str_ctrl in patches:
                    raw = patches[current_str_ctrl]
                out.append(0x01)
                out.extend(raw)
                out.append(0x00)
                logical_offset += len(raw) + 2
                continue
            if ".string" in line:
                # Plain string bytes, no implicit 0x01/0x00. Mostly for manual advanced use.
                q = extract_quoted_arg(line)
                raw = unquote_asm_string(q, encoding)
                out.extend(raw)
                logical_offset += len(raw)
                continue
            raise ValueError(f"{path}:{lineno}: unsupported statement: {line}")
    return bytes(out)


def verify_recompress(plain: bytes, comp: bytes) -> bool:
    return lzss_decompress(comp) == plain


def assemble_one(asm_path: Path, out_path: Path | None, encoding: str | None, json_patch, plain_output: bool):
    meta = read_header(asm_path)
    enc = encoding or meta.get("encoding") or DEFAULT_ENCODING
    container = meta.get("container", "lzss")
    plain = assemble_asm(asm_path, enc, json_patch)
    if plain_output or container == "plain":
        output = plain
        default_suffix = ".rebuild.plain"
    else:
        output = lzss_compress(plain)
        if not verify_recompress(plain, output):
            raise RuntimeError("internal error: recompressed MES does not decompress to rebuilt plain stream")
        default_suffix = ".rebuild.MES"
    if out_path is None:
        name = asm_path.name
        if name.endswith(".asm.txt"):
            name = name[:-8]
        out_path = asm_path.with_name(name + default_suffix)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(output)
    print(f"[asm] {asm_path} -> {out_path}")
    print(f"[asm] plain_size={len(plain)} out_size={len(output)} plain_sha256={sha256_bytes(plain)}")


def assemble_dir(asm_dir: Path, out_dir: Path, encoding: str | None, json_patch, plain_output: bool):
    files = sorted(asm_dir.rglob("*.asm.txt"))
    for ap in files:
        rel = ap.relative_to(asm_dir)
        # C_01.MES.asm.txt -> C_01.MES
        stem_name = rel.as_posix()
        if stem_name.endswith(".asm.txt"):
            stem_name = stem_name[:-8]
        target = out_dir / stem_name
        if plain_output:
            target = target.with_suffix(target.suffix + ".plain")
        assemble_one(ap, target, encoding, json_patch, plain_output)
    print(f"[batch-asm] files={len(files)} out={out_dir}")


def main(argv=None):
    ap = argparse.ArgumentParser(description="Baigui MES assembler / JSON importer")
    ap.add_argument("input", help="input .asm.txt file or directory")
    ap.add_argument("-o", "--output", help="output .MES file or directory")
    ap.add_argument("--encoding", default=None, help="override text encoding; defaults to asm header or cp932")
    ap.add_argument("--json", action="append", help="translated JSON file or directory; can be specified multiple times")
    ap.add_argument("--plain", action="store_true", help="write rebuilt plain stream instead of LZSS-compressed MES")
    args = ap.parse_args(argv)

    inp = Path(args.input)
    if inp.is_dir():
        out_dir = Path(args.output) if args.output else inp.with_name(inp.name + "_rebuilt")
        assemble_dir(inp, out_dir, args.encoding, args.json, args.plain)
    else:
        out_path = Path(args.output) if args.output else None
        assemble_one(inp, out_path, args.encoding, args.json, args.plain)


if __name__ == "__main__":
    main()
