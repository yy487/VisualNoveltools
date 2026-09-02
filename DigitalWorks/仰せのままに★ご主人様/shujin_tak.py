#!/usr/bin/env python3
"""Structured SHUJIN_TAIKEN TAK text extractor and injector."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import struct
import sys
import tempfile
from array import array
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence


FORMAT_NAME = "SHUJIN_TAIKEN TAK translation"
FORMAT_VERSION = 2
ENCODING = "cp932"
LZS_MAGIC = b"LZS\0"
MAX_SCRIPT_SIZE = 0x1000000
MAX_MESSAGE_BYTES = 318
MAX_CHOICE_BYTES = 38
MAX_NAME_DISPLAY_BYTES = 30
MAX_LZS_MATCH = 18
LZS_WINDOW = 4096
LZS_INITIAL_POSITION = 0xFEE
COLOR_CONTROLS = {b"cw", b"cy", b"cr", b"cg", b"cb"}


class TakError(Exception):
    """Raised for a user-facing format or workflow error."""


@dataclass(frozen=True)
class LzsResult:
    data: bytes
    compressed: bool
    declared_size: int
    consumed_size: int


@dataclass
class ScriptUnit:
    offset: int
    raw: bytes
    opcode: int
    text_bytes: bytes | None = None
    text: str | None = None
    record_id: int | None = None
    padding: int | None = None
    text_index: int | None = None
    entry_index: int | None = None
    kind: str | None = None
    choice_id: int | None = None
    target: int | None = None
    post_target: int | None = None
    name_unit: ScriptUnit | None = None

    @property
    def size(self) -> int:
        return len(self.raw)


@dataclass(frozen=True)
class ParsedScript:
    data: bytes
    units: tuple[ScriptUnit, ...]
    text_units: tuple[ScriptUnit, ...]
    entry_units: tuple[ScriptUnit, ...]
    ac_count: int


@dataclass(frozen=True)
class ScriptInput:
    path: Path
    file_key: str
    stored: bytes
    lzs: LzsResult
    script: ParsedScript


@dataclass(frozen=True)
class Discovery:
    scripts: tuple[ScriptInput, ...]
    scanned_files: int
    warnings: tuple[str, ...]


@dataclass(frozen=True)
class RebuildResult:
    script: bytes
    changed_entries: int
    patched_jumps: int


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _u24(data: bytes) -> int:
    if len(data) != 3:
        raise ValueError("u24 requires exactly three bytes")
    return int.from_bytes(data, "little")


def _pack_u24(value: int) -> bytes:
    if value < 0 or value > 0xFFFFFF:
        raise TakError(f"24-bit offset 0x{value:X} is out of range")
    return value.to_bytes(3, "little")


def decompress_lzs(blob: bytes) -> LzsResult:
    """Decode the game's exact LZS/LZSS wrapper, or return raw data unchanged."""

    if not blob.startswith(b"LZS"):
        if len(blob) > MAX_SCRIPT_SIZE:
            raise TakError("raw script exceeds the 16 MiB 24-bit-offset limit")
        return LzsResult(blob, False, len(blob), len(blob))
    if len(blob) < 8 or blob[:4] != LZS_MAGIC:
        raise TakError("invalid or truncated LZS header")

    declared_size = struct.unpack_from("<I", blob, 4)[0]
    if declared_size > MAX_SCRIPT_SIZE:
        raise TakError(
            f"LZS declares 0x{declared_size:X} bytes, above the 16 MiB "
            "24-bit-offset limit"
        )

    source = 8
    flags = 0
    ring = bytearray(b" " * LZS_WINDOW)
    ring_position = LZS_INITIAL_POSITION
    output = bytearray()

    while len(output) < declared_size:
        flags >>= 1
        if not flags & 0x100:
            if source >= len(blob):
                raise TakError("LZS stream ends before a flag byte")
            flags = 0xFF00 | blob[source]
            source += 1

        if flags & 1:
            if source >= len(blob):
                raise TakError("LZS stream ends before a literal byte")
            value = blob[source]
            source += 1
            output.append(value)
            ring[ring_position] = value
            ring_position = (ring_position + 1) & 0xFFF
        else:
            if source + 1 >= len(blob):
                raise TakError("LZS stream ends inside a back-reference")
            first = blob[source]
            second = blob[source + 1]
            source += 2
            reference = ((second & 0xF0) << 4) | first
            count = (second & 0x0F) + 3
            for _ in range(count):
                if len(output) == declared_size:
                    break
                value = ring[reference]
                output.append(value)
                ring[ring_position] = value
                reference = (reference + 1) & 0xFFF
                ring_position = (ring_position + 1) & 0xFFF

    return LzsResult(bytes(output), True, declared_size, source)


