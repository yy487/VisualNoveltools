# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

DEFAULT_ENCODING = "cp932"
RUNTIME_NAME_TEXT = "小十郎"
RUNTIME_NAME_CALL_RAW = bytes.fromhex("02 00 FF 00")
SUPPORTED_SUFFIXES = {".MES", ".mes"}

# 用户确认的外字规则：message 中可见映射；不能确认的直接删除。
GAIJI_BYTES_TO_TEXT: dict[bytes, str] = {
    b"\xEB\xA4": "！",
    b"\xEB\xA8": "？",
    b"\xEB\xAC": "っ",
    b"\xEB\xAA": "♪",
    b"\xEB\xAE": "♥",
    b"\xEB\xAB": "ッ",
    b"\xEB\xAD": "！！",
}
GAIJI_DROP: set[bytes] = {
    b"\xEB\xA1",
    b"\xEB\xA2",
    b"\xEB\xA3",
    b"\xEB\xA6",
    b"\xEB\xA7",
    b"\xEB\xA9",
}
# 注入时的反向映射。按最长匹配排序使用。
TEXT_TO_GAIJI_BYTES: dict[str, bytes] = {
    "！！": b"\xEB\xAD",
    "！": b"\xEB\xA4",
    "？": b"\xEB\xA8",
    "っ": b"\xEB\xAC",
    "♪": b"\xEB\xAA",
    "♥": b"\xEB\xAE",
    "ッ": b"\xEB\xAB",
}
TOKEN_RE = re.compile(r"\{\{([0-9A-Fa-f]{2})(?::([0-9A-Fa-f]{2}))?\}\}")


