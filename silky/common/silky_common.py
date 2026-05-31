"""silky_common.py — Silky MES 文本提取/注入共用逻辑。

本模块只处理 silky_op.py 反汇编后的 *.op.txt：
  - 识别 MESSAGE / STR / ruby 文本块；
  - 生成我们常用 JSON 条目：可选 name、scr_msg、message，并带 _file/_index 便于批处理定位；
  - 注入时用 _index 定位，再用 scr_msg 校验，避免译文顺序错位。

注意：scr_msg 是原始脚本文本，不能修改；翻译只改 message。
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

# 角色名块里出现的特殊 PUSH 数值。
# 本作 END05A.MES 的 name 结构为：PUSH_STR[角色名] -> PUSH[117440512] -> PUSH[486539264] -> 18[]。
# 旧版只收录了 83886080/167772160，因此会漏掉本作角色名。不同 Silky 变体可继续在这里扩展。
NAME_BLOCK_PUSH_VALS = frozenset([83886080, 117440512, 167772160])

# 块内 op：遇到这些不结束当前文本块，按 op + arg 两行跳过。
BLOCK_INTERNAL_OPCODES = frozenset([
    '#1-PUSH', '#1-PUSH_STR', '#1-RETURN',
    '#1-ff', '#1-fe', '#1-fd', '#1-fc', '#1-fb', '#1-fa',
    '#1-JUMP_2', '#1-3a', '#1-3b', '#1-3c', '#1-3d', '#1-3e', '#1-3f',
    '#1-40', '#1-41', '#1-42', '#1-43',
    '#1-34', '#1-35', '#1-37', '#1-38',
    '#1-10', '#1-11', '#1-0c', '#1-0d', '#1-0e', '#1-0f',
    '#1-02', '#1-03', '#1-04', '#1-05', '#1-06',
    '#1-17', '#1-18',
])

# 块结束 op。
BLOCK_END_OPCODES = frozenset([
    '#1-MESSAGE', '#1-JUMP', '#1-MSG_OFSETTER', '#1-SPEC_OFSETTER',
    '#1-1a', '#1-1b',
])

# 0x0A / 0x0B 都可能承载文本。
STR_OPCODE_LINES = frozenset(['#1-STR_CRYPT', '#1-STR_UNCRYPT'])


def is_label_or_free(line: str) -> bool:
    """#0- free bytes、#2-/#3- label 行不属于普通 op。"""
    return line.startswith('#0-') or line.startswith('#2-') or line.startswith('#3')


def parse_json_str(arg_line: str) -> str:
    """读取 op.txt 参数行中的第一个字符串参数。"""
    try:
        val = json.loads(arg_line)
        if isinstance(val, list) and val:
            return str(val[0])
    except (json.JSONDecodeError, IndexError, TypeError):
        pass
    return arg_line


def parse_json_first_int(arg_line: str) -> int:
    """读取 op.txt 参数行中的第一个整数参数。"""
    try:
        val = json.loads(arg_line)
        if isinstance(val, list) and val:
            return int(val[0])
    except (json.JSONDecodeError, IndexError, TypeError, ValueError):
        pass
    return 0


def write_json_arg(value: str) -> str:
    """把字符串重新写回 op.txt 参数行。"""
    return json.dumps([value], ensure_ascii=False) + '\n'


@dataclass
class TextPart:
    """文本槽位。

    kind:
      - text: 普通 STR 参数；arg_idx 指向参数行。
      - ruby: ruby 段；ruby_base_arg_idx 指向 base 字符串参数行。
      - newline: TO_NEW_STRING [0]，表示提取 JSON 里的字面 \\n。
    """
    kind: str
    arg_idx: Optional[int] = None
    text: str = ''
    ruby_reading_slots: Optional[List[Tuple[int, str, str]]] = None
    ruby_base_arg_idx: Optional[int] = None


@dataclass
class TextBlock:
    """一次可翻译文本块。"""
    parts: List[TextPart]
    end_index: int
    name: Optional[str] = None
    name_arg_idx: Optional[int] = None


