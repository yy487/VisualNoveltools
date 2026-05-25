# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import shutil
from bisect import bisect_right
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

from .script_common import (
    CHOICE_OP,
    DEFAULT_ENCODING,
    DEFAULT_OUTPUT_ENCODING,
    TEXT_OP_MID,
    SPEAKER_PREFIXES,
    encode_text,
    normalize_encoding_name,
    iter_script_files,
    load_json,
    parse_script_records,
)


DEFAULT_NAME_DICT_FILE = "_noesis_name_dict.json"
IGNORED_JSON_FILENAMES = {"_noesis_iga_manifest.json", DEFAULT_NAME_DICT_FILE, "namedict.json", "name_dict.json"}


@dataclass(frozen=True)
class Replacement:
    old_start: int
    old_end: int
    new_bytes: bytes


@dataclass(frozen=True)
class OffsetRef:
    kind: str
    inst_offset: int
    value_offset: int
    target: int


def _collect_json_entries(json_path: Path) -> dict[str, list[dict[str, Any]]]:
    by_file: dict[str, list[dict[str, Any]]] = defaultdict(list)
    if json_path.is_file():
        entries = load_json(json_path)
        for e in entries:
            file_name = e.get("_file")
            if not isinstance(file_name, str):
                raise ValueError(f"JSON entry missing _file: {e!r}")
            by_file[file_name].append(e)
        return by_file

    for jp in sorted(json_path.rglob("*.json")):
        if jp.name in IGNORED_JSON_FILENAMES:
            continue
        entries = load_json(jp)
        for e in entries:
            file_name = e.get("_file")
            if not isinstance(file_name, str):
                raise ValueError(f"JSON entry missing _file in {jp}: {e!r}")
            by_file[file_name].append(e)
    return by_file


def _load_name_dict(json_path: Path, explicit_name_dict: Path | None = None) -> dict[str, str]:
    """Load optional original-name -> translated-name mapping.

    Supported format is a plain object, for example:
        {"あやか": "绫香"}

    Empty values and identity mappings are ignored.  The dictionary is used in
    addition to per-dialogue edited `name` fields.
    """
    candidates: list[Path] = []
    if explicit_name_dict is not None:
        candidates.append(explicit_name_dict)
    elif json_path.is_dir():
        candidates.extend([
            json_path / DEFAULT_NAME_DICT_FILE,
            json_path / "namedict.json",
            json_path / "name_dict.json",
        ])
    else:
        candidates.extend([
            json_path.with_name(DEFAULT_NAME_DICT_FILE),
            json_path.with_name("namedict.json"),
            json_path.with_name("name_dict.json"),
        ])

    out: dict[str, str] = {}
    for path in candidates:
        if not path.is_file():
            continue
        with path.open("r", encoding="utf-8") as f:
            data = json.load(f)
        if not isinstance(data, dict):
            raise ValueError(f"name dict root must be an object: {path}")
        for k, v in data.items():
            if not isinstance(k, str) or not isinstance(v, str):
                continue
            key = k.strip()
            value = v.strip()
            if key and value and key != value:
                out[key] = value
        break
    return out


