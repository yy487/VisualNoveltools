# -*- coding: utf-8 -*-
"""Noesis .s script text helpers for Love es M.

The script is a mostly opaque VM stream.  For localization we only touch the
confirmed text-bearing instructions:

* text/name string op: 00 04 00 <payload_size:u8> <payload>
* choice op:           1D 08 <text_size:u16le> <target:u32le> <text>

All other bytes are preserved verbatim.
"""
from __future__ import annotations

import codecs
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable

DEFAULT_ENCODING = "cp932"
DEFAULT_OUTPUT_ENCODING = DEFAULT_ENCODING
SUPPORTED_OUTPUT_ENCODINGS = {"cp932", "shift_jis", "sjis", "gbk", "gb18030"}
TEXT_OP_MID = b"\x04\x00"
CHOICE_OP = b"\x1D\x08"
SPEAKER_PREFIXES = ("＃", "#")

# Ruby-like syntax observed in the sample:
#   <鶴<つる><谷<たに>あやか。 -> 鶴谷あやか。
RUBY_RE = re.compile(r"<([^<>]+)<[^<>]*>")


class ScriptFormatError(ValueError):
    pass


@dataclass
class ScriptTextRecord:
    kind: str                 # text / choice / name
    raw_text: str
    clean_text: str
    inst_offset: int
    text_offset: int
    size: int                 # payload size for text op, fixed text size for choice
    visible_size: int         # bytes before the first NUL for text op, size for choice
    opcode: str
    target: int | None = None
    name: str | None = None
    exported_index: int | None = None
    suffix_hex: str | None = None
    ruby_stripped: bool = False

    def to_entry(self, file_name: str, index: int, *, message_uses_clean: bool) -> dict[str, Any]:
        # Keep translator-facing fields first.  JSON object order is preserved
        # by Python's json module.
        obj: dict[str, Any] = {}

        if self.kind == "name":
            # The script stores speaker markers as "＃name"/"#name".
            # Translators should edit only the name body; the injector restores
            # the original speaker prefix automatically.
            prefix = self.raw_text[:1] if self.raw_text.startswith(SPEAKER_PREFIXES) else "＃"
            raw_body = self.raw_text[1:] if self.raw_text.startswith(SPEAKER_PREFIXES) else self.raw_text
            clean_body = self.clean_text[1:] if self.clean_text.startswith(SPEAKER_PREFIXES) else self.clean_text
            obj.update({
                "scr_msg": raw_body,
                "message": clean_body if message_uses_clean else raw_body,
                "_file": file_name,
                "_index": index,
                "_offset": self.text_offset,
                "_inst_offset": self.inst_offset,
                "_size": self.size,
                "_visible_size": self.visible_size,
                "_type": self.kind,
                "_opcode": self.opcode,
                "_encoding": DEFAULT_ENCODING,
                "_output_encoding": DEFAULT_OUTPUT_ENCODING,
                "_policy": "relocate",
                "_speaker_prefix": prefix,
            })
        else:
            if self.name:
                obj["name"] = self.name
            obj.update({
                "scr_msg": self.raw_text,
                "message": self.clean_text if message_uses_clean else self.raw_text,
                "_file": file_name,
                "_index": index,
                "_offset": self.text_offset,
                "_inst_offset": self.inst_offset,
                "_size": self.size,
                "_visible_size": self.visible_size,
                "_type": self.kind,
                "_opcode": self.opcode,
                "_encoding": DEFAULT_ENCODING,
                "_output_encoding": DEFAULT_OUTPUT_ENCODING,
                "_policy": "relocate",
            })
        if self.target is not None:
            obj["_target"] = f"0x{self.target:08X}"
        if self.ruby_stripped:
            obj["_ruby_stripped"] = True
            obj["_ruby_removed_preview"] = self.clean_text
        if self.suffix_hex:
            obj["_suffix_hex"] = self.suffix_hex
        return obj


def strip_ruby(text: str) -> str:
    """Remove Noesis ruby tags while keeping base text."""
    prev = None
    out = text
    # Repeat because tags can be adjacent or nested by syntax shape.
    while prev != out:
        prev = out
        out = RUBY_RE.sub(r"\1", out)
    return out.replace("<", "")


def decode_cp932(raw: bytes, offset: int, *, strict: bool = True) -> str:
    try:
        return raw.decode(DEFAULT_ENCODING)
    except UnicodeDecodeError as exc:
        if strict:
            raise ScriptFormatError(f"cp932 decode failed at 0x{offset:X}: {exc}") from exc
        return raw.decode(DEFAULT_ENCODING, errors="replace")


def normalize_encoding_name(encoding: str | None) -> str:
    if not encoding:
        return DEFAULT_OUTPUT_ENCODING
    enc = str(encoding).strip().lower().replace("-", "_")
    aliases = {
        "932": "cp932",
        "ms932": "cp932",
        "shiftjis": "shift_jis",
        "shift_jisx0213": "shift_jisx0213",
        "sjis": "shift_jis",
        "gb2312": "gbk",
    }
    enc = aliases.get(enc, enc)
    try:
        codecs.lookup(enc)
    except LookupError as exc:
        raise ScriptFormatError(f"unknown output encoding: {encoding!r}") from exc
    return enc