def compress_lzs(data: bytes) -> bytes:
    """Greedily encode a valid LZS stream for the runtime's LZSS decoder."""

    if len(data) > MAX_SCRIPT_SIZE:
        raise TakError("script exceeds the 16 MiB 24-bit-offset limit")

    size = len(data)
    previous = array("i", [-1]) * size
    heads: dict[bytes, int] = {}
    body = bytearray()
    position = 0

    def add_position(index: int) -> None:
        if index + 3 > size:
            return
        key = data[index : index + 3]
        previous[index] = heads.get(key, -1)
        heads[key] = index

    while position < size:
        flag = 0
        payload = bytearray()
        for bit in range(8):
            if position >= size:
                break

            best_start = -1
            best_length = 0
            if position + 3 <= size:
                key = data[position : position + 3]
                candidate = heads.get(key, -1)
                checked = 0
                limit = min(MAX_LZS_MATCH, size - position)
                while (
                    candidate >= 0
                    and position - candidate <= LZS_WINDOW
                    and checked < 64
                ):
                    length = 3
                    while (
                        length < limit
                        and data[candidate + length] == data[position + length]
                    ):
                        length += 1
                    if length > best_length:
                        best_start = candidate
                        best_length = length
                        if length == limit:
                            break
                    candidate = previous[candidate]
                    checked += 1

            if best_length >= 3:
                reference = (LZS_INITIAL_POSITION + best_start) & 0xFFF
                payload.append(reference & 0xFF)
                payload.append(((reference >> 4) & 0xF0) | (best_length - 3))
                for index in range(position, position + best_length):
                    add_position(index)
                position += best_length
            else:
                flag |= 1 << bit
                payload.append(data[position])
                add_position(position)
                position += 1

        body.append(flag)
        body.extend(payload)

    packed = LZS_MAGIC + struct.pack("<I", size) + bytes(body)
    check = decompress_lzs(packed)
    if check.data != data or check.consumed_size != len(packed):
        raise TakError("internal error: generated LZS stream failed verification")
    return packed


def _decode_cp932_exact(data: bytes, label: str) -> str:
    try:
        text = data.decode(ENCODING, "strict")
    except UnicodeDecodeError as error:
        raise TakError(
            f"{label}: invalid CP932 bytes at +0x{error.start:X}"
        ) from error
    if text.encode(ENCODING, "strict") != data:
        raise TakError(f"{label}: CP932 decode/re-encode is not byte-exact")
    return text


def _unencodable_description(text: str, error: UnicodeEncodeError) -> str:
    offending = []
    for char in text[error.start : error.end]:
        item = f"{char!r} (U+{ord(char):04X})"
        if item not in offending:
            offending.append(item)
    return ", ".join(offending) or "unknown character"


def _encode_cp932(text: str, label: str) -> bytes:
    try:
        return text.encode(ENCODING, "strict")
    except UnicodeEncodeError as error:
        detail = _unencodable_description(text, error)
        raise TakError(f"{label}: CP932 cannot encode {detail}") from error


def _validate_text_bytes(
    encoded: bytes,
    label: str,
    *,
    allow_controls: bool,
) -> None:
    if len(encoded) % 2:
        raise TakError(f"{label}: encoded length must be even, got {len(encoded)}")

    for offset in range(0, len(encoded), 2):
        pair = encoded[offset : offset + 2]
        if allow_controls and (pair == b"||" or pair in COLOR_CONTROLS):
            continue
        lead = pair[0]
        if not (0x81 <= lead <= 0x9F or 0xE0 <= lead <= 0xFC):
            printable = pair.decode("ascii", "backslashreplace")
            raise TakError(
                f"{label}: unsupported single-byte pair {printable!r} "
                f"at encoded byte +0x{offset:X}"
            )
        try:
            decoded = pair.decode(ENCODING, "strict")
        except UnicodeDecodeError as error:
            raise TakError(
                f"{label}: invalid double-byte CP932 sequence {pair.hex(' ')} "
                f"at +0x{offset:X}"
            ) from error
        if len(decoded) != 1 or decoded.encode(ENCODING, "strict") != pair:
            raise TakError(
                f"{label}: non-canonical CP932 sequence {pair.hex(' ')} "
                f"at +0x{offset:X}"
            )


def encode_editable_text(text: str, kind: str, label: str) -> bytes:
    if not isinstance(text, str):
        raise TakError(f"{label}: editable value must be a string")
    if "\0" in text:
        raise TakError(f"{label}: NUL is not allowed")
    if "\r" in text or "\n" in text:
        raise TakError(f"{label}: use || for a forced line break, not CR/LF")

    allow_controls = kind in {"message", "choice"}
    if kind == "name":
        if not text:
            raise TakError(f"{label}: displayed name cannot be empty")
        if "（" in text or "）" in text:
            raise TakError(
                f"{label}: full-width parentheses are reserved for the name alias wrapper"
            )
    elif kind == "choice" and not text:
        raise TakError(f"{label}: choice text cannot be empty")

    encoded = _encode_cp932(text, label)
    _validate_text_bytes(encoded, label, allow_controls=allow_controls)

    maximum = {
        "message": MAX_MESSAGE_BYTES,
        "choice": MAX_CHOICE_BYTES,
        "name": MAX_NAME_DISPLAY_BYTES,
    }[kind]
    if len(encoded) > maximum:
        raise TakError(
            f"{label}: {len(encoded)} encoded bytes exceed the {maximum}-byte "
            f"{kind} limit"
        )
    return encoded


