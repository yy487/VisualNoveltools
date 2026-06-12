"""
rUGP .rio static unpacker for 螺旋回廊 (rUGP 5.80.20EC)
Complete offline resource extraction — no runtime needed.

Reference: GARbro ArcRIO.cs + UnivUI.dll reverse engineering
"""
from __future__ import annotations

import argparse
import json
import struct
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Optional

# ═══════════════════════════════════════════════════════════════
# Constants
# ═══════════════════════════════════════════════════════════════

ICI_HEADER_XOR1 = 0xC92E568B
ICI_HEADER_XOR2 = 0xC92E568F
ICI_SEED = 0xB29D5A0C
PRNG_CONST = 0xA3B376C9

RIO_KEY = 0x7E6B8CE2  # CrelicUnitedGameProject resource key (GARbro)

RESOURCE_KEY_A = 0xA2FB6AD1
RESOURCE_KEY_B = 0xE7B5D9F8

SIG_ICI = 0x673CE92A
SIG_RIO = 0x596E32CD
SIG_ENCRYPTED = 0x1EDB927C

PERM_XOR_COLS = (0x18, 0x3F, 0xE2)
PERM_DELTA_XOR = 0xA5

CLASS_NAME_TABLE_5 = b"eaitrosducmnSglR"
CLASS_NAME_TABLE_6 = b"\x01COFLfBMxphyAVbI"
CLASS_NAME_TABLE_7 = b"EHTDPWXkqvNjwGz02U_K15JQZ467839\x00"


class FormatError(ValueError):
    pass


# ═══════════════════════════════════════════════════════════════
# PRNG
# ═══════════════════════════════════════════════════════════════

def prng_next(key: int) -> int:
    bit = (key >> 15) & 1
    return ~(bit + 2 * key + PRNG_CONST) & 0xFFFFFFFF


# ═══════════════════════════════════════════════════════════════
# ICI decryption
# ═══════════════════════════════════════════════════════════════

def read_encrypted(data: bytes, key: int) -> bytes:
    """PRNG-decrypt data in rUGP block format (32-byte + 2-byte checksum)."""
    if len(data) < 8:
        raise FormatError("encrypted payload too short")
    enc1, enc2 = struct.unpack_from("<II", data)
    size = ~(enc1 ^ ICI_HEADER_XOR1) & 0xFFFFFFFF
    if size != ((enc2 ^ ICI_HEADER_XOR2) >> 3):
        raise FormatError("encrypted header mismatch")
    output = bytearray(size)
    pos, dst = 8, 0
    while dst < size:
        block = min(32, size - dst)
        checksum = 0
        for weight in range(block, 0, -1):
            b = data[pos] ^ (key & 0xFF)
            pos += 1
            output[dst] = b
            dst += 1
            checksum += b * weight
            key = prng_next(key)
        if block < 32:
            break
        stored = struct.unpack_from("<H", data, pos)[0]
        pos += 2
        if (checksum & 0xFFFF) != stored:
            raise FormatError(
                f"checksum mismatch at block near offset 0x{pos - 34:X}"
            )
    return bytes(output)


def _deinterleave(data: bytes | bytearray, stride: int) -> bytearray:
    size = len(data)
    rows, tail = divmod(size, stride)
    output = bytearray(size)
    for row in range(rows):
        for col in range(stride):
            output[stride * row + col] = data[row + col * rows]
    output[stride * rows:] = data[stride * rows : stride * rows + tail]
    return output


def decrypt_ici_permutation(data: bytes) -> bytes:
    """UTIL_DecryptIci / sub_10014260 — GARbro DecryptIci."""
    d = _deinterleave(data, 6)

    prev = 0
    for i, v in enumerate(d):
        orig = v
        d[i] = ((v - prev) & 0xFF) ^ PERM_DELTA_XOR
        prev = orig

    d = _deinterleave(d, 5)

    prev = 0
    for i in range(len(d) - 1, -1, -1):
        orig = d[i]
        d[i] = (orig - prev) & 0xFF
        prev = orig

    d = _deinterleave(d, 3)
    rows = len(d) // 3
    for row in range(rows):
        base = row * 3
        d[base] ^= PERM_XOR_COLS[0]
        d[base + 1] ^= PERM_XOR_COLS[1]
        d[base + 2] ^= PERM_XOR_COLS[2]
    return bytes(d)


