# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

if __package__ is None or __package__ == "":
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    __package__ = "yuris"

from .common import DEFAULT_ENCODING, TextEntry, iter_ybn_files, rel_file_name, save_json, warn
from .crypto import key_from_text, parse_key
from .yscm import Yscm
from .yslb import Yslb
from .ystb import Ystb
from .ystl import Ystl


def script_id_from_filename(path: Path) -> int | None:
    m = re.search(r"yst0*(\d+)\.ybn$", path.name, re.IGNORECASE)
    return int(m.group(1)) if m else None


def load_key(args: argparse.Namespace) -> bytes | None:
    if args.key_text:
        return key_from_text(args.key_text)
    if args.key_hex:
        return parse_key(args.key_hex)
    return None


def extract_project(
    input_path: Path,
    output_path: Path,
    *,
    ysc_path: Path,
    ystl_path: Path | None = None,
    ysl_path: Path | None = None,
    key: bytes | None = None,
    xor_mode: str = "segment",
    encoding: str = DEFAULT_ENCODING,
    command_names: list[str] | None = None,
    include_raw_candidates: bool = False,
    split_files: bool = False,
) -> tuple[int, int, int]:
    yscm = Yscm.read(ysc_path, encoding=encoding)
    ystl = None
    if ystl_path and ystl_path.exists():
        ystl = Ystl.read(ystl_path, encoding=encoding)
    yslb = None
    if ysl_path and ysl_path.exists():
        yslb = Yslb.read(ysl_path, encoding=encoding)

    command_names = command_names or ["WORD"]
    files = [p for p in iter_ybn_files(input_path) if p.name.lower() not in {"ysc.ybn", "yst_list.ybn", "ysl.ybn", "ysv.ybn"}]
    total = 0
    parsed = 0
    failed = 0
    all_entries: list[TextEntry] = []

    root = input_path if input_path.is_dir() else input_path.parent
    for file in files:
        try:
            ystb = Ystb.read_file(file, key=key, xor_mode=xor_mode)
        except Exception as exc:
            failed += 1
            warn(f"跳过无法解析的 YSTB: {file} ({exc})")
            continue
        parsed += 1
        script_id = script_id_from_filename(file)
        source = ystl.get_source(script_id) if ystl and script_id is not None else None
        rel_name = rel_file_name(file, root)
        entries: list[TextEntry] = []
        for idx, tref in enumerate(
            ystb.iter_text_refs(
                yscm,
                command_names=command_names,
                encoding=encoding,
                include_raw_candidates=include_raw_candidates,
            )
        ):
            cmd_name = yscm.command_name(tref.command.command_id)
            labels = yslb.find(script_id, tref.command.index) if yslb and script_id is not None else []
            text_type = "message" if cmd_name.upper() == "WORD" else "candidate"
            entry = TextEntry(
                scr_msg=tref.text,
                message=tref.text,
                _file=rel_name,
                _source=source,
                _script_id=script_id,
                _index=idx,
                _cmd_index=tref.command.index,
                _cmd_offset=tref.command.abs_offset,
                _expr_index=tref.expr_index,
                _expr_id=tref.expression.expr_id,
                _expr_offset=tref.expression.instruction_offset,
                _expr_size=tref.expression.instruction_size,
                _text_format=tref.text_format,
                _opcode=cmd_name,
                _opcode_id=tref.command.command_id,
                _type=text_type,
                _label=labels[0].name if labels else None,
            )
            entries.append(entry)
        total += len(entries)
        if split_files:
            out_file = output_path / (file.stem + ".json")
            save_json(out_file, entries)
        else:
            all_entries.extend(entries)

    if not split_files:
        if output_path.suffix.lower() != ".json":
            output_path.mkdir(parents=True, exist_ok=True)
            out_file = output_path / "yuris_text.json"
        else:
            out_file = output_path
        save_json(out_file, all_entries)

    return parsed, total, failed


def build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="提取 YU-RIS YSTB(v5) 文本为统一 JSON")
    p.add_argument("input", help="输入 ysbin/脚本目录或单个 ystxxxxx.ybn")
    p.add_argument("output", help="输出 JSON 文件或目录")
    p.add_argument("--ysc", required=True, help="ysc.ybn 路径")
    p.add_argument("--yst-list", help="yst_list.ybn 路径，可选，用于写入 _source")
    p.add_argument("--ysl", help="ysl.ybn 路径，可选，用于写入 _label")
    p.add_argument("--key-text", help="用于 CRC32 计算 ybnKey 的字符串")
    p.add_argument("--key-hex", help="直接指定 4 字节 key，例如 12345678 或 0x12345678")
    p.add_argument("--xor-mode", choices=["segment", "flat"], default="segment", help="默认 segment，兼容旧工具可用 flat")
    p.add_argument("--encoding", default=DEFAULT_ENCODING, help="文本编码，默认 cp932")
    p.add_argument("--command", action="append", help="要提取的命令名，默认 WORD；可重复指定")
    p.add_argument("--include-raw-candidates", action="store_true", help="额外尝试提取其他命令中的可疑 raw/push string")
    p.add_argument("--split-files", action="store_true", help="每个 ybn 输出一个 json")
    return p


def main() -> None:
    args = build_arg_parser().parse_args()
    parsed, total, failed = extract_project(
        Path(args.input),
        Path(args.output),
        ysc_path=Path(args.ysc),
        ystl_path=Path(args.yst_list) if args.yst_list else None,
        ysl_path=Path(args.ysl) if args.ysl else None,
        key=load_key(args),
        xor_mode=args.xor_mode,
        encoding=args.encoding,
        command_names=args.command or ["WORD"],
        include_raw_candidates=args.include_raw_candidates,
        split_files=args.split_files,
    )
    print(f"[extract] 解析 YSTB：{parsed}")
    print(f"[extract] 提取文本：{total}")
    print(f"[extract] 失败文件：{failed}")


if __name__ == "__main__":
    main()
