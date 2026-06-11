# -*- coding: utf-8 -*-
"""
IDA/XAF archive unpacker and repacker for FF.exe Script.IDA.

Format notes from FF.exe:
  - magic: b"XAF\0"
  - version: 0x00011400
  - metadata entries are MFC CArchive-like serialized records
  - payload flags in entry field5:
      bit 0x01: bytewise NOT stage
      bit 0x02: chained XOR stage
      bit 0x08: chained ADD/SUB stage
      bit 0x10: zlib stream
      bit 0x04: old RLE wrapper (implemented for unpack/pack fallback)

Commands:
  python ida_xaf_tool.py list Script.IDA
  python ida_xaf_tool.py unpack Script.IDA out_dir
  python ida_xaf_tool.py pack out_dir rebuilt.IDA
  python ida_xaf_tool.py verify Script.IDA out_dir
"""
from __future__ import annotations

import argparse
import json
import re
import struct
import sys
import zlib
from dataclasses import dataclass, asdict
from hashlib import sha256
from pathlib import Path
from typing import Any

DEFAULT_ENCODING = "cp932"
MAGIC = b"XAF\x00"
VERSION = 0x00011400
SENTINEL_FLAGS = 0x1F


@dataclass
class Entry:
    index: int
    meta_offset: int
    record_size: int
    data_offset: int
    packed_size: int
    field3: int
    field4: int
    flags: int
    key: int
    filetime1_hex: str
    filetime2_hex: str
    name: str
    name2: str
    name3: str
    out_name: str
    raw_sha256: str | None = None
    decoded_sha256: str | None = None
    decoded_size: int | None = None

    def to_manifest_obj(self) -> dict[str, Any]:
        return asdict(self)


class Reader:
    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0

    def read(self, n: int) -> bytes:
        if self.pos + n > len(self.data):
            raise EOFError(f"unexpected EOF at 0x{self.pos:X}, need {n} bytes")
        out = self.data[self.pos:self.pos + n]
        self.pos += n
        return out

    def u8(self) -> int:
        return self.read(1)[0]

    def u16(self) -> int:
        return struct.unpack("<H", self.read(2))[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.read(4))[0]

    def archive_string(self, encoding: str = DEFAULT_ENCODING) -> str:
        length = self.u8()
        unicode_mode = False
        if length == 0xFF:
            marker = self.u16()
            if marker == 0xFFFE:
                unicode_mode = True
                length = self.archive_string_length()
            elif marker == 0xFFFF:
                length = self.u32()
            else:
                length = marker
        byte_len = length * 2 if unicode_mode else length
        raw = self.read(byte_len)
        if unicode_mode:
            # MFC serializes CStringW without the terminating NUL.
            return raw.decode("utf-16le")
        return raw.decode(encoding, errors="replace")

    def archive_string_length(self) -> int:
        length = self.u8()
        if length == 0xFF:
            marker = self.u16()
            if marker == 0xFFFF:
                length = self.u32()
            elif marker == 0xFFFE:
                raise ValueError("nested unicode CString marker is invalid")
            else:
                length = marker
        return length


