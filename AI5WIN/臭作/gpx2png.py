#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AIWIN/旧 ELF 系 GPX -> PNG 转换脚本

适用对象：`graph.arc` 中解包出来的 `.gpx` 图像。

格式依据：
- 当前样本 `syuusaku.exe.c` 的反编译逻辑；
- arc_conv 项目 `arc_aiw_arc.asm` 中的 `aiw_arc` 解码流程。

已确认的 GPX 结构：
    00h uint16le x
    02h uint16le y
    04h uint16le w
    06h uint16le h
    08h uint16le reverse   # 0=正常，1=转置存储
    0Ah uint8[0xEC*3] palette (R,G,B)*236
    2CEh ... packed bitstream

注意：
- GPX 不是前面 message.arc 那种 LZSS 文本压缩流；它有自己独立的图像压缩格式。
- 本脚本既支持单文件，也支持批量目录转换。
- 默认不自动抠透明；如果你确认某个调色板索引是透明色，可用
  `--transparent-index` 指定。
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from dataclasses import dataclass, asdict
from pathlib import Path
import struct
from typing import Iterable, List, Optional

from PIL import Image


HEADER_SIZE = 0x0A
PALETTE_COLORS = 0xEC
PALETTE_SIZE = PALETTE_COLORS * 3
DATA_OFFSET = HEADER_SIZE + PALETTE_SIZE  # 0x2CE

# 来自 arc_conv/arc_aiw_arc.asm 的表
OFFSET_TABLE = [0x14, 0x10, 0x0C, 0x08, 0x06, 0x04, 0x02, 0x01,
                0x00, -0x01, -0x02, -0x04, -0x06, -0x08, -0x0C, -0x10, -0x14]


class GpxFormatError(ValueError):
    pass


@dataclass
class GpxInfo:
    path: str
    x: int
    y: int
    width: int
    height: int
    reverse: int
    packed_size: int
    output_path: str
    transparent_index: Optional[int]
    most_common_index: int


class BitReader:
    """MSB-first bit reader，对应原引擎/arc_conv 的逐 bit 读取逻辑。"""

    def __init__(self, data: bytes, offset: int, size: Optional[int] = None):
        self.data = data
        self.pos = offset
        self.remaining = len(data) - offset if size is None else size
        self.mask = 0x80

    def read_bit(self) -> int:
        if self.remaining <= 0:
            raise EOFError("位流提前结束")
        value = 1 if (self.data[self.pos] & self.mask) else 0
        self.mask >>= 1
        if self.mask == 0:
            self.mask = 0x80
            self.pos += 1
            self.remaining -= 1
        return value

    def read_bits(self, n: int) -> int:
        v = 0
        for _ in range(n):
            v = (v << 1) | self.read_bit()
        return v


@dataclass(frozen=True)
class GpxHeader:
    x: int
    y: int
    width: int
    height: int
    reverse: int


@dataclass(frozen=True)
class DecodedGpx:
    header: GpxHeader
    palette: List[tuple[int, int, int]]
    pixels: bytes


def parse_gpx_header(data: bytes) -> GpxHeader:
    if len(data) < DATA_OFFSET:
        raise GpxFormatError(f"文件过短：{len(data)} < 0x{DATA_OFFSET:X}")
    x, y, w, h, reverse = struct.unpack_from("<5H", data, 0)
    if reverse not in (0, 1):
        raise GpxFormatError(f"不支持的 reverse 值: {reverse}")
    if w == 0 or h == 0:
        raise GpxFormatError(f"非法尺寸: {w}x{h}")
    return GpxHeader(x=x, y=y, width=w, height=h, reverse=reverse)


def parse_palette(data: bytes, palette_order: str = "rgb") -> List[tuple[int, int, int]]:
    """读取 GPX 调色板。

    当前 Shusaku/AIWIN GPX 样本中调色板按 RGB 存储。
    之前误按 BGR 处理会导致红蓝通道互换，例如血迹变蓝。

    为了兼容后续可能遇到的变体，保留 palette_order 参数：
    - rgb：按 R,G,B 读取（默认，当前样本正确）
    - bgr：按 B,G,R 读取
    """
    if palette_order not in {"rgb", "bgr"}:
        raise GpxFormatError(f"不支持的 palette_order: {palette_order}")

    pal = []
    base = HEADER_SIZE
    for i in range(PALETTE_COLORS):
        c0, c1, c2 = data[base + i * 3 : base + i * 3 + 3]
        if palette_order == "rgb":
            pal.append((c0, c1, c2))
        else:
            pal.append((c2, c1, c0))
    return pal


