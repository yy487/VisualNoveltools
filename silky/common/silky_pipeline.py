"""silky_pipeline.py — Silky MES 项目式流水线。

工作流：
  unpack: *.MES -> work/op/*.op.txt + work/json/*.json
  pack:   work/op/*.op.txt + work/json/*.json -> out/*.MES

JSON 条目格式遵循本项目惯例：可选 name、scr_msg、message；scr_msg 保留原文，翻译只改 message。
"""

from __future__ import annotations

import argparse
import glob
import os
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed
from typing import List, Tuple

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def _strip_ext(name: str, exts: List[str]) -> str:
    for ext in exts:
        if name.lower().endswith(ext.lower()):
            return name[:-len(ext)]
    return os.path.splitext(name)[0]


def _default_workers() -> int:
    n = os.cpu_count() or 4
    return max(1, n - 1)


def _worker_unpack(task: Tuple[str, str, str, str]):
    mes_path, op_path, json_path, encoding = task
    import silky_extract
    import silky_op
    try:
        sm = silky_op.SilkyMesScript(mes_path, op_path, encoding=encoding)
        sm.disassemble()
        n = silky_extract.extract_text(op_path, json_path, file_name=os.path.basename(mes_path))
        return os.path.basename(mes_path), n, None
    except Exception as e:  # noqa: BLE001
        return os.path.basename(mes_path), 0, repr(e)


def _worker_pack(task: Tuple[str, str, str, str, str, str, bool]):
    base, op_path, json_path, op2_path, out_mes, encoding, strict = task
    import silky_inject
    import silky_op
    try:
        stat = silky_inject.import_text(op_path, json_path, op2_path, strict=strict)
    except Exception as e:  # noqa: BLE001
        return base, 0, f'inject: {e!r}'
    try:
        sm = silky_op.SilkyMesScript(out_mes, op2_path, encoding=encoding)
        sm.assemble()
    except Exception as e:  # noqa: BLE001
        return base, int(stat.get('patched', 0)), f'asm: {e!r}'
    return base, int(stat.get('patched', 0)), None


def cmd_unpack(args) -> int:
    op_dir = os.path.join(args.workdir, 'op')
    json_dir = os.path.join(args.workdir, 'json')
    os.makedirs(op_dir, exist_ok=True)
    os.makedirs(json_dir, exist_ok=True)

    files = sorted(glob.glob(os.path.join(args.mes_dir, '*.MES')) + glob.glob(os.path.join(args.mes_dir, '*.mes')))
    # Windows 文件系统大小写不敏感；Linux 测试环境可能重复，这里去重。
    files = sorted(dict.fromkeys(files))
    if not files:
        print(f'[!] {args.mes_dir} 下没有 .MES 文件')
        return 1

    workers = args.jobs or _default_workers()
    tasks = []
    for mes in files:
        base = _strip_ext(os.path.basename(mes), ['.MES', '.mes'])
        tasks.append((
            mes,
            os.path.join(op_dir, base + '.op.txt'),
            os.path.join(json_dir, base + '.json'),
            args.encoding,
        ))

    total = 0
    failed = []
    print(f'[unpack] {len(tasks)} 个 MES / {workers} 进程')
    if workers == 1:
        results = [_worker_unpack(t) for t in tasks]
    else:
        with ProcessPoolExecutor(max_workers=workers) as ex:
            results = []
            for fut in as_completed([ex.submit(_worker_unpack, t) for t in tasks]):
                results.append(fut.result())

    for name, n, err in results:
        if err:
            failed.append((name, err))
            print(f'  [!] {name}: {err}')
        else:
            total += n
            print(f'  [+] {name}: {n} entries')

    print(f'[unpack] 完成 {len(tasks) - len(failed)}/{len(tasks)}，共 {total} 条')
    print(f'[unpack] op:   {op_dir}')
    print(f'[unpack] json: {json_dir}')
    return 0 if not failed else 2


def cmd_pack(args) -> int:
    op_dir = os.path.join(args.workdir, 'op')
    json_dir = os.path.join(args.workdir, 'json')
    op2_dir = os.path.join(args.workdir, 'op_injected')
    os.makedirs(op2_dir, exist_ok=True)
    os.makedirs(args.output_dir, exist_ok=True)

    if not os.path.isdir(op_dir):
        print(f'[!] {op_dir} 不存在，请先 unpack')
        return 1
    if not os.path.isdir(json_dir):
        print(f'[!] {json_dir} 不存在，请先 unpack')
        return 1

    op_files = sorted(glob.glob(os.path.join(op_dir, '*.op.txt')))
    if not op_files:
        print(f'[!] {op_dir} 下没有 .op.txt')
        return 1

    tasks = []
    missing = []
    for op_path in op_files:
        base = _strip_ext(os.path.basename(op_path), ['.op.txt'])
        json_path = os.path.join(json_dir, base + '.json')
        if not os.path.isfile(json_path):
            missing.append(base)
            continue
        tasks.append((
            base,
            op_path,
            json_path,
            os.path.join(op2_dir, base + '.op.txt'),
            os.path.join(args.output_dir, base + '.MES'),
            args.encoding,
            args.strict,
        ))

    workers = args.jobs or _default_workers()
    total = 0
    failed = []
    print(f'[pack] {len(tasks)} 个 op/json / {workers} 进程')
    if workers == 1:
        results = [_worker_pack(t) for t in tasks]
    else:
        with ProcessPoolExecutor(max_workers=workers) as ex:
            results = []
            for fut in as_completed([ex.submit(_worker_pack, t) for t in tasks]):
                results.append(fut.result())

    for base, n, err in results:
        if err:
            failed.append((base, err))
            print(f'  [!] {base}: {err}')
        else:
            total += n
            print(f'  [+] {base}: patched={n}')

    if missing:
        print(f'[!] 缺 JSON {len(missing)} 个: {missing[:5]}')
    print(f'[pack] 完成 {len(tasks) - len(failed)}/{len(tasks)}，共注入 {total} 条')
    print(f'[pack] 输出 MES: {args.output_dir}')
    return 0 if not failed and not missing else 2


def main() -> int:
    ap = argparse.ArgumentParser(description='Silky MES 项目式提取/注入流水线')
    sub = ap.add_subparsers(dest='cmd', required=True)

    p_u = sub.add_parser('unpack', help='MES目录 -> work/op + work/json')
    p_u.add_argument('mes_dir')
    p_u.add_argument('workdir')
    p_u.add_argument('--encoding', default='cp932')
    p_u.add_argument('-j', '--jobs', type=int, default=0, help='并行进程数，1=单进程')
    p_u.set_defaults(func=cmd_unpack)

    p_p = sub.add_parser('pack', help='work/op + work/json -> 输出目录/*.MES')
    p_p.add_argument('workdir')
    p_p.add_argument('output_dir')
    p_p.add_argument('--encoding', default='cp932')
    p_p.add_argument('--strict', action='store_true', help='scr_msg 校验失败时跳过该条')
    p_p.add_argument('-j', '--jobs', type=int, default=0, help='并行进程数，1=单进程')
    p_p.set_defaults(func=cmd_pack)

    args = ap.parse_args()
    return args.func(args)


if __name__ == '__main__':
    raise SystemExit(main())
