# -*- coding: utf-8 -*-
"""Whole-script rebuilding injector for Refrain Blue / AI5WIN MES.ARC.

This tool is deliberately different from the older EOF-trampoline injector:

* it never replaces an original TEXT opcode with OPX0A;
* it rebuilds each MES byte stream in order;
* known absolute target fields are relocated through an old_offset -> new_offset map;
* untranslated/unknown bytes are preserved verbatim.

The parser is conservative: TEXT/RUBY_TEXT spans are recognized structurally,
then known branch/jump target fields are relocated.  Unknown VM bytes remain raw
and keep their relative position unless preceding text length changes.
"""
from __future__ import annotations

import argparse
import json
import sys
import zipfile
from bisect import bisect_right
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# Load sibling modules robustly for drag/drop usage.
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import rp_arc  # type: ignore
from rp_mes_common import scan_mes_text  # type: ignore

# Local rp_mes_opcode.py cannot be imported as "opcode" because stdlib already uses that name.
import importlib.util as _importlib_util
_OPCODE_PATH = SCRIPT_DIR / "rp_mes_opcode.py"
_spec = _importlib_util.spec_from_file_location("rp_mes_opcode", _OPCODE_PATH)
if _spec is None or _spec.loader is None:
    raise ImportError(f"cannot load local rp_mes_opcode.py from {_OPCODE_PATH}")
_opcode_mod = _importlib_util.module_from_spec(_spec)
sys.modules[_spec.name] = _opcode_mod
_spec.loader.exec_module(_opcode_mod)
DEFAULT_ENCODING = _opcode_mod.DEFAULT_ENCODING

# Reuse the byte-safe TEXT/RUBY scanner from disassembler.py.
_DISASM_PATH = SCRIPT_DIR / "disassembler.py"
_dspec = _importlib_util.spec_from_file_location("rp_mes_disassembler", _DISASM_PATH)
if _dspec is None or _dspec.loader is None:
    raise ImportError(f"cannot load local disassembler.py from {_DISASM_PATH}")
_disasm = _importlib_util.module_from_spec(_dspec)
sys.modules[_dspec.name] = _disasm
_dspec.loader.exec_module(_disasm)

FULLWIDTH_SPACE_CP932 = "　".encode("cp932")
HALFWIDTH_SPACE_CP932 = b" "


@dataclass
class RelocField:
    field_off: int
    old_target: int
    kind: str


@dataclass
class Span:
    old_start: int
    old_end: int
    kind: str
    raw: bytes
    text_bytes: bytes | None = None
    prefix: bytes = b""
    reloc_fields: list[RelocField] = field(default_factory=list)

    @property
    def old_len(self) -> int:
        return self.old_end - self.old_start

    def new_len(self, text_patch: bytes | None = None) -> int:
        if self.kind in ("TEXT", "RUBY_TEXT") and text_patch is not None:
            return len(self.prefix) + len(text_patch) + 1
        return len(self.raw)


@dataclass
class Segment:
    old_start: int
    old_end: int
    new_start: int
    new_end: int
    span: Span | None


def _is_cp932_lead(b: int) -> bool:
    return 0x81 <= b <= 0x9F or 0xE0 <= b <= 0xFC


