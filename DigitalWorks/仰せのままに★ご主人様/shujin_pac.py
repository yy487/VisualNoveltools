#!/usr/bin/env python3
"""PAC inspector, unpacker, and repacker for the SHUJIN_TAIKEN executable."""

from __future__ import annotations

import argparse
import base64
import binascii
import hashlib
import json
import os
import shutil
import struct
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence


ALIGNMENT = 0x800
MAX_INDEX_ENTRIES = 65536
MAX_DECOMPRESSED_ENTRY_SIZE = 1024 * 1024 * 1024


class ArchiveError(Exception):
    """Raised when an input is not a supported, structurally valid archive."""


@dataclass(frozen=True)
class PackSpec:
    kind: str
    runtime_name: str
    table_va: int
    filename_prefix: str
    filename_extension: str

    def filename(self, file_id: int) -> str:
        return f"{self.filename_prefix}{file_id:05d}{self.filename_extension}"


PACK_SPECS = {
    spec.kind: spec
    for spec in (
        PackSpec("script", "SCRIPT.PAC", 0x447778, "TAK", ".BIN"),
        PackSpec("visual", "VISUAL.PAC", 0x447818, "VIS", ".TM2"),
        PackSpec("music", "MUSIC.PAC", 0x449D00, "BGM", ".OGG"),
        PackSpec("se", "SE.PAC", 0x449DE8, "_SE", ".OGG"),
        PackSpec("voice", "VOICE.PAC", 0x44A058, "VCE", ".OGG"),
    )
}


@dataclass(frozen=True)
class Section:
    name: str
    virtual_address: int
    virtual_size: int
    raw_offset: int
    raw_size: int


@dataclass(frozen=True)
class IndexEntry:
    table_index: int
    flags: int
    file_id: int
    offset: int
    stored_size: int


@dataclass(frozen=True)
class ParsedIndex:
    spec: PackSpec
    table_raw_offset: int
    entries: tuple[IndexEntry, ...]
    sentinel_flags: int
    sentinel_offset: int
    sentinel_size: int


@dataclass(frozen=True)
class DecodeResult:
    data: bytes
    compressed: bool
    declared_size: int
    consumed_size: int


@dataclass(frozen=True)
class PackInput:
    entry: IndexEntry
    filename: str
    data: bytes
    padding: bytes
    changed: bool


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _align_up(value: int, alignment: int = ALIGNMENT) -> int:
    return (value + alignment - 1) // alignment * alignment


def _unpack_from(fmt: str, data: bytes, offset: int, label: str) -> tuple[int, ...]:
    size = struct.calcsize(fmt)
    if offset < 0 or offset + size > len(data):
        raise ArchiveError(f"{label} is truncated at file offset 0x{offset:X}")
    return struct.unpack_from(fmt, data, offset)


class PEImage:
    """Minimal PE32 parser used only for VA-to-file-offset mapping."""

    def __init__(self, data: bytes) -> None:
        self.data = data
        if len(data) < 0x40 or data[:2] != b"MZ":
            raise ArchiveError("EXE is not an MZ executable")

        (pe_offset,) = _unpack_from("<I", data, 0x3C, "DOS header")
        if pe_offset + 24 > len(data) or data[pe_offset : pe_offset + 4] != b"PE\0\0":
            raise ArchiveError("EXE has no valid PE signature")

        section_count, optional_size = _unpack_from(
            "<H12xH", data, pe_offset + 6, "COFF header"
        )
        optional_offset = pe_offset + 24
        (optional_magic,) = _unpack_from(
            "<H", data, optional_offset, "optional header"
        )
        if optional_magic != 0x10B:
            raise ArchiveError("only PE32 executables are supported")
        (self.image_base,) = _unpack_from(
            "<I", data, optional_offset + 28, "PE32 image base"
        )

        section_offset = optional_offset + optional_size
        sections: list[Section] = []
        for section_index in range(section_count):
            offset = section_offset + section_index * 40
            if offset + 40 > len(data):
                raise ArchiveError("PE section table is truncated")
            raw_name = data[offset : offset + 8].split(b"\0", 1)[0]
            name = raw_name.decode("ascii", errors="replace")
            virtual_size, virtual_address, raw_size, raw_offset = _unpack_from(
                "<IIII", data, offset + 8, f"PE section {section_index}"
            )
            sections.append(
                Section(name, virtual_address, virtual_size, raw_offset, raw_size)
            )
        self.sections = tuple(sections)

    def va_to_raw(self, virtual_address: int) -> int:
        if virtual_address < self.image_base:
            raise ArchiveError(f"VA 0x{virtual_address:X} precedes the image base")
        rva = virtual_address - self.image_base
        for section in self.sections:
            span = max(section.virtual_size, section.raw_size)
            if section.virtual_address <= rva < section.virtual_address + span:
                delta = rva - section.virtual_address
                if delta >= section.raw_size:
                    raise ArchiveError(
                        f"VA 0x{virtual_address:X} is in uninitialized {section.name} data"
                    )
                raw = section.raw_offset + delta
                if raw >= len(self.data):
                    raise ArchiveError(
                        f"VA 0x{virtual_address:X} maps beyond the EXE file"
                    )
                return raw
        raise ArchiveError(f"VA 0x{virtual_address:X} is not mapped by a PE section")


