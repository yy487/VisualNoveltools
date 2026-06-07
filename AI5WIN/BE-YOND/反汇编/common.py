# -*- coding: utf-8 -*-
from __future__ import annotations

import importlib.util
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_opcode_path = Path(__file__).with_name("b_mes_opcode.py")
_spec = importlib.util.spec_from_file_location("b_mes_opcode", _opcode_path)
opdef = importlib.util.module_from_spec(_spec)
assert _spec and _spec.loader
_spec.loader.exec_module(opdef)

import disassembler as disasm  # same directory when run as scripts

DEFAULT_ENCODING = opdef.DEFAULT_ENCODING

# These files are resource/loader/table scripts in the supplied set.  They may
# contain byte values that look like TEXT opcodes when read linearly, but they
# are not scenario text and should not enter the translation JSON by default.
NON_SCENARIO_BASENAMES = {
    "0", "NAME", "FLAGINI", "START", "TEST",
    "STAND", "STAND2", "STAND3", "STAND4",
}
NON_SCENARIO_PREFIXES = ("L",)

_NAME_RE = re.compile(r"^【(.+?)】$")

# Project-local AI5WIN gaiji handling.
# scr_msg always keeps the raw {{EB:xx}} byte placeholders for verification.
# message applies this policy: confirmed visual glyphs are converted to readable
# characters; every other EB gaiji/control placeholder is dropped.
CONFIRMED_GAIJI_DECODE: dict[str, str] = {
    "EB:A1": "♪",
    "EB:A5": "！",
    "EB:A6": "！",
    "EB:A8": "？",
    "EB:A9": "！？",
    "EB:AA": "！",
    "EB:B9": "♪",
    "EB:BA": "ォ",
    "EB:BB": "ァ",
}

_PLACEHOLDER_RE = re.compile(r"\{\{([0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2})*)\}\}")

def normalize_gaiji_for_message(text: str) -> str:
    """Make translator-facing text readable.

    Confirmed EB gaiji are mapped to Unicode.  Unconfirmed EB gaiji/control
    placeholders are removed.  Non-EB placeholders are kept, because they may
    represent raw bytes needed for faithful reconstruction.
    """
    def repl(m: re.Match[str]) -> str:
        code = m.group(1).upper()
        if code in CONFIRMED_GAIJI_DECODE:
            return CONFIRMED_GAIJI_DECODE[code]
        if code.startswith("EB:"):
            return ""
        return m.group(0)
    return _PLACEHOLDER_RE.sub(repl, text)


@dataclass
class Instr:
    old_offset: int
    mnemonic: str
    code: int | None
    payload: bytes
    next_offset: int
    target: int | None = None
    text: str | None = None
    raw_text: bytes | None = None


def decode_cstring_payload(payload: bytes, encoding: str) -> tuple[bytes, str]:
    # payload includes trailing NUL for TEXT/SYSTEM_TEXT as produced by safe_parse_payload
    raw = payload[:-1] if payload.endswith(b"\x00") else payload
    return raw, raw.decode(encoding)


def _parse_placeholder_token(token: str) -> bytes:
    inner = token[2:-2].strip()
    if not inner:
        return b""
    return bytes(int(part, 16) for part in inner.split(":"))


def encode_text_bytes(text: str, encoding: str) -> bytes:
    """Encode JSON message text, preserving {{XX:YY}} raw-byte placeholders."""
    out = bytearray()
    i = 0
    while i < len(text):
        if text.startswith("{{", i):
            j = text.find("}}", i + 2)
            if j < 0:
                raise UnicodeEncodeError(encoding, text, i, len(text), "unterminated raw-byte placeholder")
            try:
                out += _parse_placeholder_token(text[i:j + 2])
            except Exception as e:
                raise UnicodeEncodeError(encoding, text, i, j + 2, f"bad raw-byte placeholder: {e}") from e
            i = j + 2
        else:
            out += text[i].encode(encoding)
            i += 1
    return bytes(out)


