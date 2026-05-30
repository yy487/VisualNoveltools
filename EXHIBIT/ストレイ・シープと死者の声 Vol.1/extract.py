# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from exhibit_common import (
    DEFAULT_ENCODING, DEF_SEED, auto_find_normal_seed, build_name_table, calc_normal_seed, crypt_rld,
    has_textual_japanese_or_marks, is_def_rld_name, is_definition_rld_name, make_entry, make_name_entry, parse_dlr,
    save_json, seed_for_path,
)

TEXT_OP_DIALOGUE = 0x001C
TEXT_OP_PLAIN = 0x0015
TEXT_OP_CHOICE = 0x00BF


def iter_inputs(path: Path):
    if path.is_dir():
        yield from sorted(p for p in path.rglob("*") if p.is_file() and p.suffix.lower() in {".rld", ".bin"})
    else:
        yield path


def load_script_data(path: Path, normal_seed: int | None, encoding: str, force_def: bool = False) -> bytes:
    data = path.read_bytes()
    # 注意：RLD 文件头 0x10 字节本来就是明文，encrypted .rld 也以 b"\x00DLR" 开头。
    # 因此不能只看 magic；只要扩展名是 .rld，就必须按 seed 解密。
    if path.suffix.lower() == ".rld":
        seed = seed_for_path(path, normal_seed, force_def=force_def)
        return crypt_rld(data, seed)
    if data[:4] == b"\x00DLR":
        return data
    raise ValueError(f"not DLR bin and not .rld: {path}")


def _try_load_for_name_table(path: Path, normal_seed: int | None, encoding: str):
    data = path.read_bytes()
    if path.suffix.lower() == ".bin":
        yield "bin", data
        return
    # def.rld normally uses the fixed def seed, but defChara.rld in this engine
    # family can use the normal scenario seed.  Try both and keep the one that
    # actually parses and yields opcode 0x30 names.
    tried: list[tuple[str, int]] = []
    if normal_seed is not None:
        tried.append(("normal", normal_seed))
    tried.append(("def", DEF_SEED))
    seen = set()
    for label, seed in tried:
        if seed in seen:
            continue
        seen.add(seed)
        try:
            yield label, crypt_rld(data, seed)
        except Exception:
            continue


def find_name_table(input_path: Path, normal_seed: int | None, encoding: str) -> dict[int, str]:
    candidates: list[Path] = []
    if input_path.is_dir():
        for p in sorted(input_path.rglob("*")):
            if p.is_file() and p.suffix.lower() in {".bin", ".rld"}:
                # Prefer definition files but also allow any file with opcode 0x30.
                if p.stem.lower() in {"defchara", "def"}:
                    candidates.insert(0, p)
                else:
                    candidates.append(p)
    else:
        base = input_path.parent
        for name in ("defChara.bin", "defChara.rld", "def.bin", "def.rld"):
            p = base / name
            if p.exists():
                candidates.append(p)

    merged: dict[int, str] = {}
    sources = 0
    for p in candidates:
        for label, data in _try_load_for_name_table(p, normal_seed, encoding):
            try:
                names = build_name_table(data, encoding=encoding)
            except Exception:
                continue
            if names:
                before = len(merged)
                merged.update(names)
                added = len(merged) - before
                print(f"[name] loaded {len(names)} names from {p.name} seed={label} added={added}")
                sources += 1
                break
    if not merged:
        print("[name][warn] no global name table found; 0x1C lines with character id will be exported without name")
    else:
        print(f"[name] total names={len(merged)} sources={sources}")
    return merged