def _u32le(data: bytes | bytearray, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little")


def _put_u32le(buf: bytearray, off: int, value: int) -> None:
    if not 0 <= value <= 0xFFFFFFFF:
        raise ValueError(f"relocated offset out of u32 range: 0x{value:X}")
    buf[off:off + 4] = value.to_bytes(4, "little")


def _put_u16le(buf: bytearray, off: int, value: int) -> None:
    if not 0 <= value <= 0xFFFF:
        raise ValueError(f"relocated length out of u16 range: 0x{value:X}")
    buf[off:off + 2] = value.to_bytes(2, "little")


def _iter_probable_text_op_ranges(data: bytes) -> Iterable[tuple[int, int]]:
    """Yield ranges occupied by 00/xx 04 00 len payload-style string ops.

    This is intentionally broader than exported story text.  The script also
    stores resource names in the same length-prefixed shape.  Masking these
    ranges prevents relocation scans from mistaking CP932 bytes inside strings
    for jump opcodes.
    """
    n = len(data)
    i = 0
    while i + 4 <= n:
        if data[i + 1:i + 3] == TEXT_OP_MID:
            size = data[i + 3]
            end = i + 4 + size
            if size > 0 and end <= n:
                yield i, end
                i = end
                continue
        i += 1


def _make_covered_mask(data: bytes) -> bytearray:
    mask = bytearray(len(data))
    for start, end in _iter_probable_text_op_ranges(data):
        mask[start:end] = b"\x01" * (end - start)
    # Mask confirmed choice bodies as well; the header is still a real ref.
    for rec in parse_script_records(data, export_names=True):
        if rec.kind == "choice":
            start = rec.text_offset
            end = rec.text_offset + rec.size
            mask[start:end] = b"\x01" * (end - start)
    return mask


def _is_masked(mask: bytearray, start: int, end: int) -> bool:
    if start < 0 or end > len(mask):
        return True
    return any(mask[start:end])


def _collect_offset_refs(data: bytes, records) -> list[OffsetRef]:
    """Collect script-internal absolute offset operands that must move.

    Confirmed from samples:
    * choice  1D 08 <text_len:u16> <target:u32> <text>
    * jump    0D 08 00 00 <target:u32>
    * branch  3B 08 xx 00 <target:u32>

    Other 08-family ops such as 0C08 and 1408 are page/timing/resource related
    in the current sample set and are not physical file offsets.
    """
    n = len(data)
    mask = _make_covered_mask(data)
    refs: list[OffsetRef] = []

    seen_value_offsets: set[int] = set()
    for rec in records:
        if rec.kind == "choice" and rec.target is not None:
            voff = rec.inst_offset + 4
            if 0 <= rec.target < n:
                refs.append(OffsetRef("choice", rec.inst_offset, voff, rec.target))
                seen_value_offsets.add(voff)

    i = 0
    while i + 8 <= n:
        if _is_masked(mask, i, i + 8):
            i += 1
            continue
        b0 = data[i]
        if b0 == 0x0D and data[i + 1:i + 4] == b"\x08\x00\x00":
            target = _u32le(data, i + 4)
            if 0 <= target < n and (i + 4) not in seen_value_offsets:
                refs.append(OffsetRef("jump_0D08", i, i + 4, target))
                seen_value_offsets.add(i + 4)
                i += 8
                continue
        if b0 == 0x3B and data[i + 1] == 0x08 and data[i + 3] == 0x00:
            target = _u32le(data, i + 4)
            if 0 <= target < n and (i + 4) not in seen_value_offsets:
                refs.append(OffsetRef("branch_3B08", i, i + 4, target))
                seen_value_offsets.add(i + 4)
                i += 8
                continue
        i += 1
    return sorted(refs, key=lambda r: r.value_offset)


def _build_text_instruction(original: bytes, rec, message: str, output_encoding: str) -> bytes:
    new = encode_text(message, output_encoding)
    if len(new) + 1 > 0xFF:
        raise ValueError(f"text op payload too long for u8 length: {len(new) + 1} > 255")
    # Keep the original three-byte opcode prefix; rebuild the payload as a clean
    # NUL-terminated string.  Ignored garbage after the old first NUL is dropped.
    return original[rec.inst_offset:rec.inst_offset + 3] + bytes([len(new) + 1]) + new + b"\x00"


def _build_choice_instruction(rec, message: str, output_encoding: str) -> bytes:
    new = encode_text(message, output_encoding)
    if len(new) > 0xFFFF:
        raise ValueError(f"choice text too long for u16 length: {len(new)} > 65535")
    # The target is patched after all deltas are known.
    return CHOICE_OP + len(new).to_bytes(2, "little") + (rec.target or 0).to_bytes(4, "little") + new


def _entry_output_encoding(entry: dict[str, Any], forced_output_encoding: str | None) -> str:
    if forced_output_encoding:
        return normalize_encoding_name(forced_output_encoding)
    value = entry.get("_output_encoding")
    if isinstance(value, str) and value.strip():
        return normalize_encoding_name(value)
    return DEFAULT_OUTPUT_ENCODING


def _name_body_from_record(rec) -> str:
    if rec.raw_text.startswith(SPEAKER_PREFIXES):
        return rec.raw_text[1:]
    return rec.raw_text


def _name_prefix_from_record(rec) -> str:
    if rec.raw_text.startswith(SPEAKER_PREFIXES):
        return rec.raw_text[:1]
    return "＃"


def _strip_optional_name_prefix(message: str) -> str:
    # Accept either translator-friendly "绫香" or raw script-style "＃绫香"/"#绫香".
    if message.startswith(SPEAKER_PREFIXES):
        return message[1:]
    return message


def _original_name_prefix_bytes(data: bytes, rec) -> bytes:
    """Return the original script marker bytes for a #name record.

    In this engine the speaker marker is not just display text.  The VM checks
    the original marker byte sequence, typically CP932 fullwidth sharp
    ``81 94``.  When message/name bodies are injected as GBK, the marker must
    still stay as the original bytes; otherwise the engine prints ``#绫香`` as
    a normal line instead of treating it as the speaker-name command.
    """
    if rec.raw_text.startswith(SPEAKER_PREFIXES):
        try:
            prefix_len = len(rec.raw_text[:1].encode(DEFAULT_ENCODING))
        except UnicodeEncodeError:
            prefix_len = 1
        return data[rec.text_offset:rec.text_offset + prefix_len]
    return b"#"


def _name_body_from_message(message: str) -> str:
    return _strip_optional_name_prefix(message).strip()


def _build_name_instruction(original: bytes, rec, translated_name: str, output_encoding: str) -> bytes:
    body = _name_body_from_message(translated_name)
    prefix = _original_name_prefix_bytes(original, rec)
    new = prefix + encode_text(body, output_encoding)
    if len(new) + 1 > 0xFF:
        raise ValueError(f"name op payload too long for u8 length: {len(new) + 1} > 255")
    return original[rec.inst_offset:rec.inst_offset + 3] + bytes([len(new) + 1]) + new + b"\x00"


def _scr_msg_matches_record(rec, scr_msg: str) -> bool:
    if rec.kind == "name":
        return scr_msg == rec.raw_text or scr_msg == _name_body_from_record(rec)
    return rec.raw_text == scr_msg


def _build_index_maps(records):
    """Return full/non-name index maps plus dialogue->preceding-name links.

    New JSON exported by this version includes explicit name records and uses
    full indexes.  Older JSON has no name records and uses non-name indexes.
    Both layouts are accepted during injection.
    """
    full_by_index = {i: rec for i, rec in enumerate(records)}
    non_name_records = [r for r in records if r.kind != "name"]
    non_by_index = {i: rec for i, rec in enumerate(non_name_records)}

    full_name_links: dict[int, Any] = {}
    non_name_links: dict[int, Any] = {}
    last_name = None
    non_i = 0
    for full_i, rec in enumerate(records):
        if rec.kind == "name":
            last_name = rec
            continue
        if rec.kind == "dialogue" and rec.name and last_name is not None and last_name.name == rec.name:
            full_name_links[full_i] = last_name
            non_name_links[non_i] = last_name
        last_name = None
        non_i += 1
    return full_by_index, non_by_index, full_name_links, non_name_links


def _prepare_replacements(
    data: bytes,
    records,
    entries: list[dict[str, Any]],
    *,
    output_encoding: str | None = None,
    name_dict: dict[str, str] | None = None,
) -> tuple[list[Replacement], int, int, list[str]]:
    full_by_index, non_by_index, full_name_links, non_name_links = _build_index_maps(records)

    # If JSON contains explicit name entries, its _index values are full record
    # indexes.  Otherwise keep compatibility with v1/v2/v3 JSON where indexes
    # skipped speaker-marker records.
    has_explicit_name_entries = any(e.get("_type") == "name" for e in entries)
    by_index = full_by_index if has_explicit_name_entries else non_by_index
    name_links = full_name_links if has_explicit_name_entries else non_name_links

    replacements: list[Replacement] = []
    patched = 0
    failed = 0
    warnings: list[str] = []
    replacement_by_range: dict[tuple[int, int], bytes] = {}

    def add_name_mapping(mapping: dict[str, str], original: str | None, translated: str, label: str) -> None:
        nonlocal failed
        if not original or not translated or translated == original:
            return
        old = mapping.get(original)
        if old is not None and old != translated:
            failed += 1
            warnings.append(f"conflicting name translation for {original!r}: {old!r} vs {translated!r} at {label}")
            return
        mapping[original] = translated

    def add_replacement(old_start: int, old_end: int, new_bytes: bytes, label: str) -> bool:
        nonlocal patched, failed
        key = (old_start, old_end)
        if key in replacement_by_range:
            if replacement_by_range[key] == new_bytes:
                return False
            warnings.append(f"duplicate replacement for {label}; earlier replacement kept range=0x{old_start:X}-0x{old_end:X}")
            return False
        for s, e2 in replacement_by_range:
            if not (old_end <= s or old_start >= e2):
                failed += 1
                warnings.append(f"overlapping replacement for {label}: range=0x{old_start:X}-0x{old_end:X}")
                return False
        replacement_by_range[key] = new_bytes
        replacements.append(Replacement(old_start, old_end, new_bytes))
        patched += 1
        return True

    def build_record_replacement(rec, message: str, enc: str) -> tuple[int, int, bytes]:
        if rec.kind == "choice":
            old_start = rec.inst_offset
            old_end = rec.text_offset + rec.size
            new_bytes = _build_choice_instruction(rec, message, enc)
        else:
            old_start = rec.inst_offset
            old_end = rec.text_offset + rec.size
            if rec.kind == "name":
                new_bytes = _build_name_instruction(data, rec, message, enc)
            else:
                new_bytes = _build_text_instruction(data, rec, message, enc)
        return old_start, old_end, new_bytes

    def raw_script_text_after_entry(rec, entry: dict[str, Any]) -> str:
        message = entry.get("message")
        if not isinstance(message, str):
            return rec.raw_text
        if rec.kind == "name":
            body = _name_body_from_message(message)
            return _name_prefix_from_record(rec) + body
        return message

    # Name replacement policy:
    # 1) Optional _noesis_name_dict.json maps original speaker names globally.
    # 2) If an old-style dialogue entry has `name` edited from the original
    #    script name, infer the same global mapping from that entry.
    # This keeps the normal JSON compact: translators edit `name` in dialogue
    # rows or edit the small name dictionary, without separate `_type=name`
    # translation rows.
    resolved_name_map: dict[str, str] = {}
    if name_dict:
        for original, translated in name_dict.items():
            add_name_mapping(resolved_name_map, original, translated, "name_dict")

    for e in entries:
        if has_explicit_name_entries:
            continue
        idx0 = e.get("_index")
        if not isinstance(idx0, int) or idx0 not in by_index:
            continue
        rec0 = by_index[idx0]
        if rec0.kind == "dialogue" and isinstance(e.get("name"), str):
            desired_name = e["name"].strip()
            add_name_mapping(resolved_name_map, rec0.name, desired_name, f"dialogue index={idx0}")

    # Apply name mappings to every matching #name op in the current script.
    # add_replacement deduplicates ranges, so this coexists with explicit name
    # entries and with direct dialogue edits.
    if resolved_name_map:
        name_enc = normalize_encoding_name(output_encoding) if output_encoding else None
        if name_enc is None:
            for ee in entries:
                try:
                    name_enc = _entry_output_encoding(ee, None)
                    break
                except Exception:
                    continue
        default_enc = name_enc or DEFAULT_OUTPUT_ENCODING
        for nr in records:
            if nr.kind != "name":
                continue
            old_body = _name_body_from_record(nr)
            desired = resolved_name_map.get(old_body)
            if desired is None or desired == old_body:
                continue
            try:
                name_old_start = nr.inst_offset
                name_old_end = nr.text_offset + nr.size
                name_new_bytes = _build_name_instruction(data, nr, desired, default_enc)
                add_replacement(name_old_start, name_old_end, name_new_bytes, f"name-dict {old_body!r}")
            except Exception as exc:
                failed += 1
                warnings.append(f"encode/build failed for name {old_body!r}->{desired!r}: {exc}")

    for e in sorted(entries, key=lambda x: int(x.get("_index", -1)) if isinstance(x.get("_index"), int) else 10**9):
        idx = e.get("_index")
        if not isinstance(idx, int) or idx not in by_index:
            failed += 1
            warnings.append(f"index not found: index={idx}")
            continue
        rec = by_index[idx]
        scr_msg = e.get("scr_msg")
        message = e.get("message")
        if not isinstance(scr_msg, str) or not isinstance(message, str):
            failed += 1
            warnings.append(f"missing scr_msg/message: index={idx}")
            continue
        if not _scr_msg_matches_record(rec, scr_msg):
            failed += 1
            if rec.kind == "name":
                warnings.append(f"scr_msg mismatch: index={idx} file={_name_body_from_record(rec)!r}/{rec.raw_text!r} json={scr_msg!r}")
            else:
                warnings.append(f"scr_msg mismatch: index={idx} file={rec.raw_text!r} json={scr_msg!r}")
            continue

        try:
            enc = _entry_output_encoding(e, output_encoding)

            # 1) Explicit record injection: dialogue/monologue/choice/name message.
            desired_script_text = raw_script_text_after_entry(rec, e)
            if desired_script_text != rec.raw_text:
                old_start, old_end, new_bytes = build_record_replacement(rec, message, enc)
                add_replacement(old_start, old_end, new_bytes, f"index={idx}")

        except Exception as exc:
            failed += 1
            warnings.append(f"encode/build failed: index={idx} {exc}")
            continue

    replacements.sort(key=lambda r: r.old_start)
    return replacements, patched, failed, warnings

def _make_offset_mapper(replacements: list[Replacement]):
    starts: list[int] = []
    ends: list[int] = []
    new_starts: list[int] = []
    acc_delta = 0
    for r in replacements:
        starts.append(r.old_start)
        ends.append(r.old_end)
        new_starts.append(r.old_start + acc_delta)
        acc_delta += len(r.new_bytes) - (r.old_end - r.old_start)

    def map_offset(old: int) -> int:
        # Count all replacements ending at or before this old offset.
        idx = bisect_right(ends, old)
        delta = 0
        for r in replacements[:idx]:
            delta += len(r.new_bytes) - (r.old_end - r.old_start)
        # If the offset points inside a replaced range, map it into the rebuilt
        # instruction by preserving the relative position as far as possible.
        j = bisect_right(starts, old) - 1
        if 0 <= j < len(replacements):
            r = replacements[j]
            if r.old_start <= old < r.old_end:
                rel = min(old - r.old_start, max(0, len(r.new_bytes) - 1))
                return new_starts[j] + rel
        return old + delta

    return map_offset


def _apply_replacements(data: bytes, replacements: list[Replacement]) -> bytearray:
    out = bytearray()
    pos = 0
    for r in replacements:
        if r.old_start < pos:
            raise ValueError("overlapping replacements")
        out += data[pos:r.old_start]
        out += r.new_bytes
        pos = r.old_end
    out += data[pos:]
    return out


def _patch_relocated_refs(out: bytearray, refs: list[OffsetRef], map_offset) -> list[str]:
    warnings: list[str] = []
    for ref in refs:
        new_value_off = map_offset(ref.value_offset)
        new_target = map_offset(ref.target)
        if new_value_off + 4 > len(out):
            warnings.append(f"relocated {ref.kind} value offset outside output: old=0x{ref.value_offset:X} new=0x{new_value_off:X}")
            continue
        _put_u32le(out, new_value_off, new_target)
    return warnings


def inject_one_file(
    src_path: Path,
    entries: list[dict[str, Any]],
    out_path: Path,
    *,
    output_encoding: str | None = None,
    name_dict: dict[str, str] | None = None,
) -> tuple[int, int, list[str]]:
    data = src_path.read_bytes()
    records = parse_script_records(data, export_names=True)
    refs = _collect_offset_refs(data, records)
    replacements, patched, failed, warnings = _prepare_replacements(data, records, entries, output_encoding=output_encoding, name_dict=name_dict)

    try:
        map_offset = _make_offset_mapper(replacements)
        out = _apply_replacements(data, replacements)
        warnings.extend(_patch_relocated_refs(out, refs, map_offset))
    except Exception as exc:
        failed += 1
        warnings.append(f"relocate failed: {exc}")
        out = bytearray(data)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(out)
    return patched, failed, warnings


def inject_scripts(input_path: Path, json_path: Path, output_path: Path, *, output_encoding: str | None = None, name_dict_path: Path | None = None) -> None:
    entries_by_file = _collect_json_entries(json_path)
    name_dict = _load_name_dict(json_path, name_dict_path)
    if name_dict:
        print(f"[noesis inject] name_dict mappings={len(name_dict)}")
    total_patched = 0
    total_failed = 0
    total_files = 0
    all_warnings: list[str] = []

    if input_path.is_file():
        file_entries = entries_by_file.get(input_path.name)
        if file_entries is None and len(entries_by_file) == 1:
            file_entries = next(iter(entries_by_file.values()))
        if not file_entries:
            raise FileNotFoundError(f"no JSON entries for {input_path.name}")
        patched, failed, warnings = inject_one_file(input_path, file_entries, output_path, output_encoding=output_encoding, name_dict=name_dict)
        print(f"[noesis inject] files=1 patched={patched} failed={failed} output={output_path}")
        for w in warnings[:50]:
            print(f"[noesis inject][warn] {input_path.name} {w}")
        if failed:
            raise SystemExit(1)
        return

    output_path.mkdir(parents=True, exist_ok=True)
    for src in input_path.rglob("*"):
        if src.is_file():
            rel = src.relative_to(input_path)
            dst = output_path / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)

    for src in iter_script_files(input_path):
        rel = src.relative_to(input_path).as_posix()
        file_entries = entries_by_file.get(rel) or entries_by_file.get(src.name)
        if not file_entries:
            continue
        dst = output_path / rel
        patched, failed, warnings = inject_one_file(src, file_entries, dst, output_encoding=output_encoding, name_dict=name_dict)
        total_patched += patched
        total_failed += failed
        total_files += 1
        all_warnings.extend(f"{rel} {w}" for w in warnings)

    print(f"[noesis inject] files={total_files} patched={total_patched} failed={total_failed} output={output_path}")
    for w in all_warnings[:100]:
        print(f"[noesis inject][warn] {w}")
    if len(all_warnings) > 100:
        print(f"[noesis inject][warn] ... {len(all_warnings) - 100} more")
    if total_failed:
        raise SystemExit(1)


def main() -> None:
    ap = argparse.ArgumentParser(description="Inject JSON text back into Noesis .s scripts with relocation")
    ap.add_argument("input", help="original .s file or unpacked script directory")
    ap.add_argument("json", help="translated JSON file or JSON directory")
    ap.add_argument("output", help="output .s file or directory")
    ap.add_argument("--output-encoding", "--encoding", default=None,
                    help="encoding used to encode message/name fields when injecting; defaults to per-entry _output_encoding or cp932. Use gbk for GBK hook mode.")
    ap.add_argument("--name-dict", default=None,
                    help="optional original-name -> translated-name JSON object. If omitted, _noesis_name_dict.json in/beside the JSON path is used when present.")
    args = ap.parse_args()
    inject_scripts(
        Path(args.input),
        Path(args.json),
        Path(args.output),
        output_encoding=args.output_encoding,
        name_dict_path=Path(args.name_dict) if args.name_dict else None,
    )


if __name__ == "__main__":
    main()
