#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Blue/Mizuka-style .p archive unpacker/repacker with per-file payload decode.

Validated with supplied Blue Se.p sample:
  magic       pipi
  version     7
  seed        41
  file_count  189

For v12/v13 archives the directory has a protected substitution step.  The
original engine stores that routine in an encrypted code descriptor in the EXE;
this tool emulates that routine with Unicorn so the implementation stays tied to
what the target EXE actually does.

This edition supports older v7 variable directory entries and also mirrors the runtime open/read split: for v6-v13 entries
the first 0x1000 bytes are decoded by the file-open path and the remaining
stream is decoded by the read path, so extracted PNG/SKR/TXT payloads become
usable instead of still looking encrypted.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import struct
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

MAGICS = {b"pipi", b"PCCS"}
DEFAULT_ENCODING = "cp932"

# Protected v12/v13 descriptor in the supplied engine.
V12_DESC_VA = 0x4E0628
V12_DESC_SIZE = 0x6FC
DATA_CONST_VA = 0x4E0000
DATA_CONST_SIZE = 0x20000

# Emulation layout. These are private, synthetic addresses.
EMU_BASE = 0x401000
EMU_STACK = 0x700000
EMU_DATA = 0x600000
EMU_CB = 0x500000
EMU_RET = 0xDEADB000
EMU_MEMSET = 0x510000
EMU_MEMCPY = 0x510010
EMU_MEMMOVE = 0x510020
EMU_SEED = 0x510030
EMU_RAND = 0x510040


