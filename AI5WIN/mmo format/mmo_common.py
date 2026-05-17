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
import ctypes
import os
import platform
import struct
from typing import Iterable, Optional, Tuple

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit("需要 Pillow：pip install pillow") from exc


MMO_MAGIC = b"MMO "


class _MMOFast:
    """ctypes 封装的 C 加速核心。找不到 DLL/SO 时自动回退纯 Python。"""

    def __init__(self) -> None:
        self.lib = None
        self.path: Optional[Path] = None
        self.error: Optional[str] = None
        self._load()

    def _candidate_names(self) -> list[str]:
        sysname = platform.system().lower()
        if sysname == "windows":
            return ["mmo_fast.dll"]
        if sysname == "darwin":
            return ["mmo_fast.dylib", "mmo_fast.so"]
        return ["mmo_fast.so"]

    def _load(self) -> None:
        base = Path(__file__).resolve().parent
        for name in self._candidate_names():
            path = base / name
            if not path.exists():
                continue
            try:
                lib = ctypes.CDLL(str(path))
                fn = lib.mmo_decode_rgb_fast
                fn.argtypes = [
                    ctypes.c_void_p, ctypes.c_size_t,
                    ctypes.c_uint32, ctypes.c_uint32,
                    ctypes.c_void_p, ctypes.c_size_t,
                    ctypes.POINTER(ctypes.c_size_t),
                    ctypes.c_char_p, ctypes.c_size_t,
                ]
                fn.restype = ctypes.c_int
                self.lib = lib
                self.path = path
                self.error = None
                return
            except Exception as exc:  # pragma: no cover
                self.error = f"加载 {path} 失败：{exc}"
        if self.error is None:
            self.error = "未找到 mmo_fast 动态库"

    @property
    def available(self) -> bool:
        return self.lib is not None

    def decode_rgb(self, comp: bytes | memoryview, width: int, height: int) -> tuple[bytes, int]:
        if self.lib is None:
            raise RuntimeError(self.error or "mmo_fast 不可用")
        src = bytes(comp) if isinstance(comp, memoryview) else comp
        expected = width * height * 3
        dst = ctypes.create_string_buffer(expected)
        used = ctypes.c_size_t(0)
        err = ctypes.create_string_buffer(512)
        rc = self.lib.mmo_decode_rgb_fast(
            ctypes.c_char_p(src), ctypes.c_size_t(len(src)),
            ctypes.c_uint32(width), ctypes.c_uint32(height),
            ctypes.cast(dst, ctypes.c_void_p), ctypes.c_size_t(expected),
            ctypes.byref(used), err, ctypes.c_size_t(len(err)),
        )
        if rc != 0:
            msg = err.value.decode("utf-8", errors="replace") or f"mmo_fast failed rc={rc}"
            raise RuntimeError(msg)
        return dst.raw, int(used.value)


_FAST = _MMOFast()


def mmo_fast_status() -> str:
    """返回 C 加速状态，供 CLI --fast-info 使用。"""
    if _FAST.available:
        return f"enabled: {_FAST.path}"
    return f"disabled: {_FAST.error}"


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


def decode_mmo_bytes(data: bytes, *, flip_y: bool = True, use_fast: bool = True) -> Image.Image:
    """解码 MMO bytes 为 Pillow Image。默认优先使用 C 加速，失败时回退纯 Python。"""
    header = parse_header(data)
    rgb_expected = header.width * header.height * 3
    rgb_comp = data[0x28:]

    # C 快路径：LZSS + 差分还原 + BGR->RGB 全部放进 C，避免 Python 逐字节循环。
    if use_fast and _FAST.available:
        try:
            rgb, rgb_used = _FAST.decode_rgb(rgb_comp, header.width, header.height)
            img = Image.frombytes("RGB", (header.width, header.height), rgb)
        except Exception:
            # 加速库异常时不直接中断，回退纯 Python，保证工具仍然可用。
            rgb_raw, rgb_used = lzss_decompress(rgb_comp, rgb_expected)
            rgb = restore_rgb_delta(rgb_raw, header.width, header.height)
            img = Image.frombytes("RGB", (header.width, header.height), rgb, "raw", "BGR")
    else:
        rgb_raw, rgb_used = lzss_decompress(rgb_comp, rgb_expected)
        rgb = restore_rgb_delta(rgb_raw, header.width, header.height)
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

def decode_mmo_file(input_path: str | Path, output_path: str | Path, *, flip_y: bool = True, use_fast: bool = True) -> None:
    data = Path(input_path).read_bytes()
    img = decode_mmo_bytes(data, flip_y=flip_y, use_fast=use_fast)
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
