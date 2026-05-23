# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import os
import struct
from pathlib import Path

from common import align_up, to_bytes


def pack_drs(src_folder: Path, output_path: Path) -> None:
    files = sorted([p for p in src_folder.iterdir() if p.is_file()], key=lambda p: p.name.upper())
    dir_size = (len(files) + 1) * 0x10
    current_offset = 2 + dir_size
    entries = bytearray()
    blobs = bytearray()
    for p in files:
        name = p.name.upper().encode("cp932")[:12]
        entries.extend(name.ljust(12, b"\x00"))
        entries.extend(to_bytes(current_offset, 4))
        data = p.read_bytes()
        blobs.extend(data)
        current_offset += len(data)
    entries.extend(b"\x00" * 12)
    entries.extend(to_bytes(current_offset, 4))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(to_bytes(dir_size, 2) + bytes(entries) + bytes(blobs))


def pack_mpx(src_folder: Path, output_path: Path, file_order: list[str]) -> None:
    magic = b"SM2MPX10"
    ordered = [name for name in file_order if (src_folder / name).exists()]
    num_files = len(ordered)
    header_len = 0x20 + 0x14 * num_files
    file_start = align_up(header_len, 16)
    unk1 = b"isf_r" + b"\x00" * 7 + b"\x20\x00\x00\x00"
    entries = bytearray()
    data_blobs: list[bytes] = []
    for name in ordered:
        name_bytes = name.encode("cp932")
        if len(name_bytes) > 12:
            raise ValueError(f"MPX 文件名超过 12 字节: {name}")
        data = (src_folder / name).read_bytes()
        entries.extend(name_bytes.ljust(12, b"\x00"))
        entries.extend(to_bytes(file_start, 4))
        entries.extend(to_bytes(len(data), 4))
        data_blobs.append(data)
        file_start += align_up(len(data), 16)
    out = bytearray()
    out.extend(magic)
    out.extend(to_bytes(num_files, 4))
    out.extend(to_bytes(header_len, 4))
    out.extend(unk1)
    out.extend(entries)
    out.extend(b"\x00" * (align_up(header_len, 16) - header_len))
    for data in data_blobs:
        out.extend(data)
        out.extend(b"\x00" * (align_up(len(data), 16) - len(data)))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(bytes(out))


def main() -> None:
    ap = argparse.ArgumentParser(description="按 file_order.json 重封 DRS/MPX 包")
    ap.add_argument("input_dir", help="待封包散文件目录")
    ap.add_argument("output", help="输出包路径")
    ap.add_argument("--order", default="file_order.json", help="解包时生成的 file_order.json")
    ap.add_argument("--engine", choices=["auto", "MPX", "DRS"], default="auto")
    args = ap.parse_args()

    src = Path(args.input_dir)
    out = Path(args.output)
    engine = args.engine
    order_path = Path(args.order)
    config = {}
    if order_path.exists():
        config = json.loads(order_path.read_text(encoding="utf-8"))
    if engine == "auto":
        engine = config.get("engine", "MPX")
    if engine == "MPX":
        order = config.get("file_order") or [p.name for p in sorted(src.iterdir()) if p.is_file()]
        pack_mpx(src, out, order)
    else:
        pack_drs(src, out)
    print(f"[pack] engine={engine} output={out}")


if __name__ == "__main__":
    main()