def decrypt_ici_file(path: Path) -> bytes:
    return decrypt_ici_permutation(read_encrypted(path.read_bytes(), ICI_SEED))


# ═══════════════════════════════════════════════════════════════
# MFC CArchive reader
# ═══════════════════════════════════════════════════════════════

class Reader:
    def __init__(self, data: bytes, pos: int = 0):
        self._data = data
        self.pos = pos

    def __len__(self) -> int:
        return len(self._data)

    def read(self, n: int) -> bytes:
        end = self.pos + n
        if n < 0 or end > len(self._data):
            raise FormatError(f"read past end at 0x{self.pos:X}")
        v = self._data[self.pos : end]
        self.pos = end
        return v

    def u8(self) -> int:
        return self.read(1)[0]

    def u16(self) -> int:
        return struct.unpack("<H", self.read(2))[0]

    def i32(self) -> int:
        return struct.unpack("<i", self.read(4))[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.read(4))[0]

    def i64(self) -> int:
        return struct.unpack("<q", self.read(8))[0]

    def count(self) -> int:
        v = self.u16()
        return self.u32() if v == 0xFFFF else v

    def cstring(self) -> str:
        length = self.u8()
        if length == 0xFF:
            length = self.u16()
            if length == 0xFFFF:
                length = self.u32()
        raw = self.read(length)
        try:
            return raw.decode("cp932")
        except UnicodeDecodeError:
            return raw.hex()

    def cstring_ascii(self) -> str:
        n = self.u16()
        raw = self.read(n)
        return raw.decode("ascii")

    def bool(self) -> bool:
        return self.u8() != 0

    def tell(self) -> int:
        return self.pos


# ═══════════════════════════════════════════════════════════════
# Compact class name decoder
# ═══════════════════════════════════════════════════════════════

