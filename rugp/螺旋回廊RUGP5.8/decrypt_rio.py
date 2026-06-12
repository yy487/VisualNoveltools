"""
rUGP 5.80.20EC ICI decryptor and static RIO resource extractor.

The .rio.ici files use two reversible layers:
  1. sub_10025F60: 32-byte PRNG/XOR blocks with 16-bit checksums
  2. sub_10014260: inverse column/delta permutation

The resulting MFC archive contains CObjectArcMan and CInstallSource
metadata. CObjectArcMan field_48 and field_28 statically locate the
compact COceanNode root archive in the .rio volume. The RIO directory
and resource locator records are serialized, not PRNG-encrypted.
"""

from __future__ import annotations

import argparse
import json
import mmap
import re
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ICI_HEADER_XOR1 = 0xC92E568B
ICI_HEADER_XOR2 = 0xC92E568F
ICI_SEED = 0xB29D5A0C
ICI_PRNG_CONST = 0xA3B376C9

PERM_XOR_COLS = (0x18, 0x3F, 0xE2)
PERM_DELTA_XOR = 0xA5

SIG_ICI = 0x673CE92A
SIG_RIO_NAKED = 0x1EDB927C

# CUuiGlobals (UnivUI_901 + 0x30/+0x34), initialized by sub_10004FF0.
RESOURCE_KEY_A = 0xA2FB6AD1
RESOURCE_KEY_B = 0xE7B5D9F8
RIO_SCRIPT_KEY = 0x7E6B8CE2
CRSA_HEADER_SIZE = 12

# sub_1000D230 compact CRuntimeClass-name alphabet.
CLASS_NAME_TABLE_5 = b"eaitrosducmnSglR"
CLASS_NAME_TABLE_6 = b"\x01COFLfBMxphyAVbI"
CLASS_NAME_TABLE_7 = b"EHTDPWXkqvNjwGz02U_K15JQZ467839\x00"

# External locators observed in the serialized object streams. 0xC118
# differs by bit 0x10 and is used by COptimizedObs in the second game.
DIRECT_LOCATOR_FLAGS = (0xC108, 0xC118, 0xC308)
MAX_COMPACT_CLASS_REFERENCE = 0x100


class FormatError(ValueError):
    pass


def prng_next(key: int) -> int:
    """Exact 32-bit step from sub_10025F60."""
    bit15 = (key >> 15) & 1
    return ~(bit15 + 2 * key + ICI_PRNG_CONST) & 0xFFFFFFFF


