# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import shutil
import struct
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

DEFAULT_ENCODING = "cp932"

SCW_MAGIC = b"Scw5.x\x00\x00"
SCW_HEADER_SIZE = 0x1C8

# Expression value types observed in Scw5.x statement operands.
VALUE_TYPES = {1, 2, 4, 16, 32, 64, 256, 4096}

# Confirmed text-related opcodes. Extraction is driven by statement opcode + type=16 string operands.
TEXT_OPS = {0x01A8: "message_a8", 0x01AA: "message_aa"}
CALL_TEXT_OPS = {0x00CB: "call_text_cb"}
CHOICE_OPS = {0x01CE: "choice_ce"}
SYSTEM_TEXT_OPS = {0x01C9: "system_title_c9"}
EXPORT_OPS_DEFAULT = set(TEXT_OPS) | set(CALL_TEXT_OPS) | set(CHOICE_OPS)
EXPORT_OPS_WITH_SYSTEM = EXPORT_OPS_DEFAULT | set(SYSTEM_TEXT_OPS)


@dataclass
class ScwInfo:
    file: str
    version_raw: int
    mode_flag: int
    unpacked_size: int
    packed_size: int
    count_a: int
    count_b: int
    count_c: int
    size_a: int
    size_b: int
    size_c: int


@dataclass
class Layout:
    table_a: int
    table_b: int
    table_c: int
    block_a: int
    block_b: int
    block_c: int


