# -*- coding: utf-8 -*-
"""
uniform_kanojo_pac_tool.py

PAC unpack/pack/diagnose tool for 制服カノジョ 2.5 Script.pac style archives.

Format summary derived from the supplied sample and executable export:
  header: 12 bytes
    0..2  b"PAC"
    3     ignored/uninitialized byte in the original packer
    4..7  uint32 file_count
    8..11 uint32 archive compression selector; Script.pac uses 7
  body: concatenated per-file compressed chunks
  tail: bitwise-NOT'ed custom Huffman-compressed file table + uint32 table_compressed_size
  table record: 76 bytes = name[64] + uint32 offset + uint32 original_size + uint32 compressed_size

For this title, table names in the original Script.pac are UTF-8, and normal .bin
entries are Zstandard chunks when archive compression selector is 7.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Iterable, Optional

PAC_MAGIC = b"PAC"
HEADER_SIZE = 12
TABLE_REC_SIZE = 76
DEFAULT_NAME_ENCODING = "utf-8"
DEFAULT_PACK_COMPRESSION = 7
ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
SKIP_COMPRESS_EXTS = {".ogg", ".wav", ".png", ".fnt", ".nmv", ".mpg", ".mpeg", ".avi"}


class PacError(RuntimeError):
    pass


@dataclass
class PacEntry:
    name: str
    name_hex: str
    offset: int
    original_size: int
    compressed_size: int


class BitReader:
    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0
        self.bit_pos = 0
        self.cur = 0

    def read_bit(self) -> int:
        # Port of sub_4C59E0: consume bits MSB-first; if exhausted, read next byte.
        self.bit_pos -= 1
        if self.bit_pos >= 0:
            return (self.cur >> self.bit_pos) & 1
        self.bit_pos = 7
        if self.pos < len(self.data):
            self.cur = self.data[self.pos]
            self.pos += 1
        else:
            self.cur = 0xFF
        return (self.cur >> 7) & 1

    def read_bits(self, n: int) -> int:
        value = 0
        for _ in range(n):
            value = (value << 1) | self.read_bit()
        return value


class BitWriter:
    def __init__(self):
        self.out = bytearray()
        self.bit_count = 8
        self.cur = 0

    def write_bit(self, bit: int) -> None:
        self.bit_count -= 1
        if bit:
            self.cur |= 1 << self.bit_count
        if self.bit_count == 0:
            self.out.append(self.cur & 0xFF)
            self.cur = 0
            self.bit_count = 8

    def write_bits(self, n: int, value: int) -> None:
        # Equivalent to sub_4C5B50 for the cases this format needs.
        for i in range(n - 1, -1, -1):
            self.write_bit((value >> i) & 1)

    def finish_like_game(self) -> bytes:
        # Original compressor writes seven zero bits at the end. It does not force
        # an additional flush if the current byte is still completely empty.
        self.write_bits(7, 0)
        return bytes(self.out)


class HuffmanTableCodec:
    """Custom static Huffman codec used only for the PAC file table."""

    @staticmethod
    def decompress(data: bytes, output_size: int) -> bytes:
        br = BitReader(data)
        left: dict[int, int] = {}
        right: dict[int, int] = {}
        next_node = 256

        def read_tree() -> int:
            nonlocal next_node
            if br.read_bit():
                node = next_node
                next_node += 1
                if node >= 512:
                    raise PacError("Huffman tree has too many internal nodes")
                left[node] = read_tree()
                right[node] = read_tree()
                return node
            return br.read_bits(8)

        root = read_tree()
        out = bytearray()
        while len(out) < output_size:
            node = root
            while node >= 256:
                node = right[node] if br.read_bit() else left[node]
            out.append(node & 0xFF)
        return bytes(out)

    @staticmethod
    def compress(data: bytes) -> bytes:
        if not data:
            return b""

        # Build a deterministic Huffman tree. Any valid tree is accepted because
        # the tree is serialized before the encoded table payload.
        import heapq
        from itertools import count

        freq: dict[int, int] = {}
        for b in data:
            freq[b] = freq.get(b, 0) + 1

        serial = count()
        heap: list[tuple[int, int, int]] = []
        left: dict[int, int] = {}
        right: dict[int, int] = {}
        next_node = 256
        for sym, f in sorted(freq.items()):
            heapq.heappush(heap, (f, next(serial), sym))

        if len(heap) == 1:
            root = heap[0][2]
        else:
            while len(heap) > 1:
                f1, _, n1 = heapq.heappop(heap)
                f2, _, n2 = heapq.heappop(heap)
                node = next_node
                next_node += 1
                if node >= 512:
                    raise PacError("Huffman tree has too many nodes")
                left[node] = n1
                right[node] = n2
                heapq.heappush(heap, (f1 + f2, next(serial), node))
            root = heap[0][2]

        codes: dict[int, tuple[int, int]] = {}

        def build_codes(node: int, bits: int, length: int) -> None:
            if node < 256:
                codes[node] = (bits, length)
                return
            build_codes(left[node], bits << 1, length + 1)
            build_codes(right[node], (bits << 1) | 1, length + 1)

        build_codes(root, 0, 0)

        bw = BitWriter()

        def write_tree(node: int) -> None:
            if node >= 256:
                bw.write_bit(1)
                write_tree(left[node])
                write_tree(right[node])
            else:
                bw.write_bit(0)
                bw.write_bits(8, node)

        write_tree(root)
        if root >= 256:
            for b in data:
                bits, length = codes[b]
                bw.write_bits(length, bits)
        # For a single-symbol table the decoder repeats the leaf until output_size.
        return bw.finish_like_game()


def _run_zstd(args: list[str], input_data: bytes) -> bytes:
    exe = shutil.which("zstd") or shutil.which("zstd.exe")
    if not exe:
        raise PacError("需要 zstandard Python 模块，或把 zstd/zstd.exe 放到 PATH")
    with tempfile.NamedTemporaryFile(delete=False) as f:
        f.write(input_data)
        temp_name = f.name
    try:
        proc = subprocess.run([exe] + args + [temp_name], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if proc.returncode != 0 and not proc.stdout:
            raise PacError(proc.stderr.decode("utf-8", "replace").strip() or "zstd failed")
        return proc.stdout
    finally:
        try:
            os.unlink(temp_name)
        except OSError:
            pass


def zstd_compress(data: bytes, level: int = 3) -> bytes:
    try:
        import zstandard as zstd  # type: ignore
        return zstd.ZstdCompressor(level=level).compress(data)
    except ImportError:
        return _run_zstd([f"-{level}", "-q", "-c"], data)


def zstd_decompress(data: bytes) -> bytes:
    try:
        import zstandard as zstd  # type: ignore
        return zstd.ZstdDecompressor().decompress(data)
    except ImportError:
        return _run_zstd(["-d", "-q", "-c"], data)


def read_header(data: bytes) -> tuple[int, int]:
    if len(data) < HEADER_SIZE:
        raise PacError("file too small")
    if data[:3] != PAC_MAGIC:
        raise PacError(f"bad magic: {data[:4]!r}")
    count = struct.unpack_from("<I", data, 4)[0]
    compression = struct.unpack_from("<I", data, 8)[0]
    return count, compression


def parse_table(data: bytes, name_encoding: str = DEFAULT_NAME_ENCODING) -> tuple[list[PacEntry], bytes]:
    count, _compression = read_header(data)
    if len(data) < HEADER_SIZE + 4:
        raise PacError("file too small for PAC table")
    table_comp_size = struct.unpack_from("<I", data, len(data) - 4)[0]
    table_start = len(data) - 4 - table_comp_size
    if table_start < HEADER_SIZE:
        raise PacError(f"bad table size {table_comp_size}, table_start={table_start}")
    table_comp_not = data[table_start:len(data) - 4]
    table_comp = bytes((~b) & 0xFF for b in table_comp_not)
    table_raw = HuffmanTableCodec.decompress(table_comp, count * TABLE_REC_SIZE)
    entries: list[PacEntry] = []
    for i in range(count):
        rec = table_raw[i * TABLE_REC_SIZE:(i + 1) * TABLE_REC_SIZE]
        raw_name = rec[:64].split(b"\0", 1)[0]
        try:
            name = raw_name.decode(name_encoding)
        except UnicodeDecodeError:
            name = raw_name.decode(name_encoding, "replace")
        offset, original_size, compressed_size = struct.unpack_from("<III", rec, 64)
        entries.append(PacEntry(name, raw_name.hex(), offset, original_size, compressed_size))
    return entries, table_raw


def detect_name_encoding(data: bytes) -> str:
    # Original Script.pac names are UTF-8. Some broken repacks use CP932.
    count, _ = read_header(data)
    table_comp_size = struct.unpack_from("<I", data, len(data) - 4)[0]
    table_start = len(data) - 4 - table_comp_size
    table_comp = bytes((~b) & 0xFF for b in data[table_start:len(data) - 4])
    table_raw = HuffmanTableCodec.decompress(table_comp, count * TABLE_REC_SIZE)
    names = [table_raw[i * TABLE_REC_SIZE:i * TABLE_REC_SIZE + 64].split(b"\0", 1)[0] for i in range(count)]
    utf8_ok = sum(1 for n in names if _can_decode(n, "utf-8"))
    cp932_ok = sum(1 for n in names if _can_decode(n, "cp932"))
    # ASCII names count for both; prefer UTF-8 when all names are valid UTF-8.
    return "utf-8" if utf8_ok == len(names) else "cp932" if cp932_ok >= utf8_ok else "utf-8"


def _can_decode(data: bytes, enc: str) -> bool:
    try:
        data.decode(enc)
        return True
    except UnicodeDecodeError:
        return False


def _is_zlib_stream(data: bytes) -> bool:
    if len(data) < 2:
        return False
    cmf, flg = data[0], data[1]
    return (cmf & 0x0F) == 8 and ((cmf << 8) + flg) % 31 == 0


def decompress_chunk(chunk: bytes, original_size: int, archive_compression: int) -> bytes:
    # Detection first makes the tool able to recover user-made zlib repacks too.
    if chunk.startswith(ZSTD_MAGIC):
        raw = zstd_decompress(chunk)
    elif _is_zlib_stream(chunk):
        raw = zlib.decompress(chunk)
    elif len(chunk) == original_size:
        raw = chunk
    else:
        raise PacError(
            f"unknown chunk compression: first={chunk[:8].hex()} "
            f"archive_compression={archive_compression} comp_size={len(chunk)} raw_size={original_size}"
        )
    if len(raw) != original_size:
        raise PacError(f"decompressed size mismatch: got {len(raw)}, expected {original_size}")
    return raw


def should_store_raw(name: str) -> bool:
    return Path(name).suffix.lower() in SKIP_COMPRESS_EXTS


def safe_output_path(base: Path, name: str) -> Path:
    p = Path(name)
    if p.is_absolute() or ".." in p.parts:
        raise PacError(f"unsafe archive path: {name!r}")
    return base / p


def unpack_pac(pac_path: Path, out_dir: Path, name_encoding: Optional[str] = None) -> None:
    data = pac_path.read_bytes()
    count, compression = read_header(data)
    if name_encoding is None or name_encoding == "auto":
        name_encoding = detect_name_encoding(data)
    entries, _table = parse_table(data, name_encoding)
    out_dir.mkdir(parents=True, exist_ok=True)
    for e in entries:
        chunk = data[e.offset:e.offset + e.compressed_size]
        raw = decompress_chunk(chunk, e.original_size, compression)
        out_path = safe_output_path(out_dir, e.name)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(raw)
    manifest = {
        "format": "uniform-kanojo-2.5-pac",
        "source": str(pac_path),
        "file_count": count,
        "compression": compression,
        "name_encoding": name_encoding,
        "entries": [asdict(e) for e in entries],
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[unpack] file_count={count} compression={compression} name_encoding={name_encoding} output={out_dir}")


def load_pack_entries(input_dir: Path, manifest_path: Optional[Path], name_encoding: str) -> list[tuple[str, Path]]:
    if manifest_path and manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        entries = []
        for obj in manifest.get("entries", []):
            name = obj["name"]
            path = safe_output_path(input_dir, name)
            if not path.is_file():
                raise PacError(f"manifest entry missing on disk: {name}")
            entries.append((name, path))
        return entries

    entries = []
    for path in sorted(p for p in input_dir.rglob("*") if p.is_file()):
        if path.name == "manifest.json":
            continue
        name = path.relative_to(input_dir).as_posix()
        # Validate early.
        name.encode(name_encoding)
        entries.append((name, path))
    return entries


def build_table(entries: list[PacEntry], name_encoding: str) -> bytes:
    out = bytearray()
    for e in entries:
        name_bytes = e.name.encode(name_encoding)
        if len(name_bytes) >= 64:
            raise PacError(f"file name too long for PAC table: {e.name!r} ({len(name_bytes)} bytes)")
        rec = bytearray(76)
        rec[:len(name_bytes)] = name_bytes
        struct.pack_into("<III", rec, 64, e.offset, e.original_size, e.compressed_size)
        out += rec
    return bytes(out)


def pack_pac(input_dir: Path, out_pac: Path, manifest_path: Optional[Path] = None,
             name_encoding: str = DEFAULT_NAME_ENCODING, compression: int = DEFAULT_PACK_COMPRESSION,
             zstd_level: int = 3) -> None:
    if compression != 7:
        raise PacError("当前 pack 只输出本游戏原包使用的 compression=7/Zstandard 格式")
    if manifest_path is None and (input_dir / "manifest.json").exists():
        manifest_path = input_dir / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        name_encoding = manifest.get("name_encoding", name_encoding)
    files = load_pack_entries(input_dir, manifest_path, name_encoding)
    if not files:
        raise PacError("no files to pack")

    body = bytearray(PAC_MAGIC + b"\0" + struct.pack("<II", len(files), compression))
    pac_entries: list[PacEntry] = []
    for name, path in files:
        raw = path.read_bytes()
        offset = len(body)
        if should_store_raw(name):
            comp = raw
        else:
            comp = zstd_compress(raw, level=zstd_level)
        body += comp
        pac_entries.append(PacEntry(name=name, name_hex=name.encode(name_encoding).hex(),
                                    offset=offset, original_size=len(raw), compressed_size=len(comp)))

    table_raw = build_table(pac_entries, name_encoding)
    table_comp = HuffmanTableCodec.compress(table_raw)
    table_enc = bytes((~b) & 0xFF for b in table_comp)
    body += table_enc
    body += struct.pack("<I", len(table_enc))
    out_pac.parent.mkdir(parents=True, exist_ok=True)
    out_pac.write_bytes(bytes(body))
    print(f"[pack] file_count={len(files)} compression={compression} name_encoding={name_encoding} output={out_pac} size={out_pac.stat().st_size}")


def info_pac(pac_path: Path, name_encoding: str = "auto", verbose: bool = False) -> None:
    data = pac_path.read_bytes()
    count, compression = read_header(data)
    if name_encoding == "auto":
        name_encoding = detect_name_encoding(data)
    entries, _ = parse_table(data, name_encoding)
    table_size = struct.unpack_from("<I", data, len(data) - 4)[0]
    print(f"[info] file={pac_path}")
    print(f"[info] size={len(data)} file_count={count} compression={compression} name_encoding={name_encoding} table_comp_size={table_size}")
    if verbose:
        for i, e in enumerate(entries):
            chunk = data[e.offset:e.offset + e.compressed_size]
            sig = chunk[:4].hex()
            print(f"[{i:02d}] off={e.offset:08x} raw={e.original_size:7d} comp={e.compressed_size:6d} sig={sig} name={e.name}")


def _entry_name_sets(pac_path: Path, enc: str) -> set[str]:
    return {e.name for e in parse_table(pac_path.read_bytes(), enc)[0]}


def diagnose_pac(original_pac: Path, chs_pac: Path) -> None:
    orig_data = original_pac.read_bytes()
    chs_data = chs_pac.read_bytes()
    ocnt, octype = read_header(orig_data)
    ccnt, cctype = read_header(chs_data)
    print(f"[diagnose] original: file_count={ocnt}, compression={octype}, size={len(orig_data)}")
    print(f"[diagnose] chs     : file_count={ccnt}, compression={cctype}, size={len(chs_data)}")

    oenc = detect_name_encoding(orig_data)
    cenc = detect_name_encoding(chs_data)
    print(f"[diagnose] original name encoding guess: {oenc}")
    print(f"[diagnose] chs name encoding guess     : {cenc}")

    orig_entries = parse_table(orig_data, oenc)[0]
    chs_entries_auto = parse_table(chs_data, cenc)[0]
    chs_entries_utf8 = parse_table(chs_data, "utf-8")[0]

    if ocnt != ccnt:
        print(f"[bad] file_count differs: original={ocnt}, chs={ccnt}")
    if octype != cctype:
        print(f"[bad] archive compression selector differs: original={octype}, chs={cctype}")

    if chs_entries_auto:
        first = chs_data[chs_entries_auto[0].offset:chs_entries_auto[0].offset + min(4, chs_entries_auto[0].compressed_size)]
        print(f"[diagnose] chs first chunk signature={first.hex()}")
        if first.startswith(b"\x78"):
            print("[bad] chs chunks look like zlib streams; original Script.pac chunks are Zstandard frames (28 b5 2f fd)")

    orig_names = {e.name for e in orig_entries}
    chs_names_auto = {e.name for e in chs_entries_auto}
    chs_names_utf8 = {e.name for e in chs_entries_utf8}

    missing_auto = sorted(orig_names - chs_names_auto)
    extra_auto = sorted(chs_names_auto - orig_names)
    if missing_auto or extra_auto:
        print(f"[bad] name set differs when chs is decoded as {cenc}:")
        print("      missing:", missing_auto)
        print("      extra  :", extra_auto)

    missing_utf8 = sorted(orig_names - chs_names_utf8)
    if len(missing_utf8) > len(missing_auto):
        print("[bad] chs table is not UTF-8-compatible like the original; Japanese names become mojibake under UTF-8")

    # Try reading every chs chunk to detect self-consistency.
    ok = 0
    fail = 0
    for e in chs_entries_auto:
        chunk = chs_data[e.offset:e.offset + e.compressed_size]
        try:
            decompress_chunk(chunk, e.original_size, cctype)
            ok += 1
        except Exception as ex:
            fail += 1
            print(f"[bad] cannot decompress chs entry {e.name}: {ex}")
    print(f"[diagnose] chs chunk decompress check: ok={ok}, failed={fail}")


def recover_chs_to_original_layout(original_pac: Path, chs_pac: Path, out_pac: Path,
                                   work_dir: Path, zstd_level: int = 3) -> None:
    """Best-effort repair: unpack original, overlay files recovered from chs, pack as original ctype7/UTF-8."""
    if work_dir.exists():
        shutil.rmtree(work_dir)
    orig_dir = work_dir / "orig"
    chs_dir = work_dir / "chs_cp932"
    unpack_pac(original_pac, orig_dir, "utf-8")
    unpack_pac(chs_pac, chs_dir, "cp932")

    # Overlay matching names. The broken chs package omitted the leading "機能" in this sample.
    rename_map = {
        "チュートリアル.bin": "機能チュートリアル.bin",
    }
    copied = 0
    for path in chs_dir.rglob("*"):
        if not path.is_file() or path.name == "manifest.json":
            continue
        rel = path.relative_to(chs_dir).as_posix()
        rel = rename_map.get(rel, rel)
        dst = safe_output_path(orig_dir, rel)
        if dst.exists():
            shutil.copy2(path, dst)
            copied += 1
        else:
            print(f"[recover][warn] skip chs-only file not present in original layout: {rel}")
    print(f"[recover] overlaid {copied} files; original-only files are kept, including __global.bin")
    pack_pac(orig_dir, out_pac, orig_dir / "manifest.json", name_encoding="utf-8", compression=7, zstd_level=zstd_level)


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="制服カノジョ2.5 Script.pac unpack/pack/diagnose tool")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("info", help="show PAC header/table info")
    p.add_argument("pac")
    p.add_argument("--name-encoding", default="auto", choices=["auto", "utf-8", "cp932"])
    p.add_argument("-v", "--verbose", action="store_true")

    p = sub.add_parser("unpack", help="unpack PAC to directory")
    p.add_argument("pac")
    p.add_argument("out_dir")
    p.add_argument("--name-encoding", default="auto", choices=["auto", "utf-8", "cp932"])

    p = sub.add_parser("pack", help="pack directory to game-compatible PAC; uses manifest.json order when present")
    p.add_argument("input_dir")
    p.add_argument("out_pac")
    p.add_argument("--manifest")
    p.add_argument("--name-encoding", default="utf-8", choices=["utf-8", "cp932"])
    p.add_argument("--zstd-level", type=int, default=3)

    p = sub.add_parser("diagnose", help="compare original PAC and a user-made chs PAC")
    p.add_argument("original_pac")
    p.add_argument("chs_pac")

    p = sub.add_parser("recover-chs", help="best-effort repair for the supplied Script_chs.pac layout")
    p.add_argument("original_pac")
    p.add_argument("chs_pac")
    p.add_argument("out_pac")
    p.add_argument("--work-dir", default="_pac_recover_work")
    p.add_argument("--zstd-level", type=int, default=3)

    args = parser.parse_args(argv)
    try:
        if args.cmd == "info":
            info_pac(Path(args.pac), args.name_encoding, args.verbose)
        elif args.cmd == "unpack":
            unpack_pac(Path(args.pac), Path(args.out_dir), args.name_encoding)
        elif args.cmd == "pack":
            pack_pac(Path(args.input_dir), Path(args.out_pac), Path(args.manifest) if args.manifest else None,
                     name_encoding=args.name_encoding, zstd_level=args.zstd_level)
        elif args.cmd == "diagnose":
            diagnose_pac(Path(args.original_pac), Path(args.chs_pac))
        elif args.cmd == "recover-chs":
            recover_chs_to_original_layout(Path(args.original_pac), Path(args.chs_pac), Path(args.out_pac),
                                           Path(args.work_dir), zstd_level=args.zstd_level)
        else:
            parser.error("unknown command")
        return 0
    except PacError as e:
        print(f"[error] {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