def decrypt_prng_layer(data: bytes) -> tuple[bytes, int]:
    if len(data) < 8:
        raise FormatError("ICI file is shorter than its header")

    enc_count1, enc_count2 = struct.unpack_from("<II", data)
    count1 = ~(enc_count1 ^ ICI_HEADER_XOR1) & 0xFFFFFFFF
    count2 = (enc_count2 ^ ICI_HEADER_XOR2) >> 3
    if count1 != count2:
        raise FormatError(
            f"header count mismatch: first={count1}, second={count2}"
        )

    expected_size = 8 + count1 + 2 * ((count1 + 31) // 32)
    if len(data) != expected_size:
        raise FormatError(
            f"file size mismatch: expected {expected_size}, got {len(data)}"
        )

    key = ICI_SEED
    source_pos = 8
    remaining = count1
    block_index = 0
    output = bytearray()

    while remaining:
        block_size = min(32, remaining)
        checksum = 0
        for weight in range(block_size, 0, -1):
            value = data[source_pos] ^ (key & 0xFF)
            source_pos += 1
            output.append(value)
            checksum += weight * value
            key = prng_next(key)

        stored_checksum = struct.unpack_from("<H", data, source_pos)[0]
        source_pos += 2
        if (checksum & 0xFFFF) != stored_checksum:
            raise FormatError(
                f"checksum mismatch in block {block_index}: "
                f"calculated=0x{checksum & 0xFFFF:04X}, "
                f"stored=0x{stored_checksum:04X}"
            )

        block_index += 1
        remaining -= block_size

    return bytes(output), block_index


def _deinterleave_columns(data: bytes | bytearray, stride: int) -> bytearray:
    """Undo the fixed-width column layout used by sub_10014010."""
    size = len(data)
    rows, tail = divmod(size, stride)
    output = bytearray(size)

    for row in range(rows):
        for column in range(stride):
            output[stride * row + column] = data[row + column * rows]

    output[stride * rows :] = data[stride * rows : stride * rows + tail]
    return output


def decrypt_permutation_layer(data: bytes) -> bytes:
    """Exact inverse implemented by sub_10014260."""
    stage = _deinterleave_columns(data, 6)

    previous = 0
    for index, value in enumerate(stage):
        original = value
        stage[index] = ((value - previous) & 0xFF) ^ PERM_DELTA_XOR
        previous = original

    stage = _deinterleave_columns(stage, 5)

    previous = 0
    for index in range(len(stage) - 1, -1, -1):
        original = stage[index]
        stage[index] = (original - previous) & 0xFF
        previous = original

    stage = _deinterleave_columns(stage, 3)
    full_rows = len(stage) // 3
    for row in range(full_rows):
        base = row * 3
        stage[base] ^= PERM_XOR_COLS[0]
        stage[base + 1] ^= PERM_XOR_COLS[1]
        stage[base + 2] ^= PERM_XOR_COLS[2]

    return bytes(stage)


@dataclass
class DecryptionResult:
    plaintext: bytes
    encrypted_payload_size: int
    verified_blocks: int


def decrypt_ici(path: Path) -> DecryptionResult:
    first_layer, block_count = decrypt_prng_layer(path.read_bytes())
    plaintext = decrypt_permutation_layer(first_layer)
    return DecryptionResult(plaintext, len(first_layer), block_count)


class Reader:
    def __init__(self, data: bytes, position: int = 0):
        self.data = data
        self.position = position

    def read(self, size: int) -> bytes:
        end = self.position + size
        if size < 0 or end > len(self.data):
            raise FormatError(
                f"read past end at 0x{self.position:X} (size 0x{size:X})"
            )
        value = self.data[self.position:end]
        self.position = end
        return value

    def u8(self) -> int:
        return self.read(1)[0]

    def u16(self) -> int:
        return struct.unpack("<H", self.read(2))[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.read(4))[0]

    def u64(self) -> int:
        return struct.unpack("<Q", self.read(8))[0]

    def count(self) -> int:
        value = self.u16()
        return self.u32() if value == 0xFFFF else value

    def cstring_bytes(self) -> bytes:
        length = self.u8()
        if length == 0xFF:
            length = self.u16()
            if length == 0xFFFF:
                length = self.u32()
        return self.read(length)

    def cstring(self) -> str:
        value = self.cstring_bytes()
        try:
            text = value.decode("cp932")
        except UnicodeDecodeError:
            return value.hex()
        if all(character.isprintable() for character in text):
            return text
        return value.hex()


class _BitReader:
    def __init__(self, data: bytes):
        self.data = data
        self.position = 0

    def read(self, count: int) -> int:
        if self.position + count > len(self.data) * 8:
            raise EOFError
        value = 0
        for bit_index in range(count):
            byte_index, shift = divmod(self.position, 8)
            value |= ((self.data[byte_index] >> shift) & 1) << bit_index
            self.position += 1
        return value


def decode_compact_class_name(data: bytes) -> str:
    """Decode the bit-packed CRuntimeClass name from sub_1000D230."""
    bits = _BitReader(data)
    output = bytearray()

    def append(value: int) -> None:
        if value == 0:
            raise StopIteration
        output.append(value)

    try:
        if bits.read(1) == 0:
            append(ord("C"))
        while True:
            if bits.read(1) == 0:
                append(CLASS_NAME_TABLE_5[bits.read(4)])
            elif bits.read(1) == 0:
                index = bits.read(4)
                append(
                    CLASS_NAME_TABLE_6[index]
                    if index
                    else bits.read(8)
                )
            else:
                append(CLASS_NAME_TABLE_7[bits.read(5)])
    except (EOFError, StopIteration, IndexError):
        pass

    try:
        return output.decode("ascii")
    except UnicodeDecodeError as error:
        raise FormatError("invalid compact runtime class name") from error


def _parse_install_source(reader: Reader) -> dict[str, Any]:
    start = reader.position
    version = reader.u16()
    if version != 6:
        raise FormatError(f"unsupported CInstallSource version {version}")

    result: dict[str, Any] = {
        "offset": start,
        "version": version,
        "source_name": reader.cstring(),
        "display_name": reader.cstring(),
        "source_volume": reader.cstring(),
        "string_68": reader.cstring(),
        "string_72": reader.cstring(),
        "range_start": reader.u64(),
        "range_size": reader.u64(),
        "field_128": reader.u32(),
        "volume_name": reader.cstring(),
        "volume_start": reader.u64(),
        "volume_size": reader.u64(),
        "field_60": reader.u32(),
        "description": reader.cstring(),
        "field_44": reader.u32(),
        "field_48": reader.u32(),
        "field_52": reader.u32(),
        "field_168": reader.u32(),
        "field_172": reader.u32(),
        "string_28": reader.cstring(),
    }

    dword_count = reader.count()
    result["dword_table"] = [reader.u32() for _ in range(dword_count)]

    block_size = 0x10000
    block_count = (result["volume_size"] + block_size - 1) // block_size
    bitmap_size = (block_count + 7) // 8
    bitmap_offset = reader.position
    bitmap = reader.read(bitmap_size)
    result.update(
        {
            "block_size": block_size,
            "block_count": block_count,
            "bitmap_offset": bitmap_offset,
            "bitmap_size": bitmap_size,
            "bitmap_nonzero_bytes": sum(value != 0 for value in bitmap),
            "end_offset": reader.position,
        }
    )
    return result


def parse_ici_metadata(data: bytes) -> dict[str, Any]:
    reader = Reader(data)
    signature = reader.u32()
    if signature != SIG_ICI:
        raise FormatError(
            f"unexpected plaintext signature 0x{signature:08X}"
        )

    archive_version = reader.u16()
    archive_flags = reader.u16()
    class_tag = reader.u16()
    class_schema = reader.u16()
    class_name_length = reader.u16()
    class_name = reader.read(class_name_length).decode("ascii")
    if class_name != "CObjectArcMan":
        raise FormatError(f"unexpected runtime class {class_name!r}")

    # The MFC object header has two zero bytes after this class name.
    object_start = data.find(struct.pack("<I", 10), reader.position, reader.position + 8)
    if object_start < 0:
        raise FormatError("could not locate CObjectArcMan version field")
    reader.position = object_start

    object_version = reader.u32()
    result: dict[str, Any] = {
        "signature": f"0x{signature:08X}",
        "archive_version": archive_version,
        "archive_flags": archive_flags,
        "class_tag": f"0x{class_tag:04X}",
        "class_schema": class_schema,
        "runtime_class": class_name,
        "object_offset": object_start,
        "object_version": object_version,
        "field_20": reader.u32(),
        "field_24": reader.u8(),
        "field_25": reader.u8(),
    }
    if object_version >= 10:
        result["field_28"] = reader.u32()
        result["field_32"] = reader.u32()

    result.update(
        {
            "field_36": reader.u32(),
            "field_40": reader.u32(),
            "field_44": reader.u32(),
        }
    )
    if object_version >= 6:
        result["field_48"] = reader.u32()
        result["field_52"] = reader.u32()
        result["field_168"] = reader.u32()
    if object_version >= 8:
        result["field_56"] = reader.u32()

    result.update(
        {
            "package_name": reader.cstring(),
            "field_112": reader.u32(),
            "install_path": reader.cstring(),
            "field_64": reader.u32(),
            "identifier": reader.cstring(),
            "source_root": reader.cstring(),
            "title": reader.cstring(),
            "field_120": reader.u32(),
            "string_124": reader.cstring(),
        }
    )

    string_count = reader.count()
    result["string_array"] = [reader.cstring() for _ in range(string_count)]
    result["field_104"] = reader.u32()
    if object_version >= 9:
        result["primary_volume"] = reader.cstring()
    if object_version >= 7:
        result["manual_name"] = reader.cstring()
    if object_version >= 5:
        result["field_152"] = reader.u32()

    source_count = reader.count()
    sources = []
    for _ in range(source_count):
        present = reader.u8()
        sources.append(_parse_install_source(reader) if present else None)

    result["install_sources"] = sources
    result["serialized_size"] = reader.position
    result["padding_size"] = len(data) - reader.position
    return result


def decode_resource_address(encoded: int, key: int = RESOURCE_KEY_A) -> int:
    """Decode COceanNode+0x20 to its logical address-unit index."""
    return (encoded - key) & 0xFFFFFFFF


def encode_resource_address(
    logical_address: int,
    key: int = RESOURCE_KEY_A,
) -> int:
    """Encode a logical address for COceanNode+0x20."""
    if not 0 <= logical_address <= 0xFFFFFFFF:
        raise ValueError("logical address must fit in 32 bits")
    return (key + logical_address) & 0xFFFFFFFF


# Backward-compatible name retained for callers of the earlier script.
decode_resource_block = decode_resource_address


def decode_resource_size(encoded: int, key: int = RESOURCE_KEY_B) -> int:
    """Decode the 31-bit COceanNode+0x24 resource length."""
    value = (encoded - key) & 0xFFFFFFFF
    upper = value >> 13
    return (upper | ((value - (upper & 0xFFF)) << 19)) & 0x7FFFFFFF


def encode_resource_size(size: int, key: int = RESOURCE_KEY_B) -> int:
    if not 0 <= size <= 0x7FFFFFFF:
        raise ValueError("resource size must fit in 31 bits")
    transformed = ((size << 13) & 0xFFFFFFFF) | (
        (size & 0xFFF) + ((size >> 19) & 0xFFF)
    )
    return (key + transformed) & 0xFFFFFFFF


def map_logical_address(
    logical_address: int,
    address_shift: int,
    install_sources: list[dict[str, Any] | None],
) -> tuple[dict[str, Any], int]:
    """Implement sub_10014C60's cumulative-volume address selection."""
    unit_size = 1 << address_shift
    local_address = logical_address
    for source in install_sources:
        if source is None:
            continue
        source_units = (source["volume_size"] + unit_size - 1) >> address_shift
        if local_address < source_units:
            return source, local_address << address_shift
        local_address -= source_units
    raise FormatError(
        f"logical RIO address 0x{logical_address:X} is outside all volumes"
    )


class _ArchiveLoadContext:
    """Track the shared CArchive load-array indices used by class tags."""

    def __init__(self) -> None:
        # MFC reserves indices 0 and 1 before the first loaded class.
        self.next_index = 2
        self.entries: dict[int, dict[str, Any]] = {}

    def register(self, entry: dict[str, Any]) -> int:
        index = self.next_index
        self.next_index += 1
        self.entries[index] = entry
        return index

    def reserve(self, kind: str) -> int:
        return self.register({"kind": kind})

    def resolve_class(self, index: int) -> dict[str, Any] | None:
        entry = self.entries.get(index)
        if entry is None or entry.get("kind") != "new":
            return None
        return entry


def _read_runtime_class(
    reader: Reader,
    archive: _ArchiveLoadContext | None = None,
) -> dict[str, Any]:
    tag = reader.u16()
    if tag == 0xFFFF:
        schema = reader.u16()
        length = reader.u8()
        if length == 0xFF:
            length = reader.u16()
        packed_name = reader.read(length)
        result = {
            "kind": "new",
            "tag": "0xFFFF",
            "schema": schema,
            "name": decode_compact_class_name(packed_name),
            "packed_name": packed_name.hex(),
        }
        if archive is not None:
            result["archive_index"] = archive.register(result)
        return result
    if tag == 0x7FFF:
        extended = reader.u32()
        result = {
            "kind": "reference",
            "tag": f"0x{extended:08X}",
            "reference": extended & 0x7FFFFFFF,
        }
    elif tag & 0x8000:
        result = {
            "kind": "reference",
            "tag": f"0x{tag:04X}",
            "reference": tag & 0x7FFF,
        }
    else:
        result = {
            "kind": "object_reference",
            "tag": f"0x{tag:04X}",
            "reference": tag,
        }

    if archive is not None and result["kind"] == "reference":
        resolved = archive.resolve_class(result["reference"])
        if resolved is not None:
            result["resolved_name"] = resolved["name"]
            result["resolved_schema"] = resolved["schema"]
    return result


def _runtime_class_label(runtime_class: dict[str, Any]) -> str:
    if runtime_class["kind"] == "new":
        return runtime_class["name"]
    if "resolved_name" in runtime_class:
        return runtime_class["resolved_name"]
    return f"class_ref_{runtime_class['reference']:04X}"


def _parse_rio_node(
    reader: Reader,
    address_shift: int,
    install_sources: list[dict[str, Any] | None],
    naked_archive: bool,
    archive: _ArchiveLoadContext,
    depth: int = 0,
) -> dict[str, Any]:
    if depth > 256:
        raise FormatError("RIO node tree exceeds maximum nesting depth")

    serialized_offset = reader.position
    flags = reader.u16()
    low_kind = flags & 7
    object_id: int | None = None
    if low_kind:
        if low_kind != 1:
            raise FormatError(
                f"unsupported node kind {low_kind} at 0x{serialized_offset:X}"
            )
        object_id = reader.u32()
        object_type = reader.u16()
        if object_type != 7767:
            raise FormatError(
                f"unsupported node object type {object_type} "
                f"at 0x{serialized_offset:X}"
            )
        runtime_class = _read_runtime_class(reader, archive)
        type_id: int | None = None
    else:
        type_id = reader.u8() if flags & 0x8000 else reader.u16()
        object_type = None
        runtime_class = _read_runtime_class(reader, archive)

    node: dict[str, Any] = {
        "serialized_offset": serialized_offset,
        "flags": f"0x{flags:04X}",
        "type_id": type_id,
        "object_id": object_id,
        "object_type": object_type,
        "runtime_class": runtime_class,
        "class_name": _runtime_class_label(runtime_class),
    }

    if flags & 8:
        encoded_address = reader.u32()
        encoded_size = reader.u32()
        logical_address = decode_resource_address(encoded_address)
        resource_size = decode_resource_size(encoded_size)
        source, volume_offset = map_logical_address(
            logical_address, address_shift, install_sources
        )
        node.update(
            {
                "encoded_address": f"0x{encoded_address:08X}",
                "logical_address": logical_address,
                "volume_name": source["volume_name"],
                "volume_offset": volume_offset,
                "resource_size": resource_size,
            }
        )

    children = []
    if not (naked_archive and flags & 0x200):
        child_count = reader.count()
        if child_count > 1_000_000:
            raise FormatError(
                f"implausible child count {child_count} "
                f"at 0x{reader.position:X}"
            )
        children = [
            _parse_rio_node(
                reader,
                address_shift,
                install_sources,
                naked_archive,
                archive,
                depth + 1,
            )
            for _ in range(child_count)
        ]
    node["children"] = children
    return node


def parse_rio_root(
    volume_path: Path,
    metadata: dict[str, Any],
) -> dict[str, Any]:
    address_shift = metadata["field_28"]
    root_logical_address = metadata["field_48"]
    source, root_offset = map_logical_address(
        root_logical_address,
        address_shift,
        metadata["install_sources"],
    )
    expected_path = volume_path.parent / source["volume_name"]
    if expected_path.resolve() != volume_path.resolve():
        volume_path = expected_path

    directory_limit = metadata["field_52"]
    if directory_limit <= 0:
        raise FormatError("CObjectArcMan field_52 does not bound the root directory")
    with volume_path.open("rb") as stream:
        stream.seek(root_offset)
        directory_data = stream.read(directory_limit)
    if len(directory_data) != directory_limit:
        raise FormatError("RIO root directory extends past the volume")

    reader = Reader(directory_data)
    signature = reader.u32()
    archive_version = reader.u16()
    archive_flags = reader.u16() if archive_version >= 17 else 0
    archive = _ArchiveLoadContext()
    root_class = _read_runtime_class(reader, archive)
    root_object_index = archive.reserve("root_object")
    naked_archive = signature == SIG_RIO_NAKED
    if not naked_archive:
        raise FormatError(f"unexpected RIO root signature 0x{signature:08X}")

    child_count = reader.count()
    children = [
        _parse_rio_node(
            reader,
            address_shift,
            metadata["install_sources"],
            naked_archive,
            archive,
        )
        for _ in range(child_count)
    ]
    return {
        "volume": str(volume_path),
        "root_logical_address": root_logical_address,
        "root_offset": root_offset,
        "resource_area_offset": root_offset + directory_limit,
        "directory_limit": directory_limit,
        "directory_size": reader.position,
        "directory_padding": directory_limit - reader.position,
        "signature": f"0x{signature:08X}",
        "archive_version": archive_version,
        "archive_flags": archive_flags,
        "runtime_class": root_class,
        "root_object_archive_index": root_object_index,
        "archive_class_table": {
            str(index): entry["name"]
            for index, entry in archive.entries.items()
            if entry.get("kind") == "new"
        },
        "child_count": child_count,
        "children": children,
    }


def _find_new_class_before(
    data: mmap.mmap,
    flag_offset: int,
) -> dict[str, Any] | None:
    max_length = min(80, flag_offset - 5)
    for length in range(1, max_length + 1):
        start = flag_offset - 5 - length
        if data[start : start + 2] != b"\xFF\xFF":
            continue
        if data[start + 4] != length:
            continue
        name = decode_compact_class_name(data[start + 5 : flag_offset])
        if (
            len(name) > 1
            and name.startswith("C")
            and name.isascii()
            and name.isprintable()
        ):
            return {
                "descriptor_offset": start,
                "schema": struct.unpack_from("<H", data, start + 2)[0],
                "class_name": name,
            }
    return None


def scan_rio_resource_locators(
    volume_path: Path,
    metadata: dict[str, Any],
) -> dict[str, Any]:
    """
    Find direct CPmArchive node locators.

    A record is accepted only when a valid encoded address/size follows
    a known external-locator flag and the flags are preceded by either a
    decodable new CRuntimeClass descriptor or a compact high-bit class
    reference.
    """
    volume_size = volume_path.stat().st_size
    records: list[dict[str, Any]] = []
    with volume_path.open("rb") as stream:
        data = mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ)
        try:
            for flags in DIRECT_LOCATOR_FLAGS:
                pattern = struct.pack("<H", flags)
                position = 0
                while True:
                    position = data.find(pattern, position)
                    if position < 2 or position + 10 > volume_size:
                        break

                    encoded_address, encoded_size = struct.unpack_from(
                        "<II", data, position + 2
                    )
                    logical_address = decode_resource_address(encoded_address)
                    resource_size = decode_resource_size(encoded_size)
                    try:
                        target_source, volume_offset = map_logical_address(
                            logical_address,
                            metadata["field_28"],
                            metadata["install_sources"],
                        )
                    except FormatError:
                        valid_range = False
                    else:
                        valid_range = (
                            volume_offset < target_source["volume_size"]
                            and 0
                            < resource_size
                            <= target_source["volume_size"] - volume_offset
                        )
                    if valid_range:
                        new_class = _find_new_class_before(data, position)
                        class_tag = struct.unpack_from("<H", data, position - 2)[0]
                        if new_class is not None:
                            record = {
                                **new_class,
                                "class_kind": "new",
                            }
                        elif (
                            class_tag & 0x8000
                            and (class_tag & 0x7FFF)
                            <= MAX_COMPACT_CLASS_REFERENCE
                        ):
                            record = {
                                "descriptor_offset": position - 2,
                                "class_kind": "reference",
                                "class_reference": class_tag & 0x7FFF,
                                "class_name": f"class_ref_{class_tag & 0x7FFF:04X}",
                            }
                        else:
                            position += 1
                            continue
                        record.update(
                            {
                                "record_offset": position,
                                "flags": f"0x{flags:04X}",
                                "encoded_address": f"0x{encoded_address:08X}",
                                "encoded_size": f"0x{encoded_size:08X}",
                                "logical_address": logical_address,
                                "volume_name": target_source["volume_name"],
                                "volume_offset": volume_offset,
                                "resource_size": resource_size,
                            }
                        )
                        records.append(record)
                    position += 1
        finally:
            data.close()

    # Prefer named class records for duplicate spans.
    unique: dict[tuple[int, int], dict[str, Any]] = {}
    for record in records:
        key = (
            record["volume_name"],
            record["volume_offset"],
            record["resource_size"],
        )
        previous = unique.get(key)
        if previous is None or (
            previous["class_kind"] == "reference"
            and record["class_kind"] == "new"
        ):
            unique[key] = record

    named_spans = [
        (
            record["volume_name"],
            record["volume_offset"],
            record["volume_offset"] + record["resource_size"],
        )
        for record in unique.values()
        if record["class_kind"] == "new"
    ]
    selected = []
    rejected_overlaps = 0
    selected_reference_spans: list[tuple[str, int, int]] = []
    for record in sorted(
        unique.values(),
        key=lambda item: (
            item["class_kind"] != "new",
            item["volume_offset"],
            item["resource_size"],
        ),
    ):
        start = record["volume_offset"]
        end = start + record["resource_size"]
        if record["class_kind"] == "reference":
            if any(
                record["volume_name"] == volume
                and start < right
                and left < end
                for volume, left, right in named_spans
            ):
                rejected_overlaps += 1
                continue
            if any(
                record["volume_name"] == volume
                and start < right
                and left < end
                for volume, left, right in selected_reference_spans
            ):
                rejected_overlaps += 1
                continue
            selected_reference_spans.append(
                (record["volume_name"], start, end)
            )
        selected.append(record)

    selected.sort(key=lambda item: (item["volume_offset"], item["resource_size"]))
    return {
        "volume": str(volume_path),
        "raw_record_count": len(records),
        "unique_record_count": len(unique),
        "selected_record_count": len(selected),
        "named_record_count": sum(
            record["class_kind"] == "new" for record in selected
        ),
        "reference_record_count": sum(
            record["class_kind"] == "reference" for record in selected
        ),
        "rejected_overlap_count": rejected_overlaps,
        "selected_bytes": sum(record["resource_size"] for record in selected),
        "records": selected,
    }


