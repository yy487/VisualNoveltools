# -*- coding: utf-8 -*-
"""Text extraction/injection helpers for Refrain Blue MES files.

Important correction:
The MES files inside MES.ARC contain ordinary CP932 zero-terminated strings.
Opcode 0x01 is a text command followed by a CP932 C string.  It is not a
2-byte FONT.TBL code-unit stream.
"""
from __future__ import annotations

from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Iterable
import json
import re

ENCODING = "cp932"
JAPANESE_RE = re.compile(r"[\u3040-\u30ff\u3400-\u9fff\u3000-\u303f]")
SENTENCE_END = set("。！？!?）」』…")

# 0B 15 FF 01 <ruby> 00 appears to be ruby/furigana helper text.
RUBY_PREFIX = b"\x0B\x15\xFF"


@dataclass
class TextRecord:
    scr_msg: str
    message: str
    _file: str
    _index: int
    _offset: int          # offset of opcode 01
    _text_offset: int     # offset of first byte of CP932 string
    _end: int             # offset after zero terminator
    _opcode: str = "01"
    _type: str = "message"
    _raw_hex: str = ""

    def to_json_obj(self) -> dict[str, Any]:
        return asdict(self)


def is_private_use(s: str) -> bool:
    return any(0xE000 <= ord(ch) <= 0xF8FF for ch in s)


def looks_like_translatable_text(s: str) -> bool:
    if not s or len(s.strip()) < 2:
        return False
    if is_private_use(s):
        return False
    if any((ord(ch) < 32 and ch not in "\t\r\n") for ch in s):
        return False
    if not JAPANESE_RE.search(s):
        return False
    # Resource-ish strings are usually ASCII-heavy.  Keep Japanese-only/mostly Japanese text.
    return True


def read_cstring(data: bytes, start: int, max_len: int = 500) -> tuple[bytes, int] | None:
    end_limit = min(len(data), start + max_len)
    end = data.find(b"\x00", start, end_limit)
    if end < 0:
        return None
    return data[start:end], end + 1


def decode_cp932(raw: bytes) -> str | None:
    try:
        s = raw.decode(ENCODING)
    except UnicodeDecodeError:
        return None
    if is_private_use(s):
        return None
    return s


def encode_cp932(s: str) -> bytes:
    return s.encode(ENCODING)


def scan_mes_text(data: bytes, file_name: str, include_ruby: bool = False) -> list[TextRecord]:
    """Heuristic but practical scanner for opcode 01 text commands.

    The real VM is AI5WIN v0-like and contains many expression bytes.  We do not
    treat every byte as an opcode; instead, a candidate is accepted only when
    byte 0x01 is followed by a valid zero-terminated CP932 Japanese string.
    The common ruby helper pattern 0B 15 FF 01 <ruby> 00 is skipped by default.
    """
    out: list[TextRecord] = []
    i = 0
    while i < len(data) - 2:
        if data[i] != 0x01:
            i += 1
            continue

        is_ruby = i >= 3 and data[i - 3:i] == RUBY_PREFIX
        item = read_cstring(data, i + 1)
        if item is None:
            i += 1
            continue
        raw, end = item
        if len(raw) < 2:
            i += 1
            continue
        text = decode_cp932(raw)
        if text is None or not looks_like_translatable_text(text):
            i += 1
            continue
        if is_ruby and not include_ruby:
            i = end
            continue

        rec = TextRecord(
            scr_msg=text,
            message=text,
            _file=file_name,
            _index=len(out),
            _offset=i,
            _text_offset=i + 1,
            _end=end,
            _opcode="01",
            _type="ruby" if is_ruby else "message",
            _raw_hex=data[i:end].hex(" "),
        )
        out.append(rec)
        i = end
    return out


def records_to_json(records: Iterable[TextRecord]) -> list[dict[str, Any]]:
    return [r.to_json_obj() for r in records]


