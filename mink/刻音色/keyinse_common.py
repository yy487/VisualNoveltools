# -*- coding: utf-8 -*-
"""Common parser/patcher for 刻音色 *.s scripts.

The script VM stores instructions in a compact binary stream. Most fixed
instructions use byte1 as instruction length; several string-bearing
instructions use byte1 as string start offset and another header byte as
string length. This module implements only the parts needed for text/choice
localisation while preserving all opaque instructions byte-for-byte.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable
import bisect
import json
import re

DEFAULT_ENCODING = "cp932"
TOOL_VERSION = "2026-05-24-page-mark-auto-v3"
TEXT_END = b"\x81\x94\x00"  # CP932 '＃' + NUL; text display treats 0x8194 as line/end control.
PAGE_MARK = "＃"
PAD_BYTE = 0xCD

# From keyinse.exe.c dispatch logic:
# case 1/4/7/9/0b/0c/0d/22/29/2e copy byte3 bytes from p+byte1.
VARLEN_B3 = {0x01, 0x04, 0x07, 0x09, 0x0B, 0x0C, 0x0D, 0x22, 0x29, 0x2E}
# case 1b stores choice text: byte2 bytes from p+byte1, target is dword at p+4.
CHOICE_OP = 0x1B
TEXT_OP = 0x04

# Absolute script offsets observed in the interpreter:
# op05: ip = script_base + dword(p+4)
# op1b: choice target = dword(p+4)
# op1c..21: conditional jump target = dword(p+12)
# op33: if flag then ip = script_base + dword(p+4)
TARGET_FIELD_OFFSETS: dict[int, tuple[int, ...]] = {
    0x05: (4,),
    0x1B: (4,),
    0x33: (4,),
    0x1C: (12,),
    0x1D: (12,),
    0x1E: (12,),
    0x1F: (12,),
    0x20: (12,),
    0x21: (12,),
}

VOICE_RE = re.compile(r"^[A-Za-z]{2}\d{5}$")


@dataclass(slots=True)
class Instruction:
    offset: int
    op: int
    raw: bytes

    @property
    def length(self) -> int:
        return len(self.raw)

    @property
    def b1(self) -> int:
        return self.raw[1] if len(self.raw) > 1 else 0

    @property
    def b2(self) -> int:
        return self.raw[2] if len(self.raw) > 2 else 0

    @property
    def b3(self) -> int:
        return self.raw[3] if len(self.raw) > 3 else 0


def read_json(path: str | Path) -> list[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be a list: {path}")
    return data


def write_json(path: str | Path, entries: list[dict[str, Any]]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def decode_bytes(data: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    return data.decode(encoding)


def encode_text(text: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    return text.encode(encoding)


def strip_page_marks(text: str) -> str:
    """Hide in-text page/wait marks from translator-facing message."""
    return text.replace(PAGE_MARK, "")


def page_mark_byte_offsets(scr_msg: str, encoding: str = DEFAULT_ENCODING) -> list[int]:
    """Return byte offsets, in scr_msg with PAGE_MARK removed, where PAGE_MARK occurred."""
    offsets: list[int] = []
    acc = 0
    for ch in scr_msg:
        if ch == PAGE_MARK:
            offsets.append(acc)
        else:
            acc += len(ch.encode(encoding))
    return offsets


def apply_page_marks_from_scr_msg(scr_msg: str, message: str,
                                  encoding: str = DEFAULT_ENCODING) -> str:
    """Insert PAGE_MARK into message at byte positions inherited from scr_msg.

    Translators edit message without PAGE_MARK.  On import we reconstruct the
    original page/wait marks by using the byte offsets where PAGE_MARK appeared
    in scr_msg after removing PAGE_MARK itself.  Existing PAGE_MARK in message is
    removed first to avoid accidental double insertion.
    """
    offsets = page_mark_byte_offsets(scr_msg, encoding)
    if not offsets:
        return message

    clean = strip_page_marks(message)
    out: list[str] = []
    acc = 0
    mark_i = 0
    for ch in clean:
        out.append(ch)
        acc += len(ch.encode(encoding))
        while mark_i < len(offsets) and offsets[mark_i] <= acc:
            out.append(PAGE_MARK)
            mark_i += 1

    # If the translated message is shorter than the original breakpoint offset,
    # keep the required control marks at the end rather than dropping them.
    while mark_i < len(offsets):
        out.append(PAGE_MARK)
        mark_i += 1
    return "".join(out)


def u32le(data: bytes | bytearray, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little")


def put_u32le(buf: bytearray, off: int, value: int) -> None:
    if not (0 <= value <= 0xFFFFFFFF):
        raise ValueError(f"u32 out of range: {value}")
    buf[off:off + 4] = value.to_bytes(4, "little")


def instruction_length_at(data: bytes, off: int) -> int:
    if off + 2 > len(data):
        raise ValueError(f"truncated instruction header at 0x{off:X}")
    op = data[off]
    b1 = data[off + 1]
    if b1 == 0:
        raise ValueError(f"zero instruction length/start offset at 0x{off:X}, op=0x{op:02X}")
    b2 = data[off + 2] if off + 2 < len(data) else 0
    b3 = data[off + 3] if off + 3 < len(data) else 0
    if op in VARLEN_B3:
        total = b1 + b3
    elif op == CHOICE_OP:
        total = b1 + b2
    else:
        total = b1
    if total <= 0 or off + total > len(data):
        raise ValueError(
            f"bad instruction length at 0x{off:X}: op=0x{op:02X}, computed={total}, file_size={len(data)}"
        )
    return total


def parse_instructions(data: bytes) -> list[Instruction]:
    out: list[Instruction] = []
    off = 0
    while off < len(data):
        ln = instruction_length_at(data, off)
        out.append(Instruction(offset=off, op=data[off], raw=data[off:off + ln]))
        off += ln
    if off != len(data):
        raise ValueError(f"parse ended at 0x{off:X}, file size 0x{len(data):X}")
    return out


def _strip_text_payload(payload: bytes) -> tuple[bytes, bytes]:
    """Return (text_bytes, stripped_payload_without_right_cd)."""
    stripped = payload.rstrip(bytes([PAD_BYTE]))
    if not stripped.endswith(TEXT_END):
        raise ValueError(f"text payload does not end with 819400(+cd padding): {payload[-16:].hex(' ')}")
    return stripped[:-len(TEXT_END)], stripped


def decode_text_instruction(inst: Instruction, encoding: str = DEFAULT_ENCODING) -> str:
    if inst.op != TEXT_OP:
        raise ValueError("not a text instruction")
    payload_len = inst.b3
    payload = inst.raw[inst.b1:inst.b1 + payload_len]
    text_bytes, _ = _strip_text_payload(payload)
    return decode_bytes(text_bytes, encoding)


def decode_choice_instruction(inst: Instruction, encoding: str = DEFAULT_ENCODING) -> str:
    if inst.op != CHOICE_OP:
        raise ValueError("not a choice instruction")
    text_len = inst.b2
    text_bytes = inst.raw[inst.b1:inst.b1 + text_len]
    return decode_bytes(text_bytes, encoding)


def get_inline_string(inst: Instruction, encoding: str = DEFAULT_ENCODING) -> str | None:
    if inst.op not in VARLEN_B3:
        return None
    if inst.b1 + inst.b3 > len(inst.raw):
        return None
    raw = inst.raw[inst.b1:inst.b1 + inst.b3]
    try:
        return raw.decode(encoding)
    except UnicodeDecodeError:
        return None


def build_offset_index(instructions: list[Instruction]) -> dict[int, int]:
    return {inst.offset: i for i, inst in enumerate(instructions)}


def resolve_choice_target_file(instructions: list[Instruction], target_offset: int,
                               encoding: str = DEFAULT_ENCODING) -> str | None:
    """Resolve a 0x1B choice target to the script file loaded at that target, when present.

    Two common target layouts were observed in the supplied scripts:

    * target -> 01 ... <file.s>
    * target -> 12 / 10 setup instructions -> 01 ... <file.s>

    The function intentionally returns only a best-effort annotation; injection uses the
    numeric offset target and does not depend on this string.
    """
    off_to_idx = build_offset_index(instructions)
    idx = off_to_idx.get(target_offset)
    if idx is None:
        return None
    for inst in instructions[idx:idx + 6]:
        inline = get_inline_string(inst, encoding)
        if inline and inline.lower().endswith(".s"):
            return inline
    return None


def annotate_choice_groups(instructions: list[Instruction]) -> dict[int, dict[str, int]]:
    """Return metadata for consecutive 0x1B choices.

    Choice menus in the samples are encoded as:

        1A ...
        1B ...  # choice 0
        1B ...  # choice 1
        ...
        19 04 00 count

    This annotation is diagnostic/export metadata only; the VM branch target remains the
    dword at p+4 inside each 0x1B instruction.
    """
    meta: dict[int, dict[str, int]] = {}
    group_id = 0
    i = 0
    while i < len(instructions):
        if instructions[i].op != 0x1A:
            i += 1
            continue
        j = i + 1
        choice_offsets: list[int] = []
        while j < len(instructions) and instructions[j].op == CHOICE_OP:
            choice_offsets.append(instructions[j].offset)
            j += 1
        if choice_offsets and j < len(instructions) and instructions[j].op == 0x19:
            count = instructions[j].b3
            for order, off in enumerate(choice_offsets):
                meta[off] = {
                    "group_id": group_id,
                    "group_start_offset": instructions[i].offset,
                    "group_end_offset": instructions[j].offset,
                    "choice_order": order,
                    "choice_count": count,
                }
            group_id += 1
            i = j + 1
        else:
            i += 1
    return meta


def make_text_instruction(inst: Instruction, message: str, encoding: str = DEFAULT_ENCODING,
                          keep_payload_size: int | None = None) -> bytes:
    if inst.op != TEXT_OP:
        raise ValueError("not a text instruction")
    msg = encode_text(message, encoding)
    base = msg + TEXT_END
    if keep_payload_size is None:
        payload_size = len(base)
        # Existing files keep following instruction dword-aligned by padding text payload with 0xCD.
        while (inst.b1 + payload_size) % 4 != 0:
            payload_size += 1
    else:
        payload_size = keep_payload_size
    if payload_size > 0xFF:
        raise ValueError(f"text payload too long for one-byte length: {payload_size} bytes")
    if len(base) > payload_size:
        raise ValueError(f"encoded text too long: need {len(base)} payload bytes, capacity {payload_size}")
    new_raw = bytearray(inst.raw[:inst.b1])
    new_raw[3] = payload_size
    new_raw.extend(base)
    new_raw.extend(bytes([PAD_BYTE]) * (payload_size - len(base)))
    return bytes(new_raw)


def make_choice_instruction(inst: Instruction, message: str, encoding: str = DEFAULT_ENCODING,
                            keep_text_size: int | None = None) -> bytes:
    if inst.op != CHOICE_OP:
        raise ValueError("not a choice instruction")
    msg = encode_text(message, encoding)
    text_size = len(msg) if keep_text_size is None else keep_text_size
    if text_size > 0xFF:
        raise ValueError(f"choice text too long for one-byte length: {text_size} bytes")
    if len(msg) > text_size:
        raise ValueError(f"encoded choice too long: need {len(msg)} bytes, capacity {text_size}")
    new_raw = bytearray(inst.raw[:inst.b1])
    new_raw[2] = text_size
    new_raw.extend(msg)
    if keep_text_size is not None:
        new_raw.extend(b" " * (text_size - len(msg)))
    return bytes(new_raw)


def build_entries(script_path: Path, *, encoding: str = DEFAULT_ENCODING,
                  name_map: dict[str, str] | None = None) -> list[dict[str, Any]]:
    data = script_path.read_bytes()
    instructions = parse_instructions(data)
    choice_group_meta = annotate_choice_groups(instructions)
    entries: list[dict[str, Any]] = []
    text_index = 0
    last_voice: str | None = None

    for inst in instructions:
        inline = get_inline_string(inst, encoding)
        if inline and VOICE_RE.match(inline):
            last_voice = inline

        if inst.op == TEXT_OP:
            text = decode_text_instruction(inst, encoding)
            entry: dict[str, Any] = {
                "scr_msg": text,
                "message": strip_page_marks(text),
                "_file": script_path.name,
                "_index": text_index,
                "_offset": inst.offset + inst.b1,
                "_inst_offset": inst.offset,
                "_size": inst.b3,
                "_type": "dialogue" if inst.b2 else "monologue",
                "_opcode": "0x04",
                "_flag": inst.b2,
                "_encoding": encoding,
                "_policy": "relocate",
            }
            page_offsets = page_mark_byte_offsets(text, encoding)
            if page_offsets:
                entry["_page_mark"] = PAGE_MARK
                entry["_page_mark_byte_offsets"] = page_offsets
                entry["_message_page_mark_mode"] = "auto_from_scr_msg"
            if last_voice:
                entry["_voice"] = last_voice
                entry["_speaker_code"] = last_voice[:2].lower()
                if name_map and last_voice[:2].lower() in name_map:
                    entry["name"] = name_map[last_voice[:2].lower()]
            entries.append(entry)
            text_index += 1
            last_voice = None

        elif inst.op == CHOICE_OP:
            text = decode_choice_instruction(inst, encoding)
            target = u32le(inst.raw, 4)
            entry = {
                "scr_msg": text,
                "message": strip_page_marks(text),
                "_file": script_path.name,
                "_index": text_index,
                "_offset": inst.offset + inst.b1,
                "_inst_offset": inst.offset,
                "_size": inst.b2,
                "_type": "choice",
                "_opcode": "0x1B",
                "_target": f"0x{target:X}",
                "_target_file": resolve_choice_target_file(instructions, target, encoding),
                "_encoding": encoding,
                "_policy": "relocate",
            }
            group = choice_group_meta.get(inst.offset)
            if group:
                entry["_choice_group"] = group["group_id"]
                entry["_choice_order"] = group["choice_order"]
                entry["_choice_count"] = group["choice_count"]
                entry["_choice_group_start"] = f"0x{group['group_start_offset']:X}"
                entry["_choice_group_end"] = f"0x{group['group_end_offset']:X}"
            entries.append({k: v for k, v in entry.items() if v is not None})
            text_index += 1

    return entries


def iter_script_files(path: Path) -> Iterable[Path]:
    if path.is_file():
        yield path
    else:
        yield from sorted(p for p in path.rglob("*") if p.is_file() and p.suffix.lower() == ".s")


def json_name_for_script(script_path: Path) -> str:
    return f"{script_path.name}.json"


def _entry_key(entry: dict[str, Any]) -> tuple[str, int] | None:
    f = entry.get("_file")
    idx = entry.get("_index")
    if isinstance(f, str) and isinstance(idx, int):
        return f, idx
    return None


def entries_for_file(all_entries: list[dict[str, Any]], file_name: str) -> dict[int, dict[str, Any]]:
    out: dict[int, dict[str, Any]] = {}
    for e in all_entries:
        if e.get("_file") == file_name and isinstance(e.get("_index"), int):
            out[e["_index"]] = e
    return out


def load_name_map(path: str | Path | None) -> dict[str, str] | None:
    if not path:
        return None
    data = read_json(path)
    # Also accept a plain dict JSON for convenience.
    if isinstance(data, list):
        raise ValueError("name-map must be a JSON object like {\"ka\": \"カスミ\"}")
    return None


def load_name_map_object(path: str | Path | None) -> dict[str, str] | None:
    if not path:
        return None
    with open(path, "r", encoding="utf-8") as f:
        obj = json.load(f)
    if not isinstance(obj, dict):
        raise ValueError("name-map must be a JSON object like {\"ka\": \"カスミ\"}")
    return {str(k).lower(): str(v) for k, v in obj.items()}


def _make_boundary_mapper(old_offsets: list[int], old_lengths: list[int], new_offsets: list[int]):
    old_to_new = {old: new for old, new in zip(old_offsets, new_offsets)}
    old_ends = [o + l for o, l in zip(old_offsets, old_lengths)]

    def map_offset(target: int) -> int | None:
        if target in old_to_new:
            return old_to_new[target]
        # Allow mapping an address inside an unchanged/opaque instruction, but not inside resized text/choice data.
        idx = bisect.bisect_right(old_offsets, target) - 1
        if idx >= 0 and target < old_ends[idx]:
            return new_offsets[idx] + (target - old_offsets[idx])
        return None

    return map_offset


def patch_script(original_data: bytes, file_name: str, entries: dict[int, dict[str, Any]], *,
                 encoding: str = DEFAULT_ENCODING, mode: str = "relocate",
                 strict: bool = False) -> tuple[bytes, dict[str, Any]]:
    if mode not in {"relocate", "in-place"}:
        raise ValueError("mode must be 'relocate' or 'in-place'")

    instructions = parse_instructions(original_data)
    rebuilt: list[bytearray] = []
    old_offsets: list[int] = []
    old_lengths: list[int] = []
    new_offsets: list[int] = []
    stats: dict[str, Any] = {
        "patched": 0,
        "unchanged": 0,
        "skipped": 0,
        "failed": 0,
        "warnings": [],
        "mode": mode,
        "file": file_name,
    }
    text_index = 0
    new_pos = 0

    for inst in instructions:
        raw_new = bytes(inst.raw)
        entry = None
        entry_type = None
        old_text = None
        if inst.op == TEXT_OP:
            entry = entries.get(text_index)
            entry_type = "text"
            old_text = decode_text_instruction(inst, encoding)
        elif inst.op == CHOICE_OP:
            entry = entries.get(text_index)
            entry_type = "choice"
            old_text = decode_choice_instruction(inst, encoding)

        if entry is not None:
            scr_msg = entry.get("scr_msg")
            message = entry.get("message")
            if not isinstance(scr_msg, str) or not isinstance(message, str):
                msg = f"index={text_index} missing scr_msg/message"
                stats["warnings"].append(msg)
                stats["failed"] += 1
                if strict:
                    raise ValueError(msg)
            elif scr_msg != old_text:
                msg = f"index={text_index} scr_msg mismatch at 0x{inst.offset:X}: json={scr_msg!r}, file={old_text!r}"
                stats["warnings"].append(msg)
                stats["failed"] += 1
                if strict:
                    raise ValueError(msg)
            else:
                message_for_inject = (
                    apply_page_marks_from_scr_msg(scr_msg, message, encoding)
                    if entry_type == "text" else message
                )
                if message_for_inject == old_text:
                    stats["unchanged"] += 1
                else:
                    try:
                        if entry_type == "text":
                            if mode == "in-place":
                                raw_new = make_text_instruction(inst, message_for_inject, encoding, keep_payload_size=inst.b3)
                            else:
                                raw_new = make_text_instruction(inst, message_for_inject, encoding, keep_payload_size=None)
                        else:
                            if mode == "in-place":
                                raw_new = make_choice_instruction(inst, message_for_inject, encoding, keep_text_size=inst.b2)
                            else:
                                raw_new = make_choice_instruction(inst, message_for_inject, encoding, keep_text_size=None)
                        stats["patched"] += 1
                    except Exception as ex:
                        msg = f"index={text_index} patch failed at 0x{inst.offset:X}: {ex}"
                        stats["warnings"].append(msg)
                        stats["failed"] += 1
                        if strict:
                            raise
                        raw_new = bytes(inst.raw)

        if inst.op in {TEXT_OP, CHOICE_OP}:
            text_index += 1

        old_offsets.append(inst.offset)
        old_lengths.append(inst.length)
        new_offsets.append(new_pos)
        rebuilt.append(bytearray(raw_new))
        new_pos += len(raw_new)

    map_offset = _make_boundary_mapper(old_offsets, old_lengths, new_offsets)
    fixed_targets = 0
    unresolved_targets: list[str] = []
    for inst, raw in zip(instructions, rebuilt):
        for field_off in TARGET_FIELD_OFFSETS.get(inst.op, ()):
            if field_off + 4 > len(inst.raw) or field_off + 4 > len(raw):
                continue
            old_target = u32le(inst.raw, field_off)
            mapped = map_offset(old_target)
            if mapped is None:
                # Some script references can target external files/states or unused sentinel-like values.
                unresolved_targets.append(f"0x{inst.offset:X}:op=0x{inst.op:02X}:target=0x{old_target:X}")
                continue
            if mapped != old_target:
                put_u32le(raw, field_off, mapped)
                fixed_targets += 1

    if unresolved_targets:
        stats["warnings"].append("unresolved target offsets: " + ", ".join(unresolved_targets[:20]))
        if len(unresolved_targets) > 20:
            stats["warnings"].append(f"... and {len(unresolved_targets) - 20} more unresolved targets")

    out = b"".join(bytes(x) for x in rebuilt)
    # Structural smoke parse.
    parse_instructions(out)
    stats["entries_in_file"] = text_index
    stats["fixed_targets"] = fixed_targets
    stats["old_size"] = len(original_data)
    stats["new_size"] = len(out)
    return out, stats