class BitReader:
    def __init__(self, data: bytes):
        self._data = data
        self._pos = 0

    def get(self) -> int | None:
        if self._pos // 8 >= len(self._data):
            return None
        b = self._data[self._pos // 8]
        bit = (b >> (self._pos & 7)) & 1
        self._pos += 1
        return bit

    def bits(self, n: int) -> int | None:
        v = 0
        for _ in range(n):
            b = self.get()
            if b is None:
                return None
            v = (v << 1) | b
        return v

    def bits_le(self, n: int) -> int | None:
        v = 0
        for i in range(n):
            b = self.get()
            if b is None:
                return None
            if b:
                v |= 1 << i
        return v


def decode_class_name(data: bytes) -> str:
    bits = BitReader(data)
    chars = bytearray()
    if bits.get() == 0:
        chars.append(ord("C"))
    while True:
        selector = bits.get()
        if selector is None:
            break
        if selector == 0:
            idx = bits.bits_le(4)
            if idx is None:
                break
            chars.append(CLASS_NAME_TABLE_5[idx])
        else:
            sub = bits.get()
            if sub is None:
                break
            if sub == 0:
                idx = bits.bits_le(4)
                if idx is None:
                    break
                if idx == 0:
                    c = bits.bits_le(8)
                    if c is None:
                        break
                    chars.append(c)
                else:
                    chars.append(CLASS_NAME_TABLE_6[idx])
            else:
                idx = bits.bits_le(5)
                if idx is None:
                    break
                chars.append(CLASS_NAME_TABLE_7[idx])
    try:
        return chars.decode("ascii")
    except UnicodeDecodeError:
        return chars.hex()


# ═══════════════════════════════════════════════════════════════
# Address / size codec
# ═══════════════════════════════════════════════════════════════

def decode_offset(encoded: int) -> int:
    return (encoded - RESOURCE_KEY_A) & 0xFFFFFFFF


def decode_size(encoded: int) -> int:
    x = (encoded - RESOURCE_KEY_B) & 0xFFFFFFFF
    upper = x >> 13
    return (upper | ((x - (upper & 0xFFF)) << 19)) & 0x7FFFFFFF


# ═══════════════════════════════════════════════════════════════
# ICI metadata parser
# ═══════════════════════════════════════════════════════════════

def parse_ici_metadata(data: bytes) -> dict[str, Any]:
    r = Reader(data)
    if r.u32() != SIG_ICI:
        raise FormatError("bad ICI signature")
    r.u16()  # archive_version
    r.u16()  # archive_flags
    r.u16()  # class_tag
    r.u16()  # class_schema
    nlen = r.u16()
    class_name = r.read(nlen).decode("ascii")
    if class_name != "CObjectArcMan":
        raise FormatError(f"expected CObjectArcMan, got {class_name}")

    # seek to version field
    needle = struct.pack("<I", 10)
    vpos = data.find(needle, r.pos, r.pos + 8)
    if vpos < 0:
        raise FormatError("CObjectArcMan version not found")
    r.pos = vpos
    obj_version = r.u32()

    result: dict[str, Any] = {
        "object_version": obj_version,
        "field_20": r.u32(),
        "field_24": r.u8(),
        "field_25": r.u8(),
    }
    if obj_version >= 10:
        result["field_28"] = r.u32()
        result["field_32"] = r.u32()

    result.update({
        "field_36": r.u32(), "field_40": r.u32(), "field_44": r.u32(),
    })
    if obj_version >= 6:
        result["field_48"] = r.u32()
        result["field_52"] = r.u32()
        result["field_168"] = r.u32()
    if obj_version >= 8:
        result["field_56"] = r.u32()

    result["package_name"] = r.cstring()
    result["field_112"] = r.u32()
    result["install_path"] = r.cstring()
    result["field_64"] = r.u32()
    result["identifier"] = r.cstring()
    result["source_root"] = r.cstring()
    result["title"] = r.cstring()
    result["field_120"] = r.u32()
    result["string_124"] = r.cstring()

    sc = r.count()
    result["string_array"] = [r.cstring() for _ in range(sc)]
    result["field_104"] = r.u32()
    if obj_version >= 9:
        result["primary_volume"] = r.cstring()
    if obj_version >= 7:
        result["manual_name"] = r.cstring()
    if obj_version >= 5:
        result["field_152"] = r.u32()

    install_sources = []
    src_count = r.count()
    for _ in range(src_count):
        present = r.u8()
        if present:
            install_sources.append(parse_install_source(r))
        else:
            install_sources.append(None)
    result["install_sources"] = install_sources
    result["serialized_size"] = r.pos
    result["padding_size"] = len(data) - r.pos
    return result


def parse_install_source(r: Reader) -> dict[str, Any]:
    ver = r.u16()
    if ver < 6:
        raise FormatError(f"CInstallSource version {ver} too old")
    result: dict[str, Any] = {"version": ver}
    if ver >= 7:
        r.i32(); r.i32(); r.u8(); r.cstring()
    r.cstring(); r.cstring(); r.cstring()
    r.cstring(); r.cstring()
    r.i64(); r.i64()
    if ver < 6:
        r.u32(); r.u32()
    else:
        r.u32()
    result["volume_name"] = r.cstring()
    r.i64()  # volume_start
    result["volume_size"] = r.i64()
    if ver < 6:
        r.i64()
    r.u32()
    r.cstring()
    r.u32(); r.u32(); r.u32(); r.u32(); r.u32()
    r.cstring()
    dwords = r.count()
    r.read(dwords * 4)
    # bitmap
    block_count = (result["volume_size"] + 0xFFFF) >> 16
    bitmap_size = (block_count + 7) // 8
    result["bitmap_size"] = bitmap_size
    result["bitmap_offset"] = r.tell()
    r.read(bitmap_size)
    result["end_offset"] = r.tell()
    return result


# ═══════════════════════════════════════════════════════════════
# RIO resource tree parser
# ═══════════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════════
# Resource locator scanner (flags 0xC108/0xC308)
# ═══════════════════════════════════════════════════════════════

DIRECT_LOCATOR_FLAGS = (0xC108, 0xC308)

def _find_class_before(data: bytes, flag_pos: int) -> dict | None:
    """Try to find a new class descriptor before a flag pattern."""
    for length in range(1, min(80, flag_pos - 5)):
        start = flag_pos - 5 - length
        if data[start:start+2] != b"\xff\xff":
            continue
        if data[start+4] != length:
            continue
        try:
            name = decode_class_name(data[start+5:flag_pos])
            if len(name) > 1 and name[0] == "C" and name.isascii():
                return {"class_name": name, "class_kind": "new",
                        "descriptor_offset": start,
                        "schema": struct.unpack_from("<H", data, start+2)[0]}
        except (FormatError, UnicodeDecodeError, IndexError):
            continue
    return None


def scan_rio(volume_path: Path, metadata: dict) -> dict[str, Any]:
    """Find resource locator records by scanning for flag patterns."""
    vol_size = volume_path.stat().st_size
    records = []
    ashift = metadata["field_28"]
    vol_size_limit = volume_path.stat().st_size

    with volume_path.open("rb") as f:
        data = f.read()

    for flags in DIRECT_LOCATOR_FLAGS:
        pattern = struct.pack("<H", flags)
        pos = 0
        while True:
            pos = data.find(pattern, pos)
            if pos < 2 or pos + 10 > vol_size:
                break
            enc_addr = struct.unpack_from("<I", data, pos + 2)[0]
            enc_size = struct.unpack_from("<I", data, pos + 6)[0]
            logical = decode_offset(enc_addr)
            size = decode_size(enc_size)
            byte_off = logical << ashift
            if 0 < size <= vol_size_limit and 0 <= byte_off < vol_size_limit and byte_off + size <= vol_size_limit:
                new_cls = _find_class_before(data, pos)
                class_tag = struct.unpack_from("<H", data, pos - 2)[0]
                if new_cls:
                    records.append({**new_cls, "record_offset": pos,
                                    "flags": f"0x{flags:04X}",
                                    "encoded_addr": enc_addr, "encoded_size": enc_size,
                                    "byte_offset": byte_off, "resource_size": size,
                                    "logical_addr": logical})
                elif (class_tag & 0x8000) and (class_tag & 0x7FFF) <= 0x100:
                    records.append({"class_name": f"class_ref_{class_tag & 0x7FFF:04X}",
                                    "class_kind": "reference",
                                    "class_reference": class_tag & 0x7FFF,
                                    "descriptor_offset": pos - 2,
                                    "record_offset": pos,
                                    "flags": f"0x{flags:04X}",
                                    "encoded_addr": enc_addr, "encoded_size": enc_size,
                                    "byte_offset": byte_off, "resource_size": size,
                                    "logical_addr": logical})
            pos += 1

    # Deduplicate by (offset, size) — prefer named over reference
    unique: dict[tuple[int, int], dict] = {}
    for r in records:
        key = (r["byte_offset"], r["resource_size"])
        prev = unique.get(key)
        if prev is None or (prev["class_kind"] == "reference" and r["class_kind"] == "new"):
            unique[key] = r

    selected = list(unique.values())
    selected.sort(key=lambda r: (r["byte_offset"], r["resource_size"]))
    return {
        "raw_record_count": len(records),
        "selected_record_count": len(selected),
        "named_record_count": sum(1 for r in selected if r["class_kind"] == "new"),
        "reference_record_count": sum(1 for r in selected if r["class_kind"] != "new"),
        "selected_bytes": sum(r["resource_size"] for r in selected),
        "records": selected,
    }


def parse_rio_root(volume_path: Path, metadata: dict) -> list[dict]:
    address_shift = metadata["field_28"]
    root_addr = metadata["field_48"]
    sources = metadata["install_sources"]

    byte_offset = (root_addr << address_shift)
    dir_limit = metadata["field_52"]

    with volume_path.open("rb") as f:
        f.seek(byte_offset)
        data = f.read(dir_limit)

    r = Reader(data)
    sig = r.u32()
    if sig != SIG_ENCRYPTED:
        raise FormatError(f"bad RIO root sig 0x{sig:08X}")
    r.u16()  # archive_version
    r.u16()  # archive_flags
    _read_rio_class(r)  # root class = CrelicUnitedGameProject

    child_count = r.count()
    children = []
    for _ in range(child_count):
        node = _parse_rio_node(r, address_shift, sources)
        if node is not None:
            children.append(node)
    return children


def _read_rio_class(r: Reader) -> str:
    tag = r.u16()
    if tag == 0xFFFF:
        r.u16()  # schema
        nlen = r.u8()
        if nlen == 0xFF:
            nlen = r.u16()
        return decode_class_name(r.read(nlen))
    if tag == 0x7FFF:
        r.u32()
        return "class_ref"
    return f"class_ref_{tag & 0x7FFF:04X}"


def _parse_rio_node(r: Reader, ashift: int, sources: list) -> dict | None:
    flags = r.u16()
    kind = flags & 7

    if kind == 1:
        r.u32()  # object_id
        r.u16()  # object_type (7767)
        class_name = _read_rio_class(r)
    else:
        if flags & 0x8000:
            r.u8()
        else:
            r.u16()
        class_name = _read_rio_class(r)

    node: dict[str, Any] = {
        "flags": flags,
        "class_name": class_name,
    }

    if flags & 8:
        enc_addr = r.u32()
        enc_size = r.u32()
        logical = decode_offset(enc_addr)
        size = decode_size(enc_size)
        byte_off = logical << ashift
        # map to volume
        vol_name = sources[0]["volume_name"] if sources else "?"
        node.update({
            "encoded_addr": enc_addr,
            "encoded_size": enc_size,
            "logical_addr": logical,
            "byte_offset": byte_off,
            "resource_size": size,
            "volume_name": vol_name,
        })

    children = []
    if not (flags & 0x200):
        cc = r.count()
        for _ in range(cc):
            child = _parse_rio_node(r, ashift, sources)
            if child is not None:
                children.append(child)
    node["children"] = children
    return node


# ═══════════════════════════════════════════════════════════════
# Resource extractor
# ═══════════════════════════════════════════════════════════════

def collect_resources(nodes: list[dict], prefix: str = "") -> list[dict]:
    """Flatten tree into list of resource entries."""
    entries = []
    for node in nodes:
        name = f"{prefix}/{node['class_name']}" if prefix else node["class_name"]
        if "byte_offset" in node and node["resource_size"] > 0:
            entries.append({
                **node,
                "path": name,
            })
        if node.get("children"):
            entries.extend(collect_resources(node["children"], name))
    return entries


def _extract_resource(volume_path: Path, entry: dict, output_dir: Path, key: int) -> Path | None:
    """Extract and decrypt a single resource. Returns output path or None."""
    offset = entry["byte_offset"]
    size = entry["resource_size"]
    class_name = entry["class_name"]

    with volume_path.open("rb") as f:
        f.seek(offset)
        data = f.read(size)

    if len(data) < size:
        return None

    # Attempt decryption for supported encrypted types
    output_data = data
    suffix = ".bin"

    if "Rsa" in class_name or "CrelicUnitedGameProject" in class_name:
        try:
            output_data = read_encrypted(data, key)
            suffix = ".dec"
        except FormatError:
            pass

    # Determine output extension
    ext_map = {
        "CRip": ".rip", "CRip007": ".rip007", "CRsa": ".rsa",
        "CS5i": ".s5i", "CWaveAudio": ".wav", "CIcon": ".ico",
        "CrelicHicompAudio": ".hca",
    }
    for k, v in ext_map.items():
        if k in class_name:
            suffix = v
            break

    safe_name = class_name.replace("/", "_").replace("class_ref_", "ref_")
    safe_name = "".join(c if c.isalnum() or c in "_-" else "_" for c in safe_name).strip("_") or "unknown"
    out_path = output_dir / f"{safe_name}_{offset:08X}_{size:08X}{suffix}"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(output_data)
    return out_path


# ═══════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════

def unpack_game(ici_path: Path, output_dir: Path, key: int = RIO_KEY) -> dict:
    """Unpack a single game's resources."""
    ici_name = ici_path.stem
    game_dir = Path(ici_path).parent

    print(f"  Decrypting ICI: {ici_path.name}")
    plain = decrypt_ici_file(ici_path)
    meta = parse_ici_metadata(plain)
    print(f"    CObjectArcMan v{meta['object_version']}, "
          f"{len(meta['install_sources'])} install sources")

    address_shift = meta["field_28"]
    primary_volume = meta.get("primary_volume") or meta["install_sources"][0]["volume_name"]
    volume_path = game_dir / primary_volume
    if not volume_path.is_file():
        raise FormatError(f"volume not found: {volume_path}")

    print(f"  Parsing RIO root: {volume_path.name}")
    root_children = parse_rio_root(volume_path, meta)
    tree_resources = collect_resources(root_children)
    print(f"    {len(tree_resources)} tree nodes")

    # Also scan for resource locators (flags 0xC108/0xC308) throughout RIO
    print(f"  Scanning for resource locators...")
    scan = scan_rio(volume_path, meta)
    print(f"    {scan['selected_record_count']} locators found "
          f"({scan['named_record_count']} named, "
          f"{scan['reference_record_count']} refs, "
          f"{scan['selected_bytes']/1024/1024:.1f} MB)")

    # Merge: prefer scanner records (more comprehensive)
    resources = list(scan['records'])  # each has byte_offset + resource_size + class_name

    # Create output directory
    out = output_dir / ici_name
    out.mkdir(parents=True, exist_ok=True)

    # Extract resources by type
    by_type = defaultdict(list)
    for res in resources:
        by_type[res["class_name"]].append(res)

    stats = {"total": len(resources), "extracted": 0, "by_type": {}}
    for cls_name, entries in sorted(by_type.items()):
        safe_cls = "".join(c if c.isalnum() or c in "_-" else "_" for c in cls_name).strip("_") or "unknown"
        cls_dir = out / safe_cls
        extracted = 0
        for entry in entries:
            result = _extract_resource(volume_path, entry, cls_dir, key)
            if result:
                extracted += 1
        stats["by_type"][cls_name] = {"count": len(entries), "extracted": extracted}
        stats["extracted"] += extracted
        if extracted > 0:
            print(f"    {cls_name}: {extracted}/{len(entries)}")

    # Write manifest
    manifest = {
        "ici_file": str(ici_path),
        "volume": str(volume_path),
        "address_shift": address_shift,
        "stats": stats,
        "resources": [
            {k: str(v) if isinstance(v, Path) else v
             for k, v in r.items() if k != "children"}
            for r in resources
        ],
    }
    manifest_path = out / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"    manifest: {manifest_path}")
    return stats