def extract_entries_from_data(data: bytes, rel_file: str, name_table: dict[int, str], encoding: str) -> list:
    ops = parse_dlr(data, encoding=encoding)
    entries = []
    text_index = 0
    for op in ops:
        if op.code == TEXT_OP_DIALOGUE:
            normal_name = name_table.get(op.init_values[0], "") if op.init_values else ""
            if len(op.strings) >= 2:
                display = op.strings[0].text.strip()
                name = display if display and display != "*" else normal_name
                for s in op.strings[1:]:
                    if not has_textual_japanese_or_marks(s.text):
                        continue
                    entries.append(make_entry(
                        name=name or None,
                        scr_msg=s.text,
                        file=rel_file,
                        index=text_index,
                        offset=s.offset,
                        inst_offset=op.offset,
                        opcode=op.code,
                        op_index=op.index,
                        str_index=s.index,
                        typ="dialogue" if name else "monologue",
                        encoding=encoding,
                    ))
                    text_index += 1
            else:
                name = normal_name or None
                for s in op.strings:
                    if not has_textual_japanese_or_marks(s.text):
                        continue
                    entries.append(make_entry(
                        name=name,
                        scr_msg=s.text,
                        file=rel_file,
                        index=text_index,
                        offset=s.offset,
                        inst_offset=op.offset,
                        opcode=op.code,
                        op_index=op.index,
                        str_index=s.index,
                        typ="dialogue" if name else "monologue",
                        encoding=encoding,
                    ))
                    text_index += 1

        elif op.code == TEXT_OP_PLAIN:
            for s in op.strings:
                if not has_textual_japanese_or_marks(s.text):
                    continue
                entries.append(make_entry(
                    name=None,
                    scr_msg=s.text,
                    file=rel_file,
                    index=text_index,
                    offset=s.offset,
                    inst_offset=op.offset,
                    opcode=op.code,
                    op_index=op.index,
                    str_index=s.index,
                    typ="monologue",
                    encoding=encoding,
                ))
                text_index += 1

        elif op.code == TEXT_OP_CHOICE:
            # 旧工具把 0xBF 的第一个字符串当可翻译文本。这里保守视为选择支/可选项。
            for s in op.strings[:1]:
                if not has_textual_japanese_or_marks(s.text):
                    continue
                entries.append(make_entry(
                    name=None,
                    scr_msg=s.text,
                    file=rel_file,
                    index=text_index,
                    offset=s.offset,
                    inst_offset=op.offset,
                    opcode=op.code,
                    op_index=op.index,
                    str_index=s.index,
                    typ="choice",
                    encoding=encoding,
                ))
                text_index += 1
    return entries



def extract_name_entries_from_data(data: bytes, rel_file: str, encoding: str) -> list:
    """Export editable character-name definitions from opcode 0x30.

    The name itself is stored as one CSV field inside the opcode's first C string,
    so the JSON exposes only that field while keeping the original string offset
    and op/str indexes for safe reinjection.
    """
    ops = parse_dlr(data, encoding=encoding)
    entries = []
    name_index = 0
    for op in ops:
        if op.code != 0x0030 or not op.strings:
            continue
        s = op.strings[0]
        parts = s.text.split(",")
        if len(parts) < 4:
            continue
        try:
            cid = int(parts[0].strip(), 0)
        except ValueError:
            continue
        name = parts[3].strip()
        if not name or name == "*":
            continue
        entries.append(make_name_entry(
            scr_msg=name,
            file=rel_file,
            index=name_index,
            offset=s.offset,
            inst_offset=op.offset,
            opcode=op.code,
            op_index=op.index,
            str_index=s.index,
            name_id=cid,
            name_field=3,
            encoding=encoding,
        ))
        name_index += 1
    return entries


def iter_name_source_files(input_path: Path):
    """Files that may contain editable character-name definitions."""
    if input_path.is_dir():
        preferred = []
        others = []
        for p in sorted(input_path.rglob("*")):
            if not (p.is_file() and p.suffix.lower() in {".bin", ".rld"}):
                continue
            if p.stem.lower() in {"defchara", "def"}:
                preferred.append(p)
            else:
                others.append(p)
        yield from preferred
        yield from others
    else:
        base = input_path.parent
        for name in ("defChara.bin", "defChara.rld", "def.bin", "def.rld"):
            p = base / name
            if p.exists():
                yield p
        if input_path.stem.lower() in {"defchara", "def"}:
            yield input_path


