# -*- coding: utf-8 -*-
"""
和香様の座する世界 ACV1 解包/封包工具

支持两种模式:
  script 模式 —— 处理 script.dat / scriptE.dat
  arc 模式    —— 处理 arc0.dat / arc1.dat / arc2.dat / arc3.dat ...

两种模式的核心差异（IDA 逆向确认）:
  ┌────────────┬──────────────────────────┬─────────────────────┐
  │            │ script.dat               │ arc*.dat            │
  ├────────────┼──────────────────────────┼─────────────────────┤
  │ 数据 XOR 密钥 │ key_lo ^ 0xBABA18A9    │ key_lo 直用 (无额外 XOR) │
  │ 偏移/大小修正  │ 无 (flag=6, bit1=1 跳过) │ flag 低位=0 时文件名 XOR │
  │ 压缩       │ zlib (flag=6)            │ flag≥1 用 zlib       │
  └────────────┴──────────────────────────┴─────────────────────┘

用法:
  # 自动检测模式（根据文件名判断 script/arc）
  python waka_dat_tool.py unpack <input_dat> <out_dir>

  # 显式指定模式
  python waka_dat_tool.py unpack <input_dat> <out_dir> --mode arc
  python waka_dat_tool.py unpack <input_dat> <out_dir> --mode script

  # 封包
  python waka_dat_tool.py pack <work_dir> <output_dat>

  # 校验
  python waka_dat_tool.py verify <input_dat> <work_dir>
"""

from __future__ import annotations

import argparse
import json
import re
import struct
import zlib
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any

MAGIC_ACV1 = 0x31564341  # b"ACV1"
MAGIC_XOR = 0x8B6A4E5F
COUNT_XOR_OLD = 0x26ACA46E

# 和香様の座する世界.exe 中由标题字符串 "a香様の座する世界" 计算出的 CRC64 低 32 位。
# 仅用于 script.dat 模式的数据块 dword 异或。
# arc 模式使用 entry.key_lo 直通，不混入此常量。
DAT_XOR_KEY_LO = 0xBABA18A9

DEFAULT_ENCODING = "cp932"