def main():
    parser = argparse.ArgumentParser(description="rUGP .rio static unpacker")
    parser.add_argument("files", nargs="*", type=Path, help=".ici files to unpack")
    parser.add_argument("-o", "--output", type=Path, default=Path("unpacked"),
                       help="output directory (default: unpacked)")
    parser.add_argument("-k", "--key", type=lambda x: int(x, 0), default=RIO_KEY,
                       help=f"RIO decryption key (default: 0x{RIO_KEY:08X})")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    ici_files = args.files or sorted(Path(__file__).resolve().parent.glob("*.rio.ici"))
    if not ici_files:
        parser.error("no .rio.ici files found")

    all_stats = {}
    for ici_path in ici_files:
        print(f"\n{'='*60}")
        print(f"Unpacking: {ici_path}")
        print(f"{'='*60}")
        try:
            stats = unpack_game(ici_path.resolve(), args.output.resolve(), args.key)
            all_stats[str(ici_path)] = stats
        except (OSError, FormatError) as e:
            print(f"  ERROR: {e}")
            all_stats[str(ici_path)] = {"error": str(e)}

    if args.json:
        print(json.dumps(all_stats, ensure_ascii=False, indent=2))

    total_resources = sum(s.get("total", 0) for s in all_stats.values())
    total_extracted = sum(s.get("extracted", 0) for s in all_stats.values())
    print(f"\nDone. {total_extracted}/{total_resources} resources extracted.")


if __name__ == "__main__":
    raise SystemExit(main())