def main() -> None:
    ap = argparse.ArgumentParser(description="ExHIBIT RLD/BIN 正文与选择支提取为 JSON")
    ap.add_argument("input", help="输入 .rld/.bin 文件或目录")
    ap.add_argument("output", help="输出 JSON 文件或目录")
    ap.add_argument("--exe", help="游戏 EXE，输入 .rld 时用于静态 seed")
    ap.add_argument("--ini", help="ExHIBIT.ini，输入 .rld 时用于静态 seed")
    ap.add_argument("--seed", help="手动指定普通 seed，例如 0x851C549B")
    ap.add_argument("--include-title", action="store_true", help="旧兼容模式：计算 ini checksum 时包含 TITLE")
    ap.add_argument("--fixed-seed", action="store_true", help="使用旧的固定 RT_BITMAP id=0x98 算法，不自动枚举资源/采样方式")
    ap.add_argument("--encoding", default=DEFAULT_ENCODING)
    ap.add_argument("--export-names", action="store_true", help="额外导出 defChara/0x30 角色名条目，修改这些 _type=name 的 message 可写回角色名")
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
            # 通用模式：先用一个普通 .rld 样本验证 seed，避免写死资源 id=0x98。
            sample = None
            for cand in iter_inputs(inp):
                if cand.suffix.lower() == ".rld" and not is_definition_rld_name(cand):
                    sample = cand
                    break
            if sample is None:
                raise SystemExit("auto seed requires at least one non-def .rld sample, or use --seed / --fixed-seed")
            normal_seed, info = auto_find_normal_seed(args.exe, args.ini, sample, encoding=args.encoding)
            print(f"[seed] auto normal_seed=0x{normal_seed:08X} sample={sample.name}")
            print(f"[seed] bitmap_bits=0x{info['bitmap_bits']:08X} ini_checksum=0x{info['ini_checksum']:08X} variant={info['ini_variant']}")
            print(f"[seed] bitmap_candidate={info['bitmap_candidate']}")

    name_table = find_name_table(inp, normal_seed, args.encoding)

    if args.export_names:
        name_total = 0
        written_sources: set[Path] = set()
        for nf in iter_name_source_files(inp):
            if nf in written_sources:
                continue
            written_sources.add(nf)
            rel_name = nf.relative_to(inp).as_posix() if inp.is_dir() else nf.name
            best_entries = []
            for label, data in _try_load_for_name_table(nf, normal_seed, args.encoding):
                try:
                    best_entries = extract_name_entries_from_data(data, rel_name, args.encoding)
                except Exception:
                    continue
                if best_entries:
                    print(f"[names-export] {rel_name}: names={len(best_entries)} seed={label}")
                    break
            if not best_entries:
                continue
            if inp.is_dir():
                dst = (out / rel_name).with_suffix(nf.suffix + ".json")
            else:
                dst = out if out.suffix.lower() == ".json" else out / f"{nf.name}.json"
            save_json(dst, best_entries)
            name_total += len(best_entries)
        print(f"[names-export] total={name_total}")

    total = 0
    files = 0
    for file in iter_inputs(inp):
        if is_definition_rld_name(file):
            continue
        rel = file.relative_to(inp).as_posix() if inp.is_dir() else file.name
        data = load_script_data(file, normal_seed, args.encoding)
        entries = extract_entries_from_data(data, rel, name_table, args.encoding)
        if inp.is_dir():
            dst = (out / rel).with_suffix(file.suffix + ".json")
        else:
            dst = out if out.suffix.lower() == ".json" else out / f"{file.name}.json"
        save_json(dst, entries)
        print(f"[extract] {rel}: entries={len(entries)} -> {dst}")
        total += len(entries)
        files += 1
    print(f"[done] files={files} entries={total}")


if __name__ == "__main__":
    main()
