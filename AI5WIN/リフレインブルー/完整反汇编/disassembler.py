# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable
from bisect import bisect_right

# Load local opcode.py explicitly because Python itself imports the stdlib module named "opcode" very early.
import importlib.util as _importlib_util
import sys as _sys
_OPCODE_PATH = Path(__file__).with_name("opcode.py")
_spec = _importlib_util.spec_from_file_location("rp_mes_opcode", _OPCODE_PATH)
if _spec is None or _spec.loader is None:
    raise ImportError(f"cannot load local opcode.py from {_OPCODE_PATH}")
_opcode_mod = _importlib_util.module_from_spec(_spec)
_sys.modules[_spec.name] = _opcode_mod
_spec.loader.exec_module(_opcode_mod)
DEFAULT_ENCODING = _opcode_mod.DEFAULT_ENCODING
OPCODES = _opcode_mod.OPCODES
label_name = _opcode_mod.label_name

PLACEHOLDER_OPEN = "{{"
PLACEHOLDER_CLOSE = "}}"


@dataclass
class Unit:
    offset: int
    end: int
    kind: str
    text_bytes: bytes | None = None
    target: int | None = None
    imm: int | None = None

    @property
    def size(self) -> int:
        return self.end - self.offset


def read_cstring(data: bytes, start: int, max_len: int = 0x400) -> tuple[bytes, int] | None:
    if start < 0 or start >= len(data):
        return None
    end_limit = min(len(data), start + max_len)
    end = data.find(b"\x00", start, end_limit)
    if end < 0:
        return None
    return data[start:end], end + 1


def _is_cp932_lead(b: int) -> bool:
    return 0x81 <= b <= 0x9F or 0xE0 <= b <= 0xFC


def _is_printable_char(ch: str) -> bool:
    if not ch:
        return False
    o = ord(ch)
    if o < 0x20 or o == 0x7F:
        return False
    if ch in {'"', "\\", "{", "}"}:
        return False
    return True


def bytes_to_semantic_string(raw: bytes, encoding: str) -> str:
    """Represent bytes as readable text plus {{XX}} placeholders.

    No \\x-style escapes are emitted.  Quotes, backslashes and braces are also
    represented as placeholders so the assembler can parse the string without
    ambiguity.
    """
    out: list[str] = []
    i = 0
    while i < len(raw):
        b = raw[i]
        chunk: bytes | None = None
        if _is_cp932_lead(b) and i + 1 < len(raw):
            cand = raw[i:i + 2]
            try:
                s = cand.decode(encoding)
            except Exception:
                s = ""
            if s and len(s) == 1 and _is_printable_char(s):
                chunk = cand
                out.append(s)
                i += 2
                continue
        try:
            s1 = bytes([b]).decode(encoding)
        except Exception:
            s1 = ""
        if s1 and len(s1) == 1 and _is_printable_char(s1):
            out.append(s1)
        else:
            out.append("{{%02X}}" % b)
        i += 1
    return "".join(out)


def _valid_text_payload(raw: bytes, encoding: str) -> bool:
    if not raw or len(raw) > 0x400:
        return False
    # VM text/resource C-strings seen in this engine do not contain control
    # bytes or 0xFF inside the payload.  Rejecting them avoids swallowing
    # operand streams after a coincidental 0x01 byte.
    for b in raw:
        if b < 0x20 or b == 0x7F or b == 0xFF:
            return False
    try:
        raw.decode(encoding)
    except Exception:
        return False
    return True


def _is_valid_target(target: int, size: int, known_starts: set[int] | None = None) -> bool:
    if not (0 <= target <= size):
        return False
    if known_starts is None:
        return True
    return target == size or target in known_starts


