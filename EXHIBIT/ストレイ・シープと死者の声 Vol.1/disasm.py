# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from exhibit_common import DEFAULT_ENCODING, calc_normal_seed, crypt_rld, parse_dlr, seed_for_path


def main() -> None:
    ap = argparse.ArgumentParser(description="ExHIBIT DLR/RLD 反汇编辅助")
    ap.add_argument("input")
    ap.add_argument("output")
    ap.add_argument("--exe")
    ap.add_argument("--ini")
    ap.add_argument("--seed")
    ap.add_argument("--include-title", action="store_true")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    args = ap.parse_args()
    src = Path(args.input)
    data = src.read_bytes()
    if src.suffix.lower() == ".rld":
        if args.seed:
            seed = int(args.seed, 0) & 0xFFFFFFFF
        else:
            if not args.exe or not args.ini:
                raise SystemExit("RLD input requires --exe/--ini or --seed")
            seed, bits, chk = calc_normal_seed(args.exe, args.ini, include_title=args.include_title)
            print(f"[seed] bitmap_bits=0x{bits:08X} ini_checksum=0x{chk:08X} normal_seed=0x{seed:08X}")
        data = crypt_rld(data, seed_for_path(src, seed))
    elif data[:4] != b"\x00DLR":
        raise SystemExit("input is neither .rld nor plain DLR .bin")
    ops = parse_dlr(data, encoding=args.encoding)
    lines = [f"# file={src.name}", f"# op_count={len(ops)}"]
    for op in ops:
        lines.append(f"[OP {op.index:05d}] OFF=0x{op.offset:08X} RAW=0x{op.raw:08X} CODE=0x{op.code:04X} INIT={len(op.init_values)} STR={len(op.strings)}")
        if op.init_values:
            lines.append("  INITS: " + " ".join(f"0x{x:08X}" for x in op.init_values))
        for s in op.strings:
            lines.append(f"  STR[{s.index}] OFF=0x{s.offset:08X}: {s.text.replace(chr(10), '[n]')}")
        lines.append("")
    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    Path(args.output).write_text("\n".join(lines), encoding="utf-8")
    print(f"[write] {args.output}")


if __name__ == "__main__":
    main()
