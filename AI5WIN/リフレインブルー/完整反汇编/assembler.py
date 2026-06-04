# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path

# Load local rp_mes_opcode.py explicitly because Python itself imports the stdlib module named "opcode" very early.
import importlib.util as _importlib_util
import sys as _sys
_OPCODE_PATH = Path(__file__).with_name("rp_mes_opcode.py")
_spec = _importlib_util.spec_from_file_location("rp_mes_opcode", _OPCODE_PATH)
if _spec is None or _spec.loader is None:
    raise ImportError(f"cannot load local rp_mes_opcode.py from {_OPCODE_PATH}")
_opcode_mod = _importlib_util.module_from_spec(_spec)
_sys.modules[_spec.name] = _opcode_mod
_spec.loader.exec_module(_opcode_mod)
DEFAULT_ENCODING = _opcode_mod.DEFAULT_ENCODING
OPCODES = _opcode_mod.OPCODES


@dataclass
class Record:
    kind: str
    args: tuple
    line_no: int


PLACEHOLDER_RE = re.compile(r"\{\{([0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2})*)\}\}")


def strip_comment(line: str) -> str:
    in_str = False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == '"':
            in_str = not in_str
        elif ch == ";" and not in_str:
            return line[:i]
        i += 1
    return line


def parse_int_token(tok: str) -> int:
    tok = tok.strip()
    if not tok:
        raise ValueError("empty integer")
    return int(tok, 0)


def split_args(s: str) -> list[str]:
    args: list[str] = []
    cur: list[str] = []
    in_str = False
    i = 0
    while i < len(s):
        ch = s[i]
        if ch == '"':
            in_str = not in_str
            cur.append(ch)
        elif ch == "," and not in_str:
            args.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
        i += 1
    if cur or s.strip():
        args.append("".join(cur).strip())
    return args


def parse_quoted_string(token: str, line_no: int) -> str:
    token = token.strip()
    if len(token) < 2 or token[0] != '"' or token[-1] != '"':
        raise ValueError(f"line {line_no}: expected quoted string")
    return token[1:-1]


def semantic_string_to_bytes(s: str, encoding: str) -> bytes:
    out = bytearray()
    pos = 0
    for m in PLACEHOLDER_RE.finditer(s):
        if m.start() > pos:
            out += s[pos:m.start()].encode(encoding)
        for h in m.group(1).split(":"):
            out.append(int(h, 16))
        pos = m.end()
    if pos < len(s):
        # Reject placeholder-looking but invalid constructs early.
        tail = s[pos:]
        if "{{" in tail or "}}" in tail:
            raise ValueError(f"invalid placeholder near: {tail!r}")
        out += tail.encode(encoding)
    return bytes(out)


def detect_encoding_from_header(text: str) -> str | None:
    for line in text.splitlines()[:20]:
        m = re.match(r"\s*;\s*encoding\s*:\s*([^\s]+)", line, re.I)
        if m:
            return m.group(1)
    return None


def parse_asm(text: str) -> tuple[list[Record], dict[str, int]]:
    records: list[Record] = []
    label_record_index: dict[str, int] = {}
    for line_no, raw_line in enumerate(text.splitlines(), 1):
        line = strip_comment(raw_line).strip()
        if not line:
            continue
        if line.endswith(":"):
            name = line[:-1].strip()
            if not name:
                raise ValueError(f"line {line_no}: empty label")
            if name in label_record_index:
                raise ValueError(f"line {line_no}: duplicate label {name}")
            label_record_index[name] = len(records)
            records.append(Record("LABEL", (name,), line_no))
            continue

        if line.startswith(".byte"):
            arg_s = line[len(".byte"):].strip()
            if not arg_s:
                raise ValueError(f"line {line_no}: .byte needs values")
            vals = []
            for a in split_args(arg_s):
                v = parse_int_token(a)
                if not 0 <= v <= 0xFF:
                    raise ValueError(f"line {line_no}: byte out of range: {v}")
                vals.append(v)
            records.append(Record("BYTE", (bytes(vals),), line_no))
            continue

        if line.startswith(".string"):
            # Raw string pseudo-op, no implicit opcode or terminator.
            s = parse_quoted_string(line[len(".string"):].strip(), line_no)
            records.append(Record("STRING", (s,), line_no))
            continue

        parts = line.split(None, 1)
        mnem = parts[0].upper()
        arg_s = parts[1].strip() if len(parts) > 1 else ""

        if mnem in ("TEXT", "SYSTEM_TEXT", "RUBY_TEXT"):
            s = parse_quoted_string(arg_s, line_no)
            records.append(Record(mnem, (s,), line_no))
        elif mnem == "JUMP":
            if not arg_s:
                raise ValueError(f"line {line_no}: JUMP needs label")
            records.append(Record("JUMP", (arg_s,), line_no))
        elif mnem == "CHOICE_DISPATCH":
            if not arg_s:
                raise ValueError(f"line {line_no}: CHOICE_DISPATCH needs label")
            records.append(Record("CHOICE_DISPATCH", (arg_s,), line_no))
        elif mnem == "CHOICE_BRANCH":
            args = split_args(arg_s)
            if len(args) != 2:
                raise ValueError(f"line {line_no}: CHOICE_BRANCH needs index, label")
            idx = parse_int_token(args[0])
            if not 0 <= idx <= 0xFF:
                raise ValueError(f"line {line_no}: branch index out of range")
            records.append(Record("CHOICE_BRANCH", (idx, args[1]), line_no))
        elif mnem == "NEW_LINE":
            if arg_s:
                raise ValueError(f"line {line_no}: NEW_LINE takes no operand")
            records.append(Record("NEW_LINE", (), line_no))
        else:
            raise ValueError(f"line {line_no}: unknown mnemonic {mnem!r}")
    return records, label_record_index