def _scan_primary_units(data: bytes, encoding: str) -> dict[int, Unit]:
    """Scan text/ruby/new-line units first.

    Choice and jump patterns are added in later passes only when their target
    resolves to an already known unit boundary.  This avoids interpreting random
    operand bytes in the prologue as branch commands.
    """
    units: dict[int, Unit] = {}
    i = 0
    size = len(data)
    while i < size:
        if data.startswith(OPCODES["RUBY_TEXT"].bytecode, i):
            item = read_cstring(data, i + 4)
            if item is not None:
                raw, end = item
                if _valid_text_payload(raw, encoding):
                    units[i] = Unit(i, end, "RUBY_TEXT", text_bytes=raw)
                    i = end
                    continue

        if data[i] == 0x01:
            item = read_cstring(data, i + 1)
            if item is not None:
                raw, end = item
                if _valid_text_payload(raw, encoding):
                    units[i] = Unit(i, end, "TEXT", text_bytes=raw)
                    i = end
                    continue


        i += 1
    return units


def _scan_choice_units(data: bytes, known_starts: set[int]) -> dict[int, Unit]:
    units: dict[int, Unit] = {}
    size = len(data)

    # Choice branch: 0E 02 <idx> FF 00 <end:u32le>
    i = 0
    while True:
        i = data.find(b"\x0E\x02", i)
        if i < 0 or i + 9 > size:
            break
        if data[i + 3:i + 5] == b"\xFF\x00":
            target = int.from_bytes(data[i + 5:i + 9], "little")
            if 0 <= target <= size and target > i:
                units[i] = Unit(i, i + 9, "CHOICE_BRANCH", target=target, imm=data[i + 2])
                i += 9
                continue
        i += 1

    # Choice dispatch: 09 01 FF <exit:u32le>, with a branch record nearby.
    i = 0
    while True:
        i = data.find(b"\x09\x01\xFF", i)
        if i < 0 or i + 7 > size:
            break
        target = int.from_bytes(data[i + 3:i + 7], "little")
        near = data[i + 7:min(size, i + 0x100)]
        has_branch_near = b"\x0E\x02\x00\xFF\x00" in near or b"\x0E\x02\x01\xFF\x00" in near
        if 0 <= target <= size and target > i and has_branch_near:
            units[i] = Unit(i, i + 7, "CHOICE_DISPATCH", target=target)
            i += 7
            continue
        i += 1
    return units

def _build_intervals(units: dict[int, Unit]) -> tuple[list[int], list[tuple[int, int]]]:
    intervals = sorted((u.offset, u.end) for u in units.values() if u.end - u.offset > 1)
    starts = [a for a, _ in intervals]
    return starts, intervals


def _inside_intervals(off: int, starts: list[int], intervals: list[tuple[int, int]]) -> bool:
    idx = bisect_right(starts, off) - 1
    if idx < 0:
        return False
    a, b = intervals[idx]
    return a < off < b

def _scan_jump_units(data: bytes, known_starts: set[int]) -> dict[int, Unit]:
    units: dict[int, Unit] = {}
    size = len(data)
    i = 0
    while True:
        i = data.find(b"\x0A", i)
        if i < 0 or i + 5 > size:
            break
        target = int.from_bytes(data[i + 1:i + 5], "little")
        if _is_valid_target(target, size, known_starts):
            units[i] = Unit(i, i + 5, "JUMP", target=target)
            i += 5
            continue
        i += 1
    return units

def scan_units(data: bytes, encoding: str) -> dict[int, Unit]:
    units = _scan_primary_units(data, encoding)
    starts = set(units) | {len(data)}
    choices = _scan_choice_units(data, starts)
    starts_list, intervals = _build_intervals(units)
    for off, u in choices.items():
        if _inside_intervals(off, starts_list, intervals):
            continue
        units.setdefault(off, u)
    starts = set(units) | {len(data)}
    jumps = _scan_jump_units(data, starts)
    starts_list, intervals = _build_intervals(units)
    # Do not allow a jump detected inside a longer semantic unit to split it.
    for off, u in jumps.items():
        if _inside_intervals(off, starts_list, intervals):
            continue
        units.setdefault(off, u)
    return units


