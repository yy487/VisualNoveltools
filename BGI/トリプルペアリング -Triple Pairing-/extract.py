# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from common import (
    DEFAULT_ENCODING,
    DEFAULT_FALLBACK_ENCODING,
    extract_workflow_entries_from_bsd,
    extract_workflow_entries_from_script,
    iter_sources,
    json_path_for_source,
    normalize_rel_path,
    parse_user_functions,
    save_json_entries,
)


def extract_one(source: Path, source_root: Path | None, output_root: Path, args) -> int:
    rel_name = normalize_rel_path(source, source_root)
    mode = args.mode
    if mode == "auto":
        mode = "bsd" if source.suffix.lower() == ".bsd" else "script"

    if mode == "bsd":
        entries = extract_workflow_entries_from_bsd(
            source,
            file_name=rel_name,
            user_function_names=parse_user_functions(args.user_functions),
        )
    else:
        entries = extract_workflow_entries_from_script(
            source,
            file_name=rel_name,
            encoding=args.encoding,
            fallback_encoding=args.fallback_encoding,
            user_function_names=parse_user_functions(args.user_functions),
        )

    out_json = json_path_for_source(source, source_root, output_root)
    save_json_entries(out_json, entries)
    print(f"[extract] {rel_name}: {len(entries)} -> {out_json}")
    return len(entries)


def main() -> None:
    parser = argparse.ArgumentParser(description="BGI V1 工作流 JSON 提取工具")
    parser.add_argument("input", help="输入 .bsd、编译脚本文件或目录")
    parser.add_argument("output", help="输出 JSON 文件或目录；目录输入时按相对路径输出 *.json")
    parser.add_argument("--mode", choices=["auto", "bsd", "script"], default="auto", help="输入类型，默认 auto")
    parser.add_argument("--encoding", default=DEFAULT_ENCODING, help="脚本文本主编码，默认 shift_jis")
    parser.add_argument("--fallback-encoding", default=DEFAULT_FALLBACK_ENCODING, help="解码/编码回退，默认 gbk")
    parser.add_argument("--user-functions", default="", help="额外把 f_01c 用户函数参数按选项提取，多个用逗号分隔")
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    source_root, sources = iter_sources(input_path, args.mode)

    if input_path.is_file() and output_path.suffix.lower() == ".json":
        json_root = output_path.parent
        source = sources[0]
        mode = args.mode
        if mode == "auto":
            mode = "bsd" if source.suffix.lower() == ".bsd" else "script"
        rel_name = source.name
        if mode == "bsd":
            entries = extract_workflow_entries_from_bsd(
                source,
                file_name=rel_name,
                user_function_names=parse_user_functions(args.user_functions),
            )
        else:
            entries = extract_workflow_entries_from_script(
                source,
                file_name=rel_name,
                encoding=args.encoding,
                fallback_encoding=args.fallback_encoding,
                user_function_names=parse_user_functions(args.user_functions),
            )
        save_json_entries(output_path, entries)
        print(f"[extract] {rel_name}: {len(entries)} -> {output_path}")
        return

    total = 0
    for source in sources:
        total += extract_one(source, source_root, output_path, args)
    print(f"[extract] 扫描文件：{len(sources)}")
    print(f"[extract] 提取文本：{total}")
    print(f"[extract] 输出目录：{output_path}")


if __name__ == "__main__":
    main()