def parse_index(exe_data: bytes, pac_size: int, spec: PackSpec) -> ParsedIndex:
    image = PEImage(exe_data)
    table_raw = image.va_to_raw(spec.table_va)
    entries: list[IndexEntry] = []
    seen_ids: set[int] = set()
    sentinel: tuple[int, int, int] | None = None

    for table_index in range(MAX_INDEX_ENTRIES):
        offset = table_raw + table_index * 12
        flags, file_id, data_offset, stored_size = _unpack_from(
            "<HHII", exe_data, offset, f"{spec.kind} index"
        )
        if file_id == 0:
            sentinel = flags, data_offset, stored_size
            break
        if file_id in seen_ids:
            raise ArchiveError(f"{spec.kind} index repeats file ID {file_id}")
        seen_ids.add(file_id)
        entries.append(IndexEntry(table_index, flags, file_id, data_offset, stored_size))
    else:
        raise ArchiveError(f"{spec.kind} index has no sentinel")

    if not entries or sentinel is None:
        raise ArchiveError(f"{spec.kind} index contains no entries")
    sentinel_flags, sentinel_offset, sentinel_size = sentinel
    if sentinel_offset != sentinel_size or sentinel_size != pac_size:
        raise ArchiveError(
            f"{spec.runtime_name} length mismatch: index sentinel is "
            f"0x{sentinel_offset:X}/0x{sentinel_size:X}, file is 0x{pac_size:X}"
        )

    previous_end = 0
    for entry_index, entry in enumerate(entries):
        if entry.offset + entry.stored_size > pac_size:
            raise ArchiveError(
                f"entry ID {entry.file_id} extends beyond the PAC file"
            )
        expected_offset = 0 if entry_index == 0 else _align_up(previous_end)
        if entry.offset != expected_offset:
            raise ArchiveError(
                f"entry ID {entry.file_id} starts at 0x{entry.offset:X}; "
                f"expected aligned offset 0x{expected_offset:X}"
            )
        previous_end = entry.offset + entry.stored_size

    if _align_up(previous_end) != pac_size:
        raise ArchiveError(
            f"last entry covers through 0x{previous_end:X}, inconsistent with "
            f"aligned PAC size 0x{pac_size:X}"
        )

    return ParsedIndex(
        spec,
        table_raw,
        tuple(entries),
        sentinel_flags,
        sentinel_offset,
        sentinel_size,
    )


def select_index(
    exe_data: bytes, pac_data: bytes, requested_kind: str | None
) -> ParsedIndex:
    if requested_kind is not None:
        return parse_index(exe_data, len(pac_data), PACK_SPECS[requested_kind])

    matches: list[ParsedIndex] = []
    failures: list[str] = []
    for spec in PACK_SPECS.values():
        try:
            matches.append(parse_index(exe_data, len(pac_data), spec))
        except ArchiveError as error:
            failures.append(f"{spec.kind}: {error}")
    if len(matches) == 1:
        return matches[0]
    if not matches:
        details = "; ".join(failures)
        raise ArchiveError(f"PAC does not match any known embedded index ({details})")
    kinds = ", ".join(match.spec.kind for match in matches)
    raise ArchiveError(f"PAC matches multiple indexes ({kinds}); specify --kind")