def _make_space_padding_exact(n: int) -> bytes:
    if n <= 0:
        return b""
    return FULLWIDTH_SPACE_CP932 * (n // 2) + (HALFWIDTH_SPACE_CP932 if n % 2 else b"")


def _decode(raw: bytes, encoding: str) -> str:
    return raw.decode(encoding)


def _encode(s: str, encoding: str) -> bytes:
    return s.encode(encoding)


def _read_json(path: Path) -> list[dict[str, Any]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        for key in ("entries", "texts", "messages"):
            if isinstance(data.get(key), list):
                return data[key]
    raise ValueError(f"unsupported JSON root: {path}")


def _load_json_dir(path: Path) -> dict[str, list[dict[str, Any]]]:
    """Return mapping MES_NAME -> JSON entries.

    Accepts either a directory or a zip.  JSON files are expected to be named
    03B.json, 03B.MES.json, etc.
    """
    if path.is_file() and path.suffix.lower() == ".zip":
        out: dict[str, list[dict[str, Any]]] = {}
        with zipfile.ZipFile(path, "r") as zf:
            for name in zf.namelist():
                if not name.lower().endswith(".json"):
                    continue
                base = Path(name).name
                raw = zf.read(name).decode("utf-8")
                obj = json.loads(raw)
                if isinstance(obj, dict):
                    for key in ("entries", "texts", "messages"):
                        if isinstance(obj.get(key), list):
                            obj = obj[key]
                            break
                if not isinstance(obj, list):
                    raise ValueError(f"unsupported JSON root in zip member: {name}")
                stem = base[:-5]
                if stem.upper().endswith(".MES"):
                    mes = stem.upper()
                else:
                    mes = (stem + ".MES").upper()
                out[mes] = obj
        return out

    out = {}
    for jp in path.rglob("*.json"):
        stem = jp.name[:-5]
        if stem.upper().endswith(".MES"):
            mes = stem.upper()
        else:
            mes = (stem + ".MES").upper()
        out[mes] = _read_json(jp)
    return out


def _split_message_by_original_parts(message: str, scr_parts: list[str]) -> list[str]:
    if not scr_parts:
        return [message]
    result: list[str] = []
    pos = 0
    chars = list(message)
    for part in scr_parts[:-1]:
        take = len(part)
        result.append("".join(chars[pos:pos + take]))
        pos += take
    result.append("".join(chars[pos:]))
    return result


def _build_text_patch_map(
    data: bytes,
    entries: list[dict[str, Any]],
    *,
    encoding: str,
    ruby_policy: str,
    file_name: str = "<memory>",
) -> tuple[dict[int, bytes], dict[str, Any]]:
    patches: dict[int, bytes] = {}
    # Current-file logical scan is used only as a fallback when JSON offsets come
    # from a nearby script revision.  Primary validation still uses _offset/_end.
    try:
        current_records = [r.to_json_obj() for r in scan_mes_text(data, file_name)]
    except Exception:
        current_records = []

    report: dict[str, Any] = {
        "json_entries": len(entries),
        "text_patches": 0,
        "part_patches": 0,
        "ruby_blank_fullwidth": 0,
        "skipped_same": 0,
        "warnings": [],
        "errors": [],
    }

    def validate_text(off: int, end: int, scr: str, label: str, hint: tuple[int, int] | None = None) -> tuple[int, int] | None:
        try:
            expected = _encode(scr, encoding)
        except Exception as ex:
            report["errors"].append(f"{label}: cannot encode scr_msg for validation: {ex}")
            return None

        def direct_ok(o: int, e: int) -> bool:
            return (
                0 <= o < e <= len(data)
                and data[o] == 0x01
                and data[e - 1] == 0
                and data[o + 1:e - 1] == expected
            )

        if direct_ok(off, end):
            return off, end
        if hint is not None and direct_ok(hint[0], hint[1]):
            report["warnings"].append(
                f"{label}: offset fallback 0x{off:X}->0x{hint[0]:X} by _index"
            )
            return hint

        # Offset mismatch happens when the JSON was extracted from a slightly
        # different official script revision.  Fallback is allowed only when the
        # same scr_msg occurs uniquely in this MES file.
        needle = b"\x01" + expected + b"\x00"
        hits: list[int] = []
        pos = data.find(needle)
        while pos >= 0:
            hits.append(pos)
            pos = data.find(needle, pos + 1)
        if len(hits) == 1:
            new_off = hits[0]
            new_end = new_off + len(needle)
            report["warnings"].append(
                f"{label}: offset fallback 0x{off:X}->0x{new_off:X} by unique scr_msg"
            )
            return new_off, new_end

        if not (0 <= off < end <= len(data)):
            reason = f"invalid span off=0x{off:X}, end=0x{end:X}"
        elif data[off] != 0x01:
            reason = f"0x{off:X} is not TEXT opcode 01"
        elif data[end - 1] != 0:
            reason = f"0x{end - 1:X} is not C-string terminator"
        else:
            old = data[off + 1:end - 1]
            try:
                actual = old.decode(encoding)
            except Exception:
                actual = old.hex(" ")
            reason = f"scr_msg mismatch at 0x{off:X}: json={scr!r}, file={actual!r}"
        report["errors"].append(f"{label}: {reason}; unique fallback hits={len(hits)}")
        return None

    def add_patch(off: int, end: int, scr: str, msg: str, label: str, *, is_part: bool = False, hint: tuple[int, int] | None = None) -> None:
        if msg == scr:
            report["skipped_same"] += 1
            return
        fixed = validate_text(off, end, scr, label, hint)
        if fixed is None:
            return
        off, end = fixed
        try:
            new_raw = _encode(msg, encoding)
        except Exception as ex:
            report["errors"].append(f"{label}: message is not encodable as {encoding}: {ex}")
            return
        if off in patches and patches[off] != new_raw:
            report["errors"].append(f"{label}: duplicate conflicting patch at 0x{off:X}")
            return
        patches[off] = new_raw
        report["part_patches" if is_part else "text_patches"] += 1

    def add_ruby_blank(entry: dict[str, Any], idx: int) -> None:
        if ruby_policy in ("", "keep", "none"):
            return
        if ruby_policy not in ("blank-fullwidth", "blank", "fullwidth-space"):
            report["errors"].append(f"entry[{idx}]: unsupported ruby_policy={ruby_policy!r}")
            return
        rubies = entry.get("_ruby")
        if not isinstance(rubies, list):
            if entry.get("_type") == "ruby" and all(k in entry for k in ("_offset", "_end", "scr_msg")):
                rubies = [{"rt": entry.get("scr_msg", ""), "rt_offset": entry.get("_offset"), "rt_end": entry.get("_end")}]
            else:
                return
        for ridx, r in enumerate(rubies):
            if not isinstance(r, dict):
                continue
            try:
                rt_offset = int(r.get("rt_offset"))
                rt_end = int(r.get("rt_end"))
                rt = str(r.get("rt", ""))
            except Exception as ex:
                report["errors"].append(f"entry[{idx}]._ruby[{ridx}]: invalid metadata: {ex}")
                continue
            hint = None
            if idx < len(current_records):
                cr = current_records[idx]
                cr_rubies = cr.get("_ruby")
                if isinstance(cr_rubies, list) and ridx < len(cr_rubies) and isinstance(cr_rubies[ridx], dict):
                    rr = cr_rubies[ridx]
                    if str(rr.get("rt", "")) == rt and rr.get("rt_offset") is not None and rr.get("rt_end") is not None:
                        hint = (int(rr["rt_offset"]), int(rr["rt_end"]))
            fixed = validate_text(rt_offset, rt_end, rt, f"entry[{idx}]._ruby[{ridx}]", hint)
            if fixed is None:
                continue
            rt_offset, rt_end = fixed
            old_len = rt_end - rt_offset - 2
            fill = _make_space_padding_exact(old_len)
            if len(fill) != old_len:
                report["errors"].append(f"entry[{idx}]._ruby[{ridx}]: ruby fill size mismatch")
                continue
            # Rebuilder spans RUBY_TEXT from the 0B 15 FF 01 prefix, while
            # JSON metadata stores rt_offset at the embedded opcode 01.
            patch_key = rt_offset - 3 if rt_offset >= 3 and data[rt_offset - 3:rt_offset] == b"\x0B\x15\xFF" else rt_offset
            if patch_key not in patches:
                patches[patch_key] = fill
                report["ruby_blank_fullwidth"] += 1

    for idx, e in enumerate(entries):
        if not isinstance(e, dict):
            report["warnings"].append(f"entry[{idx}] is not object; skipped")
            continue
        add_ruby_blank(e, idx)

        parts = e.get("_parts")
        if isinstance(parts, list) and parts:
            norm_parts = [p for p in parts if isinstance(p, dict)]
            if len(norm_parts) != len(parts):
                report["errors"].append(f"entry[{idx}]: _parts contains non-object item")
                continue
            scr_parts_obj = e.get("scr_msg_parts")
            if isinstance(scr_parts_obj, list) and len(scr_parts_obj) == len(norm_parts):
                scr_parts = [str(x) for x in scr_parts_obj]
            else:
                scr_parts = [str(p.get("scr_msg", "")) for p in norm_parts]
            msg = e.get("message", e.get("scr_msg"))
            scr = e.get("scr_msg")
            msg_parts_obj = e.get("message_parts")
            if isinstance(msg_parts_obj, list) and len(msg_parts_obj) == len(norm_parts):
                msg_parts = [str(x) for x in msg_parts_obj]
                if isinstance(msg, str) and isinstance(scr, str) and msg != scr and msg_parts == scr_parts:
                    msg_parts = _split_message_by_original_parts(msg, scr_parts)
                    report["warnings"].append(f"entry[{idx}]: flat message edited; split into visible parts automatically")
            elif isinstance(msg, str) and isinstance(scr, str) and msg != scr:
                msg_parts = _split_message_by_original_parts(msg, scr_parts)
                report["warnings"].append(f"entry[{idx}]: no valid message_parts; split flat message automatically")
            else:
                msg_parts = scr_parts
            for pidx, (p, part_msg) in enumerate(zip(norm_parts, msg_parts)):
                if not all(k in p for k in ("scr_msg", "_offset", "_end")):
                    report["errors"].append(f"entry[{idx}].part[{pidx}]: missing scr_msg/_offset/_end")
                    continue
                hint = None
                if idx < len(current_records):
                    cr = current_records[idx]
                    cr_parts = cr.get("_parts")
                    if isinstance(cr_parts, list) and pidx < len(cr_parts) and isinstance(cr_parts[pidx], dict):
                        cp = cr_parts[pidx]
                        if str(cp.get("scr_msg", "")) == str(p["scr_msg"]) and cp.get("_offset") is not None and cp.get("_end") is not None:
                            hint = (int(cp["_offset"]), int(cp["_end"]))
                add_patch(int(p["_offset"]), int(p["_end"]), str(p["scr_msg"]), str(part_msg), f"entry[{idx}].part[{pidx}]", is_part=True, hint=hint)
            continue

        scr = e.get("scr_msg")
        msg = e.get("message", scr)
        if not isinstance(scr, str) or not isinstance(msg, str):
            report["warnings"].append(f"entry[{idx}]: missing string scr_msg/message; skipped")
            continue
        if "_offset" not in e or "_end" not in e:
            report["warnings"].append(f"entry[{idx}]: missing _offset/_end; skipped")
            continue
        hint = None
        if idx < len(current_records):
            cr = current_records[idx]
            if str(cr.get("scr_msg", "")) == scr and cr.get("_offset") is not None and cr.get("_end") is not None:
                hint = (int(cr["_offset"]), int(cr["_end"]))
        add_patch(int(e["_offset"]), int(e["_end"]), scr, msg, f"entry[{idx}]", hint=hint)

    return patches, report


def _make_inside(intervals: list[tuple[int, int]]):
    intervals = sorted(intervals)
    starts = [a for a, _ in intervals]
    def inside(off: int) -> bool:
        j = bisect_right(starts, off) - 1
        return j >= 0 and intervals[j][0] < off < intervals[j][1]
    return inside




def _valid_target(target: int, size: int, text_intervals: list[tuple[int, int]]) -> bool:
    if not (0 <= target <= size):
        return False
    if target == size:
        return True
    for a, b in text_intervals:
        if a < target < b:
            return False
    return True


def _parse_expr_checked(data: bytes, p: int) -> tuple[int, bool]:
    """Parse AI5WIN expression bytecode using the EXE-derived length rules.

    Source: sub_413350.  The expression VM is stack-based, but for static
    rebuilding we only need *instruction boundaries*.  Do not validate the
    stack here: helper tokens 0xE0..0xF0 may pop/push internally, and rejecting
    them statically can make valid scripts fall back to unsafe gap handling.

    Token lengths:
      default/00..7F, E0..F0, F4, FF: 1 byte
      80, A0, C0, F5, F6: opcode + u8
      F1, F3: opcode + u16le
      F2: opcode + u32le
      FF: expression terminator, consumed and returned.
    """
    try:
        while p < len(data):
            b = data[p]
            p += 1
            if b == 0xFF:
                return p, True
            if b in (0x80, 0xA0, 0xC0, 0xF5, 0xF6):
                if p + 1 > len(data):
                    return p, False
                p += 1
            elif b in (0xF1, 0xF3):
                if p + 2 > len(data):
                    return p, False
                p += 2
            elif b == 0xF2:
                if p + 4 > len(data):
                    return p, False
                p += 4
            else:
                # default literals, 0xE0..0xF0 runtime helpers, and 0xF4 have
                # no immediate bytes in sub_413350.
                pass
    except Exception:
        return p, False
    return p, False


def _parse_expr_end(data: bytes, p: int) -> int | None:
    end, ok = _parse_expr_checked(data, p)
    return end if ok else None


def _parse_argblock_end(data: bytes, p: int) -> int | None:
    """Parse FUN_004059e0 argument list.

    EXE format:
      arglist := arg* 00
      arg     := 01 <cstring> 00 | 02 <expr_413350> | other_tag

    Tags other than 01/02 consume only the tag byte in the real function.  They
    are rare in MES samples, but supporting them is more faithful than failing
    and then trying an unsafe byte resync.
    """
    try:
        while p < len(data):
            tag = data[p]
            p += 1
            if tag == 0:
                return p
            if tag == 1:
                end = data.find(b"\x00", p, min(len(data), p + 0x400))
                if end < 0:
                    return None
                p = end + 1
            elif tag == 2:
                p2 = _parse_expr_end(data, p)
                if p2 is None:
                    return None
                p = p2
            else:
                # sub_4059E0 just records a parameter slot and consumes no
                # payload for unknown tags.
                continue
    except Exception:
        return None
    return None


def _parse_expr_list_until_zero(data: bytes, p: int, *, with_initial_byte: bool = False, initial_expr: bool = False) -> int | None:
    """Parse opcode families 03-08/14 that contain expression lists.

    Several EXE handlers consume one or more expressions, each followed by a
    one-byte continuation marker; 00 ends the list.  The exact destination of
    the values differs, but the byte layout is enough for instruction boundary
    protection and avoiding false target relocation.
    """
    if with_initial_byte:
        if p >= len(data):
            return None
        p += 1
    if initial_expr:
        p = _parse_expr_end(data, p)
        if p is None:
            return None
    while p < len(data):
        p2 = _parse_expr_end(data, p)
        if p2 is None or p2 >= len(data):
            return None
        marker = data[p2]
        p = p2 + 1
        if marker == 0:
            return p
    return None


def _build_inside_func(intervals: list[tuple[int, int]]):
    intervals = sorted(intervals)
    starts = [a for a, _ in intervals]
    def inside(off: int) -> bool:
        j = bisect_right(starts, off) - 1
        return j >= 0 and intervals[j][0] < off < intervals[j][1]
    def containing_end(off: int) -> int | None:
        j = bisect_right(starts, off) - 1
        if j >= 0 and intervals[j][0] <= off < intervals[j][1]:
            return intervals[j][1]
        return None
    return inside, containing_end


def _parse_top_instruction(data: bytes, pc: int, text_end_at: dict[int, int], inside_text) -> tuple[int, list[RelocField], str] | None:
    """Parse one top-level VM instruction at pc, based on EXE handlers.

    Returns (end, reloc_fields, kind).  The parser is intentionally conservative:
    invalid candidates return None instead of guessing.
    """
    size = len(data)
    if pc >= size or inside_text(pc):
        return None
    if pc in text_end_at:
        return text_end_at[pc], [], "TEXTLIKE"
    op = data[pc]
    p = pc + 1

    def need(n: int) -> bool:
        return p + n <= size

    if op == 0x00:
        return p, [], "RETURN"
    if op in (0x01, 0x02):
        item = _disasm.read_cstring(data, p) if hasattr(_disasm, 'read_cstring') else None
        if item is None:
            return None
        _raw, end = item
        return end, [], "TEXT" if op == 1 else "SYSTEM_TEXT"
    if op == 0x03:
        if p + 2 > size:
            return None
        p += 2
        while p < size:
            p2 = _parse_expr_end(data, p)
            if p2 is None or p2 >= size:
                return None
            marker = data[p2]
            p = p2 + 1
            if marker == 0:
                return p, [], "B_FLAG_SET"
    if op == 0x04:
        # FUN_00406EA0: u8 destination index, then expr values separated by marker bytes.
        end = _parse_expr_list_until_zero(data, p, with_initial_byte=True)
        return (end, [], "W_FLAG_SET") if end is not None else None
    if op == 0x05:
        # initial destination offset expression, then expr values separated by marker
        end = _parse_expr_list_until_zero(data, p, initial_expr=True)
        return (end, [], "EXT_B_FLAG_SET") if end is not None else None
    if op in (0x06, 0x07, 0x08, 0x14):
        # FUN_00406F90 / 00407010 / 004070A0 / 00406F40:
        # initial destination/index expression, one bank/index byte, then expr list.
        # 0x14 used to be grouped with 0x04, but EXE shows it is the same
        # layout as this family.  Misparsing it can stop gap parsing before
        # later 09/0A/0E targets in the same control block.
        first = _parse_expr_end(data, p)
        if first is None or first >= size:
            return None
        end = _parse_expr_list_until_zero(data, first, with_initial_byte=True)
        return (end, [], "ARRAY_SET") if end is not None else None
    if op == 0x09:
        expr_end = _parse_expr_end(data, p)
        if expr_end is None or expr_end + 4 > size:
            return None
        t = int.from_bytes(data[expr_end:expr_end + 4], "little")
        return expr_end + 4, [RelocField(expr_end, t, "JUMP_IF_FALSE")], "JUMP_IF_FALSE"
    if op == 0x0A:
        if not need(4):
            return None
        t = int.from_bytes(data[p:p + 4], "little")
        return p + 4, [RelocField(p, t, "JUMP")], "JUMP"
    if op == 0x0B:
        expr_end = _parse_expr_end(data, p)
        if expr_end is None:
            return None
        arg_end = _parse_argblock_end(data, expr_end)
        if arg_end is None:
            return None
        return arg_end, [], "SYS"
    if op in (0x0C, 0x0D, 0x0F):
        arg_end = _parse_argblock_end(data, p)
        if arg_end is None:
            return None
        return arg_end, [], {0x0C:"SYS_ARGS",0x0D:"CALL_ARGS",0x0F:"CALL"}[op]
    if op == 0x0E:
        arg_end = _parse_argblock_end(data, p)
        if arg_end is None or arg_end + 4 > size:
            return None
        t = int.from_bytes(data[arg_end:arg_end + 4], "little")
        return arg_end + 4, [RelocField(arg_end, t, "CHOICE_BRANCH")], "CHOICE_BRANCH"
    if op == 0x10:
        # EXE/sample-derived: opcode 0x10 consumes a FUN_004059E0-style
        # argument list.  It is used around animation/movie playback blocks,
        # e.g. 10 02 F1 2C 01 FF ... 01 "hanabi10.avi" 00 ... 00.
        # Treating it as a single byte makes the following 02/01 bytes look
        # like top-level SYSTEM_TEXT/TEXT and stops relocation of later jumps.
        arg_end = _parse_argblock_end(data, p)
        if arg_end is None:
            return None
        return arg_end, [], "ARG_MENU_SET"
    if op == 0x11:
        return (p + 1, [], "INTERRUPT") if need(1) else None
    if op == 0x12:
        expr_end = _parse_expr_end(data, p)
        if expr_end is None or expr_end + 4 > size:
            return None
        t = int.from_bytes(data[expr_end:expr_end + 4], "little")
        return expr_end + 4, [RelocField(expr_end, t, "CALL_SAVE_JUMP")], "CALL_SAVE_JUMP"
    if op == 0x13:
        return p, [], "NEW_LINE"
    if op == 0xCD:
        return (p + 1, [], "TEXT_CTRL_CD") if need(1) else None
    if op == 0xCE:
        return p, [], "TEXT_CTRL_CE"
    if op == 0xCF:
        return p, [], "TEXT_CTRL_CF"
    if op == 0xDD:
        return (p + 2, [], "LAYER_CTRL") if need(2) else None
    if op == 0xDF:
        item = _disasm.read_cstring(data, p) if hasattr(_disasm, 'read_cstring') else None
        if item is None:
            return None
        return item[1], [], "NAME_TEXT"
    if op == 0xEE:
        return (p + 1, [], "BYTE_CTRL_EE") if need(1) else None
    if op == 0xEF:
        if p + 4 > size:
            return None
        item = _disasm.read_cstring(data, p + 4) if hasattr(_disasm, 'read_cstring') else None
        if item is None:
            return None
        return item[1], [], "VOICE_CTRL"
    return None


def _scan_structured_reloc_spans(data: bytes, text_intervals: list[tuple[int, int]]) -> tuple[list[Span], dict[str, int]]:
    """Find target-bearing instructions through a conservative CFG walk.

    v1 searched raw bytes for 09/0A/0E, so expression bytes could be relocated
    as if they were jump operands.  That is exactly the kind of corruption that
    produces the VM "stack data remains" dialog.  v2 starts at script PC 0 and
    walks fallthrough/target edges, parsing instruction boundaries using the
    EXE-derived layout rules.  Unknown candidates stop that path; they are not
    scanned byte-by-byte as executable code.
    """
    from collections import deque

    size = len(data)
    inside_text, containing_end = _build_inside_func(text_intervals)
    text_end_at = {a: b for a, b in text_intervals}

    q: deque[int] = deque([0])
    seen: set[int] = set()
    starts: set[int] = {0, size} | set(text_end_at)
    candidates: list[tuple[int, int, str, list[RelocField]]] = []
    parse_failures = 0

    while q:
        pc = q.popleft()
        if not (0 <= pc < size) or pc in seen:
            continue
        ce = containing_end(pc)
        if ce is not None and pc not in text_end_at:
            pc = ce
            if not (0 <= pc < size) or pc in seen:
                continue
        parsed = _parse_top_instruction(data, pc, text_end_at, inside_text)
        if parsed is None:
            parse_failures += 1
            continue
        end, rfs, kind = parsed
        if end <= pc or end > size:
            parse_failures += 1
            continue

        seen.add(pc)
        starts.add(pc)
        starts.add(end)
        if rfs:
            candidates.append((pc, end, kind, rfs))

        # Conservative execution-flow approximation.  Some opcodes are
        # subroutine-like or VM-internal; walking both fallthrough and explicit
        # targets for conditional/choice records gives us coverage without raw
        # scanning arbitrary data.
        if kind == "RETURN":
            continue
        if kind == "JUMP":
            for rf in rfs:
                q.append(rf.old_target)
            continue
        if kind in ("JUMP_IF_FALSE", "CHOICE_BRANCH", "CALL_SAVE_JUMP"):
            q.append(end)
            for rf in rfs:
                q.append(rf.old_target)
            continue
        q.append(end)

    spans: list[Span] = []
    stats: dict[str, int] = {"JUMP_IF_FALSE": 0, "CHOICE_BRANCH": 0, "JUMP": 0, "CALL_SAVE_JUMP": 0, "PARSE_FAILURES": parse_failures, "CFG_INSTRUCTIONS": len(seen)}
    for pc, end, kind, rfs in candidates:
        fixed: list[RelocField] = []
        for rf in rfs:
            if _valid_target(rf.old_target, size, text_intervals) and (rf.old_target in starts or rf.old_target == size):
                fixed.append(rf)
        if fixed:
            spans.append(Span(pc, end, kind, data[pc:end], reloc_fields=fixed))
            stats[kind] = stats.get(kind, 0) + 1
    return spans, stats



def _scan_linear_gap_reloc_spans(data: bytes, text_intervals: list[tuple[int, int]]) -> tuple[list[Span], dict[str, int]]:
    """Parse raw gaps between TEXT/RUBY spans sequentially and relocate target fields.

    The CFG walk alone can miss valid target-bearing commands when an earlier
    unknown/control instruction stops a path.  This happened around normal
    display-control blocks of the form:

        TEXT ... 00
        03/07/0F/... control commands
        0E <argblock> <target>
        00
        13
        TEXT ...

    In v3 those 0E targets stayed as old offsets after whole-script length
    changes, which caused the script to jump to stale addresses and freeze.

    This pass is still safer than v1's raw byte sweep: it only starts at the
    beginning of a gap between already-recognized string units and advances by
    the EXE-derived instruction parser.  It does not resynchronise byte-by-byte
    after a failure, so expression payloads/ruby tail data are not scanned as
    independent code.
    """
    size = len(data)
    inside_text, _containing_end = _build_inside_func(text_intervals)
    text_end_at = {a: b for a, b in text_intervals}
    intervals = sorted(text_intervals)
    spans: list[Span] = []
    stats: dict[str, int] = {"JUMP_IF_FALSE": 0, "CHOICE_BRANCH": 0, "JUMP": 0, "CALL_SAVE_JUMP": 0, "GAP_PARSE_STOPS": 0, "GAP_INSTRUCTIONS": 0}

    pos = 0
    for a, b in intervals + [(size, size)]:
        gap_start, gap_end = pos, a
        pc = gap_start
        while pc < gap_end:
            parsed = _parse_top_instruction(data, pc, text_end_at, inside_text)
            if parsed is None:
                stats["GAP_PARSE_STOPS"] += 1
                break
            end, rfs, kind = parsed
            if end <= pc or end > gap_end:
                stats["GAP_PARSE_STOPS"] += 1
                break
            stats["GAP_INSTRUCTIONS"] += 1
            fixed: list[RelocField] = []
            for rf in rfs:
                if _valid_target(rf.old_target, size, text_intervals):
                    fixed.append(rf)
            if fixed:
                spans.append(Span(pc, end, kind, data[pc:end], reloc_fields=fixed))
                stats[kind] = stats.get(kind, 0) + 1
            pc = end
        pos = b
    return spans, stats

def _collect_true_text_units(data: bytes, encoding: str) -> dict[int, Span]:
    """Collect only extraction-confirmed TEXT/RUBY units.

    v4 used disassembler._scan_primary_units(), which accepts any byte 0x01
    followed by a printable C-string.  That is too broad for AI5WIN: absolute
    targets such as 0x2601 are stored little-endian as ``01 26 00 00`` and can
    be misdetected as a one-byte TEXT string.  Once that happens the surrounding
    0E instruction is split before its target field and the target is not
    relocated.

    The text extractor already applies the ruby-aware/dialogue-aware filters, so
    use its records as the authoritative set of patchable text units.
    """
    spans: dict[int, Span] = {}
    try:
        records = [r.to_json_obj() for r in scan_mes_text(data, "<memory>")]
    except Exception:
        records = []

    for rec in records:
        parts = rec.get("_parts")
        if isinstance(parts, list) and parts:
            for part in parts:
                if not isinstance(part, dict):
                    continue
                try:
                    off = int(part["_offset"])
                    end = int(part["_end"])
                except Exception:
                    continue
                if 0 <= off < end <= len(data) and data[off] == 0x01 and data[end - 1] == 0:
                    spans[off] = Span(off, end, "TEXT", data[off:end], text_bytes=data[off + 1:end - 1], prefix=b"\x01")
        else:
            try:
                off = int(rec["_offset"])
                end = int(rec["_end"])
            except Exception:
                continue
            if 0 <= off < end <= len(data) and data[off] == 0x01 and data[end - 1] == 0:
                spans[off] = Span(off, end, "TEXT", data[off:end], text_bytes=data[off + 1:end - 1], prefix=b"\x01")

        rubies = rec.get("_ruby")
        if isinstance(rubies, list):
            for r in rubies:
                if not isinstance(r, dict):
                    continue
                try:
                    rt_offset = int(r["rt_offset"])
                    rt_end = int(r["rt_end"])
                except Exception:
                    continue
                off = rt_offset - 3 if rt_offset >= 3 and data[rt_offset - 3:rt_offset] == b"\x0B\x15\xFF" else rt_offset
                if 0 <= off < rt_end <= len(data) and data[rt_offset] == 0x01 and data[rt_end - 1] == 0:
                    spans[off] = Span(off, rt_end, "RUBY_TEXT", data[off:rt_end], text_bytes=data[rt_offset + 1:rt_end - 1], prefix=data[off:rt_offset + 1])

    return spans


def _collect_spans(data: bytes, encoding: str) -> tuple[list[Span], dict[str, int]]:
    """Collect true text/ruby spans plus structurally parsed relocation fields.

    v5 fixes a v4 false-positive: a relocation target whose low byte is 0x01
    (for example 0x2601 / bytes 01 26 00 00) was mistaken for a TEXT opcode by
    the broad disassembler scanner.  The fake TEXT interval split the previous
    0E instruction, so the 0E target stayed as an old offset after rebuilding.
    """
    text_spans = _collect_true_text_units(data, encoding)
    text_intervals = sorted((s.old_start, s.old_end) for s in text_spans.values())
    spans: dict[int, Span] = dict(text_spans)

    cfg_spans, cfg_stats = _scan_structured_reloc_spans(data, text_intervals)
    gap_spans, gap_stats = _scan_linear_gap_reloc_spans(data, text_intervals)
    reloc_spans = cfg_spans + gap_spans

    occupied = sorted((s.old_start, s.old_end) for s in spans.values())
    def overlaps_existing(s: Span) -> bool:
        for a, b in occupied:
            if s.old_start < b and a < s.old_end:
                return True
        return False

    seen_reloc_keys: set[tuple[int, int, str]] = set()
    for s in sorted(reloc_spans, key=lambda x: (x.old_start, -(x.old_end - x.old_start))):
        key = (s.old_start, s.old_end, s.kind)
        if key in seen_reloc_keys:
            continue
        seen_reloc_keys.add(key)
        if overlaps_existing(s):
            continue
        spans.setdefault(s.old_start, s)
        occupied.append((s.old_start, s.old_end))
        occupied.sort()

    ordered = sorted(spans.values(), key=lambda s: s.old_start)
    stats = {
        "TEXT": sum(1 for s in ordered if s.kind == "TEXT"),
        "RUBY_TEXT": sum(1 for s in ordered if s.kind == "RUBY_TEXT"),
        "JUMP_IF_FALSE": sum(1 for s in ordered if s.kind == "JUMP_IF_FALSE"),
        "CHOICE_BRANCH": sum(1 for s in ordered if s.kind == "CHOICE_BRANCH"),
        "JUMP": sum(1 for s in ordered if s.kind == "JUMP"),
        "CALL_SAVE_JUMP": sum(1 for s in ordered if s.kind == "CALL_SAVE_JUMP"),
        "CFG_RELOC_TOTAL": sum(cfg_stats.values()),
        "GAP_RELOC_TOTAL": sum(gap_stats.values()),
        "GAP_PARSE_STOPS": gap_stats.get("GAP_PARSE_STOPS", 0),
        "GAP_INSTRUCTIONS": gap_stats.get("GAP_INSTRUCTIONS", 0),
    }
    return ordered, stats

def _plan_segments(data: bytes, spans: list[Span], patches: dict[int, bytes]) -> list[Segment]:
    segs: list[Segment] = []
    old_pos = 0
    new_pos = 0
    for s in spans:
        if s.old_start < old_pos:
            raise ValueError(f"overlapping spans at 0x{s.old_start:X}")
        if old_pos < s.old_start:
            l = s.old_start - old_pos
            segs.append(Segment(old_pos, s.old_start, new_pos, new_pos + l, None))
            new_pos += l
        patch = patches.get(s.old_start)
        nl = s.new_len(patch)
        segs.append(Segment(s.old_start, s.old_end, new_pos, new_pos + nl, s))
        new_pos += nl
        old_pos = s.old_end
    if old_pos < len(data):
        l = len(data) - old_pos
        segs.append(Segment(old_pos, len(data), new_pos, new_pos + l, None))
    return segs


def _map_offset(old: int, segs: list[Segment], old_size: int) -> int:
    if old == old_size:
        return segs[-1].new_end if segs else 0
    # Binary search could be used; list is small enough for current files.
    for seg in segs:
        if seg.old_start <= old < seg.old_end:
            if seg.span is not None and seg.old_start < old < seg.old_end:
                # No confirmed target should point inside a semantic instruction.
                # If it happens, keep the relative displacement within the rebuilt
                # span as a best-effort fallback and report through validation.
                delta = min(old - seg.old_start, max(0, seg.new_end - seg.new_start - 1))
                return seg.new_start + delta
            return seg.new_start + (old - seg.old_start)
        if old == seg.old_end:
            return seg.new_end
    raise KeyError(f"cannot map old offset 0x{old:X}")


def _encode_span(span: Span, patches: dict[int, bytes], segs: list[Segment], old_size: int) -> bytes:
    patch = patches.get(span.old_start)
    if span.kind in ("TEXT", "RUBY_TEXT") and patch is not None:
        return span.prefix + patch + b"\x00"
    if not span.reloc_fields:
        return span.raw
    out = bytearray(span.raw)
    for rf in span.reloc_fields:
        new_target = _map_offset(rf.old_target, segs, old_size)
        rel = rf.field_off - span.old_start
        out[rel:rel + 4] = int(new_target).to_bytes(4, "little")
    return bytes(out)


def rebuild_mes(data: bytes, patches: dict[int, bytes], *, encoding: str) -> tuple[bytes, dict[str, Any]]:
    spans, stats = _collect_spans(data, encoding)

    # Warn about patches that did not land on a scanned TEXT/RUBY_TEXT unit.
    span_by_start = {s.old_start: s for s in spans}
    missing = [off for off in patches if off not in span_by_start or span_by_start[off].kind not in ("TEXT", "RUBY_TEXT")]
    if missing:
        raise ValueError("patch offsets not recognized as TEXT/RUBY_TEXT: " + ", ".join(f"0x{x:X}" for x in missing[:20]))

    segs = _plan_segments(data, spans, patches)
    out = bytearray()
    for seg in segs:
        if seg.span is None:
            out += data[seg.old_start:seg.old_end]
        else:
            out += _encode_span(seg.span, patches, segs, len(data))
    report = {
        "old_size": len(data),
        "new_size": len(out),
        "delta": len(out) - len(data),
        "span_stats": stats,
        "reloc_fields": sum(len(s.reloc_fields) for s in spans),
    }
    return bytes(out), report


def inject_arc_full_rebuild(
    arc_path: Path,
    json_path: Path | None,
    output_path: Path,
    *,
    encoding: str = DEFAULT_ENCODING,
    ruby_policy: str = "keep",
    strict: bool = True,
) -> dict[str, Any]:
    entries = rp_arc.read_arc(arc_path)
    json_map = _load_json_dir(json_path) if json_path else {}
    out_entries: list[tuple[str, bytes]] = []
    reports: dict[str, Any] = {
        "input": str(arc_path),
        "output": str(output_path),
        "encoding": encoding,
        "ruby_policy": ruby_policy,
        "files": {},
        "errors": [],
        "warnings": [],
    }

    for ent in entries:
        mes_key = ent.name.upper()
        file_json = json_map.get(mes_key, [])
        patch_map: dict[int, bytes] = {}
        patch_report: dict[str, Any] = {"json_entries": 0, "text_patches": 0, "part_patches": 0, "ruby_blank_fullwidth": 0, "warnings": [], "errors": []}
        if file_json:
            patch_map, patch_report = _build_text_patch_map(ent.data, file_json, encoding=encoding, ruby_policy=ruby_policy, file_name=ent.name)
        elif ruby_policy not in ("", "keep", "none"):
            # Ruby blanking needs metadata from JSON; no JSON, no blanking.
            pass
        try:
            rebuilt, rebuild_report = rebuild_mes(ent.data, patch_map, encoding=encoding)
        except Exception as ex:
            msg = f"{ent.name}: rebuild failed: {ex}"
            reports["errors"].append(msg)
            if strict:
                raise
            rebuilt = ent.data
            rebuild_report = {"old_size": len(ent.data), "new_size": len(ent.data), "delta": 0, "failed": str(ex)}
        out_entries.append((ent.name, rebuilt))
        reports["files"][ent.name] = {
            **patch_report,
            **rebuild_report,
        }
        reports["warnings"].extend(f"{ent.name}: {w}" for w in patch_report.get("warnings", []))
        reports["errors"].extend(f"{ent.name}: {e}" for e in patch_report.get("errors", []))

    if strict and reports["errors"]:
        raise RuntimeError("errors during full rebuild inject:\n" + "\n".join(reports["errors"][:50]))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    rp_arc.write_arc(output_path, out_entries)
    return reports


def main() -> None:
    ap = argparse.ArgumentParser(description="Full-rebuild MES.ARC injector without EOF trampolines")
    ap.add_argument("input_arc", help="original MES.ARC")
    ap.add_argument("json", nargs="?", help="translation JSON directory or zip; omit with --roundtrip-only")
    ap.add_argument("output_arc", nargs="?", help="output MES.ARC; default <input>.fullrebuild.ARC")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING, help=f"text encoding, default {DEFAULT_ENCODING}")
    ap.add_argument("--ruby-policy", default="keep", choices=["keep", "blank-fullwidth", "blank", "fullwidth-space"], help="ruby reading policy")
    ap.add_argument("--roundtrip-only", action="store_true", help="rebuild without applying JSON; output should be byte-exact at ARC level")
    ap.add_argument("--report", help="write JSON report path")
    ap.add_argument("--non-strict", action="store_true", help="continue on per-file errors")
    args = ap.parse_args()

    inp = Path(args.input_arc)
    if args.roundtrip_only:
        # Drag/drop/CLI convenience: when --roundtrip-only is used, the optional
        # second positional argument is treated as output path.
        json_path = None
        out = Path(args.output_arc or args.json) if (args.output_arc or args.json) else inp.with_name(inp.stem + ".fullrebuild" + inp.suffix)
    else:
        json_path = Path(args.json) if args.json else None
        if json_path is None:
            ap.error("json path is required unless --roundtrip-only is used")
        out = Path(args.output_arc) if args.output_arc else inp.with_name(inp.stem + ".fullrebuild" + inp.suffix)

    report = inject_arc_full_rebuild(inp, json_path, out, encoding=args.encoding, ruby_policy=args.ruby_policy, strict=not args.non_strict)
    if args.report:
        Path(args.report).write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    totals = {
        "files": len(report["files"]),
        "text_patches": sum(v.get("text_patches", 0) for v in report["files"].values()),
        "part_patches": sum(v.get("part_patches", 0) for v in report["files"].values()),
        "ruby_blank_fullwidth": sum(v.get("ruby_blank_fullwidth", 0) for v in report["files"].values()),
        "errors": len(report.get("errors", [])),
        "warnings": len(report.get("warnings", [])),
    }
    print(f"[full-rebuild] input={inp} output={out}")
    print("[full-rebuild] " + " ".join(f"{k}={v}" for k, v in totals.items()))


if __name__ == "__main__":
    main()