def collect_rio_root_resources(root: dict[str, Any]) -> list[dict[str, Any]]:
    """Flatten external resource locators parsed from the root directory."""
    records: list[dict[str, Any]] = []

    def visit(
        nodes: list[dict[str, Any]],
        parent_path: tuple[str, ...] = (),
    ) -> None:
        for index, node in enumerate(nodes):
            component = (
                f"{index:03d}_{_safe_extension(node['class_name'])}"
            )
            tree_path = (*parent_path, component)
            if "volume_offset" in node and node["resource_size"] > 0:
                runtime_class = node["runtime_class"]
                record = {
                    key: value
                    for key, value in node.items()
                    if key not in {"children", "runtime_class"}
                }
                record["record_offset"] = (
                    root["root_offset"] + node["serialized_offset"]
                )
                record["class_kind"] = runtime_class["kind"]
                if "reference" in runtime_class:
                    record["class_reference"] = runtime_class["reference"]
                record["origins"] = ["root_tree"]
                record["tree_path"] = "/".join(tree_path)
                records.append(record)
            visit(node["children"], tree_path)

    visit(root["children"])
    return records


def merge_rio_resource_records(
    root: dict[str, Any],
    scan: dict[str, Any],
) -> dict[str, Any]:
    """
    Merge root-directory locators with locators found in later streams.

    The raw volume scan cannot see root records because root nodes serialize
    flags before their class descriptor. Conversely, parsing only the root
    misses locator records embedded in later serialized object streams.
    """
    root_records = collect_rio_root_resources(root)
    root_spans = {
        (
            record["volume_name"],
            record["volume_offset"],
            record["resource_size"],
        )
        for record in root_records
    }
    scan_spans = {
        (
            record["volume_name"],
            record["volume_offset"],
            record["resource_size"],
        )
        for record in scan["records"]
    }

    merged: dict[tuple[str, int, int], dict[str, Any]] = {}
    for origin, records in (
        ("root_tree", root_records),
        ("volume_scan", scan["records"]),
    ):
        for source_record in records:
            record = dict(source_record)
            key = (
                record["volume_name"],
                record["volume_offset"],
                record["resource_size"],
            )
            class_name = record["class_name"]
            existing = merged.get(key)
            if existing is None:
                record["origins"] = list(
                    dict.fromkeys(record.get("origins", [origin]))
                )
                record["class_aliases"] = [class_name]
                record["tree_paths"] = (
                    [record["tree_path"]] if "tree_path" in record else []
                )
                record["locator_offsets"] = [
                    {
                        "origin": origin,
                        "record_offset": record["record_offset"],
                    }
                ]
                merged[key] = record
                continue

            origins = list(existing.get("origins", []))
            if origin not in origins:
                origins.append(origin)
            aliases = list(existing.get("class_aliases", []))
            if class_name not in aliases:
                aliases.append(class_name)
            tree_paths = list(existing.get("tree_paths", []))
            candidate_tree_path = record.get("tree_path")
            if (
                candidate_tree_path is not None
                and candidate_tree_path not in tree_paths
            ):
                tree_paths.append(candidate_tree_path)
            locator_offsets = list(existing.get("locator_offsets", []))
            locator = {
                "origin": origin,
                "record_offset": record["record_offset"],
            }
            if locator not in locator_offsets:
                locator_offsets.append(locator)

            existing_is_reference = existing["class_name"].startswith(
                "class_ref_"
            )
            candidate_is_named = not class_name.startswith("class_ref_")
            if existing_is_reference and candidate_is_named:
                record["origins"] = origins
                record["class_aliases"] = aliases
                record["tree_paths"] = tree_paths
                record["locator_offsets"] = locator_offsets
                merged[key] = record
            else:
                existing["origins"] = origins
                existing["class_aliases"] = aliases
                existing["tree_paths"] = tree_paths
                existing["locator_offsets"] = locator_offsets

    records = sorted(
        merged.values(),
        key=lambda item: (
            item["volume_name"],
            item["volume_offset"],
            item["resource_size"],
        ),
    )
    return {
        "volume": scan["volume"],
        "root_record_count": len(root_records),
        "scan_record_count": len(scan["records"]),
        "shared_record_count": len(root_spans & scan_spans),
        "root_only_record_count": len(root_spans - scan_spans),
        "scan_only_record_count": len(scan_spans - root_spans),
        "selected_record_count": len(records),
        "named_record_count": sum(
            not record["class_name"].startswith("class_ref_")
            for record in records
        ),
        "reference_record_count": sum(
            record["class_name"].startswith("class_ref_")
            for record in records
        ),
        "selected_bytes": sum(record["resource_size"] for record in records),
        "records": records,
    }