def u16le(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from("<H", data, off)[0]


def s16le(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from("<h", data, off)[0]


def u32le(data: bytes | bytearray, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def p16(value: int) -> bytes:
    return struct.pack("<H", value & 0xFFFF)


def ps16(value: int) -> bytes:
    if not -0x8000 <= value <= 0x7FFF:
        raise ValueError(f"signed u16 field out of range: {value}")
    return struct.pack("<h", value)


def p32(value: int) -> bytes:
    return struct.pack("<I", value & 0xFFFFFFFF)


def align_up(value: int, align: int) -> int:
    return (value + align - 1) // align * align


class MizRand:
    """The engine's MT-like PRNG used by the .p table transforms."""

    def __init__(self, seed: int = 0) -> None:
        self.state = [0] * 624
        self.idx = 624
        self.seed(seed)

    def seed(self, seed: int) -> None:
        seed &= 0xFFFFFFFF
        for i in range(624):
            u = (seed * 0x10DCD + 1) & 0xFFFFFFFF
            self.state[i] = ((seed & 0xFFFF0000) | (u >> 16)) & 0xFFFFFFFF
            seed = (u * 0x10DCD + 1) & 0xFFFFFFFF
        self.idx = 624

    def _twist(self) -> None:
        mt = self.state
        n = 624
        m = 397
        matrix_a = 0x9908B0DF
        for kk in range(n - m):
            y = (mt[kk] & 0x80000000) | (mt[kk + 1] & 0x7FFFFFFF)
            mt[kk] = (mt[kk + m] ^ (y >> 1) ^ (matrix_a if (y & 1) else 0)) & 0xFFFFFFFF
        for kk in range(n - m, n - 1):
            y = (mt[kk] & 0x80000000) | (mt[kk + 1] & 0x7FFFFFFF)
            mt[kk] = (mt[kk + m - n] ^ (y >> 1) ^ (matrix_a if (y & 1) else 0)) & 0xFFFFFFFF
        y = (mt[n - 1] & 0x80000000) | (mt[0] & 0x7FFFFFFF)
        mt[n - 1] = (mt[m - 1] ^ (y >> 1) ^ (matrix_a if (y & 1) else 0)) & 0xFFFFFFFF
        self.idx = 0

    def rand(self) -> int:
        if self.idx >= 624:
            self._twist()
        y = self.state[self.idx]
        self.idx += 1
        y ^= y >> 11
        y ^= (y << 7) & 0x9D2C5680
        y &= 0xFFFFFFFF
        y ^= (y << 15) & 0xEFC60000
        y &= 0xFFFFFFFF
        y ^= y >> 18
        return y & 0xFFFFFFFF


def block_decode(buf: bytes | bytearray, seed: int, block_count: int, flag: int) -> bytes:
    """FUN_0048a0e0 direction used when opening an archive."""
    size = len(buf)
    if size < block_count:
        if not flag or block_count == 1:
            return bytes(buf)
        while size < block_count:
            block_count >>= 1
            if block_count == 1:
                return bytes(buf)
    unit = size // block_count
    total = unit * block_count
    src = bytes(buf[:total])
    out = bytearray(src)
    used = [False] * block_count
    rng = MizRand(seed)
    for out_idx in range(block_count):
        j = rng.rand() & (block_count - 1)
        while used[j]:
            j = (j + 1) & (block_count - 1)
        used[j] = True
        out[out_idx * unit:(out_idx + 1) * unit] = src[j * unit:(j + 1) * unit]
    return bytes(out) + bytes(buf[total:])


def block_encode(buf: bytes | bytearray, seed: int, block_count: int, flag: int) -> bytes:
    """Inverse of block_decode, used when packing."""
    size = len(buf)
    if size < block_count:
        if not flag or block_count == 1:
            return bytes(buf)
        while size < block_count:
            block_count >>= 1
            if block_count == 1:
                return bytes(buf)
    unit = size // block_count
    total = unit * block_count
    src = bytes(buf[:total])
    out = bytearray(src)
    used = [False] * block_count
    rng = MizRand(seed)
    for out_idx in range(block_count):
        j = rng.rand() & (block_count - 1)
        while used[j]:
            j = (j + 1) & (block_count - 1)
        used[j] = True
        out[j * unit:(j + 1) * unit] = src[out_idx * unit:(out_idx + 1) * unit]
    return bytes(out) + bytes(buf[total:])


def nswap_byte(b: int) -> int:
    return ((b >> 4) | ((b & 0x0F) << 4)) & 0xFF


def decode_mstr(data: bytes | bytearray, pos: int) -> tuple[bytes, int, int]:
    if pos + 5 > len(data):
        raise ValueError(f"truncated Mizuka string at 0x{pos:x}")
    length = u32le(data, pos)
    flag = data[pos + 4]
    start = pos + 5
    end = start + length
    if end > len(data):
        raise ValueError(f"Mizuka string overruns table at 0x{pos:x}, len={length}")
    raw = bytearray(data[start:end])
    # Runtime nibbleswaps the real bytes but leaves the terminal NUL alone.
    for i in range(max(0, length - 1)):
        raw[i] = nswap_byte(raw[i])
    if b"\x00" in raw:
        raw = raw.split(b"\x00", 1)[0]
    return bytes(raw), flag, end


def encode_mstr(raw: bytes, flag: int = 0) -> bytes:
    body = bytearray(raw + b"\x00")
    for i in range(len(raw)):
        body[i] = nswap_byte(body[i])
    return p32(len(body)) + bytes([flag & 0xFF]) + bytes(body)


def path_bytes_to_text(raw: bytes) -> str:
    for enc in (DEFAULT_ENCODING, "utf-8", "latin1"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            pass
    return raw.hex()


def path_text_to_bytes(text: str, hex_fallback: str | None = None) -> bytes:
    try:
        return text.encode(DEFAULT_ENCODING)
    except UnicodeEncodeError:
        if hex_fallback:
            return bytes.fromhex(hex_fallback)
        raise


def safe_out_path(name: str, index: int, used: set[str]) -> str:
    # Archive names in this sample are simple filenames, but keep this safe for
    # accidental absolute paths or path traversal.
    cleaned = name.replace("\\", "/").lstrip("/")
    parts = [p for p in cleaned.split("/") if p not in ("", ".", "..")]
    if not parts:
        parts = [f"entry_{index:04d}.bin"]
    cleaned = "/".join(parts)
    if cleaned in used:
        stem, ext = os.path.splitext(cleaned)
        cleaned = f"{stem}__{index:04d}{ext}"
    used.add(cleaned)
    return cleaned


def sum_key_low(key_raw: bytes) -> int:
    return sum(key_raw) & 0xFF


def data_key_from_key_raw(key_raw: bytes, version: int) -> int:
    c = sum_key_low(key_raw)
    if 6 <= version <= 13:
        if version < 8:
            c |= 0xFFFFFE00 + 0x400
        else:
            c |= 0x400
    if version >= 10:
        c |= 0x800
    return c & 0xFFFFFFFF


def decode_data_payload(cipher: bytes, key: int) -> bytes:
    if key == 0xFFFFFFFF:
        return cipher
    if key & 0x400:
        if key & 0x800:
            k = key & 0xFF
            return bytes((~((b - k) & 0xFF)) & 0xFF for b in cipher)
        return bytes((~b) & 0xFF for b in cipher)
    return cipher


def encode_data_payload(plain: bytes, key: int) -> bytes:
    if key == 0xFFFFFFFF:
        return plain
    if key & 0x400:
        if key & 0x800:
            k = key & 0xFF
            return bytes((((~b) & 0xFF) + k) & 0xFF for b in plain)
        return bytes((~b) & 0xFF for b in plain)
    return plain


def payload_prefix_len(size: int, key: int) -> int:
    """Bytes decoded by the runtime file-open path before streamed reads."""
    if key == 0xFFFFFFFF or size <= 0:
        return 0
    if key & 0x100:
        return size
    cap = 0x1000 if (key & 0x400) else 0x200
    return min(size, cap)


def sha1_seed_from_key_raw(key_raw: bytes) -> int:
    # FUN_0048a330 returns a SHA1 hex string; the runtime parses the first
    # eight hex digits into a 32-bit integer.
    return int(hashlib.sha1(key_raw).hexdigest()[:8], 16)


def _payload_prefix_block_count(key: int) -> int:
    return 0x100 if (key & 0x400) else 0x20


def _payload_prefix_block_flag(key: int) -> int:
    return ((key >> 10) & 0xFFFFFF01) & 0xFFFFFFFF


def decode_open_prefix(cipher_prefix: bytes, key_raw: bytes, key: int) -> bytes:
    """Decode the preloaded prefix exactly like FUN_0048a700."""
    if not cipher_prefix:
        return b""
    seed = sha1_seed_from_key_raw(key_raw)
    tmp = bytearray(block_decode(
        cipher_prefix,
        seed,
        _payload_prefix_block_count(key),
        _payload_prefix_block_flag(key),
    ))
    c = key & 0xFF
    if key & 0x800:
        rng = MizRand(((c * 5) + (seed & 0xFF)) & 0xFFFFFFFF)
        group_key = 0
        for i, b in enumerate(tmp):
            val = (b - c) & 0xFF
            if (i & 0x0F) == 0:
                group_key = rng.rand() & 0xFF
            val = (val - group_key) & 0xFF
            tmp[i] = (0xFF - val) & 0xFF
    else:
        for i, b in enumerate(tmp):
            tmp[i] = (c - b - 1) & 0xFF
    return bytes(tmp)


def encode_open_prefix(plain_prefix: bytes, key_raw: bytes, key: int) -> bytes:
    """Inverse of decode_open_prefix, used for byte-exact repacking."""
    if not plain_prefix:
        return b""
    seed = sha1_seed_from_key_raw(key_raw)
    c = key & 0xFF
    tmp = bytearray(plain_prefix)
    if key & 0x800:
        rng = MizRand(((c * 5) + (seed & 0xFF)) & 0xFFFFFFFF)
        group_key = 0
        for i, b in enumerate(tmp):
            if (i & 0x0F) == 0:
                group_key = rng.rand() & 0xFF
            tmp[i] = ((0xFF - b) + group_key + c) & 0xFF
    else:
        for i, b in enumerate(tmp):
            tmp[i] = (c - b - 1) & 0xFF
    return block_encode(
        tmp,
        seed,
        _payload_prefix_block_count(key),
        _payload_prefix_block_flag(key),
    )


def decode_file_payload(cipher: bytes, key_raw: bytes, version: int) -> bytes:
    """Decode a payload as the engine sees it after open + streamed reads."""
    key = data_key_from_key_raw(key_raw, version)
    n = payload_prefix_len(len(cipher), key)
    if n <= 0:
        return decode_data_payload(cipher, key)
    return decode_open_prefix(cipher[:n], key_raw, key) + decode_data_payload(cipher[n:], key)


def encode_file_payload(plain: bytes, key_raw: bytes, version: int) -> bytes:
    """Encode a payload back to archive bytes."""
    key = data_key_from_key_raw(key_raw, version)
    n = payload_prefix_len(len(plain), key)
    if n <= 0:
        return encode_data_payload(plain, key)
    return encode_open_prefix(plain[:n], key_raw, key) + encode_data_payload(plain[n:], key)


def _meta_masks(key_low: int) -> tuple[list[int], int]:
    c = key_low & 0xFF
    masks = [((0x11 << i) + c) & 0x33 for i in range(4)]
    c2 = (-1 - c) & 0xFF
    return masks, c2


def decode_obfuscated_u32(value: int, key_low: int) -> int:
    masks, c2 = _meta_masks(key_low)
    out = bytearray(4)
    for i in range(4):
        b = (value >> (8 * i)) & 0xFF
        out[i] = (masks[i] - b + c2) & 0xFF
    return int.from_bytes(out, "little")


def encode_obfuscated_u32(value: int, key_low: int) -> int:
    masks, c2 = _meta_masks(key_low)
    out = bytearray(4)
    raw = value.to_bytes(4, "little", signed=False)
    for i, b in enumerate(raw):
        out[i] = (masks[i] + c2 - b) & 0xFF
    return int.from_bytes(out, "little")


class PEImage:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.data = path.read_bytes()
        if self.data[:2] != b"MZ":
            raise ValueError(f"not a PE file: {path}")
        pe = u32le(self.data, 0x3C)
        if self.data[pe:pe + 4] != b"PE\0\0":
            raise ValueError(f"bad PE signature: {path}")
        self.image_base = u32le(self.data, pe + 0x34)
        section_count = u16le(self.data, pe + 6)
        opt_size = u16le(self.data, pe + 20)
        sec_base = pe + 24 + opt_size
        self.sections: list[dict[str, Any]] = []
        for i in range(section_count):
            off = sec_base + 40 * i
            name = self.data[off:off + 8].split(b"\0", 1)[0].decode("ascii", errors="replace")
            vsize, rva, raw_size, raw_ptr = struct.unpack_from("<IIII", self.data, off + 8)
            self.sections.append({
                "name": name,
                "vsize": vsize,
                "rva": rva,
                "raw_size": raw_size,
                "raw_ptr": raw_ptr,
            })

    def va_bytes(self, va: int, size: int) -> bytes:
        rva = va - self.image_base
        for sec in self.sections:
            start = sec["rva"]
            end = start + max(sec["vsize"], sec["raw_size"])
            if start <= rva < end:
                in_sec = rva - start
                raw_ptr = sec["raw_ptr"] + in_sec
                available = max(0, sec["raw_size"] - in_sec)
                chunk = self.data[raw_ptr:raw_ptr + min(size, available)]
                if len(chunk) < size:
                    chunk += b"\x00" * (size - len(chunk))
                return chunk
        raise KeyError(f"VA not covered by PE sections: 0x{va:x}")


class V12Transformer:
    def __init__(self, exe_path: Path) -> None:
        try:
            from unicorn import Uc, UcError, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE, UC_PROT_ALL, UC_ERR_OK
            from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EIP, UC_X86_REG_ESP
        except Exception as exc:  # pragma: no cover - depends on external package
            raise RuntimeError(
                "v12/v13 .p directory transform requires Unicorn. Install with: pip install unicorn"
            ) from exc
        self.Uc = Uc
        self.UcError = UcError
        self.UC_ARCH_X86 = UC_ARCH_X86
        self.UC_MODE_32 = UC_MODE_32
        self.UC_HOOK_CODE = UC_HOOK_CODE
        self.UC_PROT_ALL = UC_PROT_ALL
        self.UC_ERR_OK = UC_ERR_OK
        self.UC_X86_REG_EAX = UC_X86_REG_EAX
        self.UC_X86_REG_EIP = UC_X86_REG_EIP
        self.UC_X86_REG_ESP = UC_X86_REG_ESP

        pe = PEImage(exe_path)
        desc = pe.va_bytes(V12_DESC_VA, V12_DESC_SIZE)
        desc_seed, entry_off, entry_va = struct.unpack_from("<III", desc, 0)
        encrypted_blob = desc[12:]
        blob = block_decode(encrypted_blob, desc_seed, 0x100, 1)
        self.blob = blob
        self.func_va = entry_va
        self.map_base = entry_va - entry_off
        self.const_data = pe.va_bytes(DATA_CONST_VA, DATA_CONST_SIZE)
        self._subst_cache: dict[int, bytes] = {}

    def _hook_code(self, mu: Any, address: int, size: int, user_data: dict[str, Any]) -> None:
        rng: MizRand = user_data["rng"]
        if address in (EMU_MEMSET, EMU_MEMCPY, EMU_MEMMOVE, EMU_SEED, EMU_RAND):
            esp = mu.reg_read(self.UC_X86_REG_ESP)
            ret = u32le(mu.mem_read(esp, 4), 0)

            def arg(n: int) -> int:
                return u32le(mu.mem_read(esp + 4 + 4 * n, 4), 0)

            if address == EMU_MEMSET:
                dest = arg(0)
                val = arg(1) & 0xFF
                count = arg(2)
                mu.mem_write(dest, bytes([val]) * count)
                mu.reg_write(self.UC_X86_REG_EAX, dest)
            elif address in (EMU_MEMCPY, EMU_MEMMOVE):
                dest = arg(0)
                src = arg(1)
                count = arg(2)
                mu.mem_write(dest, bytes(mu.mem_read(src, count)))
                mu.reg_write(self.UC_X86_REG_EAX, dest)
            elif address == EMU_SEED:
                rng.seed(arg(1))
                mu.reg_write(self.UC_X86_REG_EAX, 0)
            elif address == EMU_RAND:
                mu.reg_write(self.UC_X86_REG_EAX, rng.rand())
            mu.reg_write(self.UC_X86_REG_ESP, esp + 4)
            mu.reg_write(self.UC_X86_REG_EIP, ret)
        elif address == EMU_RET:
            raise self.UcError(self.UC_ERR_OK)

    def decode(self, buf: bytes, seed: int) -> bytes:
        if not buf:
            return b""
        mu = self.Uc(self.UC_ARCH_X86, self.UC_MODE_32)
        blob_map_size = align_up(max(len(self.blob), 0x3000), 0x1000)
        mu.mem_map(self.map_base, blob_map_size, self.UC_PROT_ALL)
        mu.mem_write(self.map_base, self.blob + b"\x90" * (blob_map_size - len(self.blob)))
        mu.mem_map(DATA_CONST_VA, align_up(DATA_CONST_SIZE, 0x1000), self.UC_PROT_ALL)
        mu.mem_write(DATA_CONST_VA, self.const_data)
        data_map_size = align_up(len(buf) + 0x1000, 0x1000)
        mu.mem_map(EMU_DATA, data_map_size, self.UC_PROT_ALL)
        mu.mem_write(EMU_DATA, buf)
        mu.mem_map(EMU_STACK, 0x20000, self.UC_PROT_ALL)
        mu.mem_map(EMU_CB, 0x20000, self.UC_PROT_ALL)
        mu.mem_map(EMU_RET, 0x1000, self.UC_PROT_ALL)
        for addr in (EMU_MEMSET, EMU_MEMCPY, EMU_MEMMOVE, EMU_SEED, EMU_RAND, EMU_RET):
            mu.mem_write(addr, b"\xC3")
        # Callback table expected by the protected routine.
        callbacks = [
            (0x00, EMU_MEMSET),
            (0x04, EMU_MEMCPY),
            (0x08, EMU_MEMCPY),
            (0x10, 0x4E12B0),
            (0x14, 0x4E13F0),
            (0x18, 0x4E1530),
            (0x1C, 0x4E1670),
            (0x20, EMU_SEED),
            (0x24, EMU_RAND),
            (0x2C, EMU_CB + 0x100),
            (0x30, 0),
        ]
        for off, val in callbacks:
            mu.mem_write(EMU_CB + off, p32(val))
        esp = EMU_STACK + 0x10000
        # Entry stack: ret, callback_table, buffer, size, seed.
        for value in (seed, len(buf), EMU_DATA, EMU_CB, EMU_RET):
            esp -= 4
            mu.mem_write(esp, p32(value))
        mu.reg_write(self.UC_X86_REG_ESP, esp)
        mu.hook_add(self.UC_HOOK_CODE, self._hook_code, {"rng": MizRand(0)})
        try:
            mu.emu_start(self.func_va, EMU_RET, count=50_000_000)
        except self.UcError as exc:
            if exc.errno != self.UC_ERR_OK:
                eip = mu.reg_read(self.UC_X86_REG_EIP)
                raise RuntimeError(f"v12 transform emulation failed: {exc}; eip=0x{eip:x}") from exc
        return bytes(mu.mem_read(EMU_DATA, len(buf)))

    def substitution(self, seed: int) -> bytes:
        if seed not in self._subst_cache:
            self._subst_cache[seed] = self.decode(bytes(range(256)), seed)
        return self._subst_cache[seed]

    def encode(self, buf: bytes, seed: int) -> bytes:
        subst = self.substitution(seed)
        inv = bytearray(256)
        for src, dst in enumerate(subst):
            inv[dst] = src
        return bytes(inv[b] for b in buf)


@dataclass
class Entry:
    index: int
    lookup_raw: bytes
    key_raw: bytes
    lookup_flag: int
    key_flag: int
    offset: int
    size: int
    raw_a: int
    raw_b: int
    path: str = ""

    @property
    def lookup(self) -> str:
        return path_bytes_to_text(self.lookup_raw)

    @property
    def key(self) -> str:
        return path_bytes_to_text(self.key_raw)


@dataclass
class Archive:
    path: Path
    magic: bytes
    version: int
    seed: int
    file_count: int
    header_size: int
    entries: list[Entry]
    table_body: bytes


def decode_entry_body_in_place(table: bytearray, pos: int, index: int, seed: int) -> tuple[int, int]:
    nplus = s16le(table, pos)
    pos += 2
    nlen = nplus - 2
    if nlen < 0 or pos + nlen > len(table):
        raise ValueError(f"bad encrypted entry length idx={index}, nplus={nplus}, pos=0x{pos-2:x}")
    rb = MizRand(((seed + index) * 3) & 0xFFFFFFFF).rand() & 0xFF
    for j in range(nlen):
        table[pos + j] = (table[pos + j] - rb) & 0xFF
    return pos, nlen


def encode_entry_body(body: bytes, index: int, seed: int) -> bytes:
    rb = MizRand(((seed + index) * 3) & 0xFFFFFFFF).rand() & 0xFF
    return ps16(len(body) + 2) + bytes((b + rb) & 0xFF for b in body)


def parse_archive(path: Path, exe_path: Path | None = None) -> Archive:
    data = path.read_bytes()
    if len(data) < 16:
        raise ValueError("archive too small")
    magic = data[:4]
    if magic not in MAGICS:
        raise ValueError(f"unsupported magic: {magic!r}")
    version = u16le(data, 4)
    seed = u16le(data, 6)
    file_count = u32le(data, 8)
    header_size = u32le(data, 12)
    if version < 1 or version > 13:
        raise ValueError(f"unsupported .p version: {version}")
    if header_size < 16 or header_size > len(data):
        raise ValueError(f"bad header_size: {header_size}")
    table = data[16:header_size]
    if version >= 12:
        if exe_path is None:
            raise ValueError("version 12/13 archives require --exe for directory transform")
        table = V12Transformer(exe_path).decode(table, seed)
    if 6 <= version <= 13:
        table = block_decode(table, seed, 0x100 if version >= 8 else 0x20, 1 if version >= 8 else 0)

    buf = bytearray(table)
    pos = 0
    entries: list[Entry] = []
    used_paths: set[str] = set()
    for idx in range(file_count):
        if version >= 10:
            body_start, body_len = decode_entry_body_in_place(buf, pos, idx, seed)
            pos = body_start
            lookup_raw, lookup_flag, pos = decode_mstr(buf, pos)
            key_raw, key_flag, pos = decode_mstr(buf, pos)
            if pos + 16 > len(buf):
                raise ValueError(f"truncated numeric fields at entry {idx}")
            off_field, size_field, raw_a, raw_b = struct.unpack_from("<IIII", buf, pos)
            pos += 16
            expected_end = body_start + body_len
            if pos != expected_end:
                raise ValueError(
                    f"entry {idx} length mismatch: consumed={pos-body_start}, declared={body_len}"
                )
        else:
            # Older v6/v7 archives use the same Mizuka string and obfuscated
            # offset/size fields, but the entry body is not prefixed with the
            # signed length and not per-entry encrypted.
            lookup_raw, lookup_flag, pos = decode_mstr(buf, pos)
            key_raw, key_flag, pos = decode_mstr(buf, pos)
            if pos + 16 > len(buf):
                raise ValueError(f"truncated numeric fields at entry {idx}")
            off_field, size_field, raw_a, raw_b = struct.unpack_from("<IIII", buf, pos)
            pos += 16

        key_low = sum_key_low(key_raw)
        if version > 3:
            stored_rel = decode_obfuscated_u32(off_field, key_low)
            size = decode_obfuscated_u32(size_field, key_low)
            offset = stored_rel + (0 if version == 4 else header_size)
        else:
            offset = off_field
            size = size_field
        name = path_bytes_to_text(lookup_raw)
        out_path = safe_out_path(name, idx, used_paths)
        entries.append(Entry(
            index=idx,
            lookup_raw=lookup_raw,
            key_raw=key_raw,
            lookup_flag=lookup_flag,
            key_flag=key_flag,
            offset=offset,
            size=size,
            raw_a=raw_a,
            raw_b=raw_b,
            path=out_path,
        ))
    if pos != len(buf):
        raise ValueError(f"table parse did not consume all bytes: pos={pos}, len={len(buf)}")
    return Archive(path, magic, version, seed, file_count, header_size, entries, bytes(buf))


def manifest_from_archive(arc: Archive) -> dict[str, Any]:
    return {
        "format": "meguri-p",
        "source": arc.path.name,
        "magic": arc.magic.decode("ascii", errors="replace"),
        "version": arc.version,
        "seed": arc.seed,
        "header_size": arc.header_size,
        "file_count": arc.file_count,
        "encoding": DEFAULT_ENCODING,
        "entries": [
            {
                "index": e.index,
                "path": e.path,
                "lookup": e.lookup,
                "lookup_hex": e.lookup_raw.hex(),
                "lookup_flag": e.lookup_flag,
                "key": e.key,
                "key_hex": e.key_raw.hex(),
                "key_flag": e.key_flag,
                "offset": e.offset,
                "size": e.size,
                "raw_a": e.raw_a,
                "raw_b": e.raw_b,
            }
            for e in arc.entries
        ],
    }


def unpack_archive(archive_path: Path, out_dir: Path, exe_path: Path | None = None) -> dict[str, Any]:
    arc = parse_archive(archive_path, exe_path)
    data = archive_path.read_bytes()
    out_dir.mkdir(parents=True, exist_ok=True)
    manifest = manifest_from_archive(arc)
    for e in arc.entries:
        if e.offset < arc.header_size or e.offset + e.size > len(data):
            raise ValueError(f"entry {e.index} has invalid range offset={e.offset}, size={e.size}")
        plain = decode_file_payload(data[e.offset:e.offset + e.size], e.key_raw, arc.version)
        target = out_dir / e.path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(plain)
    (out_dir / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8", newline="\n")
    return manifest


def build_table_and_payload(manifest: dict[str, Any], files_dir: Path) -> tuple[bytes, bytes, int]:
    version = int(manifest["version"])
    seed = int(manifest["seed"])
    entries = manifest.get("entries")
    if not isinstance(entries, list):
        raise ValueError("manifest entries must be a list")
    # First pass: work out table length/header_size. Filename edits are allowed as
    # long as they can be encoded to cp932 or have *_hex fallback.
    body_lengths = []
    for ent in entries:
        lookup_raw = path_text_to_bytes(str(ent.get("lookup", "")), ent.get("lookup_hex"))
        key_raw = path_text_to_bytes(str(ent.get("key", "")), ent.get("key_hex"))
        lookup_flag = int(ent.get("lookup_flag", 0))
        key_flag = int(ent.get("key_flag", 0))
        body_len = len(encode_mstr(lookup_raw, lookup_flag)) + len(encode_mstr(key_raw, key_flag)) + 16
        body_lengths.append(body_len)
    table_len = sum((2 + n) if version >= 10 else n for n in body_lengths)
    header_size = 16 + table_len

    payload = bytearray()
    table = bytearray()
    current_offset = header_size
    for i, ent in enumerate(entries):
        index = int(ent.get("index", i))
        if index != i:
            # The runtime uses physical order as the per-entry key index. Keep
            # manifest order authoritative, but warn by hard-failing on mismatch.
            raise ValueError(f"entry order/index mismatch at list position {i}: index={index}")
        rel_path = str(ent.get("path") or ent.get("lookup") or f"entry_{i:04d}.bin")
        src = files_dir / rel_path
        if not src.is_file():
            raise FileNotFoundError(f"missing payload file for entry {i}: {src}")
        plain = src.read_bytes()
        lookup_raw = path_text_to_bytes(str(ent.get("lookup", "")), ent.get("lookup_hex"))
        key_raw = path_text_to_bytes(str(ent.get("key", "")), ent.get("key_hex"))
        lookup_flag = int(ent.get("lookup_flag", 0))
        key_flag = int(ent.get("key_flag", 0))
        key_low = sum_key_low(key_raw)
        rel_off = current_offset if version == 4 else current_offset - header_size
        if version > 3:
            off_field = encode_obfuscated_u32(rel_off, key_low)
            size_field = encode_obfuscated_u32(len(plain), key_low)
        else:
            off_field = rel_off
            size_field = len(plain)
        raw_a = int(ent.get("raw_a", 0))
        raw_b = int(ent.get("raw_b", 0))
        body = (
            encode_mstr(lookup_raw, lookup_flag)
            + encode_mstr(key_raw, key_flag)
            + p32(off_field)
            + p32(size_field)
            + p32(raw_a)
            + p32(raw_b)
        )
        if version >= 10:
            table += encode_entry_body(body, i, seed)
        else:
            table += body
        payload += encode_file_payload(plain, key_raw, version)
        current_offset += len(plain)
    return bytes(table), bytes(payload), header_size


def pack_archive(manifest_path: Path, files_dir: Path, out_path: Path, exe_path: Path | None = None) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("format") != "meguri-p":
        raise ValueError("manifest format is not meguri-p")
    magic_text = str(manifest.get("magic", "pipi"))
    magic = magic_text.encode("ascii")
    if magic not in MAGICS:
        raise ValueError(f"unsupported magic in manifest: {magic_text!r}")
    version = int(manifest["version"])
    seed = int(manifest["seed"])
    file_count = len(manifest["entries"])
    table, payload, header_size = build_table_and_payload(manifest, files_dir)
    encoded_table = table
    if 6 <= version <= 13:
        encoded_table = block_encode(encoded_table, seed, 0x100 if version >= 8 else 0x20, 1 if version >= 8 else 0)
    if version >= 12:
        if exe_path is None:
            raise ValueError("version 12/13 archives require --exe for directory transform")
        encoded_table = V12Transformer(exe_path).encode(encoded_table, seed)
    header = magic + p16(version) + p16(seed) + p32(file_count) + p32(header_size)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(header + encoded_table + payload)


def info_archive(archive_path: Path, exe_path: Path | None = None) -> None:
    arc = parse_archive(archive_path, exe_path)
    end = max((e.offset + e.size for e in arc.entries), default=arc.header_size)
    print(f"magic       = {arc.magic.decode('ascii', errors='replace')}")
    print(f"version     = {arc.version}")
    print(f"seed        = {arc.seed}")
    print(f"file_count  = {arc.file_count}")
    print(f"header_size = {arc.header_size}")
    print(f"data_end    = {end}")
    print(f"archive_size= {arc.path.stat().st_size}")
    for e in arc.entries[:10]:
        print(f"[{e.index:04d}] {e.path}  off={e.offset} size={e.size} key={e.key}")
    if len(arc.entries) > 10:
        print(f"... {len(arc.entries) - 10} more entries")


def verify_archive(archive_path: Path, exe_path: Path | None = None) -> bool:
    arc = parse_archive(archive_path, exe_path)
    size = archive_path.stat().st_size
    ok = True
    expected = arc.header_size
    seen: set[tuple[int, int]] = set()
    for e in arc.entries:
        if e.offset < arc.header_size or e.offset + e.size > size:
            print(f"[bad] range entry={e.index} path={e.path} off={e.offset} size={e.size}")
            ok = False
        if e.offset != expected:
            print(f"[warn] non-contiguous entry={e.index} path={e.path} off={e.offset} expected={expected}")
        expected = e.offset + e.size
        rng = (e.offset, e.offset + e.size)
        if rng in seen:
            print(f"[warn] duplicate range entry={e.index} path={e.path}")
        seen.add(rng)
    if expected != size:
        print(f"[warn] final data end={expected}, archive size={size}")
    print("[verify] OK" if ok else "[verify] FAILED")
    return ok


def roundtrip_archive(archive_path: Path, work_dir: Path, out_path: Path, exe_path: Path | None = None) -> None:
    if work_dir.exists():
        shutil.rmtree(work_dir)
    work_dir.mkdir(parents=True)
    unpack_archive(archive_path, work_dir, exe_path)
    pack_archive(work_dir / "manifest.json", work_dir, out_path, exe_path)
    h1 = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    h2 = hashlib.sha256(out_path.read_bytes()).hexdigest()
    same = archive_path.read_bytes() == out_path.read_bytes()
    print(f"original_sha256 = {h1}")
    print(f"rebuilt_sha256  = {h2}")
    print("roundtrip       = byte-exact" if same else "roundtrip       = differs")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Unpack/repack Blue/Mizuka .p archives")
    sub = ap.add_subparsers(dest="cmd", required=True)

    def add_exe(p: argparse.ArgumentParser) -> None:
        p.add_argument("--exe", type=Path, help="target game EXE; required for v12/v13 archives")

    p_info = sub.add_parser("info", help="parse and print archive summary")
    p_info.add_argument("archive", type=Path)
    add_exe(p_info)

    p_unpack = sub.add_parser("unpack", help="extract all files and write manifest.json")
    p_unpack.add_argument("archive", type=Path)
    p_unpack.add_argument("out_dir", type=Path)
    add_exe(p_unpack)

    p_pack = sub.add_parser("pack", help="rebuild archive from manifest.json and extracted files")
    p_pack.add_argument("manifest", type=Path)
    p_pack.add_argument("files_dir", type=Path)
    p_pack.add_argument("out", type=Path)
    add_exe(p_pack)

    p_verify = sub.add_parser("verify", help="parse table and verify ranges")
    p_verify.add_argument("archive", type=Path)
    add_exe(p_verify)

    p_rt = sub.add_parser("roundtrip", help="unpack then pack and compare bytes")
    p_rt.add_argument("archive", type=Path)
    p_rt.add_argument("work_dir", type=Path)
    p_rt.add_argument("out", type=Path)
    add_exe(p_rt)

    args = ap.parse_args(argv)
    try:
        if args.cmd == "info":
            info_archive(args.archive, args.exe)
        elif args.cmd == "unpack":
            manifest = unpack_archive(args.archive, args.out_dir, args.exe)
            print(f"[unpack] files={manifest['file_count']} output={args.out_dir}")
        elif args.cmd == "pack":
            pack_archive(args.manifest, args.files_dir, args.out, args.exe)
            print(f"[pack] output={args.out} size={args.out.stat().st_size}")
        elif args.cmd == "verify":
            return 0 if verify_archive(args.archive, args.exe) else 1
        elif args.cmd == "roundtrip":
            roundtrip_archive(args.archive, args.work_dir, args.out, args.exe)
        else:  # pragma: no cover
            ap.error("unknown command")
    except Exception as exc:
        print(f"[error] {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