def _parse_text_unit(data: bytes, offset: int, opcode: int, label: str) -> ScriptUnit:
    if offset + 4 > len(data):
        raise TakError(f"{label}: truncated {opcode:02X} header at 0x{offset:X}")
    if data[offset + 1] != 0:
        raise TakError(
            f"{label}: unsupported nonzero {opcode:02X} header byte at 0x{offset + 1:X}"
        )

    terminator = 0xA9 if opcode == 0xA8 else 0xAB
    cursor = offset + 4
    while True:
        if cursor + 2 > len(data):
            raise TakError(f"{label}: unterminated {opcode:02X} record at 0x{offset:X}")
        first = data[cursor]
        if first == terminator:
            end = cursor + 4
            if end > len(data) or data[cursor:end] != bytes([terminator, 0, 0, 0]):
                raise TakError(
                    f"{label}: malformed {terminator:02X} terminator at 0x{cursor:X}"
                )
            padding = 0
            break
        if first == 0:
            end = cursor + 6
            expected = b"\0\0" + bytes([terminator, 0, 0, 0])
            if end > len(data) or data[cursor:end] != expected:
                raise TakError(
                    f"{label}: malformed padded terminator at 0x{cursor:X}"
                )
            padding = 2
            break
        cursor += 2

    text_bytes = data[offset + 4 : cursor]
    text = _decode_cp932_exact(text_bytes, f"{label} record 0x{offset:X}")
    _validate_text_bytes(
        text_bytes,
        f"{label} record 0x{offset:X}",
        allow_controls=opcode == 0xAA,
    )
    expected_padding = 2 if len(text_bytes) % 4 == 2 else 0
    if padding != expected_padding:
        raise TakError(
            f"{label}: unexpected alignment padding in record 0x{offset:X}"
        )

    return ScriptUnit(
        offset=offset,
        raw=data[offset:end],
        opcode=opcode,
        text_bytes=text_bytes,
        text=text,
        record_id=struct.unpack_from("<H", data, offset + 2)[0],
        padding=padding,
        kind="name" if opcode == 0xA8 else "message",
    )


def parse_script(data: bytes, label: str = "script") -> ParsedScript:
    if not data:
        raise TakError(f"{label}: empty data is not a TAK script")
    if len(data) > MAX_SCRIPT_SIZE:
        raise TakError(f"{label}: script exceeds the 16 MiB 24-bit-offset limit")

    units: list[ScriptUnit] = []
    offset = 0
    while offset < len(data):
        if offset + 4 > len(data):
            raise TakError(f"{label}: trailing partial instruction at 0x{offset:X}")
        opcode = data[offset]
        if opcode in {0xA8, 0xAA}:
            unit = _parse_text_unit(data, offset, opcode, label)
        else:
            unit = ScriptUnit(offset, data[offset : offset + 4], opcode)
        units.append(unit)
        offset += unit.size

    choice_units: set[int] = set()
    for index, unit in enumerate(units):
        if unit.opcode != 0xA0 or unit.raw[1] != 0x03:
            continue
        if index + 2 >= len(units):
            raise TakError(f"{label}: truncated choice command at 0x{unit.offset:X}")
        argument = units[index + 1]
        text_unit = units[index + 2]
        if argument.opcode != 0xAC or text_unit.opcode != 0xAA:
            raise TakError(
                f"{label}: unsupported choice argument structure at 0x{unit.offset:X}"
            )
        if index + 2 in choice_units:
            raise TakError(f"{label}: overlapping choice at 0x{unit.offset:X}")
        choice_units.add(index + 2)
        text_unit.kind = "choice"
        text_unit.choice_id = struct.unpack_from("<H", unit.raw, 2)[0]
        text_unit.target = _u24(argument.raw[1:4])
        if index + 3 < len(units) and units[index + 3].opcode == 0xAC:
            text_unit.post_target = _u24(units[index + 3].raw[1:4])

    boundaries = {unit.offset for unit in units}
    boundaries.add(len(data))
    ac_count = 0
    for unit in units:
        if unit.opcode != 0xAC:
            continue
        ac_count += 1
        target = _u24(unit.raw[1:4])
        if target not in boundaries:
            raise TakError(
                f"{label}: AC at 0x{unit.offset:X} targets non-boundary 0x{target:X}"
            )

    text_units: list[ScriptUnit] = []
    for unit in units:
        if unit.text is None:
            continue
        unit.text_index = len(text_units)
        text_units.append(unit)

    for index, unit in enumerate(units):
        if unit.kind != "name":
            continue
        if index + 1 >= len(units) or units[index + 1].kind != "message":
            raise TakError(
                f"{label}: A8 name at 0x{unit.offset:X} is not immediately "
                "followed by an ordinary AA message"
            )
        message = units[index + 1]
        if message.name_unit is not None:
            raise TakError(f"{label}: duplicate name for AA at 0x{message.offset:X}")
        message.name_unit = unit

    entry_units: list[ScriptUnit] = []
    for unit in units:
        if unit.kind not in {"message", "choice"}:
            continue
        unit.entry_index = len(entry_units)
        if unit.name_unit is not None:
            unit.name_unit.entry_index = unit.entry_index
        entry_units.append(unit)

    return ParsedScript(
        data, tuple(units), tuple(text_units), tuple(entry_units), ac_count
    )