# ─── 工具函式 ────────────────────────────────────────────────
def u32(data: bytes, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def p32(v: int) -> bytes:
    return struct.pack("<I", v & 0xFFFFFFFF)


def xor_dwords(buf: bytearray, key: int) -> None:
    """按 dword 异或 buf 的前 (len//4) 个 dword。"""
    key &= 0xFFFFFFFF
    for off in range(0, len(buf) // 4 * 4, 4):
        struct.pack_into("<I", buf, off, struct.unpack_from("<I", buf, off)[0] ^ key)


def safe_name(s: str, max_len: int = 80) -> str:
    s = s.strip().replace("*", "")
    s = re.sub(r"[\\/:*?\"<>|\x00-\x1F\s]+", "_", s)
    s = s.strip("._")
    return (s[:max_len] or "no_label")


def key_name(lo: int, hi: int) -> str:
    return f"{hi:08X}_{lo:08X}"


def collect_labels(raw: bytes, encoding: str = DEFAULT_ENCODING) -> list[str]:
    text = raw.decode(encoding, errors="ignore")
    labels: list[str] = []
    seen: set[str] = set()
    for m in re.finditer(r"(?m)^\*([^\r\n: /\t]*)", text):
        label = m.group(1).strip()
        if label and label not in seen:
            seen.add(label)
            labels.append(label)
    return labels


# ─── 数据对象 ────────────────────────────────────────────────
@dataclass
class DatEntry:
    index: int
    key_lo: int
    key_hi: int
    flag: int
    offset: int          # 存档中存储的原始偏移 (可能含文件名 XOR)
    packed_size: int     # 存档中存储的原始大小 (可能含文件名 XOR)
    alloc_size: int

    # 修正后的实际值 (arc 模式在 parse 阶段填入)
    actual_offset: int | None = None
    actual_packed_size: int | None = None

    # 输出相关
    unpacked_size: int | None = None
    path: str | None = None
    labels: list[str] | None = None

    def to_json(self) -> dict[str, Any]:
        d = asdict(self)
        return {k: v for k, v in d.items() if v is not None}


# ─── 头部解析 ────────────────────────────────────────────────

CRC64_POLY = 0x42F0E1EBA9EA3693


def _crc64_table() -> list[int]:
    table: list[int] = []
    for i in range(256):
        crc = i << 56
        for _ in range(8):
            if (crc >> 63) & 1:
                crc = ((crc << 1) ^ CRC64_POLY) & 0xFFFFFFFFFFFFFFFF
            else:
                crc = (crc << 1) & 0xFFFFFFFFFFFFFFFF
        table.append(crc)
    return table


CRC64_TABLE = _crc64_table()


def crc64(data: bytes) -> int:
    crc = 0xFFFFFFFFFFFFFFFF
    for b in data:
        crc = (CRC64_TABLE[((crc >> 56) ^ b) & 0xFF] ^ (crc << 8)) & 0xFFFFFFFFFFFFFFFF
    return (~crc) & 0xFFFFFFFFFFFFFFFF


def entry_hash(e: DatEntry) -> int:
    return ((e.key_hi & 0xFFFFFFFF) << 32) | (e.key_lo & 0xFFFFFFFF)


def load_name_map(path: Path | None, encoding: str = DEFAULT_ENCODING, low_case: bool = False) -> dict[int, tuple[str, bytes]]:
    if path is None:
        return {}
    names: dict[int, tuple[str, bytes]] = {}
    with path.open("r", encoding="utf-8-sig") as f:
        for line in f:
            name = line.strip()
            if not name or name.startswith("#"):
                continue
            name = name.replace("\\", "/")
            raw_name = (name.lower() if low_case else name).encode(encoding)
            names[crc64(raw_name)] = (name, raw_name)
    return names


def decrypt_with_name(data: bytes | bytearray, raw_name: bytes) -> bytearray:
    out = bytearray(data)
    if not raw_name:
        return out
    block_len = len(out) // len(raw_name)
    n = 0
    for c in raw_name[:-1]:
        for _ in range(block_len):
            if n >= len(out):
                return out
            out[n] ^= c
            n += 1
    return out


def apply_name_to_arc_entry(e: DatEntry, raw_name: bytes) -> None:
    if (e.flag & 2) == 0 and raw_name:
        e.actual_offset = e.offset ^ raw_name[len(raw_name) >> 1]
        e.actual_packed_size = e.packed_size ^ raw_name[len(raw_name) >> 2]
        e.alloc_size ^= raw_name[len(raw_name) >> 3]


def parse_dat_header(data: bytes) -> tuple[bool, int, int, list[DatEntry]]:
    """解析 ACV1 头部，返回 (has_magic, table_end, count, entries)。"""
    if len(data) < 8:
        raise ValueError("file too small")
    first = u32(data, 0)
    if first == MAGIC_ACV1:
        has_magic = True
        count = u32(data, 4) ^ MAGIC_XOR
        entry_off = 8
        offset_xor = MAGIC_XOR
    else:
        has_magic = False
        count = first ^ COUNT_XOR_OLD
        entry_off = 4
        offset_xor = 0
    if count < 0 or count > 100000:
        raise ValueError(f"invalid entry count: {count}")
    need = entry_off + count * 21
    if need > len(data):
        raise ValueError(f"entry table truncated: need {need}, file size {len(data)}")

    entries: list[DatEntry] = []
    off = entry_off
    for i in range(count):
        key_lo, key_hi = struct.unpack_from("<II", data, off)
        off += 8
        flag = data[off] ^ (key_lo & 0xFF)
        off += 1
        file_off = u32(data, off) ^ key_lo ^ offset_xor
        off += 4
        packed_size = u32(data, off) ^ key_lo
        off += 4
        alloc_size = u32(data, off) ^ key_lo
        off += 4
        if file_off + packed_size > len(data) + 256:
            # 允许 256 字节宽限 (文件名 XOR 可能导致偏移误差)
            pass
        entries.append(DatEntry(i, key_lo, key_hi, flag, file_off, packed_size, alloc_size))
    return has_magic, entry_off + count * 21, count, entries


# ─── 脚本模式 解密/解压 ──────────────────────────────────────
def decrypt_entry_script(data: bytes, e: DatEntry, xor_key_lo: int = DAT_XOR_KEY_LO) -> bytes:
    packed = bytearray(data[e.offset : e.offset + e.packed_size])
    xor_dwords(packed, xor_key_lo ^ e.key_lo)
    return bytes(packed)


def unpack_entry_script(data: bytes, e: DatEntry, xor_key_lo: int = DAT_XOR_KEY_LO) -> bytes:
    packed = decrypt_entry_script(data, e, xor_key_lo)
    try:
        raw = zlib.decompress(packed)
    except zlib.error as ex:
        raise ValueError(f"zlib decompress failed at entry {e.index}: {ex}") from ex
    e.unpacked_size = len(raw)
    return raw


# ─── ARC 模式 解密 ───────────────────────────────────────────
def _resolve_arc_offset(
    data: bytes,
    e: DatEntry,
    header_end: int,
    expected_offset: int | None = None,
    next_stored_offset: int | None = None,
) -> tuple[int, int]:
    """
    Resolve ARC filename-XOR offset/size.

    The exe corrects offset with filename[len >> 1] and packed_size with
    filename[len >> 2], so the two low-byte XOR values are independent.
    We do not have the original names here, so choose the candidate that
    keeps blocks in table order and close to the next stored block.
    """
    stored_off = e.offset
    stored_sz = e.packed_size

    if (e.flag & 2) != 0:
        return stored_off, stored_sz

    target_off = header_end if expected_offset is None else expected_offset
    off_candidates = [stored_off ^ b for b in range(256)]
    off_candidates = [off for off in off_candidates if header_end <= off < len(data)]
    actual_off = min(off_candidates or [stored_off], key=lambda off: abs(off - target_off))

    if next_stored_offset is not None:
        next_candidates = [next_stored_offset ^ b for b in range(256)]
        next_candidates = [off for off in next_candidates if actual_off <= off <= len(data)]
        target_size = min(
            (off - actual_off for off in next_candidates),
            key=lambda sz: abs(sz - stored_sz),
            default=stored_sz,
        )
        actual_sz = min((stored_sz ^ b for b in range(256)), key=lambda sz: abs(sz - target_size))
    else:
        actual_sz = stored_sz

    if actual_off + actual_sz > len(data):
        actual_sz = len(data) - actual_off
    if actual_sz <= 0:
        actual_sz = stored_sz

    return actual_off, actual_sz

def _try_decompress_arc(packed: bytes, flag: int) -> bytes | None:
    """尝试对已异或的数据进行解压。返回解压结果或 None。"""
    if flag < 1:
        return None  # flag=0: 原样存储，不解压

    min_expected = max(512, len(packed) // 5)  # 有效解压至少是压缩大小的 20% 且 ≥512 字节

    def _try_decompress(data: bytes, wbits: int) -> bytes | None:
        try:
            result = zlib.decompress(data, wbits)
            if len(result) >= min_expected:
                return result
        except zlib.error:
            pass
        return None

    for wbits in (15, -15, 15 + 32):
        result = _try_decompress(packed, wbits)
        if result is not None:
            return result
    return None


def detect_payload(raw: bytes) -> tuple[int, str]:
    """Return (payload_offset, extension) for common embedded resource headers."""
    candidates: list[tuple[int, str]] = []
    search_len = min(len(raw), 0x100)
    probes = (
        (b"TLG0.0\x00sds\x1a", ".tlg"),
        (b"TLG5.0\x00raw\x1a", ".tlg"),
        (b"TLG6.0\x00raw\x1a", ".tlg"),
        (b"\xABLG5.0\x00raw\x1a", ".tlg"),
        (b"\xABLG6.0\x00raw\x1a", ".tlg"),
        (b"\x89PNG\r\n\x1a\n", ".png"),
        (b"RIFF", ".riff"),
        (b"OggS", ".ogg"),
        (b"BM", ".bmp"),
        (b"\x1bLua", ".lua"),
        (b"PK\x03\x04", ".zip"),
    )
    head = raw[:search_len]
    for magic, ext in probes:
        pos = head.find(magic)
        if pos >= 0:
            candidates.append((pos, ext))
    if not candidates:
        return 0, ".bin"
    pos, ext = min(candidates, key=lambda item: item[0])
    if ext == ".riff" and raw[pos + 8 : pos + 12] == b"WAVE":
        ext = ".wav"
    elif ext == ".riff" and raw[pos + 8 : pos + 12] == b"AVI ":
        ext = ".avi"
    return pos, ext


def unpack_entry_arc(data: bytes, e: DatEntry, raw_name: bytes | None = None, master_key: int = 0) -> bytes:
    """ARC 模式解密/解压单项。"""
    actual_off = e.actual_offset if e.actual_offset is not None else e.offset
    actual_sz = e.actual_packed_size if e.actual_packed_size is not None else e.packed_size

    if actual_off + actual_sz > len(data):
        actual_sz = len(data) - actual_off
    if actual_sz <= 0:
        e.unpacked_size = 0
        return b""

    packed = bytearray(data[actual_off : actual_off + actual_sz])
    if (e.flag & 2) != 0:
        key = (entry_hash(e) ^ master_key) & 0xFFFFFFFF if master_key else e.key_lo
        xor_dwords(packed, key)
        decompressed = _try_decompress_arc(bytes(packed), e.flag)
        if decompressed is not None:
            e.unpacked_size = len(decompressed)
            return decompressed
    elif raw_name is not None and e.flag != 0:
        packed = decrypt_with_name(packed, raw_name)


    # 解压失败 — 保留已异或的原始数据
    raw = bytes(packed)
    e.unpacked_size = len(raw)
    return raw


# ─── 解包主流程 ──────────────────────────────────────────────
def unpack_dat(
    input_dat: Path,
    out_dir: Path,
    encoding: str = DEFAULT_ENCODING,
    xor_key_lo: int = DAT_XOR_KEY_LO,
    mode: str = "auto",
    name_list: Path | None = None,
    low_case_names: bool = False,
    scheme_title: str | None = None,
    only_named: bool = False,
) -> None:
    data = input_dat.read_bytes()
    name_map = load_name_map(name_list, encoding, low_case_names) if mode in ("auto", "arc") else {}
    master_key = crc64(scheme_title.encode(encoding)) if scheme_title else 0
    has_magic, table_end, count, entries = parse_dat_header(data)

    # ── 模式判定 ──
    if mode == "auto":
        name_lower = input_dat.name.lower()
        if "arc" in name_lower:
            mode = "arc"
        elif "script" in name_lower:
            mode = "script"
        else:
            # 根据条目 flag 分布推断
            flag_vals = set(e.flag for e in entries)
            if flag_vals & {1, 5} and not (flag_vals & {6, 7}):
                mode = "arc"
            else:
                mode = "script"
    print(f"[mode] {mode}")
    print(f"[unpack] entries={count}")

    out_dir.mkdir(parents=True, exist_ok=True)
    files_dir = out_dir / "files"
    files_dir.mkdir(parents=True, exist_ok=True)

    # ── ARC 模式：预解析偏移 ──
    if mode == "arc":
        print("[arc] resolving filename-XOR offsets...")
        named = 0
        expected_offset = table_end
        for i, e in enumerate(entries):
            rec = name_map.get(entry_hash(e))
            if rec is not None:
                apply_name_to_arc_entry(e, rec[1])
                named += 1
            else:
                next_stored = entries[i + 1].offset if i + 1 < len(entries) else None
                act_off, act_sz = _resolve_arc_offset(data, e, table_end, expected_offset, next_stored)
                e.actual_offset = act_off
                e.actual_packed_size = act_sz
            expected_offset = (
                (e.actual_offset if e.actual_offset is not None else e.offset)
                + (e.actual_packed_size if e.actual_packed_size is not None else e.packed_size)
            )
        corrected = sum(1 for e in entries if e.actual_offset is not None and e.actual_offset != e.offset)
        print(f"[arc] name-list matched {named}/{count} entries")
        print(f"[arc] {corrected}/{count} entries had offset corrected")

    # ── 解压 ──
    used_names: set[str] = set()
    manifest_entries: list[dict[str, Any]] = []
    total_raw = 0
    failed = 0
    decompressed_ok = 0
    raw_entries = 0  # flag=0 或解压失败的条目（保留 XOR 后原始数据）

    for e in entries:
        known_rec = name_map.get(entry_hash(e)) if mode == "arc" else None
        if mode == "arc" and only_named and known_rec is None:
            continue
        try:
            if mode == "arc":
                raw = unpack_entry_arc(data, e, known_rec[1] if known_rec is not None else None, master_key)
                # 判断是否成功 zlib 解压
                if (e.flag & 2) != 0 and e.unpacked_size is not None:
                    if e.unpacked_size != e.actual_packed_size and e.unpacked_size != e.packed_size:
                        decompressed_ok += 1
                    else:
                        raw_entries += 1  # flag>=1 但无需/无法 zlib 解压
                else:
                    raw_entries += 1  # flag=0 原样存储
            else:
                raw = unpack_entry_script(data, e, xor_key_lo)
                decompressed_ok += 1
        except Exception as ex:
            failed += 1
            if failed <= 10:
                print(f"  [FAIL] entry {e.index}: {ex}")
            continue

        total_raw += len(raw)
        labels = collect_labels(raw, encoding) if mode == "script" else []
        e.labels = labels
        if known_rec is not None:
            name = known_rec[0].replace("\\", "/").strip("/")
        else:
            base = safe_name(labels[0]) if labels else key_name(e.key_lo, e.key_hi)
            name = f"{e.index:04d}_{base}_f{e.flag}"
            if name in used_names:
                name = f"{e.index:04d}_{base}_{key_name(e.key_lo, e.key_hi)}_f{e.flag}"
            used_names.add(name)

        ext = ""
        if mode == "arc":
            payload_off, ext = detect_payload(raw)
            if payload_off:
                raw = raw[payload_off:]
                e.unpacked_size = len(raw)

        e.path = f"files/{name if known_rec is not None else name + ext}"
        out_path = out_dir / e.path
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(raw)
        manifest_entries.append(e.to_json())

        if (e.index + 1) % 500 == 0:
            pct = (e.index + 1) * 100 // count
            print(f"  [{e.index + 1}/{count}] {pct}% total={total_raw // 1024 // 1024}MB "
                  f"zlib_ok={decompressed_ok} raw={raw_entries} fail={failed}")

    # ── manifest ──
    manifest = {
        "format": "ACV1-script-dat" if mode == "script" else "ACV1-arc-dat",
        "source": input_dat.name,
        "has_magic": has_magic,
        "count": count,
        "table_end": table_end,
        "xor_key_lo": f"0x{xor_key_lo:08X}" if mode == "script" else None,
        "encoding": encoding,
        "mode": mode,
        "entries": manifest_entries,
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"[unpack] entries={count}  total={total_raw}  "
          f"zlib_ok={decompressed_ok}  raw={raw_entries}  fail={failed}")
    print(f"[unpack] output={out_dir}")


# ─── 封包 ──────────────────────────────────────────────────
def load_manifest(work_dir: Path) -> dict[str, Any]:
    manifest_path = work_dir / "manifest.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(f"manifest not found: {manifest_path}")
    with manifest_path.open("r", encoding="utf-8") as f:
        manifest = json.load(f)
    if not isinstance(manifest, dict) or not isinstance(manifest.get("entries"), list):
        raise ValueError("bad manifest.json")
    return manifest


def pack_dat(work_dir: Path, output_dat: Path, level: int | None = None) -> None:
    manifest = load_manifest(work_dir)
    entries_json = manifest["entries"]
    has_magic = bool(manifest.get("has_magic", True))
    mode = str(manifest.get("mode", "script"))
    xor_key_s = str(manifest.get("xor_key_lo", f"0x{DAT_XOR_KEY_LO:08X}"))
    xor_key_lo = int(xor_key_s, 16) if xor_key_s.lower().startswith("0x") else int(xor_key_s)
    encoding = str(manifest.get("encoding", DEFAULT_ENCODING))

    count = len(entries_json)
    header_size = (8 if has_magic else 4) + count * 21
    file_blobs: list[bytes] = []
    new_entries: list[DatEntry] = []
    cur = header_size

    for i, obj in enumerate(entries_json):
        key_lo = int(obj["key_lo"])
        key_hi = int(obj["key_hi"])
        old_flag = int(obj.get("flag", 6))
        zlevel = old_flag if level is None else level
        if zlevel < 0 or zlevel > 9:
            zlevel = 6
        rel = obj.get("path")
        if not isinstance(rel, str):
            raise ValueError(f"entry {i} missing path")

        raw = (work_dir / rel).read_bytes()

        # 压缩
        if mode == "arc" and zlevel >= 1:
            packed = bytearray(zlib.compress(raw, zlevel))
        elif mode == "arc":
            packed = bytearray(raw)  # flag=0: 不压缩
        else:
            packed = bytearray(zlib.compress(raw, zlevel))

        # 异或
        if mode == "arc":
            if zlevel != 0:
                xor_dwords(packed, key_lo)
        else:
            xor_dwords(packed, xor_key_lo ^ key_lo)

        blob = bytes(packed)
        file_blobs.append(blob)
        new_entries.append(
            DatEntry(
                index=i,
                key_lo=key_lo,
                key_hi=key_hi,
                flag=zlevel,
                offset=cur,
                packed_size=len(blob),
                alloc_size=max(len(raw), int(obj.get("alloc_size", 0))),
                unpacked_size=len(raw),
                path=rel,
                labels=obj.get("labels") if isinstance(obj.get("labels"), list) else None,
            )
        )
        cur += len(blob)

    out = bytearray()
    if has_magic:
        out += p32(MAGIC_ACV1)
        out += p32(count ^ MAGIC_XOR)
        offset_xor = MAGIC_XOR
    else:
        out += p32(count ^ COUNT_XOR_OLD)
        offset_xor = 0

    for e in new_entries:
        out += p32(e.key_lo)
        out += p32(e.key_hi)
        out += bytes([(e.flag ^ (e.key_lo & 0xFF)) & 0xFF])
        out += p32(e.offset ^ e.key_lo ^ offset_xor)
        out += p32(e.packed_size ^ e.key_lo)
        out += p32(e.alloc_size ^ e.key_lo)
    for blob in file_blobs:
        out += blob

    output_dat.parent.mkdir(parents=True, exist_ok=True)
    output_dat.write_bytes(out)
    print(f"[pack] entries={count}  mode={mode}")
    print(f"[pack] output_size={len(out)}")
    print(f"[pack] output={output_dat}")


def verify_dat(input_dat: Path, work_dir: Path) -> None:
    data = input_dat.read_bytes()
    _, _, count, entries = parse_dat_header(data)
    manifest = load_manifest(work_dir)
    mode = str(manifest.get("mode", "script"))
    ok = 0
    bad = 0
    for e, obj in zip(entries, manifest["entries"]):
        try:
            if mode == "arc":
                if e.actual_offset is not None:
                    e.offset = e.actual_offset
                raw1 = unpack_entry_arc(data, e)
            else:
                raw1 = unpack_entry_script(data, e)
            raw2 = (work_dir / obj["path"]).read_bytes()
            if raw1 == raw2:
                ok += 1
            else:
                bad += 1
                print(
                    f"[verify][bad] index={e.index} path={obj.get('path')} "
                    f"original={len(raw1)} work={len(raw2)}"
                )
        except Exception as ex:
            bad += 1
            print(f"[verify][err] index={e.index}: {ex}")
    print(f"[verify] entries={count} ok={ok} bad={bad}")


# ─── CLI ───────────────────────────────────────────────────
def main() -> None:
    ap = argparse.ArgumentParser(
        description="和香様の座する世界 ACV1 解包/封包工具 (script + arc 双模式)"
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("unpack", help="解包 dat 文件")
    p.add_argument("input_dat")
    p.add_argument("out_dir")
    p.add_argument("--encoding", default=DEFAULT_ENCODING)
    p.add_argument(
        "--mode",
        default="auto",
        choices=["auto", "script", "arc"],
        help="处理模式: auto (按文件名推断), script, arc",
    )
    p.add_argument(
        "--xor-key-lo",
        default=f"0x{DAT_XOR_KEY_LO:08X}",
        help="script 模式的数据块 dword xor key 低 32 位",
    )
    p.add_argument("--name-list", default=None, help="ARC filename list, one path per line")
    p.add_argument("--low-case-names", action="store_true", help="hash names as lowercase cp932")
    p.add_argument("--scheme-title", default=None, help="GARbro NonColor scheme title for packed entries")
    p.add_argument("--only-named", action="store_true", help="with --name-list, export only matched ARC entries")

    p = sub.add_parser("pack", help="按 manifest.json 回封")
    p.add_argument("work_dir")
    p.add_argument("output_dat")
    p.add_argument("--level", type=int, default=None, help="zlib 压缩等级；默认沿用各 entry flag")

    p = sub.add_parser("verify", help="校验解包目录与原 dat 的明文是否一致")
    p.add_argument("input_dat")
    p.add_argument("work_dir")

    args = ap.parse_args()
    if args.cmd == "unpack":
        key = (
            int(args.xor_key_lo, 16)
            if str(args.xor_key_lo).lower().startswith("0x")
            else int(args.xor_key_lo)
        )
        unpack_dat(
            Path(args.input_dat),
            Path(args.out_dir),
            args.encoding,
            key,
            args.mode,
            Path(args.name_list) if args.name_list else None,
            args.low_case_names,
            args.scheme_title,
            args.only_named,
        )
    elif args.cmd == "pack":
        pack_dat(Path(args.work_dir), Path(args.output_dat), args.level)
    elif args.cmd == "verify":
        verify_dat(Path(args.input_dat), Path(args.work_dir))


if __name__ == "__main__":
    main()