def _encrypted_payload_header(
    data: bytes,
    offset: int = 0,
) -> tuple[int, int]:
    if len(data) < offset + 8:
        raise FormatError("encrypted payload is shorter than its header")
    encoded_size1, encoded_size2 = struct.unpack_from("<II", data, offset)
    size1 = ~(encoded_size1 ^ ICI_HEADER_XOR1) & 0xFFFFFFFF
    size2 = (encoded_size2 ^ ICI_HEADER_XOR2) >> 3
    if size1 != size2:
        raise FormatError("encrypted payload header mismatch")
    encoded_size = 8 + size1 + 2 * ((size1 + 31) // 32)
    return size1, encoded_size


def _encrypted_payload_layout(
    data: bytes,
    offset: int = 0,
) -> tuple[int, int, int]:
    size1, encoded_size = _encrypted_payload_header(data, offset)
    return size1, encoded_size, len(data) - offset - encoded_size


def decrypt_checked_payload(
    data: bytes,
    key: int,
    offset: int = 0,
) -> bytes:
    """Decrypt the checked 32-byte PRNG stream used by CRsa."""
    plaintext_size, encoded_size, trailing_size = _encrypted_payload_layout(
        data, offset
    )
    if trailing_size < 0:
        raise FormatError("encrypted payload is truncated")

    source_pos = offset + 8
    remaining = plaintext_size
    output = bytearray()
    block_index = 0
    while remaining:
        block_size = min(32, remaining)
        checksum = 0
        for weight in range(block_size, 0, -1):
            value = data[source_pos] ^ (key & 0xFF)
            source_pos += 1
            output.append(value)
            checksum += weight * value
            key = prng_next(key)

        stored_checksum = struct.unpack_from("<H", data, source_pos)[0]
        source_pos += 2
        if (checksum & 0xFFFF) != stored_checksum:
            raise FormatError(
                f"checksum mismatch in encrypted block {block_index}"
            )
        block_index += 1
        remaining -= block_size

    if source_pos != offset + encoded_size:
        raise FormatError("encrypted payload size accounting mismatch")
    return bytes(output)


def encrypt_checked_payload(plaintext: bytes, key: int) -> bytes:
    """Encode the checked PRNG stream used by CRsa and ICI archives."""
    plaintext_size = len(plaintext)
    output = bytearray(
        struct.pack(
            "<II",
            ((~plaintext_size) & 0xFFFFFFFF) ^ ICI_HEADER_XOR1,
            ((plaintext_size << 3) & 0xFFFFFFFF) ^ ICI_HEADER_XOR1,
        )
    )
    source_pos = 0
    remaining = plaintext_size
    while remaining:
        block_size = min(32, remaining)
        checksum = 0
        for weight in range(block_size, 0, -1):
            value = plaintext[source_pos]
            source_pos += 1
            output.append(value ^ (key & 0xFF))
            checksum += weight * value
            key = prng_next(key)
        output.extend(struct.pack("<H", checksum & 0xFFFF))
        remaining -= block_size
    return bytes(output)


def decrypt_crsa_script(data: bytes) -> dict[str, Any]:
    """Decrypt a serialized CRsa container without loading the game runtime."""
    if len(data) < CRSA_HEADER_SIZE + 8:
        raise FormatError("CRsa container is too small")
    plaintext_size, encoded_size, trailing_size = _encrypted_payload_layout(
        data, CRSA_HEADER_SIZE
    )
    if trailing_size < 0 or trailing_size > 16:
        raise FormatError("CRsa encrypted stream does not fit its container")

    plaintext = decrypt_checked_payload(
        data,
        RIO_SCRIPT_KEY,
        CRSA_HEADER_SIZE,
    )
    if len(plaintext) < 8:
        raise FormatError("CRsa plaintext is too small")
    archive_version = struct.unpack_from("<H", plaintext)[0]
    if not 1 <= archive_version <= 0x100:
        raise FormatError("CRsa plaintext has an implausible archive version")

    return {
        "outer_header": data[:CRSA_HEADER_SIZE].hex(),
        "plaintext_size": plaintext_size,
        "encoded_size": encoded_size,
        "trailing_size": trailing_size,
        "archive_version": archive_version,
        "plaintext": plaintext,
    }


def rebuild_crsa_script(
    data: bytes,
    plaintext: bytes,
    *,
    require_same_size: bool = True,
) -> bytes:
    """Re-encrypt a CRsa plaintext while preserving its outer fields."""
    original = decrypt_crsa_script(data)
    if require_same_size and len(plaintext) != original["plaintext_size"]:
        raise FormatError(
            "in-place CRsa rebuilding requires the original plaintext size"
        )
    payload_end = CRSA_HEADER_SIZE + original["encoded_size"]
    rebuilt = (
        data[:CRSA_HEADER_SIZE]
        + encrypt_checked_payload(plaintext, RIO_SCRIPT_KEY)
        + data[payload_end:]
    )
    if require_same_size and len(rebuilt) != len(data):
        raise FormatError("rebuilt CRsa container size changed")
    return rebuilt


def _script_string_kind(text: str) -> str | None:
    if text.startswith("&-O"):
        return "engine_command"
    if re.fullmatch(r"(?:[A-Z0-9]+_)+[^\s]+", text):
        return "identifier"
    japanese_count = sum(
        "\u3040" <= character <= "\u30ff"
        or "\u4e00" <= character <= "\u9fff"
        for character in text
    )
    if japanese_count >= 2:
        return "text"
    if japanese_count >= 1 and any(
        marker in text for marker in "、。！？「」@●\n"
    ):
        return "text"
    if text.startswith(("@", "!", "●")) and len(text) >= 4:
        return "text"
    return None


def _render_script_text(text: str) -> str:
    rendered = []
    for character in text:
        value = ord(character)
        if value < 0x20 and character not in "\r\n":
            rendered.append(f"{{{value}}}")
        else:
            rendered.append(character)
    return "".join(rendered)


def scan_crsa_strings(plaintext: bytes) -> list[dict[str, Any]]:
    """
    Find high-confidence length-prefixed strings in a decrypted CRsa stream.

    This mirrors RioX's one-byte/0xFF+WORD CString probe. VM-aware parsing can
    later refine the result; commands are retained in JSON but omitted from
    the human-readable text file.
    """
    allowed_controls = {1, 2, 3, 5, 9, 10, 13}
    candidates: list[dict[str, Any]] = []
    for prefix_offset in range(len(plaintext)):
        length = plaintext[prefix_offset]
        prefix_size = 1
        if length == 0xFF:
            if prefix_offset + 3 > len(plaintext):
                continue
            length = struct.unpack_from("<H", plaintext, prefix_offset + 1)[0]
            prefix_size = 3
            if length <= 0xFE:
                continue
        if not 3 <= length <= 0x400:
            continue

        content_offset = prefix_offset + prefix_size
        content_end = content_offset + length
        if content_end > len(plaintext):
            continue
        raw = plaintext[content_offset:content_end]
        if 0 in raw or any(
            value < 0x20 and value not in allowed_controls for value in raw
        ):
            continue
        try:
            text = raw.decode("cp932")
        except UnicodeDecodeError:
            continue
        if any("\ue000" <= character <= "\uf8ff" for character in text):
            continue

        kind = _script_string_kind(text)
        if kind is None:
            continue
        candidates.append(
            {
                "prefix_offset": prefix_offset,
                "content_offset": content_offset,
                "length": length,
                "kind": kind,
                "text": _render_script_text(text),
                "raw_hex": raw.hex(),
            }
        )

    selected = []
    for candidate in sorted(
        candidates,
        key=lambda item: (
            item["content_offset"],
            -item["length"],
        ),
    ):
        start = candidate["content_offset"]
        end = start + candidate["length"]
        if any(
            start >= previous["content_offset"]
            and end
            <= previous["content_offset"] + previous["length"]
            for previous in selected
        ):
            continue
        selected.append(candidate)
    return selected


def scan_crsa_vm_messages(plaintext: bytes) -> list[dict[str, Any]]:
    """
    Locate strings referenced by the VM message stream.

    RioX records the CString prefix offset in its ``#### offset ####``
    headings. Most messages follow a compact ``60 xx 00`` VM descriptor;
    script-entry messages and selector labels use two additional stable
    contexts.
    """
    allowed_controls = {1, 2, 3, 5, 9, 10, 13}
    script_entry_marker = bytes.fromhex(
        "94fffba231fbdce705000000"
    )
    selector_markers = {
        bytes.fromhex("d0f4c100"),
        bytes.fromhex("7cd9c100"),
    }
    messages = []
    for prefix_offset, length in enumerate(plaintext):
        if not 3 <= length <= 0xFE:
            continue
        content_offset = prefix_offset + 1
        content_end = content_offset + length
        if content_end > len(plaintext):
            continue
        raw = plaintext[content_offset:content_end]
        if 0 in raw or any(
            value < 0x20 and value not in allowed_controls for value in raw
        ):
            continue
        try:
            text = raw.decode("cp932")
        except UnicodeDecodeError:
            continue
        if any("\ue000" <= character <= "\uf8ff" for character in text):
            continue

        preceding3 = plaintext[prefix_offset - 3 : prefix_offset]
        preceding12 = plaintext[prefix_offset - 12 : prefix_offset]
        preceding32 = plaintext[max(0, prefix_offset - 32) : prefix_offset]
        if (
            len(preceding3) == 3
            and preceding3[0] == 0x60
            and preceding3[2] == 0
        ):
            context = "vm_message"
        elif preceding12 == script_entry_marker:
            context = "script_entry"
        elif (
            len(preceding12) == 12
            and preceding12[4:8] in selector_markers
        ):
            context = "selector_value"
        elif preceding32.endswith(b"&-OM_AddSelecter"):
            context = "selector_command"
        else:
            continue

        messages.append(
            {
                "prefix_offset": prefix_offset,
                "content_offset": content_offset,
                "length": length,
                "context": context,
                "text": _render_script_text(text).rstrip("\r\n"),
                "raw_hex": raw.hex(),
            }
        )
    return messages


def scan_rio_crsa_containers(
    volume_path: Path,
    address_shift: int,
) -> list[dict[str, Any]]:
    """Find every checksum-valid CRsa object embedded in a RIO volume."""
    marker = b"\x03\x01\x00\x00"
    alignment = 1 << address_shift
    records = []
    with volume_path.open("rb") as stream:
        data = mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ)
        try:
            marker_offset = 0
            while True:
                marker_offset = data.find(marker, marker_offset)
                if marker_offset < 0:
                    break
                volume_offset = marker_offset - 4
                marker_offset += 1
                if (
                    volume_offset < 0
                    or volume_offset % alignment
                    or volume_offset + CRSA_HEADER_SIZE + 8 > len(data)
                ):
                    continue
                try:
                    plaintext_size, encoded_size = _encrypted_payload_header(
                        data,
                        volume_offset + CRSA_HEADER_SIZE,
                    )
                except FormatError:
                    continue
                if not 8 <= plaintext_size <= 100_000_000:
                    continue
                container_size = CRSA_HEADER_SIZE + encoded_size
                container_end = volume_offset + container_size
                if container_end > len(data):
                    continue
                try:
                    script = decrypt_crsa_script(
                        data[volume_offset:container_end]
                    )
                except FormatError:
                    continue
                records.append(
                    {
                        "volume_name": volume_path.name,
                        "volume_offset": volume_offset,
                        "logical_address": volume_offset >> address_shift,
                        "container_size": container_size,
                        "outer_header": script["outer_header"],
                        "plaintext_size": script["plaintext_size"],
                        "encoded_size": script["encoded_size"],
                        "archive_version": script["archive_version"],
                        "plaintext": script["plaintext"],
                    }
                )
        finally:
            data.close()
    return records