def read_u32_le(data: bytes | bytearray, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little")


def write_u32_le(buf: bytearray, off: int, value: int) -> None:
    if value < 0 or value > 0xFFFFFFFF:
        raise ValueError(f"u32 target out of range: 0x{value:X}")
    buf[off:off + 4] = value.to_bytes(4, "little")


@dataclass
class ExprInfo:
    start: int
    end: int
    raw: bytes
    simple_value: int | None = None


@dataclass
class TextSlot:
    slot_id: int
    kind: str                 # opcode_text / opcode_sys_text / arg_text
    inst_index: int
    inst_offset: int
    op: int
    raw_start: int
    raw_end: int              # exclude NUL
    raw: bytes
    arg_tag_offset: int | None = None

    @property
    def size(self) -> int:
        return self.raw_end - self.raw_start


@dataclass
class JumpRef:
    inst_index: int
    inst_offset: int
    operand_offset: int
    old_target: int
    kind: str


@dataclass
class Instruction:
    index: int
    offset: int
    end: int
    op: int
    slots: list[TextSlot] = field(default_factory=list)
    jumps: list[JumpRef] = field(default_factory=list)
    meta: dict[str, Any] = field(default_factory=dict)


@dataclass
class MesProgram:
    path: Path
    data: bytes
    instructions: list[Instruction]
    slots: list[TextSlot]
    jumps: list[JumpRef]

    @property
    def slot_by_id(self) -> dict[int, TextSlot]:
        return {s.slot_id: s for s in self.slots}


def parse_cstr(data: bytes, pos: int) -> tuple[bytes, int, int]:
    end = data.index(0, pos)
    return data[pos:end], pos, end + 1


def parse_expr(data: bytes, pos: int) -> ExprInfo:
    start = pos
    n = len(data)
    values: list[int] = []
    while pos < n:
        b = data[pos]
        pos += 1
        if b == 0xFF:
            raw = data[start:pos]
            simple = values[0] if len(values) == 1 else None
            return ExprInfo(start=start, end=pos, raw=raw, simple_value=simple)
        if b <= 0x7F:
            values.append(b)
        elif b in (0x80, 0xA0, 0xC0, 0xF5, 0xF6):
            if pos >= n:
                raise EOFError("expr operand truncated")
            pos += 1
            values.append(-1)  # variable/table reference; not a simple literal
        elif b in (0xF1, 0xF3):
            if pos + 2 > n:
                raise EOFError("expr u16 truncated")
            val = int.from_bytes(data[pos:pos + 2], "little")
            pos += 2
            values.append(val if b == 0xF1 else -1)
        elif b == 0xF2:
            if pos + 4 > n:
                raise EOFError("expr u32 truncated")
            val = int.from_bytes(data[pos:pos + 4], "little")
            pos += 4
            values.append(val)
        elif 0xE0 <= b <= 0xF0 or b == 0xF4:
            # operator / dynamic read. Expression evaluator is unnecessary for rebuild.
            values.append(-1)
        else:
            # Unknown token: keep parsing defensively as zero-operand token.
            values.append(-1)
    raise EOFError("unterminated expr")


def parse_arglist(data: bytes, pos: int, inst: Instruction, slots: list[TextSlot]) -> int:
    n = len(data)
    while pos < n:
        tag_off = pos
        tag = data[pos]
        pos += 1
        if tag == 0x00:
            return pos
        if tag == 0x01:
            raw, raw_start, pos2 = parse_cstr(data, pos)
            slot = TextSlot(
                slot_id=len(slots), kind="arg_text", inst_index=inst.index,
                inst_offset=inst.offset, op=inst.op, raw_start=raw_start,
                raw_end=pos2 - 1, raw=raw, arg_tag_offset=tag_off,
            )
            slots.append(slot)
            inst.slots.append(slot)
            pos = pos2
        elif tag == 0x02:
            info = parse_expr(data, pos)
            pos = info.end
        else:
            # 实际样本中这里基本不会走到。按表达式回退解析，避免直接错位。
            info = parse_expr(data, tag_off)
            pos = info.end
    raise EOFError("unterminated arglist")


def parse_mes(path: str | Path) -> MesProgram:
    path = Path(path)
    data = path.read_bytes()
    pos = 0
    idx = 0
    instructions: list[Instruction] = []
    slots: list[TextSlot] = []
    jumps: list[JumpRef] = []
    n = len(data)

    while pos < n:
        off = pos
        op = data[pos]
        pos += 1
        inst = Instruction(index=idx, offset=off, end=pos, op=op)
        idx += 1
        try:
            if op in (0x00, 0x13):
                pass
            elif op in (0x01, 0x02):
                raw, raw_start, pos = parse_cstr(data, pos)
                kind = "opcode_text" if op == 0x01 else "opcode_sys_text"
                slot = TextSlot(
                    slot_id=len(slots), kind=kind, inst_index=inst.index,
                    inst_offset=inst.offset, op=op, raw_start=raw_start,
                    raw_end=pos - 1, raw=raw,
                )
                slots.append(slot)
                inst.slots.append(slot)
            elif op == 0x03:
                pos += 2
                while True:
                    info = parse_expr(data, pos)
                    pos = info.end
                    cont = data[pos]
                    pos += 1
                    if cont == 0:
                        break
            elif op in (0x04, 0x14):
                pos += 1
                while True:
                    info = parse_expr(data, pos)
                    pos = info.end
                    cont = data[pos]
                    pos += 1
                    if cont == 0:
                        break
            elif op == 0x05:
                info = parse_expr(data, pos)
                pos = info.end
                while True:
                    info = parse_expr(data, pos)
                    pos = info.end
                    cont = data[pos]
                    pos += 1
                    if cont == 0:
                        break
            elif op in (0x06, 0x07, 0x08):
                info = parse_expr(data, pos)
                pos = info.end
                pos += 1
                while True:
                    info = parse_expr(data, pos)
                    pos = info.end
                    cont = data[pos]
                    pos += 1
                    if cont == 0:
                        break
            elif op == 0x09:
                info = parse_expr(data, pos)
                pos = info.end
                target_pos = pos
                target = read_u32_le(data, target_pos)
                pos += 4
                jr = JumpRef(inst.index, off, target_pos, target, "jump_if")
                jumps.append(jr)
                inst.jumps.append(jr)
            elif op == 0x0A:
                target_pos = pos
                target = read_u32_le(data, target_pos)
                pos += 4
                jr = JumpRef(inst.index, off, target_pos, target, "jump")
                jumps.append(jr)
                inst.jumps.append(jr)
            elif op == 0x0B:
                info = parse_expr(data, pos)
                pos = info.end
                inst.meta["func_id"] = info.simple_value
                inst.meta["func_expr_raw"] = info.raw.hex(" ").upper()
                pos = parse_arglist(data, pos, inst, slots)
            elif op in (0x0C, 0x0D, 0x0F, 0x10):
                pos = parse_arglist(data, pos, inst, slots)
            elif op == 0x0E:
                pos = parse_arglist(data, pos, inst, slots)
                target_pos = pos
                target = read_u32_le(data, target_pos)
                pos += 4
                jr = JumpRef(inst.index, off, target_pos, target, "menu_entry_target")
                jumps.append(jr)
                inst.jumps.append(jr)
            elif op == 0x11:
                if pos >= n:
                    raise EOFError("op11 missing byte")
                inst.meta["newline_arg"] = data[pos]
                pos += 1
            elif op == 0x12:
                info = parse_expr(data, pos)
                pos = info.end
                target_pos = pos
                target = read_u32_le(data, target_pos)
                pos += 4
                jr = JumpRef(inst.index, off, target_pos, target, "interrupt_target")
                jumps.append(jr)
                inst.jumps.append(jr)
            else:
                raise ValueError(f"unknown opcode 0x{op:02X}")
        except Exception as e:
            raise RuntimeError(f"{path.name}: parse failed at 0x{off:X}, op=0x{op:02X}, pos=0x{pos:X}: {e}") from e
        inst.end = pos
        instructions.append(inst)
    return MesProgram(path=path, data=data, instructions=instructions, slots=slots, jumps=jumps)


def decode_raw(raw: bytes, *, mode: str) -> str:
    """mode='scr' 保留未知/外字 token；mode='message' 对确认外字映射，未确认外字删除。"""
    assert mode in ("scr", "message")
    out: list[str] = []
    i = 0
    while i < len(raw):
        b = raw[i]
        if b <= 0x7F or 0xA1 <= b <= 0xDF:
            out.append(bytes([b]).decode(DEFAULT_ENCODING, errors="replace"))
            i += 1
            continue
        if 0x81 <= b <= 0x9F or 0xE0 <= b <= 0xFC:
            if i + 1 >= len(raw):
                if mode == "scr":
                    out.append(f"{{{{{b:02X}}}}}")
                i += 1
                continue
            pair = raw[i:i + 2]
            try:
                out.append(pair.decode(DEFAULT_ENCODING))
            except UnicodeDecodeError:
                if mode == "message":
                    if pair in GAIJI_BYTES_TO_TEXT:
                        out.append(GAIJI_BYTES_TO_TEXT[pair])
                    elif pair in GAIJI_DROP:
                        pass
                    else:
                        # 未知外字不擅自丢，避免破坏尚未分析到的字形。
                        out.append(f"{{{{{pair[0]:02X}:{pair[1]:02X}}}}}")
                else:
                    out.append(f"{{{{{pair[0]:02X}:{pair[1]:02X}}}}}")
            i += 2
            continue
        if mode == "scr":
            out.append(f"{{{{{b:02X}}}}}")
        i += 1
    return "".join(out)


def encode_message_text(text: str) -> bytes:
    """把 message 文本编码回 MES 文本字节。支持 {{EB:A4}} token 和用户确认的可见外字映射。"""
    # 旧版 JSON 里可能残留运行时名字占位符；当前按用户要求统一写死为小十郎。
    text = text.replace("{{CALL:0F}}", RUNTIME_NAME_TEXT)
    out = bytearray()
    i = 0
    keys = sorted(TEXT_TO_GAIJI_BYTES.keys(), key=len, reverse=True)
    while i < len(text):
        m = TOKEN_RE.match(text, i)
        if m:
            a = int(m.group(1), 16)
            b = m.group(2)
            if b is None:
                out.append(a)
            else:
                out.extend([a, int(b, 16)])
            i = m.end()
            continue
        matched = False
        for k in keys:
            if text.startswith(k, i):
                out.extend(TEXT_TO_GAIJI_BYTES[k])
                i += len(k)
                matched = True
                break
        if matched:
            continue
        ch = text[i]
        try:
            out.extend(ch.encode(DEFAULT_ENCODING))
        except UnicodeEncodeError as e:
            raise UnicodeEncodeError(
                e.encoding, e.object, e.start, e.end,
                f"character {ch!r} is not encodable in cp932 and has no gaiji mapping"
            )
        i += 1
    return bytes(out)


def json_dump(path: str | Path, entries: list[dict[str, Any]]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def json_load(path: str | Path) -> list[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be a list: {path}")
    return data


def is_text_file(path: Path) -> bool:
    return path.is_file() and path.suffix in SUPPORTED_SUFFIXES


def slot_scr(slot: TextSlot) -> str:
    return decode_raw(slot.raw, mode="scr")


def slot_message(slot: TextSlot) -> str:
    return decode_raw(slot.raw, mode="message")
