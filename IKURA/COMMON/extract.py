# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path

from common import ISFFile, save_json_entries


def iter_isf_files(input_path: Path):
    if input_path.is_file():
        yield input_path, input_path.name
    else:
        for p in sorted(input_path.rglob("*")):
            if p.is_file() and p.suffix.lower() in {".isf", ".snr"}:
                yield p, p.relative_to(input_path).as_posix()


def extract_one(path: Path, rel_name: str, out_dir: Path, engine: str, include_ui: bool) -> int:
    isf = ISFFile(engine=engine)
    isf.load_path(path)
    entries = isf.build_workflow_entries(rel_name, include_ui=include_ui)
    if entries:
        out_name = rel_name.replace("/", "__") + ".json"
        save_json_entries(out_dir / out_name, entries)
    return len(entries)


def main() -> None:
    ap = argparse.ArgumentParser(description="提取 IKURA/ISF 脚本文本为项目统一 JSON")
    ap.add_argument("input", help="输入 ISF/SNR 文件或目录")
    ap.add_argument("output", help="输出 JSON 目录")
    ap.add_argument("--engine", choices=["MPX", "DRS"], default="MPX", help="脚本内部文本编码/结构分支，默认 MPX")
    ap.add_argument("--no-ui", action="store_true", help="不导出 ui/system/choice 以外的非剧情文本")
    args = ap.parse_args()

    input_path = Path(args.input)
    out_dir = Path(args.output)
    out_dir.mkdir(parents=True, exist_ok=True)

    scanned = 0
    total = 0
    for path, rel_name in iter_isf_files(input_path):
        scanned += 1
        try:
            count = extract_one(path, rel_name, out_dir, args.engine, include_ui=not args.no_ui)
            total += count
            print(f"[extract] {rel_name}: {count}")
        except Exception as e:
            print(f"[extract][warn] {rel_name}: {e}")
    print(f"[extract] scanned_files={scanned}")
    print(f"[extract] extracted_entries={total}")
    print(f"[extract] output={out_dir}")


if __name__ == "__main__":
    main()