def extract_rio_texts(
    volume_dir: Path,
    resources: dict[str, Any],
    output_dir: Path,
    address_shift: int,
) -> Path:
    """Scan the complete volume and export every checksum-valid CRsa."""
    output_dir.mkdir(parents=True, exist_ok=True)
    manifest_records = []
    volume_path = Path(resources["volume"])
    if not volume_path.is_file():
        volume_path = volume_dir / volume_path.name
    containers = scan_rio_crsa_containers(volume_path, address_shift)
    resource_records = resources["records"]
    for script_record in containers:
        plaintext = script_record["plaintext"]
        container = {
            key: value
            for key, value in script_record.items()
            if key != "plaintext"
        }
        strings = scan_crsa_strings(plaintext)
        messages = scan_crsa_vm_messages(plaintext)
        commands = [
            item for item in strings if item["kind"] == "engine_command"
        ]
        identifiers = [
            item for item in strings if item["kind"] == "identifier"
        ]
        owners = [
            record
            for record in resource_records
            if record["volume_name"] == container["volume_name"]
            and record["volume_offset"]
            <= container["volume_offset"]
            < record["volume_offset"] + record["resource_size"]
        ]
        owner = (
            min(owners, key=lambda item: item["resource_size"])
            if owners
            else None
        )
        base_name = f"{container['logical_address']:08X}"
        binary_path = output_dir / f"{base_name}.script.bin"
        text_path = output_dir / f"{base_name}.txt"
        json_path = output_dir / f"{base_name}.text.json"
        binary_path.write_bytes(plaintext)
        text_path.write_text(
            "\n\n".join(
                f"#### {item['prefix_offset']:x} ####\n"
                f"{item['text']}"
                for item in messages
            )
            + ("\n" if messages else ""),
            encoding="utf-8-sig",
        )
        json_path.write_text(
            json.dumps(
                {
                    "container": container,
                    "owner_resource": owner,
                    "message_count": len(messages),
                    "engine_command_count": len(commands),
                    "identifier_count": len(identifiers),
                    "messages": messages,
                    "strings": strings,
                },
                ensure_ascii=False,
                indent=2,
            ),
            encoding="utf-8",
        )
        manifest_records.append(
            {
                **container,
                "owner_class_name": (
                    owner["class_name"] if owner is not None else None
                ),
                "owner_volume_offset": (
                    owner["volume_offset"] if owner is not None else None
                ),
                "message_count": len(messages),
                "engine_command_count": len(commands),
                "identifier_count": len(identifiers),
                "binary_file": binary_path.name,
                "text_file": text_path.name,
                "json_file": json_path.name,
            }
        )

    manifest_path = output_dir / "text_manifest.json"
    manifest_path.write_text(
        json.dumps(
            {
                "script_count": len(manifest_records),
                "text_script_count": sum(
                    record["message_count"] > 0
                    for record in manifest_records
                ),
                "message_count": sum(
                    record["message_count"]
                    for record in manifest_records
                ),
                "records": manifest_records,
            },
            ensure_ascii=False,
            indent=2,
        ),
        encoding="utf-8",
    )
    return manifest_path