def _name_parts(unit: ScriptUnit) -> tuple[bytes, str, bool]:
    assert unit.text_bytes is not None and unit.text is not None
    opening = b"\x81\x69"
    closing = b"\x81\x6A"
    marker = unit.text_bytes.find(opening)
    if marker >= 0 and unit.text_bytes.endswith(closing):
        alias_bytes = unit.text_bytes[marker + 2 : -2]
        display = _decode_cp932_exact(alias_bytes, f"name at 0x{unit.offset:X}")
        return unit.text_bytes[:marker], display, True
    return unit.text_bytes, unit.text, False


def _entry_type(unit: ScriptUnit) -> str:
    if unit.kind == "choice":
        return "choice"
    return "dialogue" if unit.name_unit is not None else "message"


def make_translation_document(script_input: ScriptInput) -> dict[str, Any]:
    entries: list[dict[str, Any]] = []
    for unit in script_input.script.entry_units:
        assert unit.entry_index is not None
        assert unit.text is not None
        assert unit.record_id is not None
        entry: dict[str, Any] = {
            "_file": script_input.file_key,
            "_index": unit.entry_index,
            "_offset": unit.offset,
            "_size": unit.size,
            "_type": _entry_type(unit),
            "_opcode": f"{unit.opcode:02X}",
            "_id": unit.record_id,
            "_padding": unit.padding,
        }
        if unit.name_unit is not None:
            name_unit = unit.name_unit
            assert name_unit.text is not None
            assert name_unit.record_id is not None
            canonical, display, _ = _name_parts(name_unit)
            entry["_name_offset"] = name_unit.offset
            entry["_name_size"] = name_unit.size
            entry["_name_opcode"] = f"{name_unit.opcode:02X}"
            entry["_name_id"] = name_unit.record_id
            entry["_name_padding"] = name_unit.padding
            entry["_name_writable"] = b"\x81" not in canonical[2:]
            entry["_scr_name"] = name_unit.text
            entry["name"] = display
        if unit.kind == "choice":
            entry["_choice_id"] = unit.choice_id
            entry["_target"] = unit.target
            if unit.post_target is not None:
                entry["_post_target"] = unit.post_target
        entry["scr_msg"] = unit.text
        entry["message"] = unit.text
        entries.append(entry)

    return {
        "format": FORMAT_NAME,
        "format_version": FORMAT_VERSION,
        "_file": script_input.file_key,
        "_encoding": ENCODING,
        "_compression": "LZS" if script_input.lzs.compressed else "raw",
        "_stored_sha256": _sha256(script_input.stored),
        "_script_sha256": _sha256(script_input.script.data),
        "_script_size": len(script_input.script.data),
        "entries": entries,
    }


def _require_int(entry: dict[str, Any], key: str, label: str) -> int:
    value = entry.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise TakError(f"{label}: {key} must be an integer")
    return value


def _require_string(entry: dict[str, Any], key: str, label: str) -> str:
    value = entry.get(key)
    if not isinstance(value, str):
        raise TakError(f"{label}: {key} must be a string")
    return value


def _metadata_matches(actual: Any, expected: Any) -> bool:
    return type(actual) is type(expected) and actual == expected


def load_translation_document(path: Path) -> dict[str, Any]:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise TakError(f"cannot read translation JSON {path}: {error}") from error
    try:
        text = raw.decode("utf-8", "strict")
    except UnicodeDecodeError as error:
        raise TakError(f"translation JSON is not strict UTF-8: {path}") from error
    try:
        document = json.loads(text)
    except json.JSONDecodeError as error:
        raise TakError(
            f"invalid JSON in {path} at line {error.lineno}, column {error.colno}: "
            f"{error.msg}"
        ) from error
    if not isinstance(document, dict):
        raise TakError(f"translation JSON root must be an object: {path}")
    return document


def _validate_document_header(
    document: dict[str, Any], script_input: ScriptInput
) -> list[dict[str, Any]]:
    if document.get("format") != FORMAT_NAME:
        raise TakError(f"{script_input.file_key}: unsupported translation format")
    if not _metadata_matches(document.get("format_version"), FORMAT_VERSION):
        raise TakError(f"{script_input.file_key}: unsupported format_version")
    if document.get("_encoding") != ENCODING:
        raise TakError(f"{script_input.file_key}: _encoding must be {ENCODING}")
    if document.get("_file") != script_input.file_key:
        raise TakError(
            f"{script_input.file_key}: JSON _file does not match the source path"
        )
    if document.get("_script_sha256") != _sha256(script_input.script.data):
        raise TakError(
            f"{script_input.file_key}: source script hash does not match the JSON"
        )
    if not _metadata_matches(
        document.get("_script_size"), len(script_input.script.data)
    ):
        raise TakError(f"{script_input.file_key}: source script size does not match")
    raw_entries = document.get("entries")
    if not isinstance(raw_entries, list):
        raise TakError(f"{script_input.file_key}: entries must be an array")
    entries: list[dict[str, Any]] = []
    for position, raw_entry in enumerate(raw_entries):
        if not isinstance(raw_entry, dict):
            raise TakError(
                f"{script_input.file_key}: entries[{position}] must be an object"
            )
        entries.append(raw_entry)
    return entries


