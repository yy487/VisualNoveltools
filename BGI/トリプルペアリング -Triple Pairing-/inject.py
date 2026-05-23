# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from common import (
    DEFAULT_ENCODING,
    DEFAULT_FALLBACK_ENCODING,
    iter_sources,
    json_path_for_source,
    normalize_rel_path,
    output_path_for_source,
    parse_user_functions,
    patch_bsd_with_workflow_json,
    patch_script_with_workflow_json,
)


def json_for_input(source: Path, source_root: Path | None, json_root: Path) -> Path:
    if json_root.is_file():
        return json_root
    return json_path_for_source(source, source_root, json_root)


def inject_one(source: Path, source_root: Path | None, json_root: Path, output_root: Path, args) -> dict[str, int]:
    rel_name = normalize_rel_path(source, source_root)
    mode = args.mode
    if mode == "auto":
        mode = "bsd" if source.suffix.lower() == ".bsd" else "script"

    input_json = json_for_input(source, source_root, json_root)
    output_path = output_path_for_source(source, source_root, output_root)
    if not input_json.exists():
        print(f"[inject][warn] 找不到 JSON，跳过: {rel_name} -> {input_json}")
        return {"entries": 0, "json_items": 0, "patched_message": 0, "patched_name": 0, "skipped": 0, "failed": 1, "warnings": 1}

    if mode == "bsd":
        stats = patch_bsd_with_workflow_json(
            source,
            input_json,
            output_path,
            file_name=rel_name,
            user_function_names=parse_user_functions(args.user_functions),
            strict=args.strict,
        )
    else:
        stats = patch_script_with_workflow_json(
            source,
            input_json,
            output_path,
            file_name=rel_name,
            encoding=args.encoding,
            fallback_encoding=args.fallback_encoding,
            user_function_names=parse_user_functions(args.user_functions),
            strict=args.strict,
        )
    print(
        f"[inject] {rel_name}: msg={stats['patched_message']} name={stats['patched_name']} "
        f"skip={stats['skipped']} fail={stats['failed']} -> {output_path}"
    )
    return stats


def main() -> None:
    parser = argparse.ArgumentParser(description="BGI V1 工作流 JSON 注入工具")
    parser.add_argument("input", help="原始 .bsd、编译脚本文件或目录")
    parser.add_argument("json", help="翻译 JSON 文件或 JSON 目录")
    parser.add_argument("output", help="输出文件或目录")
    parser.add_argument("--mode", choices=["auto", "bsd", "script"], default="auto", help="输入类型，默认 auto")
    parser.add_argument("--encoding", default=DEFAULT_ENCODING, help="脚本文本主编码，默认 shift_jis")
    parser.add_argument("--fallback-encoding", default=DEFAULT_FALLBACK_ENCODING, help="解码/编码回退，默认 gbk")
    parser.add_argument("--user-functions", default="", help="额外把 f_01c 用户函数参数按选项提取，多个用逗号分隔")
    parser.add_argument("--strict", action="store_true", help="定位失败时直接中断")
    args = parser.parse_args()

    input_path = Path(args.input)
    json_path = Path(args.json)
    output_path = Path(args.output)
    source_root, sources = iter_sources(input_path, args.mode)

    if input_path.is_file() and not output_path.is_dir():
        source = sources[0]
        mode = args.mode
        if mode == "auto":
            mode = "bsd" if source.suffix.lower() == ".bsd" else "script"
        rel_name = source.name
        input_json = json_path
        if mode == "bsd":
            stats = patch_bsd_with_workflow_json(
                source,
                input_json,
                output_path,
                file_name=rel_name,
                user_function_names=parse_user_functions(args.user_functions),
                strict=args.strict,
            )
        else:
            stats = patch_script_with_workflow_json(
                source,
                input_json,
                output_path,
                file_name=rel_name,
                encoding=args.encoding,
                fallback_encoding=args.fallback_encoding,
                user_function_names=parse_user_functions(args.user_functions),
                strict=args.strict,
            )
        print(f"[inject] {rel_name}: {stats}")
        return

    totals = {"patched_message": 0, "patched_name": 0, "skipped": 0, "failed": 0, "warnings": 0}
    for source in sources:
        stats = inject_one(source, source_root, json_path, output_path, args)
        for key in totals:
            totals[key] += int(stats.get(key, 0))
    print(f"[inject] 扫描文件：{len(sources)}")
    print(f"[inject] 成功注入 message：{totals['patched_message']}")
    print(f"[inject] 成功注入 name：{totals['patched_name']}")
    print(f"[inject] 跳过未修改：{totals['skipped']}")
    print(f"[inject] 失败：{totals['failed']}")
    print(f"[inject] warning：{totals['warnings']}")


if __name__ == "__main__":
    main()
