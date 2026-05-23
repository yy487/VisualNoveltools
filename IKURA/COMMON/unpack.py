# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import os
import struct
from pathlib import Path

HEX_ENCODE_MAP = b"G5FXIL094MPRKWCJ3OEBVA7HQ2SU8Y6TZ1ND"
HEX_TABLE = [
    0x06, 0x21, 0x19, 0x10, 0x08, 0x01, 0x1E, 0x16, 0x1C, 0x07, 0x15, 0x13, 0x0E, 0x23, 0x12, 0x02,
    0x00, 0x17, 0x04, 0x0F, 0x0C, 0x05, 0x09, 0x22, 0x11, 0x0A, 0x18, 0x0B, 0x1A, 0x1F, 0x1B, 0x14,
    0x0D, 0x03, 0x1D, 0x20,
]

def chr2hex(c: int) -> int:
    if 48 <= c <= 57: return c - 48
    if 97 <= c <= 122: return c - 97 + 10
    if 65 <= c <= 90: return c - 65 + 10
    return 0

def chr2hexcode(c: int) -> int:
    return HEX_TABLE[chr2hex(c)]

def encode_hex(symbol: int) -> int:
    if symbol > 127: symbol -= 256
    return HEX_ENCODE_MAP[symbol % 36]

def str2hex(s: bytes | bytearray) -> int:
    v = 0
    for i, b in enumerate(s):
        v |= chr2hex(b) << ((len(s) - i - 1) << 2)
    return v

def create_key(secret: bytes) -> bytearray:
    length = bytearray(2)
    for i in range(2):
        length[i] = encode_hex((chr2hexcode(secret[0x0500 + i]) - chr2hexcode(secret[0x0100 + i])) & 0xFF)
    key_len = str2hex(length)
    key = bytearray(key_len)
    for i in range(key_len):
        key[i] = encode_hex((chr2hexcode(secret[0x0510 + i]) - chr2hexcode(secret[0x0110 + i])) & 0xFF)
    return key

def update_key(secret: bytes, key: bytearray, index: int) -> None:
    p = (index & 0x3F) * 0x10
    for i in range(len(key)):
        key[i] = encode_hex((chr2hexcode(key[i]) + chr2hexcode(secret[p + i])) & 0xFF)

def handle_isf_xor(data: bytes, secret: bytes) -> bytes:
    key = create_key(secret)
    out = bytearray(data)
    for i in range(len(out)):
        if i % len(key) == 0:
            update_key(secret, key, i // len(key))
        out[i] ^= key[i % len(key)]
    return bytes(out)

def auto_extract_secret(exe_path: Path) -> bytes | None:
    data = exe_path.read_bytes()
    off = 0
    while True:
        off = data.find(b"UOB0", off)
        if off < 0:
            return None
        sec = data[off:off + 2048]
        if len(sec) == 2048 and all(0x30 <= b <= 0x5A for b in sec):
            return sec
        off += 4

def unpack_drs(f) -> list[tuple[str, int, int]]:
    f.seek(0)
    dir_size = struct.unpack("<H", f.read(2))[0]
    count = (dir_size // 16) - 1
    f.seek(2)
    offsets = []
    names = []
    for _ in range(count + 1):
        name = f.read(12).split(b"\0")[0].decode("ascii", "ignore")
        off = struct.unpack("<I", f.read(4))[0]
        names.append(name)
        offsets.append(off)
    return [(names[i], offsets[i], offsets[i + 1] - offsets[i]) for i in range(count)]

def unpack_mpx(f) -> list[tuple[str, int, int]]:
    f.seek(8)
    count = struct.unpack("<I", f.read(4))[0]
    f.seek(0x20)
    entries = []
    for _ in range(count):
        e = f.read(0x14)
        name = e[:12].split(b"\0")[0].decode("ascii", "ignore")
        off = struct.unpack("<I", e[12:16])[0]
        size = struct.unpack("<I", e[16:20])[0]
        entries.append((name, off, size))
    return entries

def main() -> None:
    ap = argparse.ArgumentParser(description="解包 DRS/MPX，并按需剥离 ISF 外层 XOR 壳")
    ap.add_argument("package", help="输入 DRS/MPX 包")
    ap.add_argument("output_dir", help="输出散文件目录")
    ap.add_argument("--exe", help="用于自动提取 2048 字节 secret 的 EXE")
    ap.add_argument("--secret", help="直接指定 secret 文件")
    args = ap.parse_args()

    pkg = Path(args.package)
    out_dir = Path(args.output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    secret = None
    if args.secret:
        secret = Path(args.secret).read_bytes()
    elif args.exe:
        secret = auto_extract_secret(Path(args.exe))
        if secret:
            (out_dir / "secret.bin").write_bytes(secret)
            print("[unpack] secret extracted")

    with pkg.open("rb") as f:
        sig = f.read(4)
        f.seek(0)
        if sig == b"SM2M":
            engine = "MPX"
            entries = unpack_mpx(f)
        else:
            engine = "DRS"
            entries = unpack_drs(f)
        for name, off, size in entries:
            f.seek(off)
            data = f.read(size)
            if secret and name.lower().endswith((".isf", ".snr")) and data.endswith(b"SECRETFILTER100a"):
                data = handle_isf_xor(data[:-16], secret)
            p = out_dir / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
    order = {"original_name": pkg.name, "engine": engine, "file_order": [e[0] for e in entries]}
    (out_dir / "file_order.json").write_text(json.dumps(order, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"[unpack] engine={engine} files={len(entries)} output={out_dir}")

if __name__ == "__main__":
    main()
