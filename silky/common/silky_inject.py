"""silky_inject.py — Silky JSON + op.txt -> 新 op.txt 注入。

注入定位策略：
  1. 优先按 JSON 里的 _index 对应提取顺序定位；
  2. 注入前用 scr_msg 校验原文；
  3. 校验不一致时给 warning，默认仍按 _index 注入，便于旧 JSON 小改后继续使用。

只修改 message；scr_msg 用作定位与校验依据，不应修改。旧版 msg 仍作为兼容 fallback。
"""

from __future__ import annotations

import argparse
import glob
import os
from typing import Dict, List, Tuple

from silky_common import (
    apply_translation_to_block,
    build_translation_map,
    join_parts,
    iter_text_blocks,
    load_json_entries,
)


def import_text(opcode_txt_path: str, json_path: str, output_txt_path: str, *, strict: bool = False) -> Dict[str, object]:
    """把 JSON 的 message 注入 op.txt，返回统计信息。"""
    entries = load_json_entries(json_path)
    trans_map = build_translation_map(entries)

    with open(opcode_txt_path, 'r', encoding='utf-8-sig') as f:
        lines = f.readlines()

    patched = 0
    warnings: List[str] = []
    blocks = list(iter_text_blocks(lines))

    for idx, block in enumerate(blocks):
        item = trans_map.get(idx)
        if item is None:
            continue

        old_scr = join_parts(block.parts)
        json_scr = str(item.get('scr_msg', ''))
        if json_scr and json_scr != old_scr:
            msg = f'index={idx}: scr_msg mismatch, json={json_scr!r}, op={old_scr!r}'
            warnings.append(msg)
            if strict:
                continue

        new_msg = str(item.get('message', item.get('msg', json_scr if json_scr else old_scr)))
        apply_translation_to_block(lines, block, new_msg)
        patched += 1

    os.makedirs(os.path.dirname(os.path.abspath(output_txt_path)), exist_ok=True)
    with open(output_txt_path, 'w', encoding='utf-8-sig') as out:
        out.writelines(lines)

    return {
        'json_entries': len(entries),
        'op_blocks': len(blocks),
        'patched': patched,
        'warnings': warnings,
    }


def _strip_ext(name: str, exts: List[str]) -> str:
    for ext in exts:
        if name.lower().endswith(ext.lower()):
            return name[:-len(ext)]
    return os.path.splitext(name)[0]


def main() -> int:
    ap = argparse.ArgumentParser(description='Silky JSON + op.txt -> 新 op.txt 注入')
    ap.add_argument('op_txt', help='原始 *.op.txt 单文件或目录')
    ap.add_argument('json', help='翻译 JSON 单文件或目录')
    ap.add_argument('output_op_txt', help='输出 *.op.txt 单文件或目录')
    ap.add_argument('--pattern', default='*.op.txt', help='目录模式 op glob，默认 *.op.txt')
    ap.add_argument('--strict', action='store_true', help='scr_msg 不一致时跳过该条')
    args = ap.parse_args()

    if os.path.isdir(args.op_txt):
        if not os.path.isdir(args.json):
            raise SystemExit('批处理模式下 json 参数也必须是目录')
        os.makedirs(args.output_op_txt, exist_ok=True)
        files = sorted(glob.glob(os.path.join(args.op_txt, args.pattern)))
        total = 0
        failed: List[Tuple[str, str]] = []
        print(f'[inject] {len(files)} 个 op.txt -> {args.output_op_txt}')
        for path in files:
            base = _strip_ext(os.path.basename(path), ['.op.txt'])
            json_path = os.path.join(args.json, base + '.json')
            if not os.path.isfile(json_path):
                failed.append((base, 'missing json'))
                continue
            out_path = os.path.join(args.output_op_txt, base + '.op.txt')
            stat = import_text(path, json_path, out_path, strict=args.strict)
            total += int(stat['patched'])
            print(f'  [+] {base}: patched={stat["patched"]}, warnings={len(stat["warnings"])}')
            for w in stat['warnings'][:3]:
                print(f'      [warn] {w}')
        if failed:
            print(f'[!] 失败/缺失 {len(failed)} 个: {failed[:5]}')
        print(f'[inject] 完成，共注入 {total} 条')
        return 0 if not failed else 2

    stat = import_text(args.op_txt, args.json, args.output_op_txt, strict=args.strict)
    print(f'[+] patched={stat["patched"]}, warnings={len(stat["warnings"])} -> {args.output_op_txt}')
    for w in stat['warnings'][:10]:
        print(f'  [warn] {w}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
