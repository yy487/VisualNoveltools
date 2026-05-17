#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
同级生/AI5WIN 系 MMO 图像解码共用模块。

当前根据 doukyousei.exe.c 里的 MMO 读取逻辑整理：
- FUN_00422170: 读取 MMO，按 header 中 rect 建立 24bit surface。
- FUN_00421f10: 从 header+0x28 开始解 RGB LZSS 数据。
- FUN_0042c480: 0x1000 环形字典 LZSS 解码。
- FUN_0042c600: RGB 差分还原，第一行横向累加，其余行按上一行纵向累加。

注意：
- MMO 解出的 24bit 像素实际按 B,G,R 顺序存放，导出 PNG 时需要转成标准 RGB。
- 游戏内 surface 是 bottom-up 方向；导出 PNG 时通常需要 flip_y=True 才是肉眼正常方向。
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct
from typing import Iterable, Optional, Tuple

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit("需要 Pillow：pip install pillow") from exc


MMO_MAGIC = b"MMO "


@dataclass(frozen=True)
class MMORect:
    left: int
    top: int
    right: int
    bottom: int

    @property
    def width(self) -> int:
        return self.right - self.left

    @property
    def height(self) -> int:
        return self.bottom - self.top


@dataclass(frozen=True)
class MMOHeader:
    magic: bytes
    image_rect: MMORect
    alpha_rect: MMORect
    alpha_offset: int

    @property
    def width(self) -> int:
        return self.image_rect.width

    @property
    def height(self) -> int:
        return self.image_rect.height

    @property
    def has_alpha(self) -> bool:
        return self.alpha_offset != 0 and self.alpha_rect.width > 0 and self.alpha_rect.height > 0


def _u32le(data: bytes, off: int) -> int:
    return struct.unpack_from("<I", data, off)[0]


def parse_header(data: bytes) -> MMOHeader:
    """解析 0x28 字节 MMO header。"""
    if len(data) < 0x28:
        raise ValueError("文件过小，不足 0x28 字节 MMO header")
    magic = data[:4]
    if magic != MMO_MAGIC:
        raise ValueError(f"不是 MMO 文件：magic={magic!r}")

    # EXE 中按 ushort 取坐标，但样本实际以 dword 存储；低 16 位即有效坐标。
    image_rect = MMORect(
        _u32le(data, 0x04) & 0xFFFF,
        _u32le(data, 0x08) & 0xFFFF,
        _u32le(data, 0x0C) & 0xFFFF,
        _u32le(data, 0x10) & 0xFFFF,
    )
    alpha_rect = MMORect(
        _u32le(data, 0x14) & 0xFFFF,
        _u32le(data, 0x18) & 0xFFFF,
        _u32le(data, 0x1C) & 0xFFFF,
        _u32le(data, 0x20) & 0xFFFF,
    )
    alpha_offset = _u32le(data, 0x24)

    if image_rect.width <= 0 or image_rect.height <= 0:
        raise ValueError(f"非法图像尺寸：{image_rect}")
    return MMOHeader(magic, image_rect, alpha_rect, alpha_offset)


def lzss_decompress(src: bytes | memoryview, expected_size: int) -> Tuple[bytes, int]:
    """复现 FUN_0042c480 的 0x1000 字典 LZSS 解码。

    flag 字节低位优先：
    - bit=1: literal 1 字节
    - bit=0: 2 字节引用，offset = b0 | ((b1 & 0xF0) << 4)，len = (b1 & 0x0F) + 3
    返回：(解码结果, 消耗的压缩字节数)
    """
    view = memoryview(src)
    pos = 0
    flags = 0
    ring = bytearray(0x1000)
    rpos = 0xFEE
    out = bytearray()

    while len(out) < expected_size:
        flags >>= 1
        if (flags & 0x100) == 0:
            if pos >= len(view):
                raise EOFError(f"压缩流提前结束：out={len(out)}, expected={expected_size}")
            flags = view[pos] | 0xFF00
            pos += 1

        if flags & 1:
            if pos >= len(view):
                raise EOFError("literal 读取越界")
            b = view[pos]
            pos += 1
            out.append(b)
            ring[rpos] = b
            rpos = (rpos + 1) & 0xFFF
        else:
            if pos + 2 > len(view):
                raise EOFError("copy token 读取越界")
            b0 = view[pos]
            b1 = view[pos + 1]
            pos += 2
            offset = b0 | ((b1 & 0xF0) << 4)
            count = (b1 & 0x0F) + 3
            for i in range(count):
                b = ring[(offset + i) & 0xFFF]
                out.append(b)
                ring[rpos] = b
                rpos = (rpos + 1) & 0xFFF
                if len(out) >= expected_size:
                    break

    return bytes(out), pos