def _safe_extension(class_name: str) -> str:
    if class_name.startswith("C") and not class_name.startswith("class_ref_"):
        class_name = class_name[1:]
    value = "".join(
        character.lower() if character.isalnum() else "_"
        for character in class_name
    ).strip("_")
    return value or "bin"


def extract_rio_resources(
    volume_dir: Path,
    scan: dict[str, Any],
    output_dir: Path,
) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    manifest_records = []
    streams: dict[str, Any] = {}
    try:
        for record in scan["records"]:
            offset = record["volume_offset"]
            size = record["resource_size"]
            extension = _safe_extension(record["class_name"])
            volume_prefix = _safe_extension(Path(record["volume_name"]).stem)
            output_name = (
                f"{volume_prefix}_{offset:08X}_{size:08X}.{extension}"
            )
            tree_paths = record.get("tree_paths", [])
            if tree_paths:
                relative_dir = Path("root_tree") / Path(tree_paths[0])
                classification = "root_tree"
            else:
                relative_dir = (
                    Path("serialized_objects")
                    / _safe_extension(record["class_name"])
                )
                classification = "serialized_objects"
            output_path = output_dir / relative_dir / output_name
            output_path.parent.mkdir(parents=True, exist_ok=True)
            volume_name = record["volume_name"]
            source = streams.get(volume_name)
            if source is None:
                source = (volume_dir / volume_name).open("rb")
                streams[volume_name] = source
            source.seek(offset)
            remaining = size
            with output_path.open("wb") as destination:
                while remaining:
                    block = source.read(min(1024 * 1024, remaining))
                    if not block:
                        raise FormatError(
                            f"short read extracting resource at 0x{offset:X}"
                        )
                    destination.write(block)
                    remaining -= len(block)
            manifest_record = dict(record)
            manifest_record["classification"] = classification
            manifest_record["file"] = output_path.relative_to(
                output_dir
            ).as_posix()
            manifest_records.append(manifest_record)
    finally:
        for stream in streams.values():
            stream.close()

    manifest = {
        key: value for key, value in scan.items() if key != "records"
    }
    manifest["records"] = manifest_records
    manifest_path = output_dir / "manifest.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    return manifest_path


