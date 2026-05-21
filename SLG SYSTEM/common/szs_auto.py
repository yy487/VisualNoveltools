# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from szs_common import (
    CryptoInfo,
    detect_crypto,
    pack_archive,
    read_table,
    unpack_archive,
)


def cmd_list(args: argparse.Namespace) -> None:
    data = Path(args.archive).read_bytes()
    entries = read_table(data)
    for i, ent in enumerate(entries):
        print(f"{i:03d} {ent.name:<20} off=0x{ent.offset:08x} size=0x{ent.size:08x}")


def _detect_from_args(args: argparse.Namespace) -> CryptoInfo:
    if args.seed is not None:
        seed = int(args.seed, 0)
        ax = int(args.xor, 0) if args.xor is not None else 0x90
        return CryptoInfo(seed=seed, archive_xor=ax, mode=args.mode or "full_lcg_sub", source="manual")
    best, top = detect_crypto(
        Path(args.archive),
        exe_path=Path(args.exe) if args.exe else None,
        decompiler_c_path=Path(args.exe_c) if args.exe_c else None,
        archive_xor=int(args.xor, 0) if args.xor is not None else None,
        show_top=args.top,
    )
    print(f"[detect] best seed=0x{best.seed:08x} xor=0x{best.archive_xor:02x} mode={best.mode} score={best.score:.4f}")
    if args.verbose:
        print("[detect] top candidates:")
        for r in top:
            print(f"  seed=0x{r.seed:08x} xor=0x{r.archive_xor:02x} mode={r.mode} score={r.score:.4f}")
    return best


def cmd_detect(args: argparse.Namespace) -> None:
    _detect_from_args(args)


def cmd_unpack(args: argparse.Namespace) -> None:
    crypto = _detect_from_args(args)
    entries = unpack_archive(Path(args.archive), Path(args.output), crypto=crypto)
    print(f"[unpack] entries={len(entries)} out={args.output}")


def cmd_pack(args: argparse.Namespace) -> None:
    crypto = None
    if args.seed is not None:
        crypto = CryptoInfo(
            seed=int(args.seed, 0),
            archive_xor=int(args.xor, 0) if args.xor is not None else 0x90,
            mode=args.mode or "full_lcg_sub",
            source="manual",
        )
    entries = pack_archive(Path(args.input), Path(args.output), crypto=crypto)
    print(f"[pack] entries={len(entries)} out={args.output}")


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="SZS100__ auto seed detector / unpacker / packer")
    sub = p.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="list SZS table")
    p_list.add_argument("archive")
    p_list.set_defaults(func=cmd_list)

    def add_detect_options(sp: argparse.ArgumentParser) -> None:
        sp.add_argument("archive", help="input .szs")
        sp.add_argument("--exe", help="game exe; if exe.c exists beside it, it will be used too")
        sp.add_argument("--exe-c", help="Ghidra/decompiler C file, e.g. sangokuhime2.exe.c")
        sp.add_argument("--seed", help="manual seed, e.g. 0x3e9f9d19")
        sp.add_argument("--xor", help="manual archive XOR byte, default/detected usually 0x90")
        sp.add_argument("--mode", choices=["full_lcg_sub", "reseed_lcg_xor"], help="manual crypto mode; auto-detected when omitted")
        sp.add_argument("--top", type=int, default=10, help="number of candidates to show when --verbose")
        sp.add_argument("-v", "--verbose", action="store_true")

    p_detect = sub.add_parser("detect", help="detect stream seed from exe/exe.c and verify by trial decrypting the archive")
    add_detect_options(p_detect)
    p_detect.set_defaults(func=cmd_detect)

    p_unpack = sub.add_parser("unpack", help="detect crypto and unpack/decrypt")
    add_detect_options(p_unpack)
    p_unpack.add_argument("output", help="output directory")
    p_unpack.set_defaults(func=cmd_unpack)

    p_pack = sub.add_parser("pack", help="pack/encrypt from an unpack directory")
    p_pack.add_argument("input", help="unpacked directory containing manifest.json")
    p_pack.add_argument("output", help="output .szs")
    p_pack.add_argument("--seed", help="manual seed; omitted means use manifest crypto")
    p_pack.add_argument("--xor", help="manual archive XOR byte; omitted means use manifest/default")
    p_pack.add_argument("--mode", choices=["full_lcg_sub", "reseed_lcg_xor"], help="manual crypto mode; omitted means use manifest")
    p_pack.set_defaults(func=cmd_pack)

    return p


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