def decompress_lzs(data: bytes) -> DecodeResult:
    """Decode the exact LZS/LZSS variant implemented by sub_4118E0."""

    if len(data) < 3 or data[:3] != b"LZS":
        return DecodeResult(data, False, len(data), len(data))
    if len(data) < 8:
        raise ArchiveError("LZS entry is shorter than its 8-byte header")

    (declared_size,) = _unpack_from("<I", data, 4, "LZS header")
    if declared_size > MAX_DECOMPRESSED_ENTRY_SIZE:
        raise ArchiveError(
            f"LZS output size 0x{declared_size:X} exceeds the 1 GiB safety limit"
        )

    source = 8
    flags = 0
    ring = bytearray(b" " * 4096)
    ring_position = 0xFEE
    output = bytearray()

    while len(output) < declared_size:
        flags >>= 1
        if not flags & 0x100:
            if source >= len(data):
                raise ArchiveError("LZS stream ends before a flag byte")
            flags = 0xFF00 | data[source]
            source += 1

        if flags & 1:
            if source >= len(data):
                raise ArchiveError("LZS stream ends before a literal byte")
            value = data[source]
            source += 1
            output.append(value)
            ring[ring_position] = value
            ring_position = (ring_position + 1) & 0xFFF
        else:
            if source + 1 >= len(data):
                raise ArchiveError("LZS stream ends inside a back-reference")
            first, second = data[source], data[source + 1]
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

    return DecodeResult(bytes(output), True, declared_size, source)


def _entry_blob(pac_data: bytes, entry: IndexEntry) -> bytes:
    blob = pac_data[entry.offset : entry.offset + entry.stored_size]
    if len(blob) != entry.stored_size:
        raise ArchiveError(f"entry ID {entry.file_id} is truncated")
    return blob


def list_archive(
    exe_path: Path, pac_path: Path, requested_kind: str | None
) -> ParsedIndex:
    exe_data = exe_path.read_bytes()
    pac_data = pac_path.read_bytes()
    index = select_index(exe_data, pac_data, requested_kind)

    print(
        f"Pack: {index.spec.runtime_name} ({index.spec.kind}), "
        f"{len(index.entries)} files, 0x{len(pac_data):X} bytes"
    )
    print(
        f"Index: VA 0x{index.spec.table_va:X}, "
        f"EXE file offset 0x{index.table_raw_offset:X}"
    )
    print(f"{'ID':>7} {'Offset':>10} {'Stored':>10} {'Output':>10} {'Flags':>7}  File")
    print("-" * 76)
    for entry in index.entries:
        blob = _entry_blob(pac_data, entry)
        if blob[:3] == b"LZS":
            if len(blob) < 8:
                raise ArchiveError(f"entry ID {entry.file_id} has a short LZS header")
            output_size = struct.unpack_from("<I", blob, 4)[0]
            compression = "LZS"
        else:
            output_size = len(blob)
            compression = "raw"
        print(
            f"{entry.file_id:7d} 0x{entry.offset:08X} 0x{entry.stored_size:08X} "
            f"0x{output_size:08X} {entry.flags:7d}  "
            f"{index.spec.filename(entry.file_id)} [{compression}]"
        )
    return index


def _write_json(path: Path, value: object) -> None:
    text = json.dumps(value, ensure_ascii=False, indent=2) + "\n"
    with path.open("w", encoding="utf-8", newline="\n") as output:
        output.write(text)


def _commit_staging(staging: Path, output_dir: Path, overwrite: bool) -> None:
    if not output_dir.exists():
        staging.replace(output_dir)
        return
    if not overwrite:
        raise ArchiveError(f"output directory already exists: {output_dir}")
    if output_dir.is_symlink() or not output_dir.is_dir():
        raise ArchiveError("existing output must be a real directory, not a file or link")

    for staged_file in staging.iterdir():
        target = output_dir / staged_file.name
        if target.exists() and (target.is_dir() or target.is_symlink()):
            raise ArchiveError(f"refusing to replace non-file output: {target.name}")
    for staged_file in staging.iterdir():
        os.replace(staged_file, output_dir / staged_file.name)
    staging.rmdir()