def inspect_file(
    path: Path,
    write_plaintext: bool,
    inspect_rio: bool = False,
    extract_dir: Path | None = None,
    text_dir: Path | None = None,
) -> dict[str, Any]:
    decrypted = decrypt_ici(path)
    metadata = parse_ici_metadata(decrypted.plaintext)
    metadata.update(
        {
            "file": str(path),
            "encrypted_payload_size": decrypted.encrypted_payload_size,
            "verified_blocks": decrypted.verified_blocks,
        }
    )

    for source in metadata["install_sources"]:
        if source is None:
            continue
        volume_path = path.parent / source["volume_name"]
        source["volume_path"] = str(volume_path)
        source["volume_exists"] = volume_path.is_file()
        if source["volume_exists"]:
            source["actual_volume_size"] = volume_path.stat().st_size
            source["volume_size_matches"] = (
                source["actual_volume_size"] == source["volume_size"]
            )

    if inspect_rio or extract_dir is not None or text_dir is not None:
        source, _ = map_logical_address(
            metadata["field_48"],
            metadata["field_28"],
            metadata["install_sources"],
        )
        volume_path = path.parent / source["volume_name"]
        if not volume_path.is_file():
            raise FormatError(f"RIO volume not found: {volume_path}")
        root = parse_rio_root(volume_path, metadata)
        scan = scan_rio_resource_locators(volume_path, metadata)
        resources = merge_rio_resource_records(root, scan)
        metadata["rio_root"] = root
        metadata["rio_scan"] = scan
        metadata["rio_resources"] = resources
        if extract_dir is not None:
            target_dir = extract_dir / volume_path.stem
            metadata["manifest_file"] = str(
                extract_rio_resources(path.parent, resources, target_dir)
            )
        if text_dir is not None:
            target_dir = text_dir / volume_path.stem
            metadata["text_manifest_file"] = str(
                extract_rio_texts(
                    path.parent,
                    resources,
                    target_dir,
                    metadata["field_28"],
                )
            )

    if write_plaintext:
        output_path = path.with_suffix(path.suffix + ".dec")
        output_path.write_bytes(decrypted.plaintext)
        metadata["plaintext_file"] = str(output_path)

    return metadata