def detect_name_block(lines: Sequence[str], i: int) -> Optional[Tuple[str, int]]:
    """识别角色名块，返回 (name, name_arg_line_idx)。"""
    total = len(lines)
    if i + 7 >= total:
        return None
    if lines[i].rstrip('\n') != '#1-PUSH_STR':
        return None

    name = parse_json_str(lines[i + 1].rstrip('\n'))
    # 角色名一般不是纯 ASCII。这个过滤用于避免把路径/资源名误判为人名。
    try:
        name.encode('ascii')
        return None
    except UnicodeEncodeError:
        pass

    if lines[i + 2].rstrip('\n') != '#1-PUSH':
        return None
    try:
        push_val = json.loads(lines[i + 3].rstrip('\n'))
    except json.JSONDecodeError:
        return None
    if not (isinstance(push_val, list) and push_val and push_val[0] in NAME_BLOCK_PUSH_VALS):
        return None

    # Pattern A: PUSH_STR[name] -> PUSH[special] -> PUSH[...] -> 18[]
    if (i + 6 < total and
            lines[i + 4].rstrip('\n') == '#1-PUSH' and
            lines[i + 6].rstrip('\n') == '#1-18'):
        return name, i + 1

    # Pattern B: PUSH_STR[name] -> PUSH[special] -> PUSH[...] -> 34[] -> PUSH[...] -> 18[]
    if (i + 10 < total and
            lines[i + 4].rstrip('\n') == '#1-PUSH' and
            lines[i + 6].rstrip('\n') == '#1-34' and
            lines[i + 8].rstrip('\n') == '#1-PUSH' and
            lines[i + 10].rstrip('\n') == '#1-18'):
        return name, i + 1

    return None


def try_match_ruby_at_tns(lines: Sequence[str], tns_idx: int) -> Optional[Tuple[TextPart, int]]:
    """识别 ruby 段。

    常见结构：
      #1-TO_NEW_STRING [1]
      STR/UNSTR 若干 reading/separator
      #1-RETURN []
      #1-STR_CRYPT [base]

    返回 (ruby_part, end_idx)。end_idx 是 ruby 段之后的下一行下标。
    """
    total = len(lines)
    if tns_idx + 1 >= total:
        return None
    if lines[tns_idx].rstrip('\n') != '#1-TO_NEW_STRING':
        return None
    if parse_json_first_int(lines[tns_idx + 1].rstrip('\n')) != 1:
        return None

    j = tns_idx + 2
    inner: List[Tuple[int, str, str]] = []
    while j < total:
        op = lines[j].rstrip('\n')
        if op == '#1-RETURN':
            break
        if op not in ('#1-STR_CRYPT', '#1-STR_UNCRYPT'):
            return None
        if j + 1 >= total:
            return None
        val = parse_json_str(lines[j + 1].rstrip('\n'))
        if op == '#1-STR_UNCRYPT':
            kind = 'sep_uncrypt'
        elif val == '\u3000':
            kind = 'sep_full'
        else:
            kind = 'reading'
        inner.append((j + 1, val, kind))
        j += 2

    if not inner:
        return None
    if j >= total or lines[j].rstrip('\n') != '#1-RETURN':
        return None

    j += 2  # RETURN 的参数行也跳过
    if j + 1 >= total or lines[j].rstrip('\n') != '#1-STR_CRYPT':
        return None

    base_arg_idx = j + 1
    base = parse_json_str(lines[base_arg_idx].rstrip('\n'))
    part = TextPart(
        kind='ruby',
        text=base,
        ruby_reading_slots=inner,
        ruby_base_arg_idx=base_arg_idx,
    )
    return part, j + 2


def collect_text_block(lines: Sequence[str], start: int) -> TextBlock:
    """从 start 开始收集一个文本块，直到遇到块结束 op。"""
    total = len(lines)
    parts: List[TextPart] = []
    detected_name: Optional[str] = None
    name_arg_idx: Optional[int] = None
    i = start

    while i < total:
        cl = lines[i].rstrip('\n')

        name_hit = detect_name_block(lines, i)
        if name_hit is not None:
            detected_name, name_arg_idx = name_hit
            i += 2
            continue

        if cl == '#1-TO_NEW_STRING':
            arg_val = parse_json_first_int(lines[i + 1].rstrip('\n')) if i + 1 < total else -1
            if arg_val == 1:
                ruby_hit = try_match_ruby_at_tns(lines, i)
                if ruby_hit is not None:
                    ruby_part, i = ruby_hit
                    parts.append(ruby_part)
                    continue
            elif arg_val == 0:
                parts.append(TextPart(kind='newline'))
                i += 2
                continue
            i += 2
            continue

        if cl in STR_OPCODE_LINES:
            arg_line = lines[i + 1].rstrip('\n') if i + 1 < total else '[]'
            parts.append(TextPart(kind='text', arg_idx=i + 1, text=parse_json_str(arg_line)))
            i += 2
        elif cl in BLOCK_END_OPCODES:
            break
        elif cl in BLOCK_INTERNAL_OPCODES:
            i += 2
        elif is_label_or_free(cl):
            i += 1
        elif cl.startswith('#1-'):
            # 未命名但带参数的 op，保守按两行跳过。
            i += 2
        elif cl.startswith('$'):
            i += 1
        else:
            i += 1

    return TextBlock(parts=parts, end_index=i, name=detected_name, name_arg_idx=name_arg_idx)


def join_parts(parts: Iterable[TextPart]) -> str:
    """把块内槽位拼成 JSON 的 scr_msg/message。"""
    out: List[str] = []
    for p in parts:
        if p.kind in ('text', 'ruby'):
            out.append(p.text)
        elif p.kind == 'newline':
            out.append('\\n')
    return ''.join(out)