def u16(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from("<H", data, off)[0]


def u32(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def xor_by_index(buf: bytearray) -> None:
    for i in range(len(buf)):
        buf[i] ^= i & 0xFF


def lzss_decompress(src: bytes, expected_size: int | None = None) -> bytes:
    """CGsLZSS decompressor used by both SCR.pak TOC and Scw5.x body."""
    ring = bytearray(0x1000)
    r = 4078
    flags = 0
    ip = 0
    out = bytearray()
    n = len(src)
    while ip < n:
        flags >>= 1
        if (flags & 0x100) == 0:
            if ip >= n:
                break
            flags = src[ip] | 0xFF00
            ip += 1
        if flags & 1:
            if ip >= n:
                break
            c = src[ip]
            ip += 1
            out.append(c)
            ring[r] = c
            r = (r + 1) & 0xFFF
        else:
            if ip + 1 > n:
                break
            lo = src[ip]
            hi = src[ip + 1]
            ip += 2
            pos = lo | ((hi & 0xF0) << 4)
            length = (hi & 0x0F) + 3
            for k in range(length):
                c = ring[(pos + k) & 0xFFF]
                out.append(c)
                ring[r] = c
                r = (r + 1) & 0xFFF
        if expected_size is not None and len(out) >= expected_size:
            return bytes(out[:expected_size])
    return bytes(out)


def read_scw_header(data: bytes, file_name: str = "<memory>") -> ScwInfo:
    if len(data) < SCW_HEADER_SIZE or not data.startswith(SCW_MAGIC):
        raise ValueError(f"not a Scw5.x script: {file_name}")
    return ScwInfo(
        file=file_name,
        version_raw=u32(data, 0x10),
        mode_flag=u32(data, 0x14),
        unpacked_size=u32(data, 0x18),
        packed_size=u32(data, 0x1C),
        count_a=u32(data, 0x24),
        count_b=u32(data, 0x28),
        count_c=u32(data, 0x2C),
        size_a=u32(data, 0x30),
        size_b=u32(data, 0x34),
        size_c=u32(data, 0x38),
    )


def decode_scw(data: bytes, file_name: str = "<memory>") -> tuple[bytes, ScwInfo]:
    """Return decoded Scw5.x body plus header info.

    Original scripts normally use mode_flag=0xFFFFFFFF: body stream is XOR-by-index, then CGsLZSS.
    Rebuilt scripts use mode_flag=0: body stream is XOR-by-index only. The engine has this raw branch.
    """
    info = read_scw_header(data, file_name)
    body_src = bytearray(data[SCW_HEADER_SIZE:SCW_HEADER_SIZE + info.packed_size])
    if len(body_src) != info.packed_size:
        raise ValueError(f"truncated Scw body: {file_name}")
    xor_by_index(body_src)
    if info.mode_flag == 0xFFFFFFFF:
        body = lzss_decompress(bytes(body_src), info.unpacked_size)
    else:
        body = bytes(body_src[:info.unpacked_size])
    if len(body) != info.unpacked_size:
        raise ValueError(f"decoded size mismatch: {file_name}: expected={info.unpacked_size}, got={len(body)}")
    return body, info


def layout_of(info: ScwInfo) -> Layout:
    table_a = 0
    table_b = table_a + info.count_a * 8
    table_c = table_b + info.count_b * 8
    block_a = table_c + info.count_c * 8
    block_b = block_a + info.size_a
    block_c = block_b + info.size_b
    total = block_c + info.size_c
    if total != info.unpacked_size:
        raise ValueError(f"body layout mismatch: {info.file}: sum=0x{total:X}, header=0x{info.unpacked_size:X}")
    return Layout(table_a, table_b, table_c, block_a, block_b, block_c)


def iter_source(input_path: Path) -> Iterable[tuple[str, bytes]]:
    """Yield (relative_name, bytes) from one .scw file, a directory, or a zip."""
    if input_path.is_dir():
        for p in sorted(x for x in input_path.rglob("*") if x.is_file() and x.name != "manifest.json" and x.suffix.lower() != ".json"):
            yield p.relative_to(input_path).as_posix(), p.read_bytes()
    elif input_path.suffix.lower() == ".zip":
        with zipfile.ZipFile(input_path) as zf:
            for name in sorted(n for n in zf.namelist() if not n.endswith("/")):
                yield name, zf.read(name)
    else:
        yield input_path.name, input_path.read_bytes()


def safe_name(name: str) -> str:
    return name.replace("\\", "/").strip("/").replace("/", "__")


def decode_string(raw: bytes, enc: str = DEFAULT_ENCODING) -> str:
    if raw.endswith(b"\x00"):
        raw = raw[:-1]
    return raw.decode(enc)


def encode_string(text: str, enc: str = DEFAULT_ENCODING) -> bytes:
    return text.encode(enc) + b"\x00"


def parse_string_table(body: bytes, info: ScwInfo, layout: Layout, table: str = "B") -> list[dict[str, Any]]:
    if table == "B":
        count, table_base, block_base = info.count_b, layout.table_b, layout.block_b
    elif table == "C":
        count, table_base, block_base = info.count_c, layout.table_c, layout.block_c
    else:
        raise ValueError(table)
    out: list[dict[str, Any]] = []
    for i in range(count):
        rel, size = struct.unpack_from("<II", body, table_base + i * 8)
        raw = body[block_base + rel:block_base + rel + size]
        try:
            text = decode_string(raw)
        except UnicodeDecodeError:
            text = ""
        out.append({
            "index": i,
            "table_offset": table_base + i * 8,
            "block_offset": block_base + rel,
            "relative_offset": rel,
            "size": size,
            "text": text,
            "raw": raw,
        })
    return out


def iter_statements(body: bytes, info: ScwInfo, layout: Layout) -> Iterable[dict[str, Any]]:
    for idx in range(info.count_a):
        rel, size = struct.unpack_from("<II", body, layout.table_a + idx * 8)
        start = layout.block_a + rel
        chunk = body[start:start + size]
        if len(chunk) < 0x18:
            continue
        yield {
            "index": idx,
            "table_offset": layout.table_a + idx * 8,
            "block_offset": start,
            "relative_offset": rel,
            "size": size,
            "opcode": u16(chunk, 0),
            "declared_size": u16(chunk, 2),
            "line": u16(chunk, 4),
            "serial": u16(chunk, 6),
            "flags0": u32(chunk, 8),
            "flags1": u32(chunk, 12),
            "flags2": u32(chunk, 16),
            "flags3": u32(chunk, 20),
            "chunk": chunk,
        }


def parse_expr_groups(chunk: bytes) -> list[dict[str, Any]]:
    """Find 00 FF expression groups and parse their typed value pairs.

    This is intentionally conservative: a candidate is accepted only if every pair's type
    belongs to the observed VALUE_TYPES set.
    """
    groups: list[dict[str, Any]] = []
    pos = 0
    while True:
        p = chunk.find(b"\x00\xFF", pos)
        if p < 0:
            break
        if p + 4 > len(chunk):
            break
        count = u16(chunk, p + 2)
        q = p + 4
        if count <= 32 and q + count * 8 <= len(chunk):
            pairs = []
            ok = True
            for k in range(count):
                typ, val = struct.unpack_from("<II", chunk, q + k * 8)
                if typ not in VALUE_TYPES:
                    ok = False
                    break
                pairs.append({"type": typ, "value": val})
            if ok:
                groups.append({"offset_in_stmt": p, "count": count, "pairs": pairs})
        pos = p + 2
    return groups


def has_japanese(text: str) -> bool:
    return any("\u3040" <= ch <= "\u30ff" or "\u4e00" <= ch <= "\u9fff" or ch in "。、！？「」…ー・" for ch in text)


def statement_op_name(opcode: int) -> str:
    return TEXT_OPS.get(opcode) or CALL_TEXT_OPS.get(opcode) or CHOICE_OPS.get(opcode) or SYSTEM_TEXT_OPS.get(opcode) or f"op_{opcode:04X}"


def entry_type(opcode: int) -> str:
    if opcode in CHOICE_OPS:
        return "choice"
    if opcode in CALL_TEXT_OPS:
        return "system"
    if opcode in SYSTEM_TEXT_OPS:
        return "system"
    return "dialogue"


def extract_text_entries(name: str, data: bytes, include_system: bool = False) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    body, info = decode_scw(data, name)
    layout = layout_of(info)
    str_b = parse_string_table(body, info, layout, "B")
    export_ops = EXPORT_OPS_WITH_SYSTEM if include_system else EXPORT_OPS_DEFAULT
    entries: list[dict[str, Any]] = []
    for st in iter_statements(body, info, layout):
        opcode = st["opcode"]
        if opcode not in export_ops:
            continue
        groups = parse_expr_groups(st["chunk"])
        for gi, group in enumerate(groups):
            for pi, pair in enumerate(group["pairs"]):
                if pair["type"] != 16:
                    continue
                si = pair["value"]
                if si < 0 or si >= len(str_b):
                    continue
                srec = str_b[si]
                text = srec["text"]
                if not text:
                    continue
                if opcode in CALL_TEXT_OPS and not has_japanese(text):
                    continue
                obj: dict[str, Any] = {
                    "scr_msg": text,
                    "message": text,
                    "_file": name,
                    "_index": len(entries),
                    "_type": entry_type(opcode),
                    "_opcode": f"0x{opcode:04X}",
                    "_op_name": statement_op_name(opcode),
                    "_line": st["line"],
                    "_chunk_index": st["index"],
                    "_inst_offset": st["block_offset"],
                    "_inst_size": st["size"],
                    "_expr_group": gi,
                    "_expr_offset": group["offset_in_stmt"],
                    "_expr_pair": pi,
                    "_str_table": "B",
                    "_str_index": si,
                    "_offset": srec["block_offset"],
                    "_size": srec["size"],
                    "_encoding": DEFAULT_ENCODING,
                    "_policy": "relocate",
                }
                entries.append(obj)
    meta = {
        "file": name,
        "decoded_size": info.unpacked_size,
        "packed_size": info.packed_size,
        "count_a_statements": info.count_a,
        "count_b_strings": info.count_b,
        "count_c_labels": info.count_c,
        "size_a_code": info.size_a,
        "size_b_strings": info.size_b,
        "size_c_labels": info.size_c,
        "entries": len(entries),
    }
    return entries, meta


def load_json_entries(json_path: Path) -> list[dict[str, Any]]:
    data = json.loads(json_path.read_text(encoding="utf-8"))
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be list: {json_path}")
    return data


def save_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


def normalize_file_key(s: str) -> str:
    s = s.replace("\\", "/").split("/")[-1]
    if s.lower().endswith(".json"):
        s = s[:-5]
    if s.lower().endswith(".scw"):
        s = s[:-4]
    return s


def load_translation_map(json_input: Path) -> dict[str, list[dict[str, Any]]]:
    by_file: dict[str, list[dict[str, Any]]] = {}
    if json_input.is_dir():
        paths = sorted(
            p for p in json_input.rglob("*.json")
            if p.name not in {"manifest.json", "all_text.json", "inject_report.json", "roundtrip_report.json"}
        )
    else:
        paths = [json_input]
    for jp in paths:
        entries = load_json_entries(jp)
        for e in entries:
            if "_file" in e:
                key = normalize_file_key(str(e["_file"]))
            else:
                key = normalize_file_key(jp.name)
            by_file.setdefault(key, []).append(e)
    return by_file


def rebuild_scw_with_entries(name: str, data: bytes, entries: list[dict[str, Any]]) -> tuple[bytes, dict[str, Any]]:
    """Rebuild blockB/tableB by string index. No in-place truncation.

    String indices remain unchanged, so blockA operands do not need patching.
    The rebuilt script uses mode_flag=0, i.e. XOR-only body accepted by the engine.
    """
    body, info = decode_scw(data, name)
    layout = layout_of(info)
    str_b = parse_string_table(body, info, layout, "B")
    replacements: dict[int, str] = {}
    warnings: list[str] = []
    failed = 0
    patched_entries = 0
    for e in entries:
        if e.get("_str_table", "B") != "B":
            warnings.append(f"skip non-B string table entry index={e.get('_index')}")
            continue
        try:
            si = int(e["_str_index"])
        except Exception:
            warnings.append(f"missing _str_index, skip index={e.get('_index')}")
            failed += 1
            continue
        if not (0 <= si < len(str_b)):
            warnings.append(f"_str_index out of range: {si}")
            failed += 1
            continue
        scr_msg = e.get("scr_msg")
        message = e.get("message")
        if not isinstance(scr_msg, str) or not isinstance(message, str):
            warnings.append(f"missing scr_msg/message at _str_index={si}")
            failed += 1
            continue
        current = str_b[si]["text"]
        if current != scr_msg:
            warnings.append(f"scr_msg mismatch file={name} _str_index={si}: json={scr_msg!r} file={current!r}")
            failed += 1
            continue
        if si in replacements and replacements[si] != message:
            warnings.append(f"conflicting translations for _str_index={si}; keep first")
            failed += 1
            continue
        replacements[si] = message
        patched_entries += 1

    encoded_strings: list[bytes] = []
    for rec in str_b:
        si = rec["index"]
        if si in replacements:
            try:
                raw = encode_string(replacements[si], DEFAULT_ENCODING)
            except UnicodeEncodeError as ex:
                warnings.append(f"cp932 encode failed file={name} _str_index={si}: {ex}")
                failed += 1
                raw = rec["raw"]
            encoded_strings.append(raw)
        else:
            encoded_strings.append(rec["raw"])

    new_block_b = bytearray()
    new_table_b = bytearray()
    for raw in encoded_strings:
        rel = len(new_block_b)
        new_block_b.extend(raw)
        new_table_b.extend(struct.pack("<II", rel, len(raw)))

    table_a = body[layout.table_a:layout.table_b]
    table_c = body[layout.table_c:layout.block_a]
    block_a = body[layout.block_a:layout.block_b]
    block_c = body[layout.block_c:layout.block_c + info.size_c]
    new_body = table_a + bytes(new_table_b) + table_c + block_a + bytes(new_block_b) + block_c

    new_header = bytearray(data[:SCW_HEADER_SIZE])
    struct.pack_into("<I", new_header, 0x14, 0)                 # xor-only mode
    struct.pack_into("<I", new_header, 0x18, len(new_body))
    struct.pack_into("<I", new_header, 0x1C, len(new_body))
    struct.pack_into("<I", new_header, 0x34, len(new_block_b))

    body_encoded = bytearray(new_body)
    xor_by_index(body_encoded)
    out = bytes(new_header) + bytes(body_encoded)
    return out, {
        "file": name,
        "entries_seen": len(entries),
        "patched_entries": patched_entries,
        "unique_strings_replaced": len(replacements),
        "failed": failed,
        "warnings": warnings,
        "old_body_size": info.unpacked_size,
        "new_body_size": len(new_body),
        "old_blockB_size": info.size_b,
        "new_blockB_size": len(new_block_b),
        "mode": "xor-only",
    }


def copy_tree_manifest(src: Path, out: Path, clean: bool = False) -> None:
    if out.exists() and clean:
        shutil.rmtree(out)
    out.mkdir(parents=True, exist_ok=True)
    if (src / "manifest.json").exists():
        shutil.copy2(src / "manifest.json", out / "manifest.json")