def restore_rgb_delta(raw: bytes | bytearray, width: int, height: int) -> bytes:
    """复现 FUN_0042c600 的 RGB 差分还原。

    第一行：当前像素 += 左侧已还原像素。
    后续行：当前像素 += 上一行同列已还原像素。
    """
    buf = bytearray(raw)
    expected = width * height * 3
    if len(buf) != expected:
        raise ValueError(f"RGB 解码长度不符：got={len(buf)}, expected={expected}")

    # 第一行横向累加。
    for x in range(1, width):
        i = x * 3
        buf[i + 0] = (buf[i + 0] + buf[i - 3]) & 0xFF
        buf[i + 1] = (buf[i + 1] + buf[i - 2]) & 0xFF
        buf[i + 2] = (buf[i + 2] + buf[i - 1]) & 0xFF

    # 其余行只做纵向累加。
    stride = width * 3
    for y in range(1, height):
        row = y * stride
        prev = row - stride
        for x in range(width):
            i = row + x * 3
            j = prev + x * 3
            buf[i + 0] = (buf[i + 0] + buf[j + 0]) & 0xFF
            buf[i + 1] = (buf[i + 1] + buf[j + 1]) & 0xFF
            buf[i + 2] = (buf[i + 2] + buf[j + 2]) & 0xFF

    return bytes(buf)


def decode_mmo_bytes(data: bytes, *, flip_y: bool = True) -> Image.Image:
    """解码 MMO bytes 为 Pillow Image。默认垂直翻转为 PNG 正常观看方向。"""
    header = parse_header(data)
    rgb_expected = header.width * header.height * 3
    rgb_comp = data[0x28:]
    rgb_raw, rgb_used = lzss_decompress(rgb_comp, rgb_expected)
    rgb = restore_rgb_delta(rgb_raw, header.width, header.height)

    # EXE 最终写入的是 24bit DIB / surface，通道顺序为 BGR。
    # 之前这里直接按 RGB 导出，导致整体偏蓝；这里改为显式按 BGR 解读。
    img = Image.frombytes("RGB", (header.width, header.height), rgb, "raw", "BGR")

    if header.has_alpha:
        alpha_expected = header.alpha_rect.width * header.alpha_rect.height
        alpha_start = 0x28 + header.alpha_offset
        if alpha_start >= len(data):
            raise ValueError(f"alpha_offset 越界：0x{header.alpha_offset:X}")
        alpha_raw, _ = lzss_decompress(data[alpha_start:], alpha_expected)
        alpha_img = Image.frombytes("L", (header.alpha_rect.width, header.alpha_rect.height), alpha_raw)
        if alpha_img.size != img.size:
            # 暂按 rect 位置贴到完整 alpha 平面；样本没有 alpha，先保持兼容实现。
            full = Image.new("L", img.size, 255)
            full.paste(alpha_img, (header.alpha_rect.left - header.image_rect.left,
                                   header.alpha_rect.top - header.image_rect.top))
            alpha_img = full
        img.putalpha(alpha_img)

    if flip_y:
        img = img.transpose(Image.Transpose.FLIP_TOP_BOTTOM)
    return img


def decode_mmo_file(input_path: str | Path, output_path: str | Path, *, flip_y: bool = True) -> None:
    data = Path(input_path).read_bytes()
    img = decode_mmo_bytes(data, flip_y=flip_y)
    out = Path(output_path)
    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)


def iter_mmo_files(paths: Iterable[str | Path], recursive: bool = True) -> Iterable[Path]:
    """遍历输入文件/目录，返回 MMO 文件。

    v3 起目录输入默认递归扫描，后缀判断改为大小写不敏感，避免资源目录
    下存在子目录或 .Mmo/.mMo 这类混合大小写时漏文件。
    """
    seen: set[Path] = set()
    for p in map(Path, paths):
        if p.is_dir():
            iterator = p.rglob("*") if recursive else p.iterdir()
            for child in sorted(iterator):
                if child.is_file() and child.suffix.lower() == ".mmo" and child not in seen:
                    seen.add(child)
                    yield child
        else:
            if p.suffix.lower() == ".mmo" and p not in seen:
                seen.add(p)
                yield p