def parse_instructions(data: bytes, encoding: str = DEFAULT_ENCODING) -> list[Instr]:
    out: list[Instr] = []
    pos = 0
    n = len(data)
    while pos < n:
        mnemonic, payload, nxt, target = disasm.safe_parse_payload(data, pos)
        if nxt <= pos:
            nxt = pos + 1
        if mnemonic == ".byte":
            out.append(Instr(pos, mnemonic, None, payload, nxt, None))
            pos = nxt
            continue
        code = data[pos]
        text = None
        raw_text = None
        if mnemonic in ("TEXT", "SYSTEM_TEXT"):
            raw_text = payload[:-1] if payload.endswith(b"\x00") else payload
            try:
                _, text = decode_cstring_payload(payload, encoding)
            except UnicodeDecodeError:
                # Keep undecodable gaiji/custom bytes as assembler-style placeholders
                # instead of dropping the whole line from JSON.
                text = disasm.encode_asm_string(raw_text, encoding)
        out.append(Instr(pos, mnemonic, code, payload, nxt, target, text, raw_text))
        pos = nxt
    return out


def is_scenario_file(path: Path) -> bool:
    stem = path.stem.upper()
    if stem in NON_SCENARIO_BASENAMES:
        return False
    if stem.startswith(NON_SCENARIO_PREFIXES):
        # L* files in this title are loader/resource/menu helper scripts.
        return False
    # Scenario files in the sample are numbered blocks, H_* event scripts,
    # endings, and a few named top-level scenes.
    if re.match(r"^\d{4}[A-Z0-9]*$", stem):
        return True
    if re.match(r"^H_[A-Z]+\d+$", stem):
        return True
    if re.match(r"^END\d*$", stem):
        return True
    if stem in {"OPEN", "MAIN", "LAST", "DEFMAIN"}:
        return True
    return False


def is_exportable_text(text: str) -> bool:
    if text == "":
        return False
    # Reject control bytes and NEC/IBM private-use garbage usually produced by
    # accidentally reading tables as script strings.
    for ch in text:
        o = ord(ch)
        if o < 0x20 and ch not in "\t\n\r":
            return False
        if 0xE000 <= o <= 0xF8FF:
            return False
    return True


def split_name_text(text: str) -> str | None:
    m = _NAME_RE.match(text)
    if not m:
        return None
    inner = m.group(1).strip()
    return inner or None


def bracket_name(name: str) -> str:
    n = name.strip()
    if n.startswith("【") and n.endswith("】"):
        return n
    return f"【{n}】"


def classify_text(instructions: list[Instr], idx: int) -> str:
    ins = instructions[idx]
    if ins.mnemonic == "SYSTEM_TEXT":
        return "system"
    if ins.mnemonic != "TEXT":
        return "unknown"
    prev = instructions[idx - 1].mnemonic if idx > 0 else ""
    nxt = instructions[idx + 1].mnemonic if idx + 1 < len(instructions) else ""
    if ins.text and split_name_text(ins.text) is not None:
        return "name"
    # AI5WIN menu choices are emitted as:
    # REGISTER_BRANCH_AND_SKIP <arglist> <branch_end>; TEXT "choice"; RETURN
    if prev == "REGISTER_BRANCH_AND_SKIP" and nxt == "RETURN":
        return "choice"
    return "dialogue"


def make_text_obj(*, name: str | None, scr_msg: str, rel: str, text_index: int,
                  typ: str, ins: Instr, encoding: str,
                  name_ins: Instr | None = None, name_scr: str | None = None) -> dict[str, Any]:
    obj: dict[str, Any] = {}
    if name is not None:
        obj["name"] = name
    obj["scr_msg"] = scr_msg
    obj["message"] = normalize_gaiji_for_message(scr_msg)
    obj["_file"] = rel
    obj["_index"] = text_index
    obj["_type"] = typ
    obj["_inst_offset"] = ins.old_offset
    obj["_offset"] = ins.old_offset + 1
    obj["_size"] = len(ins.raw_text or b"")
    if name_ins is not None and name_scr is not None:
        obj["_name_inst_offset"] = name_ins.old_offset
        obj["_name_offset"] = name_ins.old_offset + 1
        obj["_name_size"] = len(name_ins.raw_text or b"")
        obj["_name_scr"] = name_scr
    obj["_encoding"] = encoding
    obj["_policy"] = "relocate"
    return obj


