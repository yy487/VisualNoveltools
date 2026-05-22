# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import sys
from pathlib import Path

if __package__ is None or __package__ == "":
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    __package__ = "yuris"

from .common import DEFAULT_ENCODING, is_encodable, load_json, warn
from .crypto import key_from_text, parse_key
from .extract import extract_project
from .inject import inject_project


def _load_key(args: argparse.Namespace) -> bytes | None:
    if getattr(args, "key_text", None):
        return key_from_text(args.key_text)
    if getattr(args, "key_hex", None):
        return parse_key(args.key_hex)
    return None


def cmd_extract(args: argparse.Namespace) -> None:
    parsed, total, failed = extract_project(
        Path(args.input),
        Path(args.output),
        ysc_path=Path(args.ysc) if args.ysc else None,
        ystl_path=Path(args.yst_list) if args.yst_list else None,
        ysl_path=Path(args.ysl) if args.ysl else None,
        key=_load_key(args),
        xor_mode=args.xor_mode,
        encoding=args.encoding,
        command_names=args.command or ["WORD"],
        include_raw_candidates=args.include_raw_candidates,
        split_files=args.split_files,
        extract_mode=args.extract_mode,
    )
    print(f"[extract] 解析 YSTB：{parsed}")
    print(f"[extract] 提取文本：{total}")
    print(f"[extract] 失败文件：{failed}")


def cmd_inject(args: argparse.Namespace) -> None:
    patched, skipped, failed = inject_project(
        Path(args.input),
        Path(args.json),
        Path(args.output),
        ysc_path=Path(args.ysc) if args.ysc else None,
        key=_load_key(args),
        xor_mode=args.xor_mode,
        encoding=args.encoding,
        target_encoding=args.target_encoding,
        copy_rest=not args.no_copy_rest,
        strict=args.strict,
    )
    print(f"[inject] 成功注入：{patched}")
    print(f"[inject] 跳过未修改：{skipped}")
    print(f"[inject] 失败：{failed}")


def cmd_check_json(args: argparse.Namespace) -> None:
    path = Path(args.json)
    files = [path] if path.is_file() else sorted(path.rglob("*.json"))
    total = 0
    bad = 0
    for file in files:
        for entry in load_json(file):
            total += 1
            msg = entry.get("message", "")
            if not is_encodable(msg, args.encoding):
                bad += 1
                chars = []
                for ch in msg:
                    if not is_encodable(ch, args.encoding) and ch not in chars:
                        chars.append(ch)
                warn(f"不可编码: {file.name} index={entry.get('_index')} chars={''.join(chars[:20])!r}")
    print(f"[check-json] 条目：{total}")
    print(f"[check-json] 不可编码：{bad}")


def add_common_key_args(p: argparse.ArgumentParser) -> None:
    p.add_argument("--key-text", help="用于 CRC32 计算 ybnKey 的字符串")
    p.add_argument("--key-hex", help="直接指定 4 字节 key，例如 12345678 或 0x12345678")
    p.add_argument("--xor-mode", choices=["segment", "flat"], default="segment")
    p.add_argument("--encoding", default=DEFAULT_ENCODING)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="YU-RIS 文本提取/注入工作流")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("extract", help="提取 YSTB 文本到 JSON")
    p.add_argument("input")
    p.add_argument("output")
    p.add_argument("--ysc")
    p.add_argument("--yst-list")
    p.add_argument("--ysl")
    p.add_argument("--command", action="append")
    p.add_argument("--extract-mode", choices=["word", "args", "both"], default="both")
    p.add_argument("--include-raw-candidates", action="store_true")
    p.add_argument("--split-files", action="store_true")
    add_common_key_args(p)
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser("inject", help="把 JSON 注入回 YSTB")
    p.add_argument("input")
    p.add_argument("json")
    p.add_argument("output")
    p.add_argument("--ysc")
    p.add_argument("--target-encoding")
    p.add_argument("--no-copy-rest", action="store_true")
    p.add_argument("--strict", action="store_true")
    add_common_key_args(p)
    p.set_defaults(func=cmd_inject)

    p = sub.add_parser("check-json", help="检查 message 是否可按指定编码写回")
    p.add_argument("json")
    p.add_argument("--encoding", default=DEFAULT_ENCODING)
    p.set_defaults(func=cmd_check_json)

    return parser


def main() -> None:
    args = build_parser().parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
