# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import math
from pathlib import Path


def align_up(v: int, a: int) -> int:
    return (v + a - 1) // a * a


def read_u16(b: bytes | bytearray, off: int) -> int:
    return int.from_bytes(b[off:off + 2], "little")


def read_u32(b: bytes | bytearray, off: int) -> int:
    return int.from_bytes(b[off:off + 4], "little")


def write_u16(b: bytearray, off: int, v: int) -> None:
    b[off:off + 2] = v.to_bytes(2, "little")


def write_u32(b: bytearray, off: int, v: int) -> None:
    b[off:off + 4] = v.to_bytes(4, "little")


def add_font_section(exe: bytes, tbl: bytes, fnt: bytes, section_name: bytes = b".font") -> tuple[bytes, dict]:
    data = bytearray(exe)
    if data[:2] != b"MZ":
        raise ValueError("not an MZ executable")
    pe = read_u32(data, 0x3C)
    if data[pe:pe + 4] != b"PE\x00\x00":
        raise ValueError("not a PE executable")
    coff = pe + 4
    num_sections = read_u16(data, coff + 2)
    opt_size = read_u16(data, coff + 16)
    opt = coff + 20
    magic = read_u16(data, opt)
    if magic != 0x10B:
        raise ValueError("only PE32 is supported")
    section_alignment = read_u32(data, opt + 0x20)
    file_alignment = read_u32(data, opt + 0x24)
    size_of_image_off = opt + 0x38
    size_of_headers = read_u32(data, opt + 0x3C)
    sec_table = opt + opt_size
    new_sec_hdr = sec_table + num_sections * 40
    if new_sec_hdr + 40 > size_of_headers:
        raise ValueError("no room for a new section header; use a PE editor or extend headers first")
    last = sec_table + (num_sections - 1) * 40
    last_va = read_u32(data, last + 12)
    last_raw_size = read_u32(data, last + 16)
    last_raw = read_u32(data, last + 20)
    last_vsize = read_u32(data, last + 8)
    new_va = align_up(last_va + max(last_vsize, last_raw_size), section_alignment)
    new_raw = align_up(last_raw + last_raw_size, file_alignment)

    payload = bytearray()
    tbl_rva = new_va
    payload.extend(tbl)
    # Keep the FNT on a file-aligned boundary inside the new section. This also
    # mirrors the old manual workflow where TBL and FNT were separately aligned.
    while len(payload) % file_alignment:
        payload.append(0)
    fnt_rva = new_va + len(payload)
    payload.extend(fnt)
    raw_size = align_up(len(payload), file_alignment)
    virt_size = len(payload)
    payload.extend(b"\x00" * (raw_size - len(payload)))

    if len(data) < new_raw:
        data.extend(b"\x00" * (new_raw - len(data)))
    data[new_raw:new_raw + raw_size] = payload

    name = section_name[:8].ljust(8, b"\x00")
    data[new_sec_hdr:new_sec_hdr + 8] = name
    write_u32(data, new_sec_hdr + 8, virt_size)
    write_u32(data, new_sec_hdr + 12, new_va)
    write_u32(data, new_sec_hdr + 16, raw_size)
    write_u32(data, new_sec_hdr + 20, new_raw)
    write_u32(data, new_sec_hdr + 24, 0)
    write_u32(data, new_sec_hdr + 28, 0)
    write_u16(data, new_sec_hdr + 32, 0)
    write_u16(data, new_sec_hdr + 34, 0)
    # initialized data, readable
    write_u32(data, new_sec_hdr + 36, 0x40000040)

    write_u16(data, coff + 2, num_sections + 1)
    write_u32(data, size_of_image_off, align_up(new_va + virt_size, section_alignment))
    info = {
        "section": section_name.decode("ascii", "ignore"),
        "section_rva": new_va,
        "section_raw": new_raw,
        "section_virtual_size": virt_size,
        "section_raw_size": raw_size,
        "tbl_rva": tbl_rva,
        "tbl_size": len(tbl),
        "fnt_rva": fnt_rva,
        "fnt_size": len(fnt),
        "file_alignment": file_alignment,
        "section_alignment": section_alignment,
    }
    return bytes(data), info


def apply_patch_u32(data: bytearray, spec: str, values: dict[str, int]) -> None:
    # Format: file_offset=value, where value can be tbl_rva/fnt_rva/0x1234.
    if "=" not in spec:
        raise ValueError("--patch-u32 must be OFFSET=VALUE")
    left, right = spec.split("=", 1)
    off = int(left, 0)
    key = right.strip()
    if key in values:
        val = values[key]
    else:
        val = int(key, 0)
    if not (0 <= off <= len(data) - 4):
        raise ValueError(f"patch offset out of range: {off:#x}")
    write_u32(data, off, val)


def main() -> None:
    ap = argparse.ArgumentParser(description="Append FONT.TBL/FONT.FNT to a new PE section and optionally patch u32 operands.")
    ap.add_argument("exe")
    ap.add_argument("tbl")
    ap.add_argument("fnt")
    ap.add_argument("out_exe")
    ap.add_argument("--section", default=".font")
    ap.add_argument("--patch-u32", action="append", default=[],
                    help="Optional raw file patch: OFFSET=tbl_rva or OFFSET=fnt_rva or OFFSET=0xVALUE. Repeatable.")
    args = ap.parse_args()
    patched, info = add_font_section(Path(args.exe).read_bytes(), Path(args.tbl).read_bytes(), Path(args.fnt).read_bytes(), args.section.encode("ascii"))
    data = bytearray(patched)
    vals = {"tbl_rva": info["tbl_rva"], "fnt_rva": info["fnt_rva"]}
    for spec in args.patch_u32:
        apply_patch_u32(data, spec, vals)
    Path(args.out_exe).write_bytes(bytes(data))
    for k, v in info.items():
        if isinstance(v, int):
            print(f"{k}=0x{v:X} ({v})")
        else:
            print(f"{k}={v}")
    if not args.patch_u32:
        print("NOTE: section appended only. You still need to patch the EXE font pointer/mov operands to use tbl_rva/fnt_rva.")


if __name__ == "__main__":
    main()