def _build_name_bytes(unit: ScriptUnit, translated_name: str, label: str) -> bytes:
    assert unit.text_bytes is not None
    canonical, original_display, had_alias = _name_parts(unit)
    if translated_name == original_display:
        return unit.text_bytes

    encoded_name = encode_editable_text(translated_name, "name", label)
    if b"\x81" in canonical[2:]:
        raise TakError(
            f"{label}: this canonical name contains byte 81 after its first glyph; "
            "the runtime alias parser cannot translate it safely"
        )
    if not canonical:
        raise TakError(f"{label}: canonical name prefix is empty")

    rebuilt = canonical + b"\x81\x69" + encoded_name + b"\x81\x6A"
    _decode_cp932_exact(rebuilt, label)
    _validate_text_bytes(rebuilt, label, allow_controls=False)
    if had_alias and rebuilt == unit.text_bytes:
        return unit.text_bytes
    return rebuilt


def _build_text_record(unit: ScriptUnit, replacement: bytes) -> bytes:
    assert unit.opcode in {0xA8, 0xAA}
    assert unit.text_bytes is not None
    if replacement == unit.text_bytes:
        return unit.raw
    padding = b"\0\0" if len(replacement) % 4 == 2 else b""
    terminator = 0xA9 if unit.opcode == 0xA8 else 0xAB
    return unit.raw[:4] + replacement + padding + bytes([terminator, 0, 0, 0])


def rebuild_script(
    script_input: ScriptInput, document: dict[str, Any]
) -> RebuildResult:
    entries = _validate_document_header(document, script_input)
    expected_count = len(script_input.script.entry_units)
    if len(entries) != expected_count:
        raise TakError(
            f"{script_input.file_key}: JSON has {len(entries)} entries; "
            f"expected {expected_count}"
        )

    by_index: dict[int, dict[str, Any]] = {}
    for position, entry in enumerate(entries):
        label = f"{script_input.file_key} entries[{position}]"
        index = _require_int(entry, "_index", label)
        if index in by_index:
            raise TakError(f"{label}: duplicate _index {index}")
        by_index[index] = entry

    replacement_records: dict[int, bytes] = {}
    changed_entries = 0
    for unit in script_input.script.entry_units:
        assert unit.entry_index is not None
        assert unit.text is not None
        assert unit.record_id is not None
        index = unit.entry_index
        entry = by_index.get(index)
        label = f"{script_input.file_key} entry {index}"
        if entry is None:
            raise TakError(f"{label}: missing entry")

        comparisons: dict[str, Any] = {
            "_file": script_input.file_key,
            "_offset": unit.offset,
            "_size": unit.size,
            "_type": _entry_type(unit),
            "_opcode": f"{unit.opcode:02X}",
            "_id": unit.record_id,
            "_padding": unit.padding,
        }
        for key, expected in comparisons.items():
            if not _metadata_matches(entry.get(key), expected):
                raise TakError(
                    f"{label}: {key} is {entry.get(key)!r}; expected {expected!r}"
                )
        if _require_string(entry, "scr_msg", label) != unit.text:
            raise TakError(f"{label}: scr_msg was changed or does not match the source")

        translated = _require_string(entry, "message", label)
        replacement = encode_editable_text(
            translated, unit.kind or "message", label
        )
        rebuilt_record = _build_text_record(unit, replacement)
        replacement_records[unit.offset] = rebuilt_record
        entry_changed = rebuilt_record != unit.raw

        if unit.name_unit is not None:
            name_unit = unit.name_unit
            assert name_unit.text is not None
            assert name_unit.record_id is not None
            name_comparisons: dict[str, Any] = {
                "_name_offset": name_unit.offset,
                "_name_size": name_unit.size,
                "_name_opcode": f"{name_unit.opcode:02X}",
                "_name_id": name_unit.record_id,
                "_name_padding": name_unit.padding,
            }
            for key, expected in name_comparisons.items():
                if not _metadata_matches(entry.get(key), expected):
                    raise TakError(f"{label}: {key} does not match the source")
            if entry.get("_scr_name") != name_unit.text:
                raise TakError(f"{label}: _scr_name was changed")
            canonical, _, _ = _name_parts(name_unit)
            name_writable = b"\x81" not in canonical[2:]
            if not _metadata_matches(entry.get("_name_writable"), name_writable):
                raise TakError(f"{label}: _name_writable does not match the source")
            translated_name = _require_string(entry, "name", label)
            name_replacement = _build_name_bytes(name_unit, translated_name, label)
            rebuilt_name = _build_text_record(name_unit, name_replacement)
            replacement_records[name_unit.offset] = rebuilt_name
            entry_changed = entry_changed or rebuilt_name != name_unit.raw
        else:
            name_keys = {
                "name",
                "_scr_name",
                "_name_writable",
                "_name_offset",
                "_name_size",
                "_name_opcode",
                "_name_id",
                "_name_padding",
            }
            if any(key in entry for key in name_keys):
                raise TakError(f"{label}: message/choice entry contains name fields")

        if unit.kind == "choice":
            choice_comparisons = {
                "_choice_id": unit.choice_id,
                "_target": unit.target,
            }
            if unit.post_target is not None:
                choice_comparisons["_post_target"] = unit.post_target
            elif "_post_target" in entry:
                raise TakError(f"{label}: unexpected _post_target")
            for key, expected in choice_comparisons.items():
                if not _metadata_matches(entry.get(key), expected):
                    raise TakError(f"{label}: {key} does not match the source")

        if entry_changed:
            changed_entries += 1

    parts: list[bytes] = []
    old_to_new: dict[int, int] = {}
    new_offset = 0
    for unit in script_input.script.units:
        old_to_new[unit.offset] = new_offset
        part = replacement_records.get(unit.offset, unit.raw)
        parts.append(part)
        new_offset += len(part)
    old_to_new[len(script_input.script.data)] = new_offset

    patched_jumps = 0
    for index, unit in enumerate(script_input.script.units):
        if unit.opcode != 0xAC:
            continue
        old_target = _u24(unit.raw[1:4])
        if old_target not in old_to_new:
            raise TakError(
                f"{script_input.file_key}: cannot relocate AC target 0x{old_target:X}"
            )
        new_target = old_to_new[old_target]
        if new_target > 0xFFFFFF:
            raise TakError(
                f"{script_input.file_key}: relocated AC target 0x{new_target:X} "
                "exceeds 24 bits"
            )
        if new_target != old_target:
            patched_jumps += 1
        parts[index] = bytes([0xAC]) + _pack_u24(new_target)

    rebuilt = b"".join(parts)
    reparsed = parse_script(rebuilt, f"rebuilt {script_input.file_key}")
    if len(reparsed.entry_units) != expected_count:
        raise TakError(f"{script_input.file_key}: rebuilt JSON entry count changed")

    for original, current in zip(
        script_input.script.entry_units, reparsed.entry_units
    ):
        assert original.entry_index is not None
        entry = by_index[original.entry_index]
        if current.text != entry["message"]:
            raise TakError(
                f"{script_input.file_key}: rebuilt text {original.entry_index} "
                "does not re-extract to the requested value"
            )
        if original.name_unit is not None:
            if current.name_unit is None:
                raise TakError(
                    f"{script_input.file_key}: rebuilt dialogue {original.entry_index} "
                    "lost its name record"
                )
            _, display, _ = _name_parts(current.name_unit)
            if display != entry["name"]:
                raise TakError(
                    f"{script_input.file_key}: rebuilt name {original.entry_index} "
                    "does not re-extract to the requested value"
                )
        elif current.name_unit is not None:
            raise TakError(
                f"{script_input.file_key}: rebuilt entry {original.entry_index} "
                "unexpectedly gained a name record"
            )

    return RebuildResult(rebuilt, changed_entries, patched_jumps)


