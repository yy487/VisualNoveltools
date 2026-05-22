# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import shutil
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

if __package__ is None or __package__ == "":
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    __package__ = "yuris"

from .common import DEFAULT_ENCODING, load_json, warn
from .crypto import key_from_text, parse_key
from .extract import load_key
from .yscm import Yscm
from .ystb import Ystb, encode_text_expr


def read_json_entries(path: Path) -> list[dict[str, Any]]:
    if path.is_file():
        return load_json(path)
    entries: list[dict[str, Any]] = []
    for file in sorted(path.rglob("*.json")):
        entries.extend(load_json(file))
    return entries


def group_entries(entries: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for entry in entries:
        file = entry.get("_file")
        if not file:
            warn(f"JSON 条目缺少 _file，跳过: {entry.get('scr_msg', '')[:40]}")
            continue
        grouped[file].append(entry)
    return grouped


def inject_project(
    input_path: Path,
    json_path: Path,
    output_path: Path,
    *,
    ysc_path: Path | None = None,
    key: bytes | None = None,
    xor_mode: str = "segment",
    encoding: str = DEFAULT_ENCODING,
    target_encoding: str | None = None,
    copy_rest: bool = True,
    strict: bool = False,
) -> tuple[int, int, int]:
    target_encoding = target_encoding or encoding
    yscm = Yscm.read(ysc_path, encoding=encoding) if ysc_path and ysc_path.exists() else None
    entries = read_json_entries(json_path)
    grouped = group_entries(entries)

    if input_path.is_file():
        root = input_path.parent
    else:
        root = input_path
    output_path.mkdir(parents=True, exist_ok=True)

    patched = 0
    skipped = 0
    failed = 0

    if copy_rest and input_path.is_dir():
        for src in input_path.rglob("*"):
            if src.is_file():
                dst = output_path / src.relative_to(input_path)
                dst.parent.mkdir(parents=True, exist_ok=True)
                if not dst.exists():
                    shutil.copy2(src, dst)

    for rel_file, file_entries in grouped.items():
        src = root / rel_file
        if not src.exists() and input_path.is_file() and input_path.name == rel_file:
            src = input_path
        if not src.exists():
            failed += len(file_entries)
            warn(f"找不到原始脚本: {rel_file}")
            continue
        try:
            ystb = Ystb.read_file(src, key=key, xor_mode=xor_mode)
        except Exception as exc:
            failed += len(file_entries)
            warn(f"无法解析脚本 {rel_file}: {exc}")
            continue

        for entry in file_entries:
            message = entry.get("message")
            scr_msg = entry.get("scr_msg")
            if message is None or scr_msg is None:
                failed += 1
                warn(f"缺少 scr_msg/message，跳过: {rel_file}")
                continue
            if message == scr_msg:
                skipped += 1
                continue

            tref = None
            cmd_index = entry.get("_cmd_index")
            expr_index = entry.get("_expr_index")
            if isinstance(cmd_index, int) and isinstance(expr_index, int):
                try:
                    tref = ystb.find_text_ref_by_position(yscm, cmd_index, expr_index, encoding=encoding)
                except Exception as exc:
                    warn(f"定位失败 {rel_file} cmd={cmd_index} expr={expr_index}: {exc}")

            if tref is None:
                # fallback：同文件内 scr_msg 唯一匹配。
                matches = []
                if yscm is not None:
                    matches.extend(r for r in ystb.iter_text_refs(yscm, encoding=encoding) if r.text == scr_msg)
                matches.extend(r for r in ystb.iter_args_scan_text_refs(encoding=encoding) if r.text == scr_msg)
                # 同一位置可能被结构化逻辑和 args 扫描同时命中，按位置去重。
                uniq = {}
                for r in matches:
                    uniq[(r.command.index, r.expr_index)] = r
                matches = list(uniq.values())
                if len(matches) == 1:
                    tref = matches[0]
                else:
                    failed += 1
                    warn(f"无法唯一定位: {rel_file} scr_msg={str(scr_msg)[:60]!r} matches={len(matches)}")
                    continue

            if tref.text != scr_msg:
                failed += 1
                msg = (
                    f"scr_msg 校验失败: {rel_file} cmd={tref.command.index} expr={tref.expr_index}\n"
                    f"  json: {scr_msg!r}\n  file: {tref.text!r}"
                )
                if strict:
                    raise RuntimeError(msg)
                warn(msg)
                continue

            text_format = entry.get("_text_format") or tref.text_format
            try:
                new_data = encode_text_expr(str(message), text_format, encoding=target_encoding)
            except UnicodeEncodeError as exc:
                failed += 1
                warn(f"编码失败 {rel_file} index={entry.get('_index')}: {exc}")
                continue
            except Exception as exc:
                failed += 1
                warn(f"生成表达式失败 {rel_file} index={entry.get('_index')}: {exc}")
                continue

            ystb.set_expr_data_append(tref.expression, new_data)
            patched += 1

        dst = output_path / rel_file if input_path.is_dir() else output_path / src.name
        ystb.write_file(dst, key=key, xor_mode=xor_mode)

    return patched, skipped, failed


def build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="把统一 JSON 注入回 YU-RIS YSTB(v5)")
    p.add_argument("input", help="原始 ysbin/脚本目录或单个 ystxxxxx.ybn")
    p.add_argument("json", help="翻译 JSON 文件或目录")
    p.add_argument("output", help="输出目录")
    p.add_argument("--ysc", help="ysc.ybn 路径；有 _cmd_index/_expr_index 时可省略，fallback 匹配建议提供")
    p.add_argument("--key-text", help="用于 CRC32 计算 ybnKey 的字符串")
    p.add_argument("--key-hex", help="直接指定 4 字节 key，例如 12345678 或 0x12345678")
    p.add_argument("--xor-mode", choices=["segment", "flat"], default="segment")
    p.add_argument("--encoding", default=DEFAULT_ENCODING, help="原文编码，默认 cp932")
    p.add_argument("--target-encoding", help="写回编码，默认等于 --encoding")
    p.add_argument("--no-copy-rest", action="store_true", help="不复制未修改文件")
    p.add_argument("--strict", action="store_true", help="校验失败立即中止")
    return p


def main() -> None:
    args = build_arg_parser().parse_args()
    patched, skipped, failed = inject_project(
        Path(args.input),
        Path(args.json),
        Path(args.output),
        ysc_path=Path(args.ysc) if args.ysc else None,
        key=load_key(args),
        xor_mode=args.xor_mode,
        encoding=args.encoding,
        target_encoding=args.target_encoding,
        copy_rest=not args.no_copy_rest,
        strict=args.strict,
    )
    print(f"[inject] 成功注入：{patched}")
    print(f"[inject] 跳过未修改：{skipped}")
    print(f"[inject] 失败：{failed}")


if __name__ == "__main__":
    main()
