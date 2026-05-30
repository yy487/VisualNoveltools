# -*- coding: utf-8 -*-
from __future__ import annotations

import configparser
import json
import re
import struct
from collections import OrderedDict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

DEFAULT_ENCODING = "cp932"
DLR_MAGIC = b"\x00DLR"
DEF_SEED = 0xAE85A916


def u16(data: bytes, off: int) -> int:
    return int.from_bytes(data[off:off + 2], "little")


def u32(data: bytes, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little")


def write_u32(buf: bytearray, off: int, value: int) -> None:
    struct.pack_into("<I", buf, off, value & 0xFFFFFFFF)


class PE:
    def __init__(self, path: str | Path):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        pe = u32(self.data, 0x3C)
        if self.data[pe:pe + 4] != b"PE\0\0":
            raise ValueError(f"not a PE file: {self.path}")
        coff = pe + 4
        nsec = u16(self.data, coff + 2)
        opt_size = u16(self.data, coff + 16)
        opt = coff + 20
        magic = u16(self.data, opt)
        if magic == 0x10B:
            dd_base = opt + 96
        elif magic == 0x20B:
            dd_base = opt + 112
        else:
            raise ValueError(f"unknown PE optional header magic: 0x{magic:X}")
        self.resource_rva = u32(self.data, dd_base + 8 * 2)
        self.resource_size = u32(self.data, dd_base + 8 * 2 + 4)
        sec = opt + opt_size
        self.sections: list[tuple[str, int, int, int]] = []
        for i in range(nsec):
            o = sec + i * 40
            name = self.data[o:o + 8].split(b"\0", 1)[0].decode("ascii", "replace")
            vsize = u32(self.data, o + 8)
            va = u32(self.data, o + 12)
            raw_size = u32(self.data, o + 16)
            raw_ptr = u32(self.data, o + 20)
            self.sections.append((name, va, max(vsize, raw_size), raw_ptr))

    def rva_to_off(self, rva: int) -> int:
        for _name, va, size, ptr in self.sections:
            if va <= rva < va + size:
                return ptr + (rva - va)
        raise ValueError(f"RVA not mapped: 0x{rva:X}")

    def read_rva(self, rva: int, size: int) -> bytes:
        off = self.rva_to_off(rva)
        return self.data[off:off + size]


def _res_name(pe: PE, base: int, x: int) -> int | str:
    if x & 0x80000000:
        off = base + (x & 0x7FFFFFFF)
        n = u16(pe.data, off)
        return pe.data[off + 2:off + 2 + n * 2].decode("utf-16le", "replace")
    return x


def iter_resources(pe: PE):
    base = pe.rva_to_off(pe.resource_rva)

    def walk(dir_off: int, path: list[int | str]):
        named = u16(pe.data, dir_off + 12)
        ids = u16(pe.data, dir_off + 14)
        ent = dir_off + 16
        for i in range(named + ids):
            eoff = ent + i * 8
            name_raw = u32(pe.data, eoff)
            data_raw = u32(pe.data, eoff + 4)
            name = _res_name(pe, base, name_raw)
            if data_raw & 0x80000000:
                yield from walk(base + (data_raw & 0x7FFFFFFF), path + [name])
            else:
                de = base + data_raw
                rva = u32(pe.data, de)
                size = u32(pe.data, de + 4)
                yield path + [name], rva, size

    yield from walk(base, [])


def get_bitmap_98(pe: PE) -> bytes:
    hits = []
    for path, rva, size in iter_resources(pe):
        if len(path) >= 2 and path[0] == 2 and path[1] == 0x98:
            hits.append((path, rva, size))
    if not hits:
        raise ValueError("RT_BITMAP id=0x98 not found")
    return pe.read_rva(hits[0][1], hits[0][2])


def parse_dib_rows(raw: bytes):
    header = u32(raw, 0)
    w = struct.unpack_from("<i", raw, 4)[0]
    h = struct.unpack_from("<i", raw, 8)[0]
    bpp = u16(raw, 14)
    comp = u32(raw, 16)
    if header < 40 or w != 32 or abs(h) != 32 or bpp != 24 or comp != 0:
        raise ValueError(f"unexpected DIB resource: {w}x{h} bpp={bpp} comp={comp}")
    stride = ((w * bpp + 31) // 32) * 4
    rows_file = [raw[header + y * stride:header + (y + 1) * stride] for y in range(abs(h))]
    return list(reversed(rows_file)) if h > 0 else rows_file


def sample_bitmap_bits(rows, x: int = 31, byte_index: int = 0, reverse_y: bool = True) -> int:
    rs = list(reversed(rows)) if reverse_y else rows
    acc = 0
    for y in range(32):
        acc = ((acc << 1) | (rs[y][x * 3 + byte_index] & 1)) & 0xFFFFFFFF
    return acc


def resident_checksum(text: str, encoding: str = DEFAULT_ENCODING) -> int:
    b = text.encode(encoding, errors="replace")
    s0 = s1 = s2 = s3 = 0
    i = 0
    while i < len(b):
        s0 = (s0 + b[i]) & 0xFF
        i += 1
        if i >= len(b):
            break
        s1 = (s1 + b[i]) & 0xFF
        i += 1
        if i >= len(b):
            break
        s2 = (s2 + b[i]) & 0xFF
        i += 1
        if i >= len(b):
            break
        s3 = (s3 + b[i]) & 0xFF
        i += 1
    return (s3 << 24) | (s2 << 16) | (s1 << 8) | s0


def ini_concat(path: str | Path, include_title: bool = False, encoding: str = DEFAULT_ENCODING) -> tuple[str, int]:
    cp = configparser.ConfigParser(interpolation=None, strict=False)
    cp.optionxform = str
    raw = Path(path).read_bytes().decode(encoding, errors="replace")
    cp.read_string(raw)

    def get(key: str, default: str = "") -> str:
        return cp.get("setting", key, fallback=default)

    try:
        flags = int(get("FLAGS", "0"), 0)
    except Exception:
        flags = 0

    keys = ["CLASS"]
    if include_title:
        keys.append("TITLE")
    keys += [
        "W_VIEW", "H_VIEW", "N_REG", "N_STRREG", "N_SYSREG", "N_LOCREG",
        "N_USAVE", "N_ASAVE", "N_QSAVE", "N_CG", "N_MESSAGE", "N_SCENE", "N_SOUND",
    ]
    if flags & 4:
        keys += ["FLAGS", "GUID", "SVDATA"]
    return "".join(get(k, "") for k in keys), flags


def calc_normal_seed(exe_path: str | Path, ini_path: str | Path, include_title: bool = False) -> tuple[int, int, int]:
    pe = PE(exe_path)
    rows = parse_dib_rows(get_bitmap_98(pe))
    bits = sample_bitmap_bits(rows, x=31, byte_index=0, reverse_y=True)
    concat, _flags = ini_concat(ini_path, include_title=include_title)
    chk = resident_checksum(concat)
    return bits ^ chk, bits, chk


def resident_rng_key_table(seed: int) -> list[int]:
    # resident.dll 自定义 seed 初始化：LCG 填充 624 状态，然后按 MT19937 twist/temper 输出。
    mt: list[int] = []
    eax = seed & 0xFFFFFFFF
    for _ in range(624):
        ecx = (eax * 0x10DCD) & 0xFFFFFFFF
        edx = eax & 0xFFFF0000
        ecx = (ecx + 1) & 0xFFFFFFFF
        mt.append(((ecx >> 16) | edx) & 0xFFFFFFFF)
        eax = ((ecx * 0x10DCD) + 1) & 0xFFFFFFFF
    for i in range(624):
        y = (mt[i] & 0x80000000) | (mt[(i + 1) % 624] & 0x7FFFFFFF)
        mt[i] = (mt[(i + 397) % 624] ^ (y >> 1) ^ (0x9908B0DF if y & 1 else 0)) & 0xFFFFFFFF
    out: list[int] = []
    for i in range(256):
        y = mt[i]
        y ^= y >> 11
        y ^= (y << 7) & 0x9D2C5680
        y ^= (y << 15) & 0xEFC60000
        y ^= y >> 18
        out.append(y & 0xFFFFFFFF)
    return out


def crypt_rld(data: bytes, seed: int) -> bytes:
    keys = resident_rng_key_table(seed)
    buf = bytearray(data)
    if len(buf) < 0x10:
        raise ValueError("RLD too small")
    count = ((len(buf) - 0x10) >> 2) & 0xFFFF
    count = min(count, 0x3FF0)
    for k in range(count):
        off = 0x10 + k * 4
        v = struct.unpack_from("<I", buf, off)[0]
        v ^= (keys[k & 0xFF] ^ seed) & 0xFFFFFFFF
        struct.pack_into("<I", buf, off, v)
    return bytes(buf)


def is_def_key_rld_name(path: str | Path) -> bool:
    """Files that actually use the fixed def seed.

    Important: in the tested build, defChara.rld stores character definitions but uses
    the normal scenario seed, not DEF_SEED.  Do not classify every def* file as
    a def-key file.
    """
    stem = Path(path).stem.lower()
    return stem == "def"


def is_definition_rld_name(path: str | Path) -> bool:
    """Definition/system RLD files that should not be exported as scenario text."""
    stem = Path(path).stem.lower()
    return stem in {"def", "defchara"}


def is_def_rld_name(path: str | Path) -> bool:
    """Backward-compatible alias used by older CLI code for output skipping."""
    return is_definition_rld_name(path)


def seed_for_path(path: str | Path, normal_seed: int | None = None, force_def: bool = False) -> int:
    if force_def or is_def_key_rld_name(path):
        return DEF_SEED
    if normal_seed is None:
        raise ValueError("normal seed is required for non-def RLD files")
    return normal_seed




@dataclass
class BitmapCandidate:
    path: list[int | str]
    rva: int
    size: int
    width: int
    height: int
    bpp: int
    comp: int
    rows: list[bytes]


def parse_dib_rows_any(raw: bytes):
    """Parse a RT_BITMAP DIB resource. Return (w,h,bpp,comp,rows_top_down)."""
    if len(raw) < 40:
        raise ValueError("DIB too small")
    header = u32(raw, 0)
    if header < 40 or len(raw) < header:
        raise ValueError(f"bad DIB header size: {header}")
    w = struct.unpack_from("<i", raw, 4)[0]
    h = struct.unpack_from("<i", raw, 8)[0]
    planes = u16(raw, 12)
    bpp = u16(raw, 14)
    comp = u32(raw, 16)
    if planes != 1 or w == 0 or h == 0 or bpp not in (1, 4, 8, 16, 24, 32):
        raise ValueError(f"unsupported DIB: {w}x{h} planes={planes} bpp={bpp} comp={comp}")
    if comp != 0:
        raise ValueError(f"compressed DIB unsupported: comp={comp}")
    abs_h = abs(h)
    stride = ((abs(w) * bpp + 31) // 32) * 4
    need = header + abs_h * stride
    if need > len(raw):
        raise ValueError(f"truncated DIB pixels: need={need} have={len(raw)}")
    rows_file = [raw[header + y * stride:header + (y + 1) * stride] for y in range(abs_h)]
    rows = list(reversed(rows_file)) if h > 0 else rows_file
    return abs(w), abs_h, bpp, comp, rows


def iter_bitmap_candidates(pe: PE, require_32x32: bool = True) -> list[BitmapCandidate]:
    """Enumerate all valid RT_BITMAP DIB resources instead of assuming id=0x98."""
    out: list[BitmapCandidate] = []
    for path, rva, size in iter_resources(pe):
        if not path or path[0] != 2:
            continue
        raw = pe.read_rva(rva, size)
        try:
            w, h, bpp, comp, rows = parse_dib_rows_any(raw)
        except Exception:
            continue
        if require_32x32 and (w != 32 or h != 32 or bpp not in (24, 32)):
            continue
        out.append(BitmapCandidate(path, rva, size, w, h, bpp, comp, rows))
    return out


def sample_bitmap_bits_generic(rows: list[bytes], width: int, bpp: int, *, x: int, byte_index: int, reverse_y: bool) -> int:
    rs = list(reversed(rows)) if reverse_y else rows
    step = bpp // 8
    if step <= 0:
        raise ValueError("packed bitmap sampling is unsupported")
    if not (0 <= x < width):
        raise ValueError(f"x out of range: {x}")
    if not (0 <= byte_index < step):
        raise ValueError(f"byte_index out of range: {byte_index}")
    acc = 0
    n = min(32, len(rs))
    for y in range(n):
        pos = x * step + byte_index
        if pos >= len(rs[y]):
            raise ValueError("row too short")
        acc = ((acc << 1) | (rs[y][pos] & 1)) & 0xFFFFFFFF
    return acc


def iter_bitmap_bit_candidates(pe: PE):
    """Yield (bits, description) for all plausible Exhibit bitmap steganography candidates."""
    cands = iter_bitmap_candidates(pe, require_32x32=True)
    # Prefer the historically observed id=0x98 but still try every bitmap.
    def rank(c: BitmapCandidate):
        return (0 if len(c.path) >= 2 and c.path[1] == 0x98 else 1, str(c.path))
    for cand in sorted(cands, key=rank):
        step = cand.bpp // 8
        xs = []
        for x in (31, 0, cand.width - 1):
            if 0 <= x < cand.width and x not in xs:
                xs.append(x)
        # Then try every column as fallback; some titles do not use the right edge.
        for x in range(cand.width):
            if x not in xs:
                xs.append(x)
        for reverse_y in (True, False):
            for byte_index in range(step):
                for x in xs:
                    try:
                        bits = sample_bitmap_bits_generic(cand.rows, cand.width, cand.bpp, x=x, byte_index=byte_index, reverse_y=reverse_y)
                    except Exception:
                        continue
                    desc = f"res_path={cand.path} {cand.width}x{cand.height}x{cand.bpp} x={x} byte={byte_index} reverse_y={reverse_y}"
                    yield bits, desc


def ini_concat_variants(path: str | Path, encoding: str = DEFAULT_ENCODING):
    """Generate common Exhibit ini checksum variants across different title builds."""
    # The two confirmed variants differ mainly by whether TITLE is included.
    seen: set[str] = set()
    for include_title in (False, True):
        text, flags = ini_concat(path, include_title=include_title, encoding=encoding)
        label = "with_TITLE" if include_title else "no_TITLE"
        if text not in seen:
            seen.add(text)
            yield label, text, resident_checksum(text, encoding=encoding), flags


def validate_decrypted_dlr(data: bytes, encoding: str = DEFAULT_ENCODING) -> tuple[bool, str, int]:
    """Strict-ish validation used for auto seed search. Returns (ok, reason, score)."""
    if len(data) < 0x10 or data[:4] != DLR_MAGIC:
        return False, "bad magic", 0
    op_off = u32(data, 8)
    op_count = u32(data, 12)
    if op_off < 0x10 or op_off >= len(data) or op_count <= 0 or op_count > 200000:
        return False, f"bad header op_off=0x{op_off:X} op_count={op_count}", 0
    off = op_off
    score = 0
    for op_index in range(op_count):
        if off + 4 > len(data):
            return False, f"truncated op {op_index}", score
        raw = u32(data, off)
        off += 4
        high = (raw >> 28) & 0xF
        code = raw & 0xFFFF
        init_count = (raw >> 16) & 0xFF
        str_count = (raw >> 24) & 0x0F
        # The upper nibble is used by some builds/opcodes; do not reject it.
        if init_count > 64 or str_count > 12:
            return False, f"bad counts op={op_index} init={init_count} str={str_count}", score
        if off + init_count * 4 > len(data):
            return False, f"truncated init op={op_index}", score
        off += init_count * 4
        for si in range(str_count):
            z = data.find(b"\x00", off)
            if z < 0:
                return False, f"unterminated string op={op_index} str={si}", score
            if z - off > 4096:
                return False, f"string too long op={op_index} str={si}", score
            raw_s = data[off:z]
            try:
                raw_s.decode(encoding)
                score += 1
            except UnicodeDecodeError:
                # Resource strings are CP932; one bad string is suspicious but not always fatal.
                return False, f"decode failed op={op_index} str={si}", score
            off = z + 1
        if code in (0x0015, 0x001C, 0x0030, 0x00BF):
            score += 5
    return True, f"ok op_count={op_count} end=0x{off:X}", score + min(op_count, 1000)


def auto_find_normal_seed(exe_path: str | Path, ini_path: str | Path, sample_rld: str | Path,
                          encoding: str = DEFAULT_ENCODING, max_report: int = 8) -> tuple[int, dict[str, object]]:
    """Find normal RLD seed by testing bitmap/checksum candidates against a real .rld."""
    pe = PE(exe_path)
    enc_data = Path(sample_rld).read_bytes()
    best: list[tuple[int, int, str, str, int, int, str]] = []
    for bits, bits_desc in iter_bitmap_bit_candidates(pe):
        for chk_label, _concat, chk, _flags in ini_concat_variants(ini_path, encoding=encoding):
            seed = (bits ^ chk) & 0xFFFFFFFF
            try:
                dec = crypt_rld(enc_data, seed)
                ok, reason, score = validate_decrypted_dlr(dec, encoding=encoding)
            except Exception as exc:
                ok, reason, score = False, str(exc), 0
            if ok:
                best.append((score, seed, bits_desc, chk_label, bits, chk, reason))
    if not best:
        # Fallback to old fixed logic so callers get the original precise exception context when needed.
        raise ValueError("auto seed search failed: no bitmap/checksum candidate produced a valid DLR")
    best.sort(reverse=True, key=lambda x: x[0])
    score, seed, bits_desc, chk_label, bits, chk, reason = best[0]
    info: dict[str, object] = {
        "seed": seed,
        "bitmap_bits": bits,
        "ini_checksum": chk,
        "ini_variant": chk_label,
        "bitmap_candidate": bits_desc,
        "validation": reason,
        "score": score,
        "matches": len(best),
        "top_matches": [
            {
                "score": s,
                "seed": f"0x{sd:08X}",
                "bitmap_bits": f"0x{bt:08X}",
                "ini_checksum": f"0x{ck:08X}",
                "ini_variant": lab,
                "bitmap_candidate": desc,
                "validation": rsn,
            }
            for s, sd, desc, lab, bt, ck, rsn in best[:max_report]
        ],
    }
    return seed, info

@dataclass
class StringRef:
    index: int
    offset: int
    text: str


@dataclass
class DlrOp:
    index: int
    offset: int
    raw: int
    code: int
    init_values: list[int]
    strings: list[StringRef]


def parse_dlr(data: bytes, encoding: str = DEFAULT_ENCODING) -> list[DlrOp]:
    if data[:4] != DLR_MAGIC:
        raise ValueError("invalid DLR magic")
    op_off = u32(data, 8)
    op_count = u32(data, 12)
    off = op_off
    ops: list[DlrOp] = []
    for op_index in range(op_count):
        if off + 4 > len(data):
            raise ValueError(f"truncated opcode at index={op_index}")
        inst_off = off
        raw = u32(data, off)
        off += 4
        code = raw & 0xFFFF
        init_count = (raw >> 16) & 0xFF
        str_count = (raw >> 24) & 0x0F
        if off + init_count * 4 > len(data):
            raise ValueError(f"truncated init args at op={op_index}")
        inits = [u32(data, off + i * 4) for i in range(init_count)]
        off += init_count * 4
        strings: list[StringRef] = []
        for si in range(str_count):
            s_off = off
            z = data.find(b"\x00", off)
            if z < 0:
                raise ValueError(f"unterminated string at op={op_index} str={si}")
            text = data[off:z].decode(encoding, errors="replace")
            strings.append(StringRef(si, s_off, text))
            off = z + 1
        ops.append(DlrOp(op_index, inst_off, raw, code, inits, strings))
    return ops


def validate_dlr(data: bytes, encoding: str = DEFAULT_ENCODING, max_ops: int = 100) -> tuple[int, str]:
    try:
        ops = parse_dlr(data, encoding=encoding)
    except Exception as exc:
        return 0, str(exc)
    return min(len(ops), max_ops), "ok"


def build_name_table(data: bytes, encoding: str = DEFAULT_ENCODING) -> dict[int, str]:
    names: dict[int, str] = {}
    for op in parse_dlr(data, encoding=encoding):
        if op.code != 0x30 or not op.strings:
            continue
        parts = op.strings[0].text.split(",")
        if len(parts) >= 4:
            try:
                cid = int(parts[0].strip(), 0)
                name = parts[3].strip()
                if name and name != "*":
                    names[cid] = name
            except ValueError:
                pass
    return names


_PUA_RE = re.compile(r"[\uE000-\uF8FF]")
_PUA_TAG_RE = re.compile(r"<PUA_([0-9A-Fa-f]{4})>")


def escape_text_for_json(text: str) -> str:
    """Make engine private-use control chars visible and hard to delete by accident."""
    text = text.replace("\n", "[n]")
    return _PUA_RE.sub(lambda m: f"<PUA_{ord(m.group(0)):04X}>", text)


def unescape_text_from_json(text: str) -> str:
    text = text.replace("[n]", "\n")
    return _PUA_TAG_RE.sub(lambda m: chr(int(m.group(1), 16)), text)


def is_probably_resource_or_config(text: str) -> bool:
    s = text.strip()
    if not s or s == "*":
        return True
    low = s.lower()
    if "\\" in s or "/" in s:
        return True
    if any(low.endswith(ext) for ext in (".gyu", ".wav", ".ogg", ".rld", ".png", ".bmp", ".jpg", ".ini")):
        return True
    if "=" in s and not any(ch in s for ch in "。、，！？「」ぁあア亜唖漢字"):
        return True
    # 纯数字/逗号/星号配置串。
    allowed = set("0123456789abcdefABCDEFxX,-.*;: ")
    if len(s) >= 3 and all(ch in allowed for ch in s):
        return True
    return False


def has_textual_japanese_or_marks(text: str) -> bool:
    if not text or text == "*":
        return False
    if is_probably_resource_or_config(text):
        return False
    for ch in text:
        o = ord(ch)
        if (
            0x3040 <= o <= 0x30FF or 0x3400 <= o <= 0x9FFF or
            0xE000 <= o <= 0xF8FF or
            ch in "「」『』、。，．！？ー…―─・"
        ):
            return True
    return False


def make_entry(*, name: str | None, scr_msg: str, file: str, index: int, offset: int, inst_offset: int,
               opcode: int, op_index: int, str_index: int, typ: str, encoding: str = DEFAULT_ENCODING) -> OrderedDict:
    obj: OrderedDict[str, object] = OrderedDict()
    if name:
        obj["name"] = name
    obj["scr_msg"] = escape_text_for_json(scr_msg)
    obj["message"] = escape_text_for_json(scr_msg)
    obj["_file"] = file
    obj["_index"] = index
    obj["_offset"] = offset
    obj["_inst_offset"] = inst_offset
    obj["_opcode"] = f"0x{opcode:04X}"
    obj["_op_index"] = op_index
    obj["_str_index"] = str_index
    obj["_type"] = typ
    obj["_encoding"] = encoding
    obj["_policy"] = "relocate"
    return obj




def make_name_entry(*, scr_msg: str, file: str, index: int, offset: int, inst_offset: int,
                    opcode: int, op_index: int, str_index: int, name_id: int,
                    name_field: int = 3, encoding: str = DEFAULT_ENCODING) -> OrderedDict:
    """Create a JSON entry for an editable character-name slot in opcode 0x30.

    ExHIBIT character definitions store the display name as one CSV field inside
    a single C string, usually field index 3.  The JSON entry exposes only that
    field as scr_msg/message, while _offset/_op_index/_str_index still point to
    the whole original C string so the injector can rebuild the CSV safely.
    """
    obj: OrderedDict[str, object] = OrderedDict()
    obj["scr_msg"] = escape_text_for_json(scr_msg)
    obj["message"] = escape_text_for_json(scr_msg)
    obj["_file"] = file
    obj["_index"] = index
    obj["_offset"] = offset
    obj["_inst_offset"] = inst_offset
    obj["_opcode"] = f"0x{opcode:04X}"
    obj["_op_index"] = op_index
    obj["_str_index"] = str_index
    obj["_type"] = "name"
    obj["_name_id"] = name_id
    obj["_name_field"] = name_field
    obj["_encoding"] = encoding
    obj["_policy"] = "relocate"
    return obj


def load_json(path: str | Path):
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f, object_pairs_hook=OrderedDict)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be list: {path}")
    return data


def save_json(path: str | Path, entries: list[OrderedDict]) -> None:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def replace_cstrings(data: bytes, replacements: dict[int, str], encoding: str = DEFAULT_ENCODING, errors: str = "strict") -> bytes:
    if not replacements:
        return data
    new = bytearray()
    cur = 0
    for off, text in sorted(replacements.items()):
        if off < cur:
            raise ValueError(f"overlapping replacement at 0x{off:X}")
        z = data.find(b"\x00", off)
        if z < 0:
            raise ValueError(f"unterminated original cstring at 0x{off:X}")
        new.extend(data[cur:off])
        new.extend(unescape_text_from_json(text).encode(encoding, errors=errors))
        new.append(0)
        cur = z + 1
    new.extend(data[cur:])
    return bytes(new)