def print_summary(metadata: dict[str, Any]) -> None:
    print(f"File: {metadata['file']}")
    print(
        f"  PRNG blocks: {metadata['verified_blocks']} verified, "
        f"{metadata['encrypted_payload_size']} bytes"
    )
    print(
        f"  Archive: {metadata['signature']}, "
        f"{metadata['runtime_class']} v{metadata['object_version']}"
    )
    print(
        f"  Serialized data: 0x{metadata['serialized_size']:X} bytes; "
        f"padding: {metadata['padding_size']} bytes"
    )
    for source in metadata["install_sources"]:
        if source is None:
            print("  Install source: null")
            continue
        print(f"  Volume: {source['volume_name']}")
        print(
            f"    size={source['volume_size']:,}, "
            f"blocks={source['block_count']}, "
            f"bitmap={source['bitmap_size']} bytes "
            f"at 0x{source['bitmap_offset']:X}"
        )
        if source.get("volume_exists"):
            match = "yes" if source["volume_size_matches"] else "NO"
            print(f"    on-disk size match: {match}")
    if "plaintext_file" in metadata:
        print(f"  Plaintext: {metadata['plaintext_file']}")
    if "rio_root" in metadata:
        root = metadata["rio_root"]
        print(
            f"  RIO root: 0x{root['root_offset']:X}, "
            f"{root['child_count']} direct children, "
            f"0x{root['directory_size']:X}/0x{root['directory_limit']:X} "
            "directory bytes"
        )
        scan = metadata["rio_scan"]
        print(
            f"  Scanned resource locators: {scan['selected_record_count']} "
            f"selected ({scan['named_record_count']} named classes, "
            f"{scan['reference_record_count']} class references)"
        )
        resources = metadata["rio_resources"]
        print(
            f"  Merged resources: {resources['selected_record_count']} "
            f"({resources['root_only_record_count']} root-only, "
            f"{resources['scan_only_record_count']} scan-only, "
            f"{resources['shared_record_count']} shared)"
        )
        print(f"    selected payload bytes: {resources['selected_bytes']:,}")
    if "manifest_file" in metadata:
        print(f"  Extract manifest: {metadata['manifest_file']}")
    if "text_manifest_file" in metadata:
        print(f"  Text manifest: {metadata['text_manifest_file']}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Decrypt and inspect rUGP .rio.ici metadata"
    )
    parser.add_argument("files", nargs="*", type=Path)
    parser.add_argument(
        "--write",
        action="store_true",
        help="write each 2048-byte plaintext buffer as .ici.dec",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="print complete parsed metadata as JSON",
    )
    parser.add_argument(
        "--rio",
        action="store_true",
        help="statically parse the RIO root tree and resource locators",
    )
    parser.add_argument(
        "--extract",
        type=Path,
        help="extract statically located RIO resources under this directory",
    )
    parser.add_argument(
        "--text",
        type=Path,
        help="decrypt CRsa scripts and export high-confidence text candidates",
    )
    args = parser.parse_args()

    files = args.files or sorted(Path(__file__).resolve().parent.glob("*.rio.ici"))
    if not files:
        parser.error("no .rio.ici files found")

    try:
        extract_dir = args.extract.resolve() if args.extract else None
        text_dir = args.text.resolve() if args.text else None
        results = [
            inspect_file(
                path.resolve(),
                args.write,
                inspect_rio=args.rio,
                extract_dir=extract_dir,
                text_dir=text_dir,
            )
            for path in files
        ]
    except (OSError, FormatError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(results, ensure_ascii=False, indent=2))
    else:
        for index, result in enumerate(results):
            if index:
                print()
            print_summary(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


# ============================================================================
# Container format analyzer
# ============================================================================

def analyze_crip(data: bytes, name: str) -> dict:
    """Parse CRip container header and attempt image extraction."""
    if len(data) < 32:
        return {"error": "too small", "size": len(data)}
    
    d0 = struct.unpack_from('<I', data, 0)[0]
    d1 = struct.unpack_from('<I', data, 4)[0]
    d2 = struct.unpack_from('<I', data, 8)[0]
    wA = struct.unpack_from('<H', data, 0x0C)[0]
    wB = struct.unpack_from('<H', data, 0x0E)[0]
    
    result = {
        "magic": f"0x{d0:08X}",
        "field_04": d1,
        "field_08": f"0x{d2:08X}",
        "dim_hint_a": wA,
        "dim_hint_b": wB,
        "size": len(data),
    }
    
    # Try to find palette region (low-entropy repeating patterns)
    # Then find pixel data region (high-entropy)
    palette_start = 0x20
    palette_end = 0x400  # default
    
    # Find actual palette boundary
    prev_unique = 0
    for offset in range(0x20, min(len(data) - 64, 0x1000), 0x10):
        chunk = data[offset:offset+64]
        unique = len(set(chunk))
        if unique > 15 and prev_unique < 10:
            palette_end = offset
            break
        prev_unique = unique
    
    palette_size = palette_end - palette_start
    pixel_size = len(data) - palette_end
    
    result["palette_offset"] = palette_start
    result["palette_entries"] = palette_size // 4
    result["pixel_offset"] = palette_end
    result["pixel_bytes"] = pixel_size
    
    # Try to find reasonable image dimensions
    if pixel_size > 0:
        for w in range(4, 2048):
            h = pixel_size // w
            if h > 0 and w * h == pixel_size:
                result["guessed_dims"] = f"{w}x{h}"
                break
            if h > 0 and pixel_size - w * h < w and pixel_size - w * h >= 0:
                result["guessed_dims"] = f"{w}x{h} (approx, {pixel_size - w*h} extra bytes)"
                break
    
    return result


def analyze_optimizedobs(data: bytes, name: str) -> dict:
    """Parse COptimizedObs header."""
    if len(data) < 8:
        return {"error": "too small", "size": len(data)}
    
    tag = struct.unpack_from('<H', data, 0)[0]
    schema = struct.unpack_from('<H', data, 2)[0]
    ver = struct.unpack_from('<I', data, 4)[0]
    
    result = {
        "mfc_tag": f"0x{tag:04X}",
        "mfc_schema": schema,
        "version_dword": f"0x{ver:08X}",
        "size": len(data),
    }
    
    # After 8-byte header, the rest is payload (possibly compressed)
    payload = data[8:]
    payload_size = len(payload)
    
    # Check entropy
    unique_ratio = len(set(payload[:min(1024, payload_size)])) / min(1024, payload_size)
    result["payload_size"] = payload_size
    result["entropy_ratio"] = f"{unique_ratio:.3f}"
    
    if unique_ratio > 0.3:
        result["likely_format"] = "raw_pixels_or_compressed"
    else:
        result["likely_format"] = "structured_or_encrypted"
    
    # Try raw BGRA dimensions
    if payload_size % 4 == 0:
        px = payload_size // 4
        sq = int(px ** 0.5)
        result["guessed_bgra_dims"] = f"~{sq}x{sq}"
    
    return result


def analyze_crsa(data: bytes, name: str) -> dict:
    """Analyze CRsa script container."""
    result = {"size": len(data)}
    
    # Check if first bytes could be PRNG header
    if len(data) >= 8:
        d0 = struct.unpack_from('<I', data, 0)[0]
        d1 = struct.unpack_from('<I', data, 4)[0]
        
        # Try ICI header
        bc1 = (~(d0 ^ ICI_HEADER_XOR1)) & 0xFFFFFFFF
        bc2 = ((d1 ^ ICI_HEADER_XOR2) >> 3) & 0xFFFFFFFF
        result["ici_header_check"] = f"bc1={bc1}, bc2={bc2}, match={bc1==bc2}"
        
        # Entropy
        unique_ratio = len(set(data[:min(1024, len(data))])) / min(1024, len(data))
        result["entropy_ratio"] = f"{unique_ratio:.3f}"
        result["likely_encrypted"] = unique_ratio > 0.4
    
    return result


def analyze_container(filepath: str) -> dict:
    """Analyze a single extracted container file."""
    path = Path(filepath)
    name = path.stem
    ext = path.suffix.lower()
    
    with open(filepath, 'rb') as f:
        data = f.read()
    
    if 'optimizedobs' in ext or 'optimizedobs' in name.lower():
        return {"type": "COptimizedObs", **analyze_optimizedobs(data, name)}
    elif ext == '.rip':
        return {"type": "CRip", **analyze_crip(data, name)}
    elif 'rip007' in ext or 'rip007' in name.lower():
        return {"type": "CRip007", "size": len(data), "note": "compressed variant, needs SIA/LZSS decoder"}
    elif 'rsa' in ext or 'rsa' in name.lower():
        return {"type": "CRsa", **analyze_crsa(data, name)}
    elif 'stdb' in ext or 'stdb' in name.lower():
        return {"type": "CStdb", "size": len(data), "note": "encrypted string database"}
    else:
        return {"type": "other", "size": len(data), "extension": ext}


# Extend main to support --containers flag
def _extend_parser(parser):
    parser.add_argument("--containers", type=Path,
                       help="analyze extracted container files in this directory")
