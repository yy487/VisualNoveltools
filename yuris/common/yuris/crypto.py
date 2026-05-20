# -*- coding: utf-8 -*-
"""YU-RIS ybn key 与 xor 处理。"""
from __future__ import annotations

import binascii
from dataclasses import dataclass


@dataclass(frozen=True)
class Segment:
    offset: int
    size: int


def key_from_text(text: str) -> bytes:
    """按 YuRis_Tool 的方式计算 ybnKey：CRC32(ASCII(text)) 后反转字节序。"""
    crc = binascii.crc32(text.encode("ascii")) & 0xFFFFFFFF
    return crc.to_bytes(4, "little")[::-1]


def parse_key(value: str | int | bytes | None) -> bytes | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        if len(value) != 4:
            raise ValueError("key bytes 必须是 4 字节")
        return value
    if isinstance(value, int):
        if not 0 <= value <= 0xFFFFFFFF:
            raise ValueError("key int 超出 32bit 范围")
        return value.to_bytes(4, "little")
    s = str(value).strip()
    if not s:
        return None
    if s.lower().startswith("0x"):
        return int(s, 16).to_bytes(4, "little")
    compact = s.replace(" ", "").replace("-", "")
    if len(compact) == 8 and all(c in "0123456789abcdefABCDEF" for c in compact):
        return bytes.fromhex(compact)
    return int(s, 0).to_bytes(4, "little")


def xor_region(buf: bytearray, offset: int, size: int, key: bytes) -> None:
    if not key:
        return
    if len(key) != 4:
        raise ValueError("YU-RIS xor key 必须是 4 字节")
    for i in range(size):
        buf[offset + i] ^= key[i & 3]


def xor_segments(buf: bytearray, segments: list[Segment], key: bytes | None) -> None:
    if not key:
        return
    for seg in segments:
        xor_region(buf, seg.offset, seg.size, key)


def xor_flat_after_header(buf: bytearray, key: bytes | None, header_size: int = 0x20) -> None:
    if not key:
        return
    xor_region(buf, header_size, len(buf) - header_size, key)
