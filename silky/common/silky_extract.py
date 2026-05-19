"""silky_extract.py — Silky op.txt -> 项目 JSON 提取。

输出 JSON 条目格式：
  {
    "_file": "xxx.MES",
    "_index": 0,
    "name": "角色名",      # 可选
    "scr_msg": "原文",     # 原始脚本文本，不要修改
    "message": "原文"      # 翻译/修改只改这里
  }
"""

from __future__ import annotations

import argparse
import glob
import os
from typing import List

from silky_common import make_json_entries, save_json_entries


def extract_text(opcode_txt_path: str, json_path: str, file_name: str = '') -> int:
    """从 op.txt 提取文本并写出 JSON，返回条目数。"""
    with open(opcode_txt_path, 'r', encoding='utf-8-sig') as f:
        lines = f.readlines()
    if not file_name:
        base = os.path.basename(opcode_txt_path)
        file_name = base[:-7] + '.MES' if base.lower().endswith('.op.txt') else base
    entries = make_json_entries(lines, file_name=file_name)
    save_json_entries(json_path, entries)
    return len(entries)


def _strip_ext(name: str, exts: List[str]) -> str:
    for ext in exts:
        if name.lower().endswith(ext.lower()):
            return name[:-len(ext)]
    return os.path.splitext(name)[0]


def main() -> int:
    ap = argparse.ArgumentParser(description='Silky op.txt -> JSON 文本提取')
    ap.add_argument('input', help='单个 *.op.txt 文件，或包含 *.op.txt 的目录')
    ap.add_argument('output', help='单个 JSON 输出路径，或输出目录')
    ap.add_argument('--pattern', default='*.op.txt', help='目录模式 glob，默认 *.op.txt')
    args = ap.parse_args()

    if os.path.isdir(args.input):
        os.makedirs(args.output, exist_ok=True)
        files = sorted(glob.glob(os.path.join(args.input, args.pattern)))
        total = 0
        print(f'[extract] {len(files)} 个 op.txt -> {args.output}')
        for path in files:
            base = _strip_ext(os.path.basename(path), ['.op.txt'])
            out = os.path.join(args.output, base + '.json')
            n = extract_text(path, out, file_name=base + '.MES')
            total += n
            print(f'  [+] {base}: {n} entries')
        print(f'[extract] 完成，共 {total} 条')
        return 0

    n = extract_text(args.input, args.output)
    print(f'[+] extracted {n} entries: {args.output}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