def iter_text_blocks(lines: Sequence[str]) -> Iterable[TextBlock]:
    """按 extract/inject 的共同顺序遍历所有可翻译文本块。"""
    i = 0
    total = len(lines)
    while i < total:
        line = lines[i].rstrip('\n')
        if line == '#1-MESSAGE':
            i += 2
            block = collect_text_block(lines, i)
            i = block.end_index
            if join_parts(block.parts):
                yield block
        elif line in STR_OPCODE_LINES or line == '#1-TO_NEW_STRING':
            block = collect_text_block(lines, i)
            i = block.end_index
            if join_parts(block.parts):
                yield block
        else:
            i += 1


def make_json_entries(lines: Sequence[str], file_name: str = '') -> List[Dict[str, Any]]:
    """由 op.txt 行生成我们项目常用 JSON 列表。"""
    entries: List[Dict[str, Any]] = []
    for idx, block in enumerate(iter_text_blocks(lines)):
        scr_msg = join_parts(block.parts)
        # 字段顺序按项目翻译习惯组织：定位字段在前，name 放到 scr_msg/message 前。
        item: Dict[str, Any] = {
            '_file': file_name,
            '_index': idx,
        }
        if block.name:
            item['name'] = block.name
        item['scr_msg'] = scr_msg
        item['message'] = scr_msg
        entries.append(item)
    return entries


def load_json_entries(json_path: str) -> List[Dict[str, Any]]:
    """读取翻译 JSON。兼容单个 list，或 {"entries": [...]}。"""
    with open(json_path, 'r', encoding='utf-8-sig') as f:
        data = json.load(f)
    if isinstance(data, list):
        return data
    if isinstance(data, dict) and isinstance(data.get('entries'), list):
        return data['entries']
    raise ValueError(f'不支持的 JSON 结构: {json_path}')


def save_json_entries(json_path: str, entries: Sequence[Dict[str, Any]]) -> None:
    os.makedirs(os.path.dirname(os.path.abspath(json_path)), exist_ok=True)
    with open(json_path, 'w', encoding='utf-8') as f:
        json.dump(list(entries), f, ensure_ascii=False, indent=2)
        f.write('\n')


def split_visible_newline(text: str) -> List[str]:
    """按项目约定的字面 \\n 切分，而不是按真实换行。"""
    return text.split('\\n')


def apply_translation_to_block(
    lines: List[str],
    block: TextBlock,
    message: str,
    name: Optional[str] = None,
) -> None:
    """把一个 JSON 条目的 message/name 写回对应 TextBlock。

    - message：写回正文字符串槽。
    - name：如果 JSON 条目里存在 name，且当前块识别到了 name_arg_idx，则直接写回原 name 参数行。

    规则与旧版一致：以 TO_NEW_STRING[0] 形成的字面 \\n 为段组边界；每段组第一个 text/ruby 槽位写入译文，后续槽位清空。
    ruby reading 槽位不暴露给译者，注入时按原 reading 长度写全角空格占位。
    """
    if name is not None and block.name_arg_idx is not None:
        lines[block.name_arg_idx] = write_json_arg(str(name))

    groups: List[List[TextPart]] = [[]]
    for p in block.parts:
        if p.kind == 'newline':
            groups.append([])
        else:
            groups[-1].append(p)
    groups = [g for g in groups if g]
    trans_parts = split_visible_newline(message)

    for gi, group in enumerate(groups):
        seg_text = trans_parts[gi] if gi < len(trans_parts) else ''
        assigned = False
        for p in group:
            if p.kind == 'text':
                if p.arg_idx is not None:
                    lines[p.arg_idx] = write_json_arg(seg_text if not assigned else '')
                    assigned = True
            elif p.kind == 'ruby':
                # ruby reading 只保留占位，避免译文污染注音控制结构。
                for arg_idx, orig_val, slot_kind in (p.ruby_reading_slots or []):
                    if slot_kind == 'reading':
                        # 按原 reading 非全角空格字符数生成占位。
                        count = len(orig_val.replace('\u3000', ''))
                        lines[arg_idx] = write_json_arg('\u3000' * count)
                if p.ruby_base_arg_idx is not None:
                    lines[p.ruby_base_arg_idx] = write_json_arg(seg_text if not assigned else '')
                    assigned = True


def build_translation_map(entries: Sequence[Dict[str, Any]]) -> Dict[int, Dict[str, Any]]:
    """按 _index 建立译文表；缺 _index 时按列表顺序兜底。"""
    result: Dict[int, Dict[str, Any]] = {}
    for fallback_idx, item in enumerate(entries):
        try:
            idx = int(item.get('_index', fallback_idx))
        except (TypeError, ValueError):
            idx = fallback_idx
        result[idx] = item
    return result