def archive_string_bytes(text: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    raw = text.encode(encoding)
    n = len(raw)
    if n < 0xFF:
        return bytes([n]) + raw
    if n < 0xFFFE:
        return b"\xFF" + struct.pack("<H", n) + raw
    return b"\xFF\xFF\xFF" + struct.pack("<I", n) + raw


def safe_name(name: str, index: int) -> str:
    # Keep the original name readable but avoid unsafe Windows/path characters.
    s = re.sub(r"[\\/:*?\"<>|]+", "_", name).strip()
    s = s.rstrip(". ")
    if not s:
        s = f"entry_{index:04d}"
    return s


def decode_chained(data: bytes, flags: int, key: int) -> bytes:
    """Decode the bytewise transform used before compression/plain payload.

    This matches FF.exe sub_47E570/read-side transform:
        if flags&8: b += previous_decoded
        if flags&2: b ^= previous_decoded
        if flags&1: b = ~b
        previous_decoded = b
    """
    if not (flags & 0x0B):
        return data
    prev = key & 0xFF
    out = bytearray(data)
    for i, x in enumerate(out):
        y = x
        if flags & 0x08:
            y = (y + prev) & 0xFF
        if flags & 0x02:
            y ^= prev
        if flags & 0x01:
            y = (~y) & 0xFF
        out[i] = y
        prev = y
    return bytes(out)


def encode_chained(data: bytes, flags: int, key: int) -> bytes:
    """Inverse of decode_chained, used while rebuilding the archive."""
    if not (flags & 0x0B):
        return data
    prev = key & 0xFF
    out = bytearray(data)
    for i, x in enumerate(out):
        y = x
        if flags & 0x01:
            y = (~y) & 0xFF
        if flags & 0x02:
            y ^= prev
        if flags & 0x08:
            y = (y - prev) & 0xFF
        out[i] = y
        prev = x
    return bytes(out)


def rle_decode(data: bytes) -> bytes:
    """Decode FF.exe sub_47E300 RLE block.

    Encoded layout: u32le uncompressed_size followed by literal/run tokens.
    """
    if len(data) < 4:
        raise ValueError("RLE block is shorter than size header")
    out_size = struct.unpack_from("<I", data, 0)[0]
    pos = 4
    out = bytearray()
    while len(out) < out_size:
        if pos >= len(data):
            raise ValueError("RLE block ended inside token stream")
        ctl = data[pos]
        pos += 1
        if ctl & 0x80:
            mode = ctl & 0x03
            if mode == 0:
                if pos + 1 > len(data):
                    raise ValueError("RLE extended u8 length truncated")
                count = data[pos]
                pos += 1
            elif mode == 1:
                if pos + 2 > len(data):
                    raise ValueError("RLE extended u16 length truncated")
                count = struct.unpack_from("<H", data, pos)[0]
                pos += 2
            elif mode == 3:
                if pos + 4 > len(data):
                    raise ValueError("RLE extended u32 length truncated")
                count = struct.unpack_from("<I", data, pos)[0]
                pos += 4
            else:
                raise ValueError(f"RLE invalid extended length mode: {mode}")
        else:
            count = ctl & 0x3F
        if ctl & 0x40:
            if pos >= len(data):
                raise ValueError("RLE run byte missing")
            out.extend(bytes([data[pos]]) * count)
            pos += 1
        else:
            if pos + count > len(data):
                raise ValueError("RLE literal payload truncated")
            out.extend(data[pos:pos + count])
            pos += count
    if len(out) != out_size:
        raise ValueError(f"RLE output overrun: got {len(out)}, expected {out_size}")
    return bytes(out)


def _rle_count_prefix(count: int, run: bool) -> bytes:
    flag = 0x40 if run else 0
    if count <= 0x3F:
        return bytes([flag | count])
    if count <= 0xFF:
        return bytes([flag | 0x80]) + bytes([count])
    if count <= 0xFFFF:
        return bytes([flag | 0x81]) + struct.pack("<H", count)
    return bytes([flag | 0x83]) + struct.pack("<I", count)


def rle_encode(data: bytes) -> bytes:
    """Compatible encoder for sub_47E300. Only used when an input archive uses flag 0x04."""
    out = bytearray(struct.pack("<I", len(data)))
    i = 0
    n = len(data)
    while i < n:
        # Prefer runs of at least three bytes, matching the engine encoder's threshold.
        run_len = 1
        while i + run_len < n and data[i + run_len] == data[i] and run_len < 0xFFFFFFFF:
            run_len += 1
        if run_len >= 3:
            out.extend(_rle_count_prefix(run_len, True))
            out.append(data[i])
            i += run_len
            continue
        lit_start = i
        i += 1
        while i < n:
            run_len = 1
            while i + run_len < n and data[i + run_len] == data[i] and run_len < 0xFFFFFFFF:
                run_len += 1
            if run_len >= 3:
                break
            i += 1
        lit = data[lit_start:i]
        # Split huge literals to keep simple control sizes manageable.
        off = 0
        while off < len(lit):
            chunk = lit[off:off + 0xFFFF]
            out.extend(_rle_count_prefix(len(chunk), False))
            out.extend(chunk)
            off += len(chunk)
    return bytes(out)


def decode_payload(raw: bytes, flags: int, key: int) -> bytes:
    data = decode_chained(raw, flags, key)
    if flags & 0x04:
        data = rle_decode(data)
    if flags & 0x10:
        data = zlib.decompress(data)
    return data


def encode_payload(decoded: bytes, flags: int, key: int, compression_level: int = 9) -> bytes:
    data = decoded
    if flags & 0x10:
        data = zlib.compress(data, compression_level)
    if flags & 0x04:
        data = rle_encode(data)
    data = encode_chained(data, flags, key)
    return data


def parse_archive(path: Path) -> tuple[bytes, list[Entry], bytes, int]:
    blob = path.read_bytes()
    if len(blob) < 8 or blob[:4] != MAGIC:
        raise ValueError(f"not an XAF archive: {path}")
    version = struct.unpack_from("<I", blob, 4)[0]
    if version != VERSION:
        raise ValueError(f"unsupported XAF version 0x{version:08X}; expected 0x{VERSION:08X}")

    r = Reader(blob)
    r.pos = 8
    entries: list[Entry] = []
    used_names: set[str] = set()
    sentinel_raw = b""
    first_data_offset = None

    while r.pos < len(blob):
        start = r.pos
        if start + 44 > len(blob):
            raise ValueError(f"metadata truncated at 0x{start:X}")
        fields = list(struct.unpack("<7I", r.read(28)))
        ft1 = r.read(8)
        ft2 = r.read(8)
        name = r.archive_string()
        name2 = r.archive_string()
        name3 = r.archive_string()
        physical_end = r.pos
        rec_size = fields[0]
        if rec_size == 0:
            sentinel_raw = blob[start:physical_end]
            first_data_offset = physical_end
            break
        expected_end = start + rec_size
        if physical_end != expected_end:
            raise ValueError(
                f"metadata record size mismatch at index {len(entries)}: "
                f"logical end 0x{physical_end:X}, record_size end 0x{expected_end:X}"
            )
        data_offset = fields[1]
        packed_size = fields[2]
        if data_offset + packed_size > len(blob):
            raise ValueError(
                f"payload out of range index={len(entries)} name={name!r}: "
                f"off=0x{data_offset:X} size=0x{packed_size:X} archive=0x{len(blob):X}"
            )
        out = safe_name(name, len(entries))
        if out in used_names:
            stem = out
            suffix = 1
            while f"{stem}_{suffix}" in used_names:
                suffix += 1
            out = f"{stem}_{suffix}"
        used_names.add(out)
        entries.append(
            Entry(
                index=len(entries),
                meta_offset=start,
                record_size=rec_size,
                data_offset=data_offset,
                packed_size=packed_size,
                field3=fields[3],
                field4=fields[4],
                flags=fields[5],
                key=fields[6],
                filetime1_hex=ft1.hex(),
                filetime2_hex=ft2.hex(),
                name=name,
                name2=name2,
                name3=name3,
                out_name=out,
            )
        )
        r.pos = expected_end

    if not sentinel_raw:
        raise ValueError("missing XAF sentinel metadata record")
    if first_data_offset != entries[0].data_offset:
        # The original sample has first payload immediately after the sentinel.
        # Non-fatal for archives with padding, but record it via stderr.
        print(
            f"[warn] first payload offset 0x{entries[0].data_offset:X} != metadata end 0x{first_data_offset:X}",
            file=sys.stderr,
        )
    return blob, entries, sentinel_raw, first_data_offset or 0


def serialize_entry_meta(e: Entry, data_offset: int, packed_size: int, encoding: str = DEFAULT_ENCODING) -> bytes:
    s1 = archive_string_bytes(e.name, encoding)
    s2 = archive_string_bytes(e.name2, encoding)
    s3 = archive_string_bytes(e.name3, encoding)
    rec_size = 44 + len(s1) + len(s2) + len(s3)
    head = struct.pack(
        "<7I",
        rec_size,
        data_offset,
        packed_size,
        e.field3,
        e.field4,
        e.flags,
        e.key,
    )
    return head + bytes.fromhex(e.filetime1_hex) + bytes.fromhex(e.filetime2_hex) + s1 + s2 + s3


def manifest_path(out_dir: Path) -> Path:
    return out_dir / "_ida_xaf_manifest.json"


def write_manifest(out_dir: Path, archive_name: str, entries: list[Entry], sentinel_raw: bytes) -> None:
    obj = {
        "format": "IDA-XAF",
        "magic_hex": MAGIC.hex(),
        "version": VERSION,
        "archive_name": archive_name,
        "sentinel_raw_hex": sentinel_raw.hex(),
        "encoding": DEFAULT_ENCODING,
        "entries": [e.to_manifest_obj() for e in entries],
    }
    manifest_path(out_dir).write_text(json.dumps(obj, ensure_ascii=False, indent=2), encoding="utf-8")


def read_manifest(in_dir: Path) -> dict[str, Any]:
    mp = manifest_path(in_dir)
    if not mp.exists():
        raise FileNotFoundError(f"missing manifest: {mp}")
    obj = json.loads(mp.read_text(encoding="utf-8"))
    if obj.get("format") != "IDA-XAF":
        raise ValueError(f"manifest format mismatch: {mp}")
    return obj


def unpack_archive(archive: Path, out_dir: Path, write_raw: bool = False) -> None:
    blob, entries, sentinel_raw, _ = parse_archive(archive)
    files_dir = out_dir / "files"
    raw_dir = out_dir / "raw" if write_raw else None
    files_dir.mkdir(parents=True, exist_ok=True)
    if raw_dir:
        raw_dir.mkdir(parents=True, exist_ok=True)

    decoded_total = 0
    for e in entries:
        raw = blob[e.data_offset:e.data_offset + e.packed_size]
        decoded = decode_payload(raw, e.flags, e.key)
        e.raw_sha256 = sha256(raw).hexdigest()
        e.decoded_sha256 = sha256(decoded).hexdigest()
        e.decoded_size = len(decoded)
        (files_dir / e.out_name).write_bytes(decoded)
        decoded_total += len(decoded)
        if raw_dir:
            (raw_dir / f"{e.index:04d}_{e.out_name}.raw").write_bytes(raw)

    write_manifest(out_dir, archive.name, entries, sentinel_raw)
    print(f"[unpack] archive={archive}")
    print(f"[unpack] entries={len(entries)} decoded_bytes={decoded_total}")
    print(f"[unpack] output={out_dir}")


def pack_archive(in_dir: Path, out_archive: Path, compression_level: int = 9) -> None:
    obj = read_manifest(in_dir)
    entries = [Entry(**e) for e in obj["entries"]]
    files_dir = in_dir / "files"
    if not files_dir.is_dir():
        raise FileNotFoundError(f"missing files dir: {files_dir}")

    # First encode all payloads so we can calculate the final offset table.
    payloads: list[bytes] = []
    for e in entries:
        p = files_dir / e.out_name
        if not p.exists():
            raise FileNotFoundError(f"missing extracted file for entry {e.index}: {p}")
        decoded = p.read_bytes()
        payloads.append(encode_payload(decoded, e.flags, e.key, compression_level=compression_level))

    # Data starts after header + all metadata + sentinel. Metadata record sizes are stable
    # because names are not changed by this tool.
    sentinel_raw = bytes.fromhex(obj.get("sentinel_raw_hex", ""))
    if not sentinel_raw:
        # Fallback: default empty entry generated by FF.exe.
        sentinel_raw = struct.pack("<7I", 0, 0, 0, 0, 0, SENTINEL_FLAGS, 0) + b"\x00" * 16 + b"\x00\x00\x00"

    # Metadata length can be calculated using placeholder offsets.
    dummy_meta_len = 8 + sum(len(serialize_entry_meta(e, 0, 0)) for e in entries) + len(sentinel_raw)
    data_offset = dummy_meta_len
    metas = []
    payload_offset = data_offset
    for e, payload in zip(entries, payloads):
        metas.append(serialize_entry_meta(e, payload_offset, len(payload)))
        payload_offset += len(payload)

    out = bytearray()
    out += MAGIC
    out += struct.pack("<I", VERSION)
    out += b"".join(metas)
    out += sentinel_raw
    if len(out) != data_offset:
        raise AssertionError(f"internal metadata size mismatch: {len(out)} != {data_offset}")
    for payload in payloads:
        out += payload

    out_archive.parent.mkdir(parents=True, exist_ok=True)
    out_archive.write_bytes(out)
    print(f"[pack] entries={len(entries)} archive_size={len(out)}")
    print(f"[pack] output={out_archive}")


def list_archive(archive: Path, limit: int | None = None) -> None:
    blob, entries, _, _ = parse_archive(archive)
    print(f"archive={archive} size={len(blob)} entries={len(entries)}")
    for e in entries[:limit]:
        print(
            f"[{e.index:04d}] {e.name} off=0x{e.data_offset:08X} "
            f"packed={e.packed_size:6d} flags=0x{e.flags:02X} out={e.out_name}"
        )
    if limit is not None and len(entries) > limit:
        print(f"... {len(entries) - limit} more")


def verify_archive(archive: Path, unpacked_dir: Path) -> None:
    blob, entries, _, _ = parse_archive(archive)
    obj = read_manifest(unpacked_dir)
    files_dir = unpacked_dir / "files"
    manifest_entries = {e["index"]: e for e in obj["entries"]}
    failed = 0
    for e in entries:
        raw = blob[e.data_offset:e.data_offset + e.packed_size]
        decoded = decode_payload(raw, e.flags, e.key)
        p = files_dir / manifest_entries[e.index]["out_name"]
        disk = p.read_bytes()
        if decoded != disk:
            print(f"[verify][fail] index={e.index} name={e.name} path={p}")
            failed += 1
    if failed:
        raise SystemExit(f"[verify] failed={failed}")
    print(f"[verify] ok entries={len(entries)}")


def roundtrip_check(archive: Path, temp_dir: Path) -> None:
    import shutil
    if temp_dir.exists():
        shutil.rmtree(temp_dir)
    unpack_archive(archive, temp_dir)
    rebuilt = temp_dir / "rebuilt.IDA"
    pack_archive(temp_dir, rebuilt)
    reparsed_dir = temp_dir / "rebuilt_unpacked"
    unpack_archive(rebuilt, reparsed_dir)
    orig_files = sorted((temp_dir / "files").iterdir())
    failed = 0
    for p in orig_files:
        q = reparsed_dir / "files" / p.name
        if p.read_bytes() != q.read_bytes():
            print(f"[roundtrip][fail] {p.name}")
            failed += 1
    if failed:
        raise SystemExit(f"[roundtrip] decoded mismatch count={failed}")
    print(f"[roundtrip] ok entries={len(orig_files)}")


def main() -> None:
    ap = argparse.ArgumentParser(description="IDA/XAF Script.IDA unpacker/repacker")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("list", help="list archive entries")
    p.add_argument("archive")
    p.add_argument("--limit", type=int, default=40)

    p = sub.add_parser("unpack", help="extract decoded files")
    p.add_argument("archive")
    p.add_argument("out_dir")
    p.add_argument("--raw", action="store_true", help="also dump original encoded payloads")

    p = sub.add_parser("pack", help="rebuild archive from unpacked directory")
    p.add_argument("in_dir")
    p.add_argument("out_archive")
    p.add_argument("--level", type=int, default=9, choices=range(0, 10), help="zlib compression level")

    p = sub.add_parser("verify", help="compare an archive with an unpacked directory")
    p.add_argument("archive")
    p.add_argument("unpacked_dir")

    p = sub.add_parser("roundtrip", help="unpack -> pack -> unpack decoded-content check")
    p.add_argument("archive")
    p.add_argument("temp_dir")

    args = ap.parse_args()
    if args.cmd == "list":
        list_archive(Path(args.archive), args.limit)
    elif args.cmd == "unpack":
        unpack_archive(Path(args.archive), Path(args.out_dir), write_raw=args.raw)
    elif args.cmd == "pack":
        pack_archive(Path(args.in_dir), Path(args.out_archive), compression_level=args.level)
    elif args.cmd == "verify":
        verify_archive(Path(args.archive), Path(args.unpacked_dir))
    elif args.cmd == "roundtrip":
        roundtrip_check(Path(args.archive), Path(args.temp_dir))


if __name__ == "__main__":
    main()