def unpack_archive(
    exe_path: Path,
    pac_path: Path,
    output_dir: Path,
    requested_kind: str | None,
    raw_mode: bool,
    overwrite: bool,
) -> dict[str, object]:
    exe_data = exe_path.read_bytes()
    pac_data = pac_path.read_bytes()
    index = select_index(exe_data, pac_data, requested_kind)

    if output_dir.exists() and not overwrite:
        raise ArchiveError(
            f"output already exists: {output_dir} (use --overwrite explicitly)"
        )
    if output_dir.is_symlink():
        raise ArchiveError("output directory may not be a symbolic link")

    output_dir.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output_dir.name}.staging-", dir=output_dir.parent)
    )
    try:
        manifest_entries: list[dict[str, object]] = []
        filenames: set[str] = set()
        for position, entry in enumerate(index.entries):
            filename = index.spec.filename(entry.file_id)
            if filename in filenames or Path(filename).name != filename:
                raise ArchiveError(f"unsafe or duplicate generated filename: {filename}")
            filenames.add(filename)

            stored = _entry_blob(pac_data, entry)
            next_offset = (
                index.entries[position + 1].offset
                if position + 1 < len(index.entries)
                else len(pac_data)
            )
            padding = pac_data[entry.offset + entry.stored_size : next_offset]
            decoded = decompress_lzs(stored)
            output_data = stored if raw_mode else decoded.data
            (staging / filename).write_bytes(output_data)
            manifest_entries.append(
                {
                    "_index": entry.table_index,
                    "id": entry.file_id,
                    "flags": entry.flags,
                    "_offset": entry.offset,
                    "_size": entry.stored_size,
                    "filename": filename,
                    "compression": "LZS" if decoded.compressed else "raw",
                    "declared_decompressed_size": decoded.declared_size,
                    "compressed_stream_consumed": decoded.consumed_size,
                    "stored_sha256": _sha256(stored),
                    "_padding_size": len(padding),
                    "_padding_base64": base64.b64encode(padding).decode("ascii"),
                    "output_size": len(output_data),
                    "output_sha256": _sha256(output_data),
                }
            )

        manifest: dict[str, object] = {
            "format": "SHUJIN_TAIKEN embedded-index PAC",
            "manifest_version": 2,
            "pack_kind": index.spec.kind,
            "runtime_name": index.spec.runtime_name,
            "mode": "raw" if raw_mode else "decompressed",
            "alignment": ALIGNMENT,
            "table_virtual_address": index.spec.table_va,
            "table_exe_file_offset": index.table_raw_offset,
            "sentinel": {
                "flags": index.sentinel_flags,
                "offset": index.sentinel_offset,
                "size": index.sentinel_size,
            },
            "source_exe_name": exe_path.name,
            "source_exe_sha256": _sha256(exe_data),
            "source_pac_name": pac_path.name,
            "source_pac_size": len(pac_data),
            "source_pac_sha256": _sha256(pac_data),
            "entries": manifest_entries,
        }
        _write_json(staging / "manifest.json", manifest)
        _commit_staging(staging, output_dir, overwrite)
    except Exception:
        if staging.exists():
            shutil.rmtree(staging)
        raise

    print(
        f"Unpacked {len(index.entries)} files from {index.spec.runtime_name} "
        f"to {output_dir} ({'raw' if raw_mode else 'decompressed'})"
    )
    return manifest


def _manifest_int(value: object, label: str, maximum: int = 0xFFFFFFFF) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ArchiveError(f"manifest field {label} must be an integer")
    if value < 0 or value > maximum:
        raise ArchiveError(f"manifest field {label} is out of range")
    return value