def record_size(rec: Record, encoding: str) -> int:
    k = rec.kind
    if k == "LABEL":
        return 0
    if k == "BYTE":
        return len(rec.args[0])
    if k == "STRING":
        return len(semantic_string_to_bytes(rec.args[0], encoding))
    if k == "TEXT":
        return 1 + len(semantic_string_to_bytes(rec.args[0], encoding)) + 1
    if k == "SYSTEM_TEXT":
        return 1 + len(semantic_string_to_bytes(rec.args[0], encoding)) + 1
    if k == "RUBY_TEXT":
        return 4 + len(semantic_string_to_bytes(rec.args[0], encoding)) + 1
    if k == "JUMP":
        return 5
    if k == "CHOICE_DISPATCH":
        return 7
    if k == "CHOICE_BRANCH":
        return 9
    if k == "NEW_LINE":
        return 1
    raise AssertionError(k)


def resolve_labels(records: list[Record], encoding: str) -> dict[str, int]:
    labels: dict[str, int] = {}
    offset = 0
    for rec in records:
        if rec.kind == "LABEL":
            labels[rec.args[0]] = offset
        else:
            offset += record_size(rec, encoding)
    return labels


def target_value(tok: str, labels: dict[str, int], line_no: int) -> int:
    tok = tok.strip()
    if tok in labels:
        return labels[tok]
    try:
        return int(tok, 0)
    except Exception:
        raise ValueError(f"line {line_no}: unresolved label {tok!r}") from None


def encode_records(records: list[Record], encoding: str) -> bytes:
    labels = resolve_labels(records, encoding)
    out = bytearray()
    for rec in records:
        k = rec.kind
        if k == "LABEL":
            continue
        if k == "BYTE":
            out += rec.args[0]
        elif k == "STRING":
            out += semantic_string_to_bytes(rec.args[0], encoding)
        elif k == "TEXT":
            out += OPCODES["TEXT"].bytecode + semantic_string_to_bytes(rec.args[0], encoding) + b"\x00"
        elif k == "SYSTEM_TEXT":
            out += OPCODES["SYSTEM_TEXT"].bytecode + semantic_string_to_bytes(rec.args[0], encoding) + b"\x00"
        elif k == "RUBY_TEXT":
            out += OPCODES["RUBY_TEXT"].bytecode + semantic_string_to_bytes(rec.args[0], encoding) + b"\x00"
        elif k == "JUMP":
            t = target_value(rec.args[0], labels, rec.line_no)
            if not 0 <= t <= 0xFFFFFFFF:
                raise ValueError(f"line {rec.line_no}: JUMP target out of u32 range")
            out += OPCODES["JUMP"].bytecode + t.to_bytes(4, "little")
        elif k == "CHOICE_DISPATCH":
            t = target_value(rec.args[0], labels, rec.line_no)
            if not 0 <= t <= 0xFFFFFFFF:
                raise ValueError(f"line {rec.line_no}: CHOICE_DISPATCH target out of u32 range")
            out += OPCODES["CHOICE_DISPATCH"].bytecode + t.to_bytes(4, "little")
        elif k == "CHOICE_BRANCH":
            idx, label = rec.args
            t = target_value(label, labels, rec.line_no)
            if not 0 <= t <= 0xFFFFFFFF:
                raise ValueError(f"line {rec.line_no}: CHOICE_BRANCH target out of u32 range")
            out += b"\x0E\x02" + bytes([idx]) + b"\xFF\x00" + t.to_bytes(4, "little")
        elif k == "NEW_LINE":
            out += b"\x13"
        else:
            raise AssertionError(k)
    return bytes(out)


def assemble(text: str, encoding: str | None = None) -> bytes:
    if encoding is None:
        encoding = detect_encoding_from_header(text) or DEFAULT_ENCODING
    records, _ = parse_asm(text)
    return encode_records(records, encoding)


def main() -> None:
    ap = argparse.ArgumentParser(description="Assemble Refrain Blue MES semantic asm.txt back to binary")
    ap.add_argument("input_asm", help="input asm.txt")
    ap.add_argument("-o", "--output", help="output binary path; default <asm_basename>.rebuild")
    ap.add_argument("--encoding", help="text encoding; default is header encoding or cp932")
    args = ap.parse_args()

    inp = Path(args.input_asm)
    text = inp.read_text(encoding="utf-8")
    enc = args.encoding or detect_encoding_from_header(text) or DEFAULT_ENCODING
    data = assemble(text, enc)
    out = Path(args.output) if args.output else inp.with_suffix(".rebuild")
    out.write_bytes(data)
    print(f"[asm] input={inp} size={len(data)} encoding={enc} output={out}")


if __name__ == "__main__":
    main()
