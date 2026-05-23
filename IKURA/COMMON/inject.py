# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from collections import defaultdict

from common import ISFFile, load_json_entries


def load_json_dir(json_path: Path) -> dict[str, list[dict]]:
    by_file: dict[str, list[dict]] = defaultdict(list)
    paths = [json_path] if json_path.is_file() else sorted(json_path.rglob("*.json"))
    for jp in paths:
        entries = load_json_entries(jp)
        for e in entries:
            f = e.get("_file")
            if isinstance(f, str):
                by_file[f].append(e)
            elif json_path.is_file():
                # 单文件 JSON 缺 _file 时，按 JSON 文件名回退；正常不应发生。
                by_file[jp.stem].append(e)
    return by_file


def main() -> None:
    ap = argparse.ArgumentParser(description="从项目统一 JSON 注入回 IKURA/ISF 脚本")
    ap.add_argument("input", help="原始 ISF/SNR 文件或目录")
    ap.add_argument("json", help="翻译 JSON 文件或目录")
    ap.add_argument("output", help="输出 ISF/SNR 文件或目录")
    ap.add_argument("--engine", choices=["MPX", "DRS"], default="MPX", help="脚本内部文本编码/结构分支，默认 MPX")
    ap.add_argument("--non-strict", action="store_true", help="scr_msg 校验失败时仍尝试写入，不建议常规使用")
    args = ap.parse_args()

    input_path = Path(args.input)
    json_path = Path(args.json)
    output_path = Path(args.output)
    by_file = load_json_dir(json_path)

    patched_total = failed_total = 0
    if input_path.is_file():
        candidates = list(by_file.values())
        entries = candidates[0] if candidates else []
        isf = ISFFile(engine=args.engine)
        isf.load_path(input_path)
        stats = isf.apply_workflow_entries(entries, strict=not args.non_strict)
        isf.save_path(output_path)
        patched_total += stats["patched"]
        failed_total += stats["failed"]
        print(f"[inject] {input_path.name}: {stats}")
    else:
        output_path.mkdir(parents=True, exist_ok=True)
        for src in sorted(input_path.rglob("*")):
            rel = src.relative_to(input_path).as_posix()
            dst = output_path / rel
            if not src.is_file():
                continue
            if src.suffix.lower() not in {".isf", ".snr"}:
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dst)
                continue
            entries = by_file.get(rel, [])
            if not entries:
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dst)
                continue
            try:
                isf = ISFFile(engine=args.engine)
                isf.load_path(src)
                stats = isf.apply_workflow_entries(entries, strict=not args.non_strict)
                isf.save_path(dst)
                patched_total += stats["patched"]
                failed_total += stats["failed"]
                print(f"[inject] {rel}: {stats}")
            except Exception as e:
                failed_total += len(entries)
                print(f"[inject][warn] {rel}: {e}")
    print(f"[inject] patched={patched_total}")
    print(f"[inject] failed={failed_total}")
    print(f"[inject] output={output_path}")


if __name__ == "__main__":
    main()