def _read_script(path: Path, file_key: str) -> ScriptInput:
    try:
        stored = path.read_bytes()
    except OSError as error:
        raise TakError(f"cannot read {path}: {error}") from error
    lzs = decompress_lzs(stored)
    if lzs.compressed and lzs.consumed_size != len(stored):
        raise TakError(
            f"{file_key}: LZS stream consumed {lzs.consumed_size} of {len(stored)} bytes"
        )
    script = parse_script(lzs.data, file_key)
    return ScriptInput(path, file_key, stored, lzs, script)


def discover_scripts(source: Path) -> Discovery:
    try:
        resolved = source.resolve(strict=True)
    except FileNotFoundError as error:
        raise TakError(f"source does not exist: {source}") from error

    if resolved.is_file():
        script = _read_script(resolved, resolved.name)
        return Discovery((script,), 1, ())
    if not resolved.is_dir():
        raise TakError(f"source is neither a file nor directory: {source}")

    files = sorted((path for path in resolved.rglob("*") if path.is_file()))
    scripts: list[ScriptInput] = []
    warnings: list[str] = []
    for path in files:
        relative = path.relative_to(resolved).as_posix()
        try:
            stored = path.read_bytes()
        except OSError as error:
            raise TakError(f"cannot read {path}: {error}") from error
        looks_structured = stored.startswith(b"LZS") or bool(stored) and stored[0] in {
            0xA0,
            0xA8,
            0xAA,
        }
        try:
            scripts.append(_read_script(path, relative))
        except TakError as error:
            if looks_structured:
                raise
            warnings.append(f"skipped non-TAK file {relative}: {error}")

    if not scripts:
        raise TakError(f"no structured TAK scripts found in {source}")
    return Discovery(tuple(scripts), len(files), tuple(warnings))


