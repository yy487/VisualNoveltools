# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from typing import Iterable

import importlib.util

_opcode_path = Path(__file__).with_name("b_mes_opcode.py")
_spec = importlib.util.spec_from_file_location("b_mes_opcode", _opcode_path)
opdef = importlib.util.module_from_spec(_spec)
assert _spec and _spec.loader
_spec.loader.exec_module(opdef)

PLACEHOLDER_SAFE = set(range(0x20, 0x7F)) - {0x22, 0x5C, 0x7B, 0x7D}


def u32le(data: bytes, pos: int) -> int:
    if pos + 4 > len(data):
        raise ValueError(f"u32 out of range at 0x{pos:08X}")
    return int.from_bytes(data[pos:pos + 4], "little")


def bytes_placeholder(bs: bytes) -> str:
    return "{{" + ":".join(f"{b:02X}" for b in bs) + "}}"


def encode_asm_string(raw: bytes, encoding: str) -> str:
    out: list[str] = []
    i = 0
    while i < len(raw):
        b = raw[i]
        if b == 0:
            out.append("{{00}}")
            i += 1
            continue
        if b in (0x09,):
            out.append("{{09}}")
            i += 1
            continue
        if b in PLACEHOLDER_SAFE:
            out.append(chr(b))
            i += 1
            continue
        if opdef.is_sjis_lead(b) and i + 1 < len(raw):
            pair = raw[i:i + 2]
            try:
                ch = pair.decode(encoding)
                if ch not in {'"', '\\', '{', '}'} and ch.isprintable() and ch.encode(encoding, errors='strict') == pair:
                    out.append(ch)
                else:
                    out.append(bytes_placeholder(pair))
            except UnicodeDecodeError:
                out.append(bytes_placeholder(pair))
            i += 2
            continue
        try:
            ch = bytes([b]).decode(encoding)
            if ch not in {'"', '\\', '{', '}'} and ch.isprintable() and ch.encode(encoding, errors='strict') == bytes([b]):
                out.append(ch)
            else:
                out.append(bytes_placeholder(bytes([b])))
        except UnicodeDecodeError:
            out.append(bytes_placeholder(bytes([b])))
        i += 1
    return "".join(out)


def parse_cstring(data: bytes, pos: int) -> tuple[bytes, int]:
    end = data.find(b"\x00", pos)
    if end < 0:
        raise ValueError(f"unterminated string at 0x{pos:08X}")
    return data[pos:end], end + 1


def parse_expr(data: bytes, pos: int) -> int:
    n = len(data)
    while pos < n:
        t = data[pos]
        pos += 1
        if t == 0xFF:
            return pos
        extra = opdef.EXPR_TOKEN_PAYLOAD.get(t, 0)
        if pos + extra > n:
            raise ValueError("expression overruns file")
        pos += extra
    raise ValueError("unterminated expression")


def parse_expr_list_00(data: bytes, pos: int) -> int:
    n = len(data)
    # AI5WIN list setters always read at least one sub_40AFC0 expression;
    # the list terminator is checked only after an expression has returned.
    pos = parse_expr(data, pos)
    while True:
        if pos >= n:
            raise ValueError("unterminated expr_list")
        if data[pos] == 0:
            return pos + 1
        pos = parse_expr(data, pos)


def parse_arglist_00(data: bytes, pos: int) -> int:
    n = len(data)
    while True:
        if pos >= n:
            raise ValueError("unterminated arglist")
        kind = data[pos]
        pos += 1
        if kind == 0:
            return pos
        if kind == 1:
            _, pos = parse_cstring(data, pos)
        elif kind == 2:
            pos = parse_expr(data, pos)
        else:
            # This should not normally happen according to sub_403690.
            # Stop parsing this command and let caller fall back to raw bytes.
            raise ValueError(f"bad arg kind 0x{kind:02X}")


