# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path
from exhibit_common import (
    DEFAULT_ENCODING, auto_find_normal_seed, calc_normal_seed, crypt_rld, load_json, parse_dlr, replace_cstrings,
    seed_for_path, unescape_text_from_json,
)


def collect_json_entries(json_path: Path):
    mapping: dict[str, list] = defaultdict(list)
    paths = sorted(json_path.rglob("*.json")) if json_path.is_dir() else [json_path]
    for jp in paths:
        for e in load_json(jp):
            file = e.get("_file")
            if not file:
                print(f"[json][warn] missing _file in {jp}: {e}")
                continue
            mapping[str(file)].append(e)
    return mapping


def source_for_rel(input_root: Path, rel: str) -> Path:
    p = input_root / rel
    if p.exists():
        return p
    # JSON 由单文件提取时，_file 是文件名；目录注入时尝试直接匹配文件名。
    matches = list(input_root.rglob(Path(rel).name)) if input_root.is_dir() else []
    if len(matches) == 1:
        return matches[0]
    raise FileNotFoundError(f"cannot locate source file for _file={rel}")


def out_for_source(src: Path, input_root: Path, out_root: Path) -> Path:
    if input_root.is_dir():
        return out_root / src.relative_to(input_root)
    return out_root if out_root.suffix else out_root / src.name



def _replace_csv_field_preserve_ws(field: str, new_value: str) -> str:
    leading_len = len(field) - len(field.lstrip())
    trailing_len = len(field) - len(field.rstrip())
    leading = field[:leading_len]
    trailing = field[len(field) - trailing_len:] if trailing_len else ""
    return leading + new_value + trailing


def build_replacements(data: bytes, entries: list, encoding: str, strict_scr: bool = True) -> dict[int, str]:
    ops = parse_dlr(data, encoding=encoding)
    replacements: dict[int, str] = {}
    failed = 0
    for e in entries:
        msg = e.get("message")
        scr = e.get("scr_msg")
        if not isinstance(msg, str) or not isinstance(scr, str):
            print(f"[inject][warn] missing scr_msg/message: {e}")
            failed += 1
            continue
        off = e.get("_offset")
        op_index = e.get("_op_index")
        str_index = e.get("_str_index")
        target_off = None
        old_text = None
        if isinstance(op_index, int) and isinstance(str_index, int) and 0 <= op_index < len(ops):
            for s in ops[op_index].strings:
                if s.index == str_index:
                    target_off = s.offset
                    old_text = s.text
                    break
        if target_off is None and isinstance(off, int):
            z = data.find(b"\x00", off)
            if z >= 0:
                target_off = off
                old_text = data[off:z].decode(encoding, errors="replace")
        if target_off is None or old_text is None:
            print(f"[inject][warn] cannot locate entry index={e.get('_index')} scr={scr}")
            failed += 1
            continue

        typ = e.get("_type")
        scr_unesc = unescape_text_from_json(scr)

        if typ == "name":
            # Name entries expose only one CSV field inside opcode 0x30's first C string.
            # Rebuild the whole original CSV string, but validate and replace only that field.
            name_field = e.get("_name_field", 3)
            if not isinstance(name_field, int):
                name_field = 3
            parts = old_text.split(",")
            if name_field < 0 or name_field >= len(parts):
                print(f"[inject][warn] bad _name_field={name_field} for name entry: {e}")
                failed += 1
                continue
            old_name = parts[name_field].strip()
            if strict_scr and old_name != scr_unesc:
                print(f"[inject][warn] name scr_msg mismatch at 0x{target_off:X}: json={scr_unesc!r} file={old_name!r}")
                failed += 1
                continue
            parts[name_field] = _replace_csv_field_preserve_ws(parts[name_field], unescape_text_from_json(msg))
            replacements[target_off] = ",".join(parts)
            continue

        if strict_scr and old_text != scr_unesc:
            print(f"[inject][warn] scr_msg mismatch at 0x{target_off:X}: json={scr_unesc!r} file={old_text!r}")
            failed += 1
            continue
        replacements[target_off] = msg
    if failed:
        raise RuntimeError(f"inject failed entries={failed}")
    return replacements