def encode_text(text: str, encoding: str | None = None) -> bytes:
    enc = normalize_encoding_name(encoding)
    return text.encode(enc)


def encode_cp932(text: str) -> bytes:
    return encode_text(text, DEFAULT_ENCODING)


def visible_payload(payload: bytes) -> tuple[bytes, int, bytes]:
    """Return visible bytes, first-NUL position, and suffix from first NUL.

    The observed text op payload often has ignored garbage/padding after the
    first NUL.  It must be preserved for conservative in-place patching.
    """
    nul = payload.find(b"\x00")
    if nul < 0:
        return payload, len(payload), b""
    return payload[:nul], nul, payload[nul:]


def looks_like_story_text(text: str) -> bool:
    if not text:
        return False
    # Keep speaker tags and Japanese/markup text.  Reject pure control-ish ASCII.
    if text.startswith(SPEAKER_PREFIXES):
        return True
    for ch in text:
        o = ord(ch)
        if (
            0x3040 <= o <= 0x30FF or  # kana
            0x3400 <= o <= 0x9FFF or  # CJK
            0xFF00 <= o <= 0xFFEF or  # fullwidth
            ch in "「」『』、。…―！？・<>（）()"
        ):
            return True
    return False


def parse_script_records(data: bytes, *, export_names: bool = False, strict: bool = False) -> list[ScriptTextRecord]:
    records: list[ScriptTextRecord] = []
    pending_name: str | None = None
    i = 0
    n = len(data)

    while i < n:
        # Text/name op: 00 04 00 <payload_size:u8> <payload>
        if i + 4 <= n and data[i + 1:i + 3] == TEXT_OP_MID:
            payload_size = data[i + 3]
            payload_start = i + 4
            payload_end = payload_start + payload_size
            if payload_size > 0 and payload_end <= n:
                payload = data[payload_start:payload_end]
                visible, visible_size, suffix = visible_payload(payload)
                try:
                    raw_text = decode_cp932(visible, payload_start, strict=strict)
                except ScriptFormatError:
                    i += 1
                    continue
                if looks_like_story_text(raw_text):
                    clean = strip_ruby(raw_text)
                    ruby_stripped = clean != raw_text
                    if raw_text.startswith(SPEAKER_PREFIXES):
                        name_text = raw_text[1:]
                        pending_name = strip_ruby(name_text)
                        if export_names:
                            prefix = raw_text[:1]
                            records.append(ScriptTextRecord(
                                kind="name",
                                raw_text=raw_text,
                                clean_text=prefix + pending_name,
                                inst_offset=i,
                                text_offset=payload_start,
                                size=payload_size,
                                visible_size=visible_size,
                                opcode=f"{data[i]:02X}04",
                                name=pending_name,
                                suffix_hex=suffix.hex() if suffix else None,
                                ruby_stripped=ruby_stripped,
                            ))
                        i = payload_end
                        continue
                    kind = "dialogue" if pending_name else "monologue"
                    records.append(ScriptTextRecord(
                        kind=kind,
                        raw_text=raw_text,
                        clean_text=clean,
                        inst_offset=i,
                        text_offset=payload_start,
                        size=payload_size,
                        visible_size=visible_size,
                        opcode=f"{data[i]:02X}04",
                        name=pending_name,
                        suffix_hex=suffix.hex() if suffix else None,
                        ruby_stripped=ruby_stripped,
                    ))
                    pending_name = None
                    i = payload_end
                    continue

        # Choice op: 1D 08 <text_size:u16le> <target:u32le> <text>
        if i + 8 <= n and data[i:i + 2] == CHOICE_OP:
            text_size = int.from_bytes(data[i + 2:i + 4], "little")
            target = int.from_bytes(data[i + 4:i + 8], "little")
            text_start = i + 8
            text_end = text_start + text_size
            if 0 < text_size <= 0x100 and text_end <= n:
                raw = data[text_start:text_end]
                try:
                    text = decode_cp932(raw, text_start, strict=True)
                except ScriptFormatError:
                    i += 1
                    continue
                if looks_like_story_text(text):
                    clean = strip_ruby(text)
                    records.append(ScriptTextRecord(
                        kind="choice",
                        raw_text=text,
                        clean_text=clean,
                        inst_offset=i,
                        text_offset=text_start,
                        size=text_size,
                        visible_size=text_size,
                        opcode="1D08",
                        target=target,
                        ruby_stripped=clean != text,
                    ))
                    i = text_end
                    continue
        i += 1

    return records


def iter_script_files(input_path: Path) -> Iterable[Path]:
    if input_path.is_file():
        yield input_path
    else:
        for p in sorted(input_path.rglob("*.s")):
            if p.is_file():
                yield p


def save_json(path: Path, entries: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def load_json(path: Path) -> list[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be a list: {path}")
    return data


def json_name_for_script(script_rel: str) -> str:
    return script_rel.replace("/", "__").replace("\\", "__") + ".json"


def parse_target(value: Any) -> int | None:
    if value is None:
        return None
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        return int(value, 0)
    return None
