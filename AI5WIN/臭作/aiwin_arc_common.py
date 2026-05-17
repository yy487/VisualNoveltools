#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AIWIN/旧 ELF 系 ARC 公用模块

已确认 ARC 外层格式：
  uint32le file_count
  repeated file_count times:
      char name[16]      # NUL 结尾，不足补 0
      uint32le offset    # 文件数据在 ARC 内的偏移
      uint32le size      # 文件数据大小；是否压缩取决于资源类型

当前样本结论：
  - message.arc 里的 .MES 为引擎 LZSS-like 压缩流，需要解压。
  - graph.arc 里的 .GPX 为原始图像资源，不应套 LZ 解压，直接切出即可。

因此入口脚本默认使用 auto 模式：按扩展名决定是否解压。
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct
from typing import Iterable, List, Literal


ENTRY_SIZE = 0x18
NAME_SIZE = 0x10
WINDOW_SIZE = 0x1000
MAX_ENTRY_COUNT = 1000  # 反编译代码里也有 count <= 1000 的检查

# 目前确认需要 LZ 解压的脚本类资源。后续如果遇到其它压缩扩展名，在这里补。
COMPRESSED_EXTS = {".mes"}

ExtractMode = Literal["auto", "raw", "decompress"]


class ArcFormatError(ValueError):
    """ARC 结构不合法时抛出。"""


@dataclass(frozen=True)
class ArcEntry:
    index: int
    name: str
    offset: int
    packed_size: int

    @property
    def end_offset(self) -> int:
        return self.offset + self.packed_size

    @property
    def suffix(self) -> str:
        return Path(self.name).suffix.lower()

    @property
    def should_decompress_by_name(self) -> bool:
        """auto 模式下按文件扩展名判断是否需要 LZ 解压。"""
        return self.suffix in COMPRESSED_EXTS


def _decode_name(raw: bytes) -> str:
    """目录项文件名目前是 ASCII/CP932 兼容的短文件名。"""
    raw = raw.split(b"\x00", 1)[0]
    if not raw:
        return ""
    return raw.decode("cp932", errors="replace")


def parse_arc(data: bytes) -> List[ArcEntry]:
    """解析 ARC 目录表并做基本边界校验。"""
    if len(data) < 4:
        raise ArcFormatError("文件太短，缺少 count 字段")

    count = struct.unpack_from("<I", data, 0)[0]
    if count > MAX_ENTRY_COUNT:
        raise ArcFormatError(f"目录项数量异常: {count} > {MAX_ENTRY_COUNT}")

    header_size = 4 + count * ENTRY_SIZE
    if header_size > len(data):
        raise ArcFormatError(f"目录表越界: header_size=0x{header_size:X}, file_size=0x{len(data):X}")

    entries: List[ArcEntry] = []
    for i in range(count):
        base = 4 + i * ENTRY_SIZE
        name = _decode_name(data[base : base + NAME_SIZE])
        offset, packed_size = struct.unpack_from("<II", data, base + NAME_SIZE)
        if not name:
            raise ArcFormatError(f"第 {i} 个目录项文件名为空")
        if offset < header_size:
            raise ArcFormatError(f"{name}: 数据偏移落在目录表内 offset=0x{offset:X}")
        if offset + packed_size > len(data):
            raise ArcFormatError(
                f"{name}: 数据越界 offset=0x{offset:X}, size=0x{packed_size:X}, file=0x{len(data):X}"
            )
        entries.append(ArcEntry(i, name, offset, packed_size))

    return entries


class BitReader:
    """MSB-first bit reader，对应反编译中的逐 bit 读取逻辑。"""

    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0
        self.mask = 0
        self.current = 0

    def read_bit(self) -> int:
        if self.mask == 0:
            if self.pos >= len(self.data):
                raise ArcFormatError("压缩流提前结束，未读到结束标记")
            self.current = self.data[self.pos]
            self.pos += 1
            self.mask = 0x80
        bit = 1 if (self.current & self.mask) else 0
        self.mask >>= 1
        return bit

    def read_bits(self, n: int) -> int:
        value = 0
        for _ in range(n):
            value = (value << 1) | self.read_bit()
        return value


def decompress_lzss(packed: bytes) -> bytes:
    """解压 ARC 内的 LZSS-like 条目数据。

    注意：ARC 目录只保存条目数据大小，不保存解压大小；压缩流以 12bit offset=0 作为结束标记。
    graph.arc 这类图像资源不是该压缩流，不能调用本函数处理。
    """
    br = BitReader(packed)
    window = bytearray(WINDOW_SIZE)
    write_pos = 1
    out = bytearray()

    while True:
        # 连续 literal，直到读到 0 控制位
        while br.read_bit() != 0:
            value = br.read_bits(8)
            out.append(value)
            window[write_pos] = value
            write_pos = (write_pos + 1) & 0xFFF

        ref_pos = br.read_bits(12)
        if ref_pos == 0:
            break

        length = br.read_bits(4) + 2
        for _ in range(length):
            value = window[ref_pos]
            ref_pos = (ref_pos + 1) & 0xFFF
            out.append(value)
            window[write_pos] = value
            write_pos = (write_pos + 1) & 0xFFF

    return bytes(out)


def extract_payload(entry: ArcEntry, packed: bytes, mode: ExtractMode = "auto") -> tuple[bytes, str]:
    """按指定模式取得输出数据。

    返回值为 (payload, actual_mode)：
      - actual_mode == "raw"：直接输出原始条目数据。
      - actual_mode == "decompressed"：输出 LZ 解压后的数据。
    """
    if mode == "raw":
        return packed, "raw"
    if mode == "decompress":
        return decompress_lzss(packed), "decompressed"
    if mode == "auto":
        if entry.should_decompress_by_name:
            return decompress_lzss(packed), "decompressed"
        return packed, "raw"
    raise ArcFormatError(f"未知解包模式: {mode}")


def iter_arc_files(data: bytes, *, mode: ExtractMode = "auto") -> Iterable[tuple[ArcEntry, bytes, str]]:
    """遍历 ARC 内文件，返回目录项、输出数据、实际处理模式。"""
    for entry in parse_arc(data):
        packed = data[entry.offset : entry.end_offset]
        payload, actual_mode = extract_payload(entry, packed, mode=mode)
        yield entry, payload, actual_mode


def safe_output_path(root: Path, name: str) -> Path:
    """防止目录项文件名中出现路径穿越。"""
    name = name.replace("\\", "/")
    parts = [p for p in name.split("/") if p not in ("", ".", "..")]
    if not parts:
        raise ArcFormatError(f"非法输出文件名: {name!r}")
    return root.joinpath(*parts)