def read_copy_length(br: BitReader) -> int:
    # 对应 arc_conv @@5
    if br.read_bit() == 1:
        return br.read_bit() + 2
    if br.read_bit() == 1:
        return br.read_bits(2) + 4
    if br.read_bit() == 1:
        return br.read_bits(3) + 8
    if br.read_bit() == 1:
        return br.read_bits(6) + 0x10
    if br.read_bit() == 1:
        return br.read_bits(8) + 0x50
    return br.read_bits(10) + 0x150


def unpack_gpx_stream(data: bytes, offset: int, size: int, width: int, height: int) -> bytes:
    """解码一块 GPX 图像数据到线性索引缓冲区。"""
    br = BitReader(data, offset, size)
    out = bytearray(width * height)
    pos = 0

    for _row in range(height):
        remaining_in_row = width
        while remaining_in_row > 0:
            # literal
            if br.read_bit() == 1:
                value = br.read_bits(8) - 0x0A
                if not (0 <= value < PALETTE_COLORS):
                    raise GpxFormatError(f"literal 调色板索引越界: {value}")
                out[pos] = value
                pos += 1
                remaining_in_row -= 1
                continue

            # back-reference
            if br.read_bit() == 1:
                if br.read_bit() == 1:
                    # 上一行 + 横向表
                    delta = -(1 * width) - OFFSET_TABLE[br.read_bits(4)]
                else:
                    # 同一行内短距离回拷，使用表后半段
                    delta = OFFSET_TABLE[9 + br.read_bits(3)]
            else:
                if br.read_bit() == 1:
                    # 往上 2 或 3 行
                    row_back = 2 + br.read_bit()
                else:
                    # 往上 4~7 行
                    row_back = br.read_bits(2) + 4
                delta = -(row_back * width) - OFFSET_TABLE[br.read_bits(4)]

            src = pos + delta
            run_len = read_copy_length(br)
            remaining_in_row -= run_len
            if remaining_in_row < 0:
                raise GpxFormatError("回拷长度越过当前行边界")
            if src < 0:
                raise GpxFormatError("回拷源位置越界（位于输出起点之前）")

            # 原实现用 rep movsb，允许重叠，按前向 memmove 复制。
            for _ in range(run_len):
                out[pos] = out[src]
                pos += 1
                src += 1

    return bytes(out)


def decode_gpx(data: bytes, palette_order: str = "rgb") -> DecodedGpx:
    header = parse_gpx_header(data)
    palette = parse_palette(data, palette_order=palette_order)
    packed_size = len(data) - DATA_OFFSET
    if packed_size < 0:
        raise GpxFormatError("数据区偏移越界")

    if header.reverse == 0:
        pixels = unpack_gpx_stream(data, DATA_OFFSET, packed_size, header.width, header.height)
    else:
        # reverse=1 时，数据以转置方式编码，需要先按 h*w 解码，再转回 w*h
        temp = unpack_gpx_stream(data, DATA_OFFSET, packed_size, header.height, header.width)
        restored = bytearray(header.width * header.height)
        src = 0
        for x in range(header.width):
            for y in range(header.height):
                restored[y * header.width + x] = temp[src]
                src += 1
        pixels = bytes(restored)

    return DecodedGpx(header=header, palette=palette, pixels=pixels)


def build_png_image(decoded: DecodedGpx, transparent_index: Optional[int] = None) -> Image.Image:
    w = decoded.header.width
    h = decoded.header.height
    img = Image.frombytes("P", (w, h), decoded.pixels)

    flat_palette = [0] * 768
    for i, (r, g, b) in enumerate(decoded.palette):
        flat_palette[i * 3 : i * 3 + 3] = [r, g, b]
    img.putpalette(flat_palette)

    if transparent_index is not None:
        img.info["transparency"] = transparent_index
    return img


def auto_detect_transparent_index(decoded: DecodedGpx) -> Optional[int]:
    """保守的自动透明检测：
    - 只在“某个索引占据明显大面积”时考虑；
    - 且该颜色必须是常见 key 色之一（纯黑、纯绿、纯洋红）。
    这样尽量避免误伤正常黑底图。
    """
    counter = Counter(decoded.pixels)
    if not counter:
        return None
    idx, count = counter.most_common(1)[0]
    ratio = count / len(decoded.pixels)
    if ratio < 0.5:
        return None
    color = decoded.palette[idx]
    if color in {(0, 0, 0), (0, 255, 0), (255, 0, 255)}:
        return idx
    return None