def main() -> None:
    ap = argparse.ArgumentParser(description="ExHIBIT JSON 注入回 RLD/BIN。正文写回 message；_type=name 会写回 defChara 角色名；正文条目的 name 仍只作上下文。")
    ap.add_argument("input", help="原始 .rld/.bin 文件或目录")
    ap.add_argument("json", help="翻译 JSON 文件或目录")
    ap.add_argument("output", help="输出文件或目录")
    ap.add_argument("--exe", help="游戏 EXE，输入 .rld 时用于静态 seed")
    ap.add_argument("--ini", help="ExHIBIT.ini，输入 .rld 时用于静态 seed")
    ap.add_argument("--seed", help="手动指定普通 seed，例如 0x851C549B")
    ap.add_argument("--include-title", action="store_true", help="旧兼容模式：计算 ini checksum 时包含 TITLE")
    ap.add_argument("--fixed-seed", action="store_true", help="使用旧的固定 RT_BITMAP id=0x98 算法，不自动枚举资源/采样方式")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--errors", default="strict", choices=["strict", "replace", "ignore"], help="编码错误处理")
    ap.add_argument("--no-strict-scr", action="store_true", help="关闭 scr_msg 校验，不建议")
    args = ap.parse_args()

    inp = Path(args.input)
    out = Path(args.output)
    json_entries = collect_json_entries(Path(args.json))

    normal_seed = None
    if args.seed:
        normal_seed = int(args.seed, 0) & 0xFFFFFFFF
        print(f"[seed] manual normal_seed=0x{normal_seed:08X}")
    elif args.exe and args.ini:
        if args.fixed_seed:
            normal_seed, bits, chk = calc_normal_seed(args.exe, args.ini, include_title=args.include_title)
            print(f"[seed] fixed bitmap_bits=0x{bits:08X} ini_checksum=0x{chk:08X} normal_seed=0x{normal_seed:08X}")
        else:
            sample = None
            if inp.is_dir():
                for rel in json_entries.keys():
                    cand = source_for_rel(inp, rel)
                    if cand.suffix.lower() == ".rld" and not cand.name.lower().startswith("def"):
                        sample = cand
                        break
            else:
                sample = inp if inp.suffix.lower() == ".rld" else None
            if sample is None:
                raise SystemExit("auto seed requires a non-def .rld source, or use --seed / --fixed-seed")
            normal_seed, info = auto_find_normal_seed(args.exe, args.ini, sample, encoding=args.encoding)
            print(f"[seed] auto normal_seed=0x{normal_seed:08X} sample={sample.name}")
            print(f"[seed] bitmap_bits=0x{info['bitmap_bits']:08X} ini_checksum=0x{info['ini_checksum']:08X} variant={info['ini_variant']}")
            print(f"[seed] bitmap_candidate={info['bitmap_candidate']}")

    patched_files = 0
    patched_entries = 0
    for rel, entries in json_entries.items():
        src = source_for_rel(inp, rel) if inp.is_dir() else inp
        original = src.read_bytes()
        # RLD 头部本来就是明文 magic，所以不能用 magic 判断是否需要解密。
        is_rld = src.suffix.lower() == ".rld"
        seed = None
        data = original
        if is_rld:
            seed = seed_for_path(src, normal_seed)
            data = crypt_rld(original, seed)
        replacements = build_replacements(data, entries, args.encoding, strict_scr=not args.no_strict_scr)
        rebuilt = replace_cstrings(data, replacements, encoding=args.encoding, errors=args.errors)
        final = crypt_rld(rebuilt, seed) if is_rld and seed is not None else rebuilt
        dst = out_for_source(src, inp, out)
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(final)
        print(f"[inject] {src.name}: patched={len(replacements)} -> {dst}")
        patched_files += 1
        patched_entries += len(replacements)
    print(f"[done] files={patched_files} patched={patched_entries}")


if __name__ == "__main__":
    main()