def _json_bytes(document: dict[str, Any]) -> bytes:
    return (json.dumps(document, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def _ensure_distinct(source: Path, output: Path) -> None:
    source_resolved = source.resolve(strict=True)
    output_resolved = output.resolve(strict=False)
    if os.path.normcase(str(source_resolved)) == os.path.normcase(str(output_resolved)):
        raise TakError(f"output must not overwrite the source: {source}")


def _ensure_separate_output_directory(source: Path, output: Path) -> Path:
    source_resolved = source.resolve(strict=True)
    output_resolved = output.resolve(strict=False)
    source_key = os.path.normcase(str(source_resolved))
    output_key = os.path.normcase(str(output_resolved))
    parent_keys = {os.path.normcase(str(parent)) for parent in output_resolved.parents}
    if output_key == source_key or source_key in parent_keys:
        raise TakError("output directory must be outside the source directory")
    if output_resolved.exists() and not output_resolved.is_dir():
        raise TakError(f"output directory path is a file: {output}")
    return output_resolved


def _prepare_targets(targets: Sequence[tuple[Path, bytes]], overwrite: bool) -> None:
    seen: set[str] = set()
    for path, _ in targets:
        key = os.path.normcase(str(path.resolve(strict=False)))
        if key in seen:
            raise TakError(f"duplicate output path: {path}")
        seen.add(key)
        if path.exists() and not overwrite:
            raise TakError(f"output already exists; use --overwrite: {path}")
        if path.exists() and not path.is_file():
            raise TakError(f"output path is not a regular file: {path}")


def _write_atomic(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = tempfile.NamedTemporaryFile(
        mode="wb", prefix=f".{path.name}.", suffix=".tmp", dir=path.parent, delete=False
    )
    temporary = Path(handle.name)
    try:
        with handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            temporary.unlink(missing_ok=True)
        finally:
            raise


def _write_targets(targets: Sequence[tuple[Path, bytes]]) -> None:
    for path, data in targets:
        _write_atomic(path, data)


def extract_command(source: Path, output: Path, overwrite: bool) -> None:
    discovery = discover_scripts(source)
    source_is_file = source.resolve(strict=True).is_file()
    targets: list[tuple[Path, bytes]] = []
    entry_count = 0

    if source_is_file:
        _ensure_distinct(source, output)
        only = discovery.scripts[0]
        document = make_translation_document(only)
        targets.append((output, _json_bytes(document)))
        entry_count = len(only.script.entry_units)
    else:
        output_root = _ensure_separate_output_directory(source, output)
        for script_input in discovery.scripts:
            target = output_root / f"{script_input.file_key}.json"
            document = make_translation_document(script_input)
            targets.append((target, _json_bytes(document)))
            entry_count += len(script_input.script.entry_units)

    _prepare_targets(targets, overwrite)
    _write_targets(targets)
    for warning in discovery.warnings:
        print(f"Warning: {warning}", file=sys.stderr)
    print(
        f"Scanned {discovery.scanned_files} files; wrote {len(targets)} JSON files; "
        f"extracted {entry_count} entries; warnings {len(discovery.warnings)}; errors 0"
    )


def _translation_path(
    source_is_file: bool, translations: Path, file_key: str
) -> Path:
    if source_is_file:
        if translations.is_dir():
            return translations / f"{file_key}.json"
        return translations
    if not translations.is_dir():
        raise TakError("directory source requires a translation directory")
    return translations / f"{file_key}.json"


def inject_command(
    source: Path,
    translations: Path,
    output: Path,
    overwrite: bool,
) -> None:
    discovery = discover_scripts(source)
    source_resolved = source.resolve(strict=True)
    source_is_file = source_resolved.is_file()
    if source_is_file:
        _ensure_distinct(source, output)
    else:
        _ensure_separate_output_directory(source, output)

    targets: list[tuple[Path, bytes]] = []
    changed_entries = 0
    patched_jumps = 0
    total_script_size = 0
    total_packed_size = 0
    for script_input in discovery.scripts:
        json_path = _translation_path(source_is_file, translations, script_input.file_key)
        document = load_translation_document(json_path)
        rebuilt = rebuild_script(script_input, document)
        if rebuilt.script == script_input.script.data and script_input.lzs.compressed:
            packed = script_input.stored
        else:
            packed = compress_lzs(rebuilt.script)
        check = decompress_lzs(packed)
        if (
            not check.compressed
            or check.data != rebuilt.script
            or check.consumed_size != len(packed)
        ):
            raise TakError(
                f"{script_input.file_key}: final LZS verification failed before writing"
            )

        target = output if source_is_file else output / Path(script_input.file_key)
        targets.append((target, packed))
        changed_entries += rebuilt.changed_entries
        patched_jumps += rebuilt.patched_jumps
        total_script_size += len(rebuilt.script)
        total_packed_size += len(packed)

    _prepare_targets(targets, overwrite)
    _write_targets(targets)
    for warning in discovery.warnings:
        print(f"Warning: {warning}", file=sys.stderr)
    print(
        f"Rebuilt {len(targets)} LZS TAK files; changed entries {changed_entries}; "
        f"patched AC targets {patched_jumps}; script bytes {total_script_size}; "
        f"stored bytes {total_packed_size}; warnings {len(discovery.warnings)}; errors 0"
    )


def verify_command(source: Path) -> None:
    discovery = discover_scripts(source)
    dialogues = messages = choices = ac_count = 0
    script_bytes = packed_bytes = 0
    for script_input in discovery.scripts:
        for unit in script_input.script.entry_units:
            if unit.kind == "choice":
                choices += 1
            elif unit.name_unit is not None:
                dialogues += 1
            else:
                messages += 1
        ac_count += script_input.script.ac_count
        script_bytes += len(script_input.script.data)
        packed = compress_lzs(script_input.script.data)
        packed_bytes += len(packed)
        check = decompress_lzs(packed)
        if check.data != script_input.script.data or check.consumed_size != len(packed):
            raise TakError(f"{script_input.file_key}: compression verification failed")

    for warning in discovery.warnings:
        print(f"Warning: {warning}", file=sys.stderr)
    print(
        f"Verified {len(discovery.scripts)} TAK files: dialogues {dialogues}, "
        f"narration/messages {messages}, choices {choices}, AC targets {ac_count}, "
        f"script bytes {script_bytes}, verified LZS bytes {packed_bytes}, "
        f"warnings {len(discovery.warnings)}, errors 0"
    )


def _prompt(label: str, default: str = "") -> str:
    suffix = f" [{default}]" if default else ""
    value = input(f"{label}{suffix}: ").strip()
    return value or default


def _prompt_path(label: str, default: str = "") -> Path:
    value = _prompt(label, default).strip().strip('"')
    if not value:
        raise TakError(f"{label} is required")
    return Path(value)


def interactive(prefill: str | None = None) -> int:
    source_default = prefill or ""
    while True:
        print("\nSHUJIN_TAIKEN TAK 文本工具")
        print("1. 提取 UTF-8 JSON")
        print("2. 注入 JSON 并生成 LZS TAK")
        print("3. 只读验证")
        print("4. 退出")
        try:
            choice = _prompt("选择", "4")
        except EOFError:
            return 0
        if choice == "4":
            return 0
        try:
            if choice == "1":
                source = _prompt_path("源 TAK 或目录", source_default)
                output = _prompt_path("JSON 输出文件或目录")
                overwrite = _prompt("允许覆盖已有输出? (y/N)", "N").lower() in {
                    "y",
                    "yes",
                }
                if _prompt("确认开始写入? (y/N)", "N").lower() not in {"y", "yes"}:
                    print("已取消。")
                    continue
                extract_command(source, output, overwrite)
            elif choice == "2":
                source = _prompt_path("原始 TAK 或目录", source_default)
                translations = _prompt_path("翻译 JSON 或目录")
                output = _prompt_path("LZS TAK 输出文件或目录")
                overwrite = _prompt("允许覆盖已有输出? (y/N)", "N").lower() in {
                    "y",
                    "yes",
                }
                if _prompt("确认开始写入? (y/N)", "N").lower() not in {"y", "yes"}:
                    print("已取消。")
                    continue
                inject_command(source, translations, output, overwrite)
            elif choice == "3":
                source = _prompt_path("源 TAK 或目录", source_default)
                verify_command(source)
            else:
                print("无效选择。")
        except (TakError, OSError, ValueError) as error:
            print(f"Error: {error}", file=sys.stderr)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Extract and inject structured SHUJIN_TAIKEN TAK text with strict "
            "CP932 validation, 24-bit jump relocation, and LZS output."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    extract_parser = subparsers.add_parser("extract", help="extract UTF-8 JSON")
    extract_parser.add_argument("source", type=Path, help="TAK file or directory")
    extract_parser.add_argument("output", type=Path, help="JSON file or directory")
    extract_parser.add_argument(
        "--overwrite", action="store_true", help="replace existing output files"
    )

    inject_parser = subparsers.add_parser(
        "inject", help="inject JSON and write verified LZS TAK output"
    )
    inject_parser.add_argument("source", type=Path, help="original TAK file or directory")
    inject_parser.add_argument(
        "translations", type=Path, help="translation JSON file or directory"
    )
    inject_parser.add_argument("output", type=Path, help="new TAK file or directory")
    inject_parser.add_argument(
        "--overwrite", action="store_true", help="replace existing output files"
    )

    verify_parser = subparsers.add_parser(
        "verify", help="validate scripts, CP932 text, targets, and generated LZS"
    )
    verify_parser.add_argument("source", type=Path, help="TAK file or directory")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    commands = {"extract", "inject", "verify"}
    if not arguments:
        return interactive()
    if len(arguments) == 1 and arguments[0] not in commands and not arguments[0].startswith("-"):
        return interactive(arguments[0])

    parser = build_parser()
    args = parser.parse_args(arguments)
    try:
        if args.command == "extract":
            extract_command(args.source, args.output, args.overwrite)
        elif args.command == "inject":
            inject_command(args.source, args.translations, args.output, args.overwrite)
        elif args.command == "verify":
            verify_command(args.source)
        else:
            parser.error("unknown command")
    except (TakError, OSError, ValueError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