def _load_manifest(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except UnicodeDecodeError as error:
        raise ArchiveError(f"manifest is not valid UTF-8: {error}") from error
    except json.JSONDecodeError as error:
        raise ArchiveError(f"manifest is not valid JSON: {error}") from error
    if not isinstance(value, dict):
        raise ArchiveError("manifest root must be an object")
    return value


def _is_within(path: Path, parent: Path) -> bool:
    try:
        path.resolve(strict=False).relative_to(parent.resolve(strict=False))
        return True
    except ValueError:
        return False


def _same_path(first: Path, second: Path) -> bool:
    first_name = os.path.normcase(os.path.abspath(first))
    second_name = os.path.normcase(os.path.abspath(second))
    return first_name == second_name


def _read_pack_inputs(
    exe_data: bytes, input_dir: Path, manifest_path: Path
) -> tuple[dict[str, object], ParsedIndex, tuple[PackInput, ...]]:
    manifest = _load_manifest(manifest_path)
    if manifest.get("format") != "SHUJIN_TAIKEN embedded-index PAC":
        raise ArchiveError("manifest format is not supported")
    manifest_version = manifest.get("manifest_version")
    if manifest_version not in {1, 2}:
        raise ArchiveError("only manifest_version 1 or 2 is supported")
    if manifest.get("alignment") != ALIGNMENT:
        raise ArchiveError(f"manifest alignment must be {ALIGNMENT}")

    kind_value = manifest.get("pack_kind")
    if not isinstance(kind_value, str) or kind_value not in PACK_SPECS:
        raise ArchiveError("manifest pack_kind is missing or unknown")
    spec = PACK_SPECS[kind_value]
    if manifest.get("table_virtual_address") != spec.table_va:
        raise ArchiveError("manifest table_virtual_address does not match pack_kind")

    source_exe_hash = manifest.get("source_exe_sha256")
    if not isinstance(source_exe_hash, str) or source_exe_hash.lower() != _sha256(exe_data):
        raise ArchiveError("source EXE SHA-256 does not match the manifest")
    source_pac_size = _manifest_int(
        manifest.get("source_pac_size"), "source_pac_size"
    )
    index = parse_index(exe_data, source_pac_size, spec)

    mode = manifest.get("mode")
    if mode not in {"raw", "decompressed"}:
        raise ArchiveError("manifest mode must be raw or decompressed")
    manifest_entries = manifest.get("entries")
    if not isinstance(manifest_entries, list):
        raise ArchiveError("manifest entries must be an array")
    if len(manifest_entries) != len(index.entries):
        raise ArchiveError(
            f"manifest has {len(manifest_entries)} entries; EXE index has {len(index.entries)}"
        )

    input_root = input_dir.resolve(strict=True)
    pack_inputs: list[PackInput] = []
    for position, (entry, manifest_entry) in enumerate(
        zip(index.entries, manifest_entries)
    ):
        if not isinstance(manifest_entry, dict):
            raise ArchiveError(f"manifest entry {position} must be an object")
        comparisons = {
            "_index": entry.table_index,
            "id": entry.file_id,
            "flags": entry.flags,
            "_offset": entry.offset,
            "_size": entry.stored_size,
        }
        for key, expected in comparisons.items():
            actual = _manifest_int(manifest_entry.get(key), f"entries[{position}].{key}")
            if actual != expected:
                raise ArchiveError(
                    f"manifest entry {position} {key} is {actual}; expected {expected}"
                )

        filename = spec.filename(entry.file_id)
        if manifest_entry.get("filename") != filename or Path(filename).name != filename:
            raise ArchiveError(f"manifest entry {position} has an unsafe or wrong filename")
        input_path = input_dir / filename
        try:
            resolved_input = input_path.resolve(strict=True)
        except FileNotFoundError as error:
            raise ArchiveError(f"missing input entry: {filename}") from error
        if input_root not in resolved_input.parents or not resolved_input.is_file():
            raise ArchiveError(f"input entry is not a regular file inside input_dir: {filename}")
        data = resolved_input.read_bytes()

        if mode == "raw" and data[:3] == b"LZS":
            decompress_lzs(data)
        elif mode == "decompressed" and data[:3] == b"LZS":
            raise ArchiveError(
                f"decompressed input {filename} starts with LZS and would be "
                "misdetected as compressed by the game"
            )

        padding = b""
        if manifest_version == 2:
            padding_value = manifest_entry.get("_padding_base64")
            if not isinstance(padding_value, str):
                raise ArchiveError(
                    f"manifest entry {position} is missing _padding_base64"
                )
            try:
                padding = base64.b64decode(padding_value, validate=True)
            except (binascii.Error, ValueError) as error:
                raise ArchiveError(
                    f"manifest entry {position} has invalid padding data"
                ) from error
            declared_padding_size = _manifest_int(
                manifest_entry.get("_padding_size"),
                f"entries[{position}]._padding_size",
            )
            next_offset = (
                index.entries[position + 1].offset
                if position + 1 < len(index.entries)
                else source_pac_size
            )
            expected_padding_size = next_offset - entry.offset - entry.stored_size
            if declared_padding_size != expected_padding_size or len(padding) != expected_padding_size:
                raise ArchiveError(
                    f"manifest entry {position} padding length is inconsistent"
                )

        original_output_hash = manifest_entry.get("output_sha256")
        changed = not isinstance(original_output_hash, str) or _sha256(data) != original_output_hash
        pack_inputs.append(PackInput(entry, filename, data, padding, changed))

    return manifest, index, tuple(pack_inputs)


def _assemble_pac(
    pack_inputs: Sequence[PackInput],
    preserve_padding: bool = False,
) -> tuple[bytes, tuple[IndexEntry, ...]]:
    pac_data = bytearray()
    rebuilt_entries: list[IndexEntry] = []
    for pack_input in pack_inputs:
        if len(pack_input.data) > 0xFFFFFFFF:
            raise ArchiveError(f"input entry is too large: {pack_input.filename}")
        offset = len(pac_data)
        if offset > 0xFFFFFFFF:
            raise ArchiveError("rebuilt PAC exceeds the 32-bit offset range")
        entry = pack_input.entry
        rebuilt_entries.append(
            IndexEntry(
                entry.table_index,
                entry.flags,
                entry.file_id,
                offset,
                len(pack_input.data),
            )
        )
        pac_data.extend(pack_input.data)
        aligned_size = _align_up(len(pac_data))
        if aligned_size > 0xFFFFFFFF:
            raise ArchiveError("rebuilt PAC exceeds the 32-bit size range")
        padding_size = aligned_size - len(pac_data)
        if preserve_padding:
            preserved = pack_input.padding[:padding_size]
            pac_data.extend(preserved)
            pac_data.extend(b"\0" * (padding_size - len(preserved)))
        else:
            pac_data.extend(b"\0" * padding_size)
    return bytes(pac_data), tuple(rebuilt_entries)


def _patch_exe_index(
    exe_data: bytes,
    source_index: ParsedIndex,
    rebuilt_entries: Sequence[IndexEntry],
    pac_size: int,
) -> bytes:
    if len(rebuilt_entries) != len(source_index.entries):
        raise ArchiveError("rebuilt entry count does not match the source index")
    if pac_size < 0 or pac_size > 0xFFFFFFFF:
        raise ArchiveError("rebuilt PAC size exceeds the 32-bit range")

    patched = bytearray(exe_data)
    for source, rebuilt in zip(source_index.entries, rebuilt_entries):
        if (
            source.table_index != rebuilt.table_index
            or source.flags != rebuilt.flags
            or source.file_id != rebuilt.file_id
        ):
            raise ArchiveError("rebuild attempted to change index identity or flags")
        struct.pack_into(
            "<HHII",
            patched,
            source_index.table_raw_offset + source.table_index * 12,
            source.flags,
            source.file_id,
            rebuilt.offset,
            rebuilt.stored_size,
        )
    struct.pack_into(
        "<HHII",
        patched,
        source_index.table_raw_offset + len(source_index.entries) * 12,
        source_index.sentinel_flags,
        0,
        pac_size,
        pac_size,
    )
    return bytes(patched)


def _validate_output_paths(
    source_exe: Path,
    input_dir: Path,
    output_exe: Path,
    output_pac: Path,
    overwrite: bool,
) -> None:
    if _same_path(output_exe, output_pac):
        raise ArchiveError("output EXE and output PAC must be different files")
    if _same_path(source_exe, output_exe) or _same_path(source_exe, output_pac):
        raise ArchiveError("outputs may not overwrite the source EXE")
    if _is_within(output_exe, input_dir) or _is_within(output_pac, input_dir):
        raise ArchiveError("outputs must be outside the input directory")

    for label, path in (("output EXE", output_exe), ("output PAC", output_pac)):
        if path.is_symlink():
            raise ArchiveError(f"{label} may not be a symbolic link")
        if path.exists() and not overwrite:
            raise ArchiveError(f"{label} already exists: {path} (use --overwrite explicitly)")
        if path.exists() and not path.is_file():
            raise ArchiveError(f"{label} must be a file path")


def _stage_file(path: Path, data: bytes) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.staging-", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
    except Exception:
        Path(temporary_name).unlink(missing_ok=True)
        raise
    return Path(temporary_name)


def pack_archive(
    source_exe: Path,
    input_dir: Path,
    output_exe: Path,
    output_pac: Path,
    manifest_path: Path | None,
    overwrite: bool,
) -> dict[str, object]:
    if not input_dir.is_dir():
        raise ArchiveError(f"input directory does not exist: {input_dir}")
    if input_dir.is_symlink():
        raise ArchiveError("input directory may not be a symbolic link")
    actual_manifest = manifest_path or input_dir / "manifest.json"
    if not actual_manifest.is_file():
        raise ArchiveError(f"manifest does not exist: {actual_manifest}")

    _validate_output_paths(source_exe, input_dir, output_exe, output_pac, overwrite)
    exe_data = source_exe.read_bytes()
    manifest, source_index, pack_inputs = _read_pack_inputs(
        exe_data, input_dir, actual_manifest
    )
    pac_data, rebuilt_entries = _assemble_pac(
        pack_inputs, preserve_padding=manifest["mode"] == "raw"
    )
    patched_exe = _patch_exe_index(
        exe_data, source_index, rebuilt_entries, len(pac_data)
    )

    verified_index = parse_index(patched_exe, len(pac_data), source_index.spec)
    if verified_index.entries != rebuilt_entries:
        raise ArchiveError("internal verification of the rebuilt EXE index failed")
    for rebuilt, pack_input in zip(rebuilt_entries, pack_inputs):
        if _entry_blob(pac_data, rebuilt) != pack_input.data:
            raise ArchiveError(f"internal verification failed for {pack_input.filename}")

    staged_pac: Path | None = None
    staged_exe: Path | None = None
    try:
        staged_pac = _stage_file(output_pac, pac_data)
        staged_exe = _stage_file(output_exe, patched_exe)
        os.replace(staged_pac, output_pac)
        staged_pac = None
        os.replace(staged_exe, output_exe)
        staged_exe = None
    finally:
        if staged_pac is not None:
            staged_pac.unlink(missing_ok=True)
        if staged_exe is not None:
            staged_exe.unlink(missing_ok=True)

    changed_count = sum(pack_input.changed for pack_input in pack_inputs)
    report: dict[str, object] = {
        "pack_kind": source_index.spec.kind,
        "input_mode": manifest["mode"],
        "entries": len(pack_inputs),
        "changed_entries": changed_count,
        "output_pac_size": len(pac_data),
        "output_pac_sha256": _sha256(pac_data),
        "output_exe_sha256": _sha256(patched_exe),
    }
    print(
        f"Packed {len(pack_inputs)} files into {output_pac} "
        f"(0x{len(pac_data):X} bytes, {changed_count} changed)"
    )
    print(f"Patched embedded {source_index.spec.kind} index in new EXE: {output_exe}")
    return report


def _kind_or_none(value: str) -> str | None:
    normalized = value.strip().lower()
    if not normalized or normalized == "auto":
        return None
    if normalized not in PACK_SPECS:
        choices = ", ".join(PACK_SPECS)
        raise ArchiveError(f"unknown pack kind {value!r}; choose one of: {choices}")
    return normalized


def _prompt(label: str, default: str | None = None) -> str:
    suffix = f" [{default}]" if default else ""
    value = input(f"{label}{suffix}: ").strip()
    return value or (default or "")


def _classify_prefill(
    path: Path,
) -> tuple[str | None, str | None, str | None]:
    if path.is_dir():
        return None, None, str(path)
    if path.is_file():
        try:
            if path.read_bytes()[:2] == b"MZ":
                return str(path), None, None
        except OSError:
            pass
        return None, str(path), None
    return None, None, None


def interactive(initial_path: str | None = None) -> int:
    exe_default: str | None = None
    pac_default: str | None = None
    input_default: str | None = None
    if initial_path:
        exe_default, pac_default, input_default = _classify_prefill(Path(initial_path))
        print(f"Prefilled path (no files have been written): {initial_path}")

    while True:
        print("\nSHUJIN_TAIKEN PAC tool")
        print("  1. List archive")
        print("  2. Unpack archive")
        print("  3. Pack archive")
        print("  4. Exit")
        try:
            choice = input("Select: ").strip().lower()
        except EOFError:
            print()
            return 0
        if choice in {"4", "q", "quit", "exit"}:
            return 0
        if choice not in {"1", "2", "3"}:
            print("Invalid selection.")
            continue

        try:
            exe_value = _prompt("EXE path", exe_default)
            if not exe_value:
                print("Cancelled: an EXE path is required.")
                continue
            exe_default = exe_value

            if choice == "3":
                input_value = _prompt("Input directory", input_default)
                if not input_value:
                    print("Cancelled: an input directory is required.")
                    continue
                input_default = input_value
                manifest_value = _prompt(
                    "Manifest path", str(Path(input_value) / "manifest.json")
                )
                output_exe_value = _prompt("New EXE path")
                output_pac_value = _prompt("New PAC path")
                if not output_exe_value or not output_pac_value:
                    print("Cancelled: both output paths are required.")
                    continue
                output_exe = Path(output_exe_value)
                output_pac = Path(output_pac_value)
                overwrite = False
                if output_exe.exists() or output_pac.exists():
                    overwrite = _prompt(
                        "Output exists. Overwrite output files? (y/N)", "N"
                    ).lower() in {"y", "yes"}
                    if not overwrite:
                        print("Cancelled: existing outputs were not approved.")
                        continue
                confirm = _prompt(
                    "Create a new PAC and patched EXE copy? (y/N)", "N"
                ).lower()
                if confirm not in {"y", "yes"}:
                    print("Cancelled: nothing was written.")
                    continue
                pack_archive(
                    Path(exe_value),
                    Path(input_value),
                    output_exe,
                    output_pac,
                    Path(manifest_value),
                    overwrite,
                )
                continue

            pac_value = _prompt("PAC path", pac_default)
            kind = _kind_or_none(_prompt("Kind (auto/script/visual/music/se/voice)", "auto"))
            if not pac_value:
                print("Cancelled: a PAC path is required.")
                continue
            pac_default = pac_value
            if choice == "1":
                list_archive(Path(exe_value), Path(pac_value), kind)
                continue

            output_value = _prompt("Output directory")
            if not output_value:
                print("Cancelled: no output directory was selected.")
                continue
            raw_mode = _prompt("Keep compressed entry bytes? (y/N)", "N").lower() in {
                "y",
                "yes",
            }
            output_dir = Path(output_value)
            overwrite = False
            if output_dir.exists():
                overwrite = _prompt("Output exists. Overwrite known files? (y/N)", "N").lower() in {
                    "y",
                    "yes",
                }
                if not overwrite:
                    print("Cancelled: existing output was not approved.")
                    continue
            confirm = _prompt("Proceed with unpacking? (y/N)", "N").lower()
            if confirm not in {"y", "yes"}:
                print("Cancelled: nothing was written.")
                continue
            unpack_archive(
                Path(exe_value),
                Path(pac_value),
                output_dir,
                kind,
                raw_mode,
                overwrite,
            )
        except (ArchiveError, OSError, ValueError) as error:
            print(f"Error: {error}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Inspect, unpack, or rebuild SHUJIN_TAIKEN PAC files. "
            "Packing always writes a new EXE/PAC pair."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="validate and list PAC entries")
    list_parser.add_argument("exe", type=Path, help="matching game executable")
    list_parser.add_argument("pac", type=Path, help="PAC payload file")
    list_parser.add_argument(
        "--kind", choices=tuple(PACK_SPECS), help="pack kind; normally auto-detected"
    )

    unpack_parser = subparsers.add_parser("unpack", help="extract PAC entries")
    unpack_parser.add_argument("exe", type=Path, help="matching game executable")
    unpack_parser.add_argument("pac", type=Path, help="PAC payload file")
    unpack_parser.add_argument("output", type=Path, help="new output directory")
    unpack_parser.add_argument(
        "--kind", choices=tuple(PACK_SPECS), help="pack kind; normally auto-detected"
    )
    unpack_parser.add_argument(
        "--raw", action="store_true", help="keep LZS entry bytes instead of decompressing"
    )
    unpack_parser.add_argument(
        "--overwrite",
        action="store_true",
        help="explicitly allow replacing known files in an existing output directory",
    )

    pack_parser = subparsers.add_parser(
        "pack", help="rebuild a PAC and patch its index in a new EXE copy"
    )
    pack_parser.add_argument("exe", type=Path, help="source game executable")
    pack_parser.add_argument(
        "input", type=Path, help="directory produced by unpack, with edited entry files"
    )
    pack_parser.add_argument("output_exe", type=Path, help="new patched executable")
    pack_parser.add_argument("output_pac", type=Path, help="new rebuilt PAC")
    pack_parser.add_argument(
        "--manifest",
        type=Path,
        help="explicit manifest path; defaults to manifest.json inside input",
    )
    pack_parser.add_argument(
        "--overwrite",
        action="store_true",
        help="explicitly allow replacing existing output EXE/PAC files",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    commands = {"list", "unpack", "pack"}
    if not arguments:
        return interactive()
    if len(arguments) == 1 and arguments[0] not in commands and not arguments[0].startswith("-"):
        return interactive(arguments[0])

    parser = build_parser()
    args = parser.parse_args(arguments)
    try:
        if args.command == "list":
            list_archive(args.exe, args.pac, args.kind)
        elif args.command == "unpack":
            unpack_archive(
                args.exe,
                args.pac,
                args.output,
                args.kind,
                args.raw,
                args.overwrite,
            )
        elif args.command == "pack":
            pack_archive(
                args.exe,
                args.input,
                args.output_exe,
                args.output_pac,
                args.manifest,
                args.overwrite,
            )
        else:
            parser.error(f"unsupported command: {args.command}")
    except (ArchiveError, OSError, ValueError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