def load_json_entries(path: str | Path) -> list[dict[str, Any]]:
    obj = json.loads(Path(path).read_text(encoding="utf-8"))
    if isinstance(obj, list):
        return obj
    if isinstance(obj, dict):
        for key in ("entries", "texts", "messages"):
            if isinstance(obj.get(key), list):
                return obj[key]
    raise ValueError(f"unsupported JSON format: {path}")


def save_json(path: str | Path, entries: list[dict[str, Any]]) -> None:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(entries, ensure_ascii=False, indent=2), encoding="utf-8")


def validate_entry_at(data: bytes, entry: dict[str, Any]) -> tuple[int, int, bytes]:
    """Return (offset, end, old_raw) after validating scr_msg at original offset."""
    off = int(entry["_offset"])
    end = int(entry["_end"])
    if off < 0 or end > len(data) or off >= end:
        raise ValueError(f"invalid offset/end: off=0x{off:X}, end=0x{end:X}")
    if data[off] != 0x01:
        raise ValueError(f"offset 0x{off:X} is no longer opcode 01; inject from original MES/ARC")
    old_raw = encode_cp932(str(entry["scr_msg"]))
    actual = data[off + 1:end - 1]
    if actual != old_raw:
        try:
            actual_s = actual.decode(ENCODING)
        except Exception:
            actual_s = actual.hex(" ")
        raise ValueError(
            f"scr_msg mismatch at 0x{off:X}: json={entry['scr_msg']!r}, file={actual_s!r}"
        )
    return off, end, old_raw


def patch_mes_non_equal(data: bytes, entries: list[dict[str, Any]], *, force_jump: bool = False) -> tuple[bytes, dict[str, Any]]:
    """Patch MES text using in-place replacement or EOF jump stubs.

    If the new encoded text fits in the original command span, it is written in
    place unless force_jump is set.  Otherwise the original command is replaced
    by `0A append_offset`, and EOF receives `01 new_text 00 0A return_offset`.
    """
    buf = bytearray(data)
    appended = bytearray()
    report: dict[str, Any] = {
        "json_entries": len(entries),
        "patched": 0,
        "skipped_same": 0,
        "inplace": 0,
        "jump": 0,
        "errors": [],
        "warnings": [],
    }

    # Patch in original order.  Offsets are from the original file and remain valid because
    # we never insert in the middle of the existing file.
    for idx, e in enumerate(entries):
        msg = e.get("message", e.get("scr_msg"))
        scr = e.get("scr_msg")
        if not isinstance(scr, str) or not isinstance(msg, str):
            report["warnings"].append(f"entry[{idx}] missing string scr_msg/message; skipped")
            continue
        if msg == scr:
            report["skipped_same"] += 1
            continue
        try:
            off, end, old_raw = validate_entry_at(data, e)
            new_raw = encode_cp932(msg)
        except Exception as ex:
            report["errors"].append(f"entry[{idx}] {ex}")
            continue

        old_span = end - off
        new_command = b"\x01" + new_raw + b"\x00"
        if not force_jump and len(new_command) <= old_span:
            buf[off:off + len(new_command)] = new_command
            if len(new_command) < old_span:
                buf[off + len(new_command):end] = b"\x00" * (old_span - len(new_command))
            report["patched"] += 1
            report["inplace"] += 1
            continue

        if old_span < 5:
            report["errors"].append(
                f"entry[{idx}] original span too small for jump: span={old_span}, off=0x{off:X}"
            )
            continue

        append_off = len(buf) + len(appended)
        return_off = end
        buf[off:off + 5] = b"\x0A" + append_off.to_bytes(4, "little")
        if old_span > 5:
            buf[off + 5:end] = b"\x00" * (old_span - 5)
        appended += new_command + b"\x0A" + return_off.to_bytes(4, "little")
        report["patched"] += 1
        report["jump"] += 1

    if appended:
        buf += appended
    report["old_size"] = len(data)
    report["new_size"] = len(buf)
    report["appended_bytes"] = len(appended)
    return bytes(buf), report