def safe_parse_payload(data: bytes, start: int) -> tuple[str, bytes, int, int | None]:
    code = data[start]
    spec = opdef.OPCODES.get(code)
    if spec is None:
        return ".byte", data[start:start + 1], start + 1, None
    fmt = spec["format"]
    pos = start + 1
    target = None
    try:
        if fmt == "none":
            pass
        elif fmt == "cstring":
            _, pos = parse_cstring(data, pos)
        elif fmt == "u8":
            pos += 1
        elif fmt == "u16 expr_list_00":
            pos += 2
            pos = parse_expr_list_00(data, pos)
        elif fmt == "u8 expr_list_00":
            pos += 1
            pos = parse_expr_list_00(data, pos)
        elif fmt == "expr expr_list_00":
            pos = parse_expr(data, pos)
            pos = parse_expr_list_00(data, pos)
        elif fmt == "expr u8 expr_list_00":
            pos = parse_expr(data, pos)
            pos += 1
            pos = parse_expr_list_00(data, pos)
        elif fmt == "expr target:u32le":
            pos = parse_expr(data, pos)
            target = u32le(data, pos)
            pos += 4
        elif fmt == "target:u32le":
            target = u32le(data, pos)
            pos += 4
        elif fmt == "expr arglist_00":
            pos = parse_expr(data, pos)
            pos = parse_arglist_00(data, pos)
        elif fmt == "arglist_00":
            pos = parse_arglist_00(data, pos)
        elif fmt == "arglist_00 target:u32le":
            pos = parse_arglist_00(data, pos)
            target = u32le(data, pos)
            pos += 4
        else:
            raise ValueError(f"unsupported format {fmt}")
        if pos > len(data):
            raise ValueError("instruction overruns file")
        return spec["mnemonic"], data[start + 1:pos], pos, target
    except Exception:
        return ".byte", data[start:start + 1], start + 1, None


def collect_targets(data: bytes) -> set[int]:
    targets: set[int] = {0}
    pos = 0
    while pos < len(data):
        mnemonic, payload, nxt, target = safe_parse_payload(data, pos)
        if target is not None and 0 <= target < len(data):
            targets.add(target)
        pos = max(nxt, pos + 1)
    return targets


def disassemble(data: bytes, encoding: str) -> str:
    labels = collect_targets(data)
    lines = [
        "; b_mes semantic disassembly",
        f"; encoding: {encoding}",
        "; source is expected to be already decompressed MES bytecode",
        "",
    ]
    pos = 0
    first_label = True
    while pos < len(data):
        if pos in labels:
            if not first_label:
                lines.append("")
            lines.append(f"{opdef.label_name(pos)}:")
            first_label = False
        mnemonic, payload, nxt, target = safe_parse_payload(data, pos)
        if mnemonic == ".byte":
            lines.append(f"    .byte 0x{payload[0]:02X}")
            pos = nxt
            continue
        code = data[pos]
        fmt = opdef.OPCODES[code]["format"]
        if fmt == "cstring":
            raw = payload[:-1] if payload.endswith(b"\x00") else payload
            s = encode_asm_string(raw, encoding)
            lines.append(f"    {mnemonic} \"{s}\"")
        elif fmt in ("target:u32le",):
            label = opdef.label_name(target if target is not None else 0)
            lines.append(f"    {mnemonic} {label}")
        elif fmt in ("expr target:u32le", "arglist_00 target:u32le"):
            raw = payload[:-4]
            label = opdef.label_name(target if target is not None else 0)
            lines.append(f"    {mnemonic} {bytes_placeholder(raw)} {label}")
        else:
            if payload:
                lines.append(f"    {mnemonic} {bytes_placeholder(payload)}")
            else:
                lines.append(f"    {mnemonic}")
        pos = nxt
    return "\n".join(lines) + "\n"


def default_output_path(inp: Path) -> Path:
    return inp.with_suffix(inp.suffix + ".asm.txt")


def main() -> None:
    ap = argparse.ArgumentParser(description="Disassemble already-decompressed AI5WIN MES bytecode")
    ap.add_argument("input")
    ap.add_argument("-o", "--output")
    ap.add_argument("--encoding", default=opdef.DEFAULT_ENCODING)
    args = ap.parse_args()
    inp = Path(args.input)
    out = Path(args.output) if args.output else default_output_path(inp)
    data = inp.read_bytes()
    out.write_text(disassemble(data, args.encoding), encoding="utf-8", newline="\n")
    print(f"[disasm] {inp} -> {out} size={len(data)} encoding={args.encoding}")

if __name__ == "__main__":
    main()