def text_entries_for_file(path: Path, root: Path | None = None, encoding: str = DEFAULT_ENCODING,
                          include_system: bool = False, include_non_scenario: bool = False) -> list[dict[str, Any]]:
    if not include_non_scenario and not is_scenario_file(path):
        return []
    data = path.read_bytes()
    instrs = parse_instructions(data, encoding)
    entries: list[dict[str, Any]] = []
    text_index = 0
    rel = path.relative_to(root).as_posix() if root else path.name
    pending_name: str | None = None
    pending_name_ins: Instr | None = None
    pending_name_scr: str | None = None

    for i, ins in enumerate(instrs):
        if ins.mnemonic not in ("TEXT", "SYSTEM_TEXT"):
            continue
        typ = classify_text(instrs, i)
        if typ == "system":
            if not include_system:
                continue
            if ins.text is None or not is_exportable_text(ins.text):
                continue
            entries.append(make_text_obj(name=None, scr_msg=ins.text, rel=rel, text_index=text_index,
                                         typ="system", ins=ins, encoding=encoding))
            text_index += 1
            continue
        if ins.mnemonic != "TEXT" or ins.text is None or not is_exportable_text(ins.text):
            continue
        if typ == "name":
            nm = split_name_text(ins.text)
            pending_name = normalize_gaiji_for_message(nm)
            pending_name_ins = ins
            pending_name_scr = ins.text
            continue
        if typ == "choice":
            # Options must not inherit the previous speaker name.
            entries.append(make_text_obj(name=None, scr_msg=ins.text, rel=rel, text_index=text_index,
                                         typ="choice", ins=ins, encoding=encoding))
            text_index += 1
            continue

        entries.append(make_text_obj(name=pending_name, scr_msg=ins.text, rel=rel, text_index=text_index,
                                     typ="dialogue", ins=ins, encoding=encoding,
                                     name_ins=pending_name_ins, name_scr=pending_name_scr))
        text_index += 1
        pending_name = None
        pending_name_ins = None
        pending_name_scr = None
    return entries


def dump_json(path: Path, obj: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2)


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def assemble_with_replacements(data: bytes, replacements: dict[int, str], encoding: str = DEFAULT_ENCODING) -> tuple[bytes, list[str]]:
    """Rebuild one MES with variable-length TEXT replacements.

    replacements maps original instruction offset (_inst_offset or _name_inst_offset)
    to new text.  Jump/call/branch absolute targets are relocated when they target
    known original instruction offsets.
    """
    instrs = parse_instructions(data, encoding)
    warnings: list[str] = []

    new_payloads: dict[int, bytes] = {}
    for idx, ins in enumerate(instrs):
        if ins.mnemonic not in ("TEXT", "SYSTEM_TEXT"):
            continue
        if ins.old_offset in replacements:
            try:
                raw = encode_text_bytes(replacements[ins.old_offset], encoding)
            except UnicodeEncodeError as e:
                raise UnicodeEncodeError(e.encoding, e.object, e.start, e.end,
                                         f"{e.reason}; inst_offset=0x{ins.old_offset:08X}") from e
            new_payloads[idx] = raw + b"\x00"

    # First pass: compute new offset of every original instruction start.
    old_to_new: dict[int, int] = {}
    pc = 0
    for i, ins in enumerate(instrs):
        old_to_new[ins.old_offset] = pc
        if i in new_payloads:
            pc += 1 + len(new_payloads[i])
        else:
            pc += ins.next_offset - ins.old_offset

    out = bytearray()
    for i, ins in enumerate(instrs):
        if ins.mnemonic == ".byte":
            out += ins.payload
            continue
        assert ins.code is not None
        out.append(ins.code)
        fmt = opdef.OPCODES[ins.code]["format"]
        if i in new_payloads:
            out += new_payloads[i]
            continue
        if fmt in ("target:u32le",):
            old_target = ins.target
            if old_target in old_to_new:
                out += old_to_new[old_target].to_bytes(4, "little")
            elif old_target is not None:
                if 0 <= old_target < len(data):
                    warnings.append(f"target 0x{old_target:08X} at 0x{ins.old_offset:08X} not at instruction boundary; kept original")
                out += old_target.to_bytes(4, "little")
            else:
                out += ins.payload
            continue
        if fmt in ("expr target:u32le", "arglist_00 target:u32le"):
            prefix = ins.payload[:-4]
            old_target = ins.target
            out += prefix
            if old_target in old_to_new:
                out += old_to_new[old_target].to_bytes(4, "little")
            elif old_target is not None:
                if 0 <= old_target < len(data):
                    warnings.append(f"target 0x{old_target:08X} at 0x{ins.old_offset:08X} not at instruction boundary; kept original")
                out += old_target.to_bytes(4, "little")
            else:
                out += ins.payload[-4:]
            continue
        out += ins.payload
    return bytes(out), warnings
