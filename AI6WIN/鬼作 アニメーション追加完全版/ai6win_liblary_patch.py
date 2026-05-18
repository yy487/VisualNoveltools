#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AI6WIN liblary.lib 文本查看/等长注入工具

说明：
  liblary.lib 是 AI6WIN 的 LZSS 压缩脚本库。解压后仍是 VM 指令流，文本通常是：
    0x0A/0x0B/0x33 + CP932字符串 + 00
  本工具默认只做“等字节长度注入”，不移动后续 VM 指令位置，安全性最高。

用法：
  # 1) 查看/导出文本
  python ai6win_liblary_patch.py extract liblary.lib liblary.json

  # 2) 修改 liblary.json 里的 msg 后注入；短于原文会用全角空格补齐，长于原文会报错
  python ai6win_liblary_patch.py inject liblary.lib liblary.json liblary.new.lib

  # 3) 只解压/只压缩，用于手工分析
  python ai6win_liblary_patch.py decompress liblary.lib liblary.dec
  python ai6win_liblary_patch.py compress liblary.dec liblary.repack.lib
"""
import argparse
import json
import sys
from pathlib import Path

# 与 ai6win_extract.py 相同的 LZSS 参数
WINDOW_SIZE = 0x1000
WINDOW_MASK = 0xFFF
WINDOW_INIT_POS = 0xFEE
MIN_MATCH = 3
MAX_MATCH = 18

OPS = {
    0x00:'', 0x01:'', 0x02:'', 0x03:'', 0x04:'', 0x05:'',
    0x06:'', 0x07:'', 0x08:'', 0x09:'',
    0x0A:'S',   # STR_PRIMARY
    0x0B:'S',   # STR_SUPPLEMENT
    0x0C:'', 0x0D:'', 0x0E:'', 0x0F:'',
    0x10:'', 0x11:'', 0x12:'', 0x13:'',
    0x14:'>I', 0x15:'>I', 0x16:'>I',
    0x17:'', 0x18:'',
    0x19:'>I', 0x1A:'>I',
    0x1B:'B',
    0x1D:'',
    0x32:'>i',
    0x33:'S',   # PUSH_STR，liblary 里很多系统字符串是这个
    0x34:'', 0x35:'', 0x36:'', 0x37:'', 0x38:'', 0x39:'',
    0x3A:'', 0x3B:'', 0x3C:'', 0x3D:'', 0x3E:'', 0x3F:'',
    0x40:'', 0x41:'', 0x42:'', 0x43:'',
    0xFA:'', 0xFB:'', 0xFC:'', 0xFD:'', 0xFE:'', 0xFF:'',
}
TEXT_OPS = {0x0A, 0x0B}
STRING_OPS = {0x0A, 0x0B, 0x33}


def lzss_decompress(src: bytes, usize: int | None = None) -> bytes:
    """LZSS解压；usize=None 时一直解到压缩流结束。"""
    out = bytearray()
    window = bytearray(b'\x00' * WINDOW_SIZE)
    wp = WINDOW_INIT_POS
    sp = 0
    limit = usize if usize is not None else 1 << 60

    while sp < len(src) and len(out) < limit:
        flags = src[sp]
        sp += 1
        for bit in range(8):
            if sp >= len(src) or len(out) >= limit:
                break
            if flags & (1 << bit):
                b = src[sp]
                sp += 1
                out.append(b)
                window[wp] = b
                wp = (wp + 1) & WINDOW_MASK
            else:
                if sp + 1 >= len(src):
                    break
                lo = src[sp]
                hi = src[sp + 1]
                sp += 2
                off = lo | ((hi & 0xF0) << 4)
                ml = (hi & 0x0F) + MIN_MATCH
                for k in range(ml):
                    if len(out) >= limit:
                        break
                    b = window[(off + k) & WINDOW_MASK]
                    out.append(b)
                    window[wp] = b
                    wp = (wp + 1) & WINDOW_MASK
    return bytes(out)


def lzss_compress(data: bytes) -> bytes:
    """保守重压缩：全部按 literal 写出。

    这样生成的文件会比原始 LZSS 大，但解压逻辑最稳定：
    每 8 个字节一个 flags=0xFF，后面跟 8 个原始字节。
    对于 liblary.lib 这种小文件，体积增大通常不是问题；如果要放回 ARC，
    让封包器更新 compressed_size 即可。
    """
    out = bytearray()
    pos = 0
    while pos < len(data):
        chunk = data[pos:pos + 8]
        flags = (1 << len(chunk)) - 1
        out.append(flags)
        out += chunk
        pos += len(chunk)
    return bytes(out)


def header_size(data: bytes) -> int:
    if len(data) < 4:
        return 0
    mc = int.from_bytes(data[:4], 'little')
    hs = 4 + mc * 4
    if hs < 4 or hs > len(data):
        return 0
    return hs


def iter_strings(data: bytes, include_push: bool = True):
    pos = header_size(data)
    wanted = STRING_OPS if include_push else TEXT_OPS
    idx = 0
    while pos < len(data):
        op_pos = pos
        op = data[pos]
        if op not in OPS:
            pos += 1
            continue
        fmt = OPS[op]
        pos += 1
        if fmt == '':
            continue
        if fmt == 'B':
            pos += 1
            continue
        if fmt in ('>I', '>i'):
            pos += 4
            continue
        if fmt == 'S':
            end = data.find(b'\x00', pos)
            if end < 0:
                break
            raw = data[pos:end]
            try:
                text = raw.decode('cp932')
            except UnicodeDecodeError:
                text = None
            if op in wanted and text is not None:
                idx += 1
                yield {
                    'id': idx,
                    'op_offset': op_pos,
                    'str_offset': pos,
                    'end_offset': end,
                    'op': f'0x{op:02X}',
                    'byte_len': len(raw),
                    'scr_msg': text,
                    'msg': text,
                }
            pos = end + 1


def cmd_extract(args):
    comp = Path(args.input).read_bytes()
    dec = lzss_decompress(comp, args.usize)
    entries = list(iter_strings(dec, include_push=args.include_push))
    Path(args.output).write_text(json.dumps(entries, ensure_ascii=False, indent=2), encoding='utf-8')
    print(f'[OK] {args.input}: compressed={len(comp)} decompressed={len(dec)} strings={len(entries)} -> {args.output}')
    for e in entries:
        if args.find and args.find in e['scr_msg']:
            print(f"  hit id={e['id']} op={e['op']} str=0x{e['str_offset']:X} len={e['byte_len']} text={e['scr_msg']}")


def cmd_inject(args):
    comp = Path(args.input).read_bytes()
    dec = bytearray(lzss_decompress(comp, args.usize))
    entries = json.loads(Path(args.json).read_text(encoding='utf-8'))
    pad = args.pad.encode('cp932')
    if len(pad) == 0:
        raise SystemExit('[ERROR] pad 不能为空')

    patched = 0
    for e in entries:
        old = e.get('scr_msg', '')
        new = e.get('msg', old)
        if new == old:
            continue
        start = int(e['str_offset'])
        old_len = int(e['byte_len'])
        end = start + old_len
        cur = bytes(dec[start:end])
        try:
            cur_text = cur.decode('cp932')
        except UnicodeDecodeError:
            cur_text = None
        if cur_text != old:
            raise SystemExit(f"[ERROR] 校验失败 id={e.get('id')} offset=0x{start:X}: 当前={cur_text!r}, JSON={old!r}")
        try:
            nb = new.encode('cp932')
        except UnicodeEncodeError as ex:
            raise SystemExit(f"[ERROR] CP932不可编码 id={e.get('id')} text={new!r}: {ex}")
        if len(nb) > old_len:
            raise SystemExit(f"[ERROR] 新文本过长 id={e.get('id')}: {len(nb)} > {old_len}。当前工具默认只支持等长/短改。")
        while len(nb) < old_len:
            need = old_len - len(nb)
            if need >= len(pad):
                nb += pad
            else:
                # 剩余 1 字节时只能补半角空格，否则会破坏双字节字符
                nb += b' ' * need
        dec[start:end] = nb
        patched += 1

    new_comp = lzss_compress(bytes(dec))
    check = lzss_decompress(new_comp, len(dec))
    if check != bytes(dec):
        raise SystemExit('[ERROR] 内部校验失败：重压缩后解压结果不一致')
    Path(args.output).write_bytes(new_comp)
    print(f'[OK] patched={patched} decompressed={len(dec)} recompressed={len(new_comp)} -> {args.output}')


def cmd_decompress(args):
    comp = Path(args.input).read_bytes()
    dec = lzss_decompress(comp, args.usize)
    Path(args.output).write_bytes(dec)
    print(f'[OK] {len(comp)} -> {len(dec)} bytes: {args.output}')


def cmd_compress(args):
    dec = Path(args.input).read_bytes()
    comp = lzss_compress(dec)
    if lzss_decompress(comp, len(dec)) != dec:
        raise SystemExit('[ERROR] 内部校验失败：重压缩后解压结果不一致')
    Path(args.output).write_bytes(comp)
    print(f'[OK] {len(dec)} -> {len(comp)} bytes: {args.output}')


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)

    p = sub.add_parser('extract')
    p.add_argument('input')
    p.add_argument('output')
    p.add_argument('--usize', type=int, default=None, help='已知解压大小；liblary.lib 可省略')
    p.add_argument('--no-push', dest='include_push', action='store_false', help='不导出 0x33 PUSH_STR')
    p.add_argument('--find', default='')
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser('inject')
    p.add_argument('input')
    p.add_argument('json')
    p.add_argument('output')
    p.add_argument('--usize', type=int, default=None)
    p.add_argument('--pad', default='　', help='短文本补齐字符，默认全角空格')
    p.set_defaults(func=cmd_inject)

    p = sub.add_parser('decompress')
    p.add_argument('input')
    p.add_argument('output')
    p.add_argument('--usize', type=int, default=None)
    p.set_defaults(func=cmd_decompress)

    p = sub.add_parser('compress')
    p.add_argument('input')
    p.add_argument('output')
    p.set_defaults(func=cmd_compress)

    args = ap.parse_args()
    args.func(args)


if __name__ == '__main__':
    main()
