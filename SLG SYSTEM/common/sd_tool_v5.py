#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
sd_tool_v5.py

SLG System .sd 文本提取/原地注入工具。

相对 sd_tool_v4 的变化：
  1. 兼容三国姫5 的新文本头：
       [0F xx xx xx xx][A:u32][B:u32][name_cp932\x00][01 00 00 00][type:01/02/03][text_cp932][END]
     旧 v4 会从最后一个 00 + type 处误判文本头，因此 name 全部丢失，并漏掉大量对话。

  2. 行分隔符改为按实际字节保存：
       00 00 00 00 + N 个 FF + 可选 0A
     三国姫5 样本中常见的是：
       00 00 00 00 FF FF FF FF FF FF FF FF 0A
     注入时不再写死 12/13 字节，而是使用 meta 中记录的 sep_hex 原样还原。

  3. JSON 兼容旧字段 pre_jp，同时增加标准字段 scr_msg/message。

用法：
  提取: python sd_tool_v5.py extract <sd_dir> <out_dir>
  注入: python sd_tool_v5.py inject  <sd_dir> <json_dir> <out_dir>
"""

from __future__ import annotations

import os
import sys
import json
import struct
from pathlib import Path
from typing import Optional

ENC = 'cp932'
FWSP = b'\x81\x40'

# 旧版固定 END：4 个 00 + 8 个 FF。v5 仍以它作为最常见模式，
# 但扫描时允许 FF 数量变化，注入时保存实际 separator bytes。
END_MIN_PREFIX = b'\x00\x00\x00\x00' + b'\xff' * 4
DEFAULT_END_MARK = b'\x00\x00\x00\x00' + b'\xff' * 8


def is_valid_jp(text: str) -> bool:
    """有效日文文本：含日文/全角字符，不含私用区字符，不含控制字符。"""
    has_jp = any(0x3000 < ord(c) <= 0x9FFF or 0xFF00 <= ord(c) <= 0xFFEF for c in text)
    has_private = any(0xE000 <= ord(c) <= 0xF8FF for c in text)
    has_ctrl = any(ord(c) < 0x20 for c in text)
    return has_jp and not has_private and not has_ctrl


def decode_cp932(raw: bytes) -> Optional[str]:
    try:
        return raw.decode(ENC)
    except Exception:
        return None


def find_end_marker(data: bytes, start: int, max_line_len: int = 400) -> Optional[tuple[int, bytes]]:
    """从 start 往后找文本结束标记，返回 (marker_offset, marker_bytes)。

    标记形态：00 00 00 00 + 至少 4 个 FF，最多 32 个 FF。
    常见为 4 个 00 + 8 个 FF。
    """
    search_end = min(len(data), start + max_line_len + 64)
    pos = data.find(END_MIN_PREFIX, start, search_end)
    while pos != -1:
        if pos - start <= 0 or pos - start > max_line_len:
            pos = data.find(END_MIN_PREFIX, pos + 1, search_end)
            continue
        p = pos + 4
        ff_count = 0
        while p < len(data) and data[p] == 0xFF and ff_count < 32:
            p += 1
            ff_count += 1
        if ff_count >= 4:
            return pos, data[pos:p]
        pos = data.find(END_MIN_PREFIX, pos + 1, search_end)
    return None


def collect_text_lines(data: bytes, text_start: int) -> Optional[dict]:
    """读取一条可能的多行文本。

    返回：
      full_text, line_lengths, sep_bytes, text_end, text_len_without_final_end

    注意：text_len 不包含最后一行后的 END_MARK；只包含正文和行间 separator。
    """
    first = find_end_marker(data, text_start)
    if not first:
        return None
    em_pos, marker = first
    raw = data[text_start:em_pos]
    line_text = decode_cp932(raw)
    if line_text is None or not is_valid_jp(line_text):
        return None

    lines: list[tuple[int, int, str]] = [(text_start, em_pos - text_start, line_text)]
    sep_bytes: list[bytes] = []
    cur_end = em_pos
    cur_marker = marker

    while True:
        after_marker = cur_end + len(cur_marker)
        # 三国姫5 的行间分隔：END_MARK + 0A。
        # 没有 0A 时视为当前文本结束，避免把下一条 opcode 后文本误合并。
        if after_marker >= len(data) or data[after_marker] != 0x0A:
            break
        sep = data[cur_end:after_marker + 1]
        next_start = after_marker + 1
        nxt = find_end_marker(data, next_start)
        if not nxt:
            break
        next_em, next_marker = nxt
        if next_em - next_start <= 0 or next_em - next_start > 400:
            break
        lt = decode_cp932(data[next_start:next_em])
        if lt is None or not is_valid_jp(lt):
            break
        sep_bytes.append(sep)
        lines.append((next_start, next_em - next_start, lt))
        cur_end = next_em
        cur_marker = next_marker

    return {
        'full_text': ''.join(t for _, _, t in lines),
        'line_lengths': [ln for _, ln, _ in lines],
        'sep_bytes': sep_bytes,
        'text_end': lines[-1][0] + lines[-1][1],
        'text_len': sum(ln for _, ln, _ in lines) + sum(len(s) for s in sep_bytes),
    }


def find_v5_header(data: bytes, flag_pos: int) -> Optional[dict]:
    """解析三国姫5 风格文本头。

    flag_pos 指向 01 00 00 00；其前一个字节必须是 name 的 NUL 结尾。
    往前在短距离内寻找 0F xx xx xx xx + A:u32 + B:u32。
    """
    if flag_pos <= 0 or data[flag_pos - 1] != 0x00:
        return None

    # 从近到远寻找 0F opcode。name 通常很短，但保守放宽到 128 字节。
    for op4_off in range(flag_pos - 1, max(-1, flag_pos - 160), -1):
        if data[op4_off] != 0x0F:
            continue
        name_start = op4_off + 5 + 8
        name_end = flag_pos - 1
        if name_start > name_end:
            continue
        name_bytes = data[name_start:name_end]
        if len(name_bytes) > 80 or b'\x00' in name_bytes:
            continue
        if op4_off + 13 > len(data):
            continue
        try:
            a_val = struct.unpack_from('<I', data, op4_off + 5)[0]
            b_val = struct.unpack_from('<I', data, op4_off + 9)[0]
        except Exception:
            continue
        # 只做宽松约束，避免把大一点的脚本编号误过滤。
        if a_val > 0x2000000 or b_val > 0x2000000:
            continue
        name = None
        if name_bytes:
            name = decode_cp932(name_bytes)
            if name is None or any(ord(c) < 0x20 for c in name):
                continue
        return {
            'op4_off': op4_off,
            'name_off': name_start,
            'name_len': len(name_bytes),
            'name': name,
            'a_val': a_val,
            'b_val': b_val,
        }
    return None


def scan_entries_v5(data: bytes) -> list[dict]:
    """扫描三国姫5 风格：[name\0][01 00 00 00][type][text]。"""
    entries: list[dict] = []
    seen: set[int] = set()
    i = 0
    length = len(data)

    while True:
        flag_pos = data.find(b'\x01\x00\x00\x00', i)
        if flag_pos < 0 or flag_pos + 5 >= length:
            break
        prefix = data[flag_pos + 4]
        if prefix not in (0x01, 0x02, 0x03):
            i = flag_pos + 1
            continue
        text_off = flag_pos + 5
        if text_off in seen:
            i = flag_pos + 1
            continue
        lines = collect_text_lines(data, text_off)
        if not lines:
            i = flag_pos + 1
            continue
        header = find_v5_header(data, flag_pos)
        if not header:
            i = flag_pos + 1
            continue

        seen.add(text_off)
        name = header['name']
        entries.append({
            'kind': 'dialog' if name else 'narr',
            'header_style': 'v5',
            'prefix': prefix,
            'op4_off': header['op4_off'],
            'text_off': text_off,
            'text_end': lines['text_end'],
            'text_len': lines['text_len'],
            'line_lengths': lines['line_lengths'],
            'sep_bytes': lines['sep_bytes'],
            'sep_sizes': [len(s) for s in lines['sep_bytes']],
            'full_text': lines['full_text'],
            'name': name,
            'name_off': header['name_off'],
            'name_len': header['name_len'],
            'a_val': header['a_val'],
            'b_val': header['b_val'],
        })
        i = flag_pos + 1

    entries.sort(key=lambda e: e['text_off'])
    return entries


def scan_entries_legacy(data: bytes, skip_offsets: set[int] | None = None) -> list[dict]:
    """兼容旧 v4 风格：[name\0][prefix:01/02/03][text]。"""
    skip_offsets = skip_offsets or set()
    entries: list[dict] = []
    seen: set[int] = set()
    i = 0
    length = len(data)

    while i < length - 15:
        if not (data[i] == 0x00 and data[i + 1] in (0x01, 0x02, 0x03)):
            i += 1
            continue
        prefix_pos = i
        prefix = data[i + 1]
        text_off = i + 2
        if text_off in skip_offsets or text_off in seen:
            i += 1
            continue
        lines = collect_text_lines(data, text_off)
        if not lines:
            i += 1
            continue

        null_pos = prefix_pos
        k = null_pos - 1
        while k >= 0 and data[k] != 0x00:
            k -= 1
        name_start = k + 1
        name_bytes = bytes(data[name_start:null_pos])
        name_len = len(name_bytes)

        if name_start < 12:
            i += 1
            continue
        op4_off = name_start - 12
        try:
            a_val = struct.unpack_from('<I', data, op4_off + 4)[0]
        except Exception:
            i += 1
            continue
        if a_val >= 100000:
            i += 1
            continue

        if name_len > 0:
            name = decode_cp932(name_bytes)
            if name is None or any(ord(c) < 0x20 for c in name):
                i += 1
                continue
            kind = 'dialog'
        else:
            name = None
            kind = 'narr'

        seen.add(text_off)
        entries.append({
            'kind': kind,
            'header_style': 'legacy',
            'prefix': prefix,
            'op4_off': op4_off,
            'text_off': text_off,
            'text_end': lines['text_end'],
            'text_len': lines['text_len'],
            'line_lengths': lines['line_lengths'],
            'sep_bytes': lines['sep_bytes'],
            'sep_sizes': [len(s) for s in lines['sep_bytes']],
            'full_text': lines['full_text'],
            'name': name,
            'name_off': name_start,
            'name_len': name_len,
        })
        i += 1

    entries.sort(key=lambda e: e['text_off'])
    return entries


def scan_entries(data: bytes) -> list[dict]:
    """自动扫描 v5 + legacy，两者冲突时优先 v5。"""
    v5_entries = scan_entries_v5(data)
    used = {e['text_off'] for e in v5_entries}
    legacy = scan_entries_legacy(data, used)
    entries = v5_entries + legacy
    entries.sort(key=lambda e: e['text_off'])
    return entries


def cp932_fit_line(text: str, capacity: int, label: str) -> bytes:
    """将文本编码为 CP932，按字符边界截断到 capacity 字节，不足补全角空格。"""
    try:
        encoded = text.encode(ENC)
    except UnicodeEncodeError as e:
        raise ValueError(f'{label} CP932编码失败: {e}')

    if len(encoded) > capacity:
        buf = bytearray()
        for ch in text:
            try:
                cb = ch.encode(ENC)
            except UnicodeEncodeError:
                cb = b'?'
            if len(buf) + len(cb) > capacity:
                break
            buf += cb
        encoded = bytes(buf)

    result = bytearray(encoded)
    while len(result) < capacity:
        if capacity - len(result) >= 2:
            result += FWSP
        else:
            result += b'\x20'
    return bytes(result[:capacity])


def cp932_multiline(text: str, line_lengths: list[int], sep_bytes: list[bytes], label: str) -> bytes:
    """按原始行容量分配翻译文本，并原样插入行间 separator bytes。"""
    try:
        encoded = text.encode(ENC)
    except UnicodeEncodeError as e:
        raise ValueError(f'{label} CP932编码失败: {e}')

    result = bytearray()
    pos = 0
    for idx, capacity in enumerate(line_lengths):
        line_buf = bytearray()
        i = pos
        while i < len(encoded) and len(line_buf) < capacity:
            b = encoded[i]
            if 0x81 <= b <= 0x9F or 0xE0 <= b <= 0xFC:
                if len(line_buf) + 2 <= capacity and i + 1 < len(encoded):
                    line_buf.append(encoded[i])
                    line_buf.append(encoded[i + 1])
                    i += 2
                else:
                    break
            else:
                line_buf.append(b)
                i += 1
        pos = i

        while len(line_buf) < capacity:
            if capacity - len(line_buf) >= 2:
                line_buf += FWSP
            else:
                line_buf += b'\x20'

        result += line_buf
        if idx < len(line_lengths) - 1:
            if idx < len(sep_bytes):
                result += sep_bytes[idx]
            else:
                result += DEFAULT_END_MARK + b'\x0A'
    return bytes(result)


def extract_file(sd_path: str, json_path: str) -> int:
    data = open(sd_path, 'rb').read()
    entries = scan_entries(data)
    if not entries:
        return 0

    json_out = []
    meta_out = []

    for eid, e in enumerate(entries):
        rec = {
            'id': eid,
            'scr_msg': e['full_text'],
            'pre_jp': e['full_text'],      # 兼容旧流程
            'message': e['full_text'],
        }
        if e['kind'] == 'dialog':
            rec['name'] = e['name']
        json_out.append(rec)

        meta_out.append({
            'id': eid,
            'kind': e['kind'],
            'header_style': e.get('header_style'),
            'prefix': e.get('prefix'),
            'text_off': e['text_off'],
            'text_end': e['text_end'],
            'text_len': e['text_len'],
            'line_lengths': e['line_lengths'],
            'sep_sizes': e.get('sep_sizes', []),
            'sep_hex': [s.hex() for s in e.get('sep_bytes', [])],
            'orig_text': e['full_text'],
            'name': e['name'],
            'name_off': e['name_off'],
            'name_len': e['name_len'],
            'op4_off': e.get('op4_off'),
        })

    Path(json_path).parent.mkdir(parents=True, exist_ok=True)
    with open(json_path, 'w', encoding='utf-8', newline='\n') as f:
        json.dump(json_out, f, ensure_ascii=False, indent=2)
    meta_path = json_path + '.meta.json'
    with open(meta_path, 'w', encoding='utf-8', newline='\n') as f:
        json.dump({'file': os.path.basename(sd_path), 'entries': meta_out}, f, ensure_ascii=False, indent=2)

    narr = sum(1 for e in entries if e['kind'] == 'narr')
    dialog = sum(1 for e in entries if e['kind'] == 'dialog')
    v5 = sum(1 for e in entries if e.get('header_style') == 'v5')
    legacy = sum(1 for e in entries if e.get('header_style') == 'legacy')
    print(f'  [提取] {Path(sd_path).name}: 旁白{narr} 对话{dialog} 共{len(entries)}条 '
          f'(v5={v5}, legacy={legacy})')
    return len(entries)


def inject_file(sd_path: str, json_path: str, out_path: str):
    data = bytearray(open(sd_path, 'rb').read())
    meta_path = json_path + '.meta.json'

    if not os.path.exists(json_path):
        print(f'  [跳过] 无 JSON: {json_path}')
        return
    if not os.path.exists(meta_path):
        print(f'  [跳过] 无 meta: {meta_path}')
        return

    trans = {t['id']: t for t in json.load(open(json_path, encoding='utf-8'))}
    meta = json.load(open(meta_path, encoding='utf-8'))

    injected = 0
    skipped = 0
    truncated = 0
    name_inj = 0

    for m in meta['entries']:
        eid = m['id']
        orig_text = m['orig_text']
        tr = trans.get(eid)
        if tr is None:
            skipped += 1
            continue

        new_msg = tr.get('message', orig_text)
        if new_msg == orig_text:
            skipped += 1
            continue

        ll = m['line_lengths']
        if 'sep_hex' in m:
            sep_bytes = [bytes.fromhex(x) for x in m.get('sep_hex', [])]
        else:
            sep_bytes = []
            for sz in m.get('sep_sizes', [13] * (len(ll) - 1)):
                sep_bytes.append(DEFAULT_END_MARK + (b'\x0A' if sz == 13 else b''))
        off = m['text_off']
        total = sum(ll) + sum(len(s) for s in sep_bytes)

        try:
            enc = new_msg.encode(ENC)
        except UnicodeEncodeError as e:
            print(f'  [错误] id={eid} CP932编码失败: {e}，跳过')
            skipped += 1
            continue
        if len(enc) > sum(ll):
            truncated += 1
            print(f'  [截断] id={eid} 译文{len(enc)}B 超出容量{sum(ll)}B，将截断')

        label = f'id={eid}'
        try:
            if len(ll) == 1:
                nb = cp932_fit_line(new_msg, ll[0], label)
            else:
                nb = cp932_multiline(new_msg, ll, sep_bytes, label)
        except ValueError as e:
            print(f'  [错误] {e}，跳过')
            skipped += 1
            continue

        if len(nb) != total:
            print(f'  [错误] id={eid} 重建长度异常: new={len(nb)} old={total}，跳过')
            skipped += 1
            continue
        data[off:off + total] = nb
        injected += 1

        if m['kind'] == 'dialog' and m.get('name_len', 0) > 0:
            new_name = tr.get('name', m['name'])
            if new_name and new_name != m['name']:
                try:
                    nb_name = new_name.encode(ENC)
                except UnicodeEncodeError:
                    print(f'  [人名警告] id={eid} 译名"{new_name}" CP932编码失败，保留原名')
                    continue
                if len(nb_name) > m['name_len']:
                    print(f'  [人名警告] id={eid} 译名"{new_name}"({len(nb_name)}B)'
                          f'超出原名字节数({m["name_len"]}B)，保留原名')
                    continue
                nb_name = bytearray(nb_name)
                while len(nb_name) < m['name_len']:
                    if m['name_len'] - len(nb_name) >= 2:
                        nb_name += FWSP
                    else:
                        nb_name += b'\x20'
                data[m['name_off']:m['name_off'] + m['name_len']] = bytes(nb_name)
                name_inj += 1

    Path(out_path).parent.mkdir(parents=True, exist_ok=True)
    open(out_path, 'wb').write(bytes(data))

    orig_size = os.path.getsize(sd_path)
    size_ok = '✓' if len(data) == orig_size else f'✗ 大小变化! 原{orig_size} 现{len(data)}'
    print(f'  [注入] {Path(sd_path).name}: 注入{injected} 截断{truncated} '
          f'跳过{skipped} 人名{name_inj} 大小{size_ok}')


def cmd_extract(sd_dir: str, out_dir: str):
    base = Path(sd_dir)
    files = sorted(set(base.rglob('*.sd')) | set(base.rglob('*.SD')), key=lambda p: str(p).lower())
    if not files:
        print(f'[错误] {sd_dir} 中无 .sd 文件')
        return
    print(f'[提取] {len(files)} 个文件...')
    total = 0
    for sf in files:
        rel = sf.relative_to(base)
        jf = Path(out_dir) / rel.parent / (sf.stem + '.json')
        try:
            n = extract_file(str(sf), str(jf))
            total += n
            if n == 0:
                print(f'  [跳过] {rel}: 0条文本')
        except Exception as e:
            print(f'  [错误] {rel}: {e}')
    print(f'[提取完成] 共 {total} 条')


def cmd_inject(sd_dir: str, json_dir: str, out_dir: str):
    base = Path(sd_dir)
    files = sorted(set(base.rglob('*.sd')) | set(base.rglob('*.SD')), key=lambda p: str(p).lower())
    if not files:
        print(f'[错误] {sd_dir} 中无 .sd 文件')
        return
    print(f'[注入] {len(files)} 个文件...')
    for sf in files:
        rel = sf.relative_to(base)
        jf = Path(json_dir) / rel.parent / (sf.stem + '.json')
        if not jf.exists():
            print(f'  [跳过] {rel}: 无JSON')
            continue
        of = Path(out_dir) / rel
        try:
            inject_file(str(sf), str(jf), str(of))
        except Exception as e:
            print(f'  [错误] {rel}: {e}')
    print('[注入完成]')


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        sys.exit(0)
    cmd = sys.argv[1].lower()
    if cmd == 'extract':
        cmd_extract(sys.argv[2], sys.argv[3])
    elif cmd == 'inject':
        if len(sys.argv) < 5:
            print('用法: python sd_tool_v5.py inject <sd_dir> <json_dir> <out_dir>')
            sys.exit(1)
        cmd_inject(sys.argv[2], sys.argv[3], sys.argv[4])
    else:
        print(f'未知命令: {cmd}')
        sys.exit(1)


if __name__ == '__main__':
    main()
