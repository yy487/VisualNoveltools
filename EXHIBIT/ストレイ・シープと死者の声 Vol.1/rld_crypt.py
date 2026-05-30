# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from exhibit_common import auto_find_normal_seed, calc_normal_seed, crypt_rld, is_def_rld_name, seed_for_path, validate_dlr, DEF_SEED


def iter_files(path: Path, suffix: str):
    if path.is_dir():
        yield from sorted(p for p in path.rglob(f"*{suffix}") if p.is_file())
    else:
        yield path


def out_path_for(in_file: Path, in_root: Path, out_root: Path, suffix: str) -> Path:
    if in_root.is_dir():
        rel = in_file.relative_to(in_root)
        return (out_root / rel).with_suffix(suffix)
    return out_root if out_root.suffix else (out_root / in_file.with_suffix(suffix).name)


def main() -> None:
    ap = argparse.ArgumentParser(description="ExHIBIT RLD 静态解密/加密工具。XOR 对称，decrypt/encrypt 都使用同一算法。")
    sub = ap.add_subparsers(dest="cmd", required=True)

    for name, in_suf, out_suf, desc in [
        ("decrypt", ".rld", ".bin", "rld -> bin"),
        ("encrypt", ".bin", ".rld", "bin -> rld"),
    ]:
        sp = sub.add_parser(name, help=desc)
        sp.add_argument("input", help="输入文件或目录")
        sp.add_argument("output", help="输出文件或目录")
        sp.add_argument("--exe", help="游戏 EXE，用于计算普通 seed")
        sp.add_argument("--ini", help="ExHIBIT.ini，用于计算普通 seed")
        sp.add_argument("--seed", help="手动指定普通 seed，例如 0x851C549B")
        sp.add_argument("--include-title", action="store_true", help="旧兼容模式：计算 ini checksum 时包含 TITLE")
        sp.add_argument("--fixed-seed", action="store_true", help="使用旧的固定 RT_BITMAP id=0x98 算法，不自动枚举资源/采样方式")
        sp.add_argument("--force-def", action="store_true", help="强制使用 def seed 0xAE85A916")
        sp.add_argument("--sample-rld", help="用于自动验证 seed 的普通 .rld 样本；encrypt 输入为 .bin 时建议提供")
        sp.add_argument("--encoding", default="cp932")

    args = ap.parse_args()
    inp = Path(args.input)
    out = Path(args.output)

    normal_seed = None
    if args.seed:
        normal_seed = int(args.seed, 0) & 0xFFFFFFFF
        print(f"[seed] manual normal_seed=0x{normal_seed:08X}")
    elif args.exe and args.ini:
        if args.fixed_seed:
            normal_seed, bits, chk = calc_normal_seed(args.exe, args.ini, include_title=args.include_title)
            print(f"[seed] fixed bitmap_bits=0x{bits:08X} ini_checksum=0x{chk:08X} normal_seed=0x{normal_seed:08X}")
        else:
            sample = Path(args.sample_rld) if args.sample_rld else None
            if sample is None and args.cmd == "decrypt":
                for cand in iter_files(inp, ".rld"):
                    if not is_def_rld_name(cand):
                        sample = cand
                        break
            if sample is None:
                raise SystemExit("auto seed requires --sample-rld when encrypting .bin, or use --seed / --fixed-seed")
            normal_seed, info = auto_find_normal_seed(args.exe, args.ini, sample, encoding=args.encoding)
            print(f"[seed] auto normal_seed=0x{normal_seed:08X} sample={sample.name}")
            print(f"[seed] bitmap_bits=0x{info['bitmap_bits']:08X} ini_checksum=0x{info['ini_checksum']:08X} variant={info['ini_variant']}")
            print(f"[seed] bitmap_candidate={info['bitmap_candidate']}")
    elif not args.force_def:
        raise SystemExit("non-def files require --exe/--ini or --seed; use --force-def only for def files")

    in_suffix = ".rld" if args.cmd == "decrypt" else ".bin"
    out_suffix = ".bin" if args.cmd == "decrypt" else ".rld"
    count = 0
    for file in iter_files(inp, in_suffix):
        seed = DEF_SEED if args.force_def else seed_for_path(file, normal_seed)
        data = file.read_bytes()
        result = crypt_rld(data, seed)
        dst = out_path_for(file, inp, out, out_suffix)
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(result)
        if args.cmd == "decrypt":
            good, msg = validate_dlr(result, encoding=args.encoding)
            print(f"[decrypt] {file} -> {dst} seed=0x{seed:08X} validate={msg} good_ops={good}")
        else:
            print(f"[encrypt] {file} -> {dst} seed=0x{seed:08X}")
        count += 1
    print(f"[done] files={count}")


if __name__ == "__main__":
    main()