def collect_label_offsets(units: dict[int, Unit], size: int) -> set[int]:
    labels = {0}
    # Labels may point into raw .byte regions.  Those regions are split during
    # output so assembler offsets remain exact.  If a candidate target lands
    # inside a semantic unit payload, keep it numeric instead of making an
    # un-emittable label.
    semantic_interiors: list[tuple[int, int]] = [(u.offset + 1, u.end) for u in units.values() if u.size > 1]
    for u in units.values():
        if u.target is None or not (0 <= u.target <= size):
            continue
        inside = any(a <= u.target < b for a, b in semantic_interiors)
        if not inside:
            labels.add(u.target)
    return labels


def _format_byte_line(chunk: bytes) -> str:
    return "    .byte " + ", ".join(f"0x{b:02X}" for b in chunk)


def _target_token(target: int, labels: set[int]) -> str:
    return label_name(target) if target in labels else f"0x{target:X}"


def _format_unit(u: Unit, encoding: str, labels: set[int]) -> str:
    if u.kind in ("TEXT", "RUBY_TEXT"):
        s = bytes_to_semantic_string(u.text_bytes or b"", encoding)
        return f"    {u.kind} \"{s}\""
    if u.kind == "JUMP":
        return f"    JUMP {_target_token(int(u.target), labels)}"
    if u.kind == "CHOICE_DISPATCH":
        return f"    CHOICE_DISPATCH {_target_token(int(u.target), labels)}"
    if u.kind == "CHOICE_BRANCH":
        return f"    CHOICE_BRANCH {int(u.imm)}, {_target_token(int(u.target), labels)}"
    if u.kind == "NEW_LINE":
        return "    NEW_LINE"
    raise AssertionError(u.kind)

def disassemble(data: bytes, encoding: str, source_name: str = "") -> str:
    units = scan_units(data, encoding)
    labels = collect_label_offsets(units, len(data))
    unit_starts = sorted(units)
    lines: list[str] = []
    lines.append("; Refrain Blue MES semantic assembly")
    lines.append(f"; encoding: {encoding}")
    lines.append(f"; source: {source_name or '<memory>'}")
    lines.append(f"; source_size: 0x{len(data):X}")
    lines.append("; special bytes inside strings use {{XX}} placeholders; no raw hex dump comments are emitted")

    pos = 0
    while pos < len(data):
        if pos in labels:
            if lines and lines[-1] != "":
                lines.append("")
            lines.append(f"{label_name(pos)}:")

        u = units.get(pos)
        if u is not None:
            lines.append(_format_unit(u, encoding, labels))
            pos = u.end
            continue

        # Emit data until next unit or label.  Labels inside data split the .byte
        # stream so assembler label offsets remain exact.
        next_points = [x for x in unit_starts if x > pos]
        next_points += [x for x in labels if x > pos]
        run_end = min(next_points) if next_points else len(data)
        raw = data[pos:run_end]
        for j in range(0, len(raw), 16):
            lines.append(_format_byte_line(raw[j:j + 16]))
        pos = run_end

    if len(data) in labels:
        if lines and lines[-1] != "":
            lines.append("")
        lines.append(f"{label_name(len(data))}:")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Disassemble a Refrain Blue MES script into semantic asm.txt")
    ap.add_argument("input_script", help="input .MES script file")
    ap.add_argument("-o", "--output", help="output asm path; default <input_basename>.asm.txt")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING, help=f"script text encoding, default {DEFAULT_ENCODING}")
    args = ap.parse_args()

    inp = Path(args.input_script)
    out = Path(args.output) if args.output else inp.with_suffix(".asm.txt")
    data = inp.read_bytes()
    asm = disassemble(data, args.encoding, inp.name)
    out.write_text(asm, encoding="utf-8", newline="\n")
    print(f"[disasm] input={inp} size={len(data)} encoding={args.encoding} output={out}")


if __name__ == "__main__":
    main()
