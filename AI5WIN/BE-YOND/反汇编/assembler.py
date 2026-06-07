# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
import shlex

import importlib.util

_opcode_path = Path(__file__).with_name("b_mes_opcode.py")
_spec = importlib.util.spec_from_file_location("b_mes_opcode", _opcode_path)
opdef = importlib.util.module_from_spec(_spec)
assert _spec and _spec.loader
_spec.loader.exec_module(opdef)


def parse_placeholder(text: str) -> bytes:
    text = text.strip()
    if not (text.startswith("{{") and text.endswith("}}")):
        raise ValueError(f"bad placeholder: {text}")
    inner = text[2:-2].strip()
    if not inner:
        return b""
    return bytes(int(part, 16) for part in inner.split(":"))


def decode_asm_string(s: str, encoding: str) -> bytes:
    out = bytearray()
    i = 0
    while i < len(s):
        if s.startswith("{{", i):
            j = s.find("}}", i + 2)
            if j < 0:
                raise ValueError("unterminated placeholder in string")
            out += parse_placeholder(s[i:j + 2])
            i = j + 2
        else:
            out += s[i].encode(encoding)
            i += 1
    return bytes(out)


def strip_comment(line: str) -> str:
    in_quote = False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == '"':
            in_quote = not in_quote
        elif ch == ';' and not in_quote:
            return line[:i]
        i += 1
    return line


def logical_lines(text: str) -> list[str]:
    out = []
    for raw in text.splitlines():
        line = strip_comment(raw).strip()
        if not line:
            continue
        out.append(line)
    return out


def instruction_size(line: str, encoding: str) -> int:
    if line.endswith(":"):
        return 0
    if line.startswith(".byte"):
        return 1
    parts = shlex.split(line)
    if not parts:
        return 0
    m = parts[0]
    if m in ("TEXT", "SYSTEM_TEXT"):
        return 1 + len(decode_asm_string(parts[1], encoding)) + 1
    code = opdef.MNEMONIC_TO_OPCODE.get(m)
    if code is None:
        raise ValueError(f"unknown mnemonic: {m}")
    fmt = opdef.OPCODES[code]["format"]
    if fmt == "none":
        return 1
    if fmt == "u8":
        return 2
    if fmt == "target:u32le":
        return 5
    if fmt in ("expr target:u32le", "arglist_00 target:u32le"):
        return 1 + len(parse_placeholder(parts[1])) + 4
    if len(parts) == 1:
        return 1
    return 1 + len(parse_placeholder(parts[1]))


def first_pass(lines: list[str], encoding: str) -> dict[str, int]:
    labels: dict[str, int] = {}
    pc = 0
    for line in lines:
        if line.endswith(":"):
            labels[line[:-1]] = pc
        else:
            pc += instruction_size(line, encoding)
    return labels


def assemble(text: str, encoding: str) -> bytes:
    lines = logical_lines(text)
    labels = first_pass(lines, encoding)
    out = bytearray()
    for line in lines:
        if line.endswith(":"):
            continue
        if line.startswith(".byte"):
            val = line.split(None, 1)[1].strip()
            out.append(int(val, 0) & 0xFF)
            continue
        parts = shlex.split(line)
        if not parts:
            continue
        m = parts[0]
        if m in ("TEXT", "SYSTEM_TEXT"):
            out.append(opdef.MNEMONIC_TO_OPCODE[m])
            out += decode_asm_string(parts[1], encoding)
            out.append(0)
            continue
        code = opdef.MNEMONIC_TO_OPCODE.get(m)
        if code is None:
            raise ValueError(f"unknown mnemonic: {m}")
        fmt = opdef.OPCODES[code]["format"]
        out.append(code)
        if fmt == "none":
            continue
        if fmt == "u8":
            if len(parts) >= 2 and parts[1].startswith("{{"):
                out += parse_placeholder(parts[1])
            else:
                out.append(int(parts[1], 0) & 0xFF)
            continue
        if fmt == "target:u32le":
            target = labels.get(parts[1], opdef.parse_label_name(parts[1]))
            if target is None:
                raise ValueError(f"unknown label: {parts[1]}")
            out += int(target).to_bytes(4, "little")
            continue
        if fmt in ("expr target:u32le", "arglist_00 target:u32le"):
            out += parse_placeholder(parts[1])
            target = labels.get(parts[2], opdef.parse_label_name(parts[2]))
            if target is None:
                raise ValueError(f"unknown label: {parts[2]}")
            out += int(target).to_bytes(4, "little")
            continue
        if len(parts) >= 2:
            out += parse_placeholder(parts[1])
    return bytes(out)


def default_output_path(inp: Path) -> Path:
    name = inp.name
    if name.endswith(".asm.txt"):
        return inp.with_name(name[:-8] + ".rebuild")
    return inp.with_suffix(inp.suffix + ".rebuild")


def main() -> None:
    ap = argparse.ArgumentParser(description="Assemble b_mes asm.txt back to MES bytecode")
    ap.add_argument("input")
    ap.add_argument("-o", "--output")
    ap.add_argument("--encoding", default=opdef.DEFAULT_ENCODING)
    args = ap.parse_args()
    inp = Path(args.input)
    out = Path(args.output) if args.output else default_output_path(inp)
    data = assemble(inp.read_text(encoding="utf-8"), args.encoding)
    out.write_bytes(data)
    print(f"[asm] {inp} -> {out} size={len(data)} encoding={args.encoding}")

if __name__ == "__main__":
    main()