def iter_gpx_files(path: Path) -> Iterable[Path]:
    if path.is_file():
        yield path
        return
    for p in sorted(path.rglob("*")):
        if p.is_file() and p.suffix.lower() == ".gpx":
            yield p


def default_output_path(input_path: Path, input_root: Path, output_root: Optional[Path]) -> Path:
    if input_path.is_file() and input_root.is_file():
        return (output_root or input_path.with_suffix(".png"))

    assert input_root.is_dir()
    rel = input_path.relative_to(input_root)
    base = output_root or input_root.with_name(input_root.name + "_png")
    return (base / rel).with_suffix(".png")


def convert_one(
    src: Path,
    input_root: Path,
    output_root: Optional[Path],
    transparent_index: Optional[int],
    auto_transparent: bool,
    overwrite: bool,
    palette_order: str,
) -> GpxInfo:
    data = src.read_bytes()
    decoded = decode_gpx(data, palette_order=palette_order)

    chosen_transparent = transparent_index
    if chosen_transparent is None and auto_transparent:
        chosen_transparent = auto_detect_transparent_index(decoded)

    out_path = default_output_path(src, input_root, output_root)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    if out_path.exists() and not overwrite:
        raise FileExistsError(f"输出已存在：{out_path}")

    img = build_png_image(decoded, chosen_transparent)
    img.save(out_path)

    common_index = Counter(decoded.pixels).most_common(1)[0][0]
    return GpxInfo(
        path=str(src),
        x=decoded.header.x,
        y=decoded.header.y,
        width=decoded.header.width,
        height=decoded.header.height,
        reverse=decoded.header.reverse,
        packed_size=len(data) - DATA_OFFSET,
        output_path=str(out_path),
        transparent_index=chosen_transparent,
        most_common_index=common_index,
    )


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="AIWIN/旧 ELF 系 GPX -> PNG 转换")
    p.add_argument("input", help="输入 .gpx 文件，或包含多个 .gpx 的目录")
    p.add_argument("output", nargs="?", help="输出 .png 文件，或批量输出目录")
    p.add_argument(
        "--transparent-index",
        type=int,
        default=None,
        help="把指定调色板索引设为透明（例如 0、2 等）",
    )
    p.add_argument(
        "--auto-transparent",
        action="store_true",
        help="自动尝试把“占比很高且颜色像 key 色”的索引设为透明",
    )
    p.add_argument("--overwrite", action="store_true", help="允许覆盖已有输出")
    p.add_argument(
        "--palette-order",
        choices=("rgb", "bgr"),
        default="rgb",
        help="GPX 调色板通道顺序，默认 rgb；若遇到红蓝互换的变体可试 bgr",
    )
    p.add_argument(
        "--manifest",
        default=None,
        help="批量转换时输出 manifest.json 路径；单文件时也可用来记录元数据",
    )
    return p.parse_args()


def main() -> int:
    ns = parse_args()
    input_path = Path(ns.input)
    if not input_path.exists():
        raise SystemExit(f"输入不存在：{input_path}")

    output_root = Path(ns.output) if ns.output else None
    infos: List[GpxInfo] = []

    files = list(iter_gpx_files(input_path))
    if not files:
        raise SystemExit("没有找到 .gpx 文件")

    if input_path.is_file() and len(files) == 1:
        info = convert_one(
            files[0],
            input_root=input_path,
            output_root=output_root,
            transparent_index=ns.transparent_index,
            auto_transparent=ns.auto_transparent,
            overwrite=ns.overwrite,
            palette_order=ns.palette_order,
        )
        infos.append(info)
        print(f"[OK] {info.path} -> {info.output_path}  ({info.width}x{info.height}, x={info.x}, y={info.y}, reverse={info.reverse})")
    else:
        if output_root is not None and output_root.suffix.lower() == ".png":
            raise SystemExit("批量模式下 output 不能是单个 .png 文件，请给目录")
        for f in files:
            info = convert_one(
                f,
                input_root=input_path,
                output_root=output_root,
                transparent_index=ns.transparent_index,
                auto_transparent=ns.auto_transparent,
                overwrite=ns.overwrite,
                palette_order=ns.palette_order,
            )
            infos.append(info)
            print(f"[OK] {info.path} -> {info.output_path}")

    if ns.manifest:
        manifest_path = Path(ns.manifest)
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(
            json.dumps([asdict(x) for x in infos], ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        print(f"[OK] 写出 manifest: {manifest_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
