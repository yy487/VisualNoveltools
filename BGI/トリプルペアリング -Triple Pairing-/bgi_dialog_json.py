import os
import json
import re
import tempfile
import asdis

RE_PUSH_STRING = re.compile(r'^\s*push_string\("((?:\\.|[^"\\])*)"\);\s*$')
RE_PUSH_DWORD = re.compile(r'^\s*push_dword\((-?\d+)\);\s*$')
RE_CALL_NOARGS = re.compile(r'^\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*\(\);\s*$')
RE_CALL_NAME = re.compile(r'^\s*((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)\(\);\s*$')
RE_PUSH_BASE_OFFSET = re.compile(r'^\s*push_base_offset\((-?\d+)\);\s*$')
RE_MOVE = re.compile(r'^\s*move\((-?\d+)\);\s*$')
RE_NARGS = re.compile(r'^\s*nargs\((-?\d+)\);\s*$')
RE_ADD = re.compile(r'^\s*add\(\);\s*$')
RE_MUL = re.compile(r'^\s*mul\(\);\s*$')
RE_V0_CALL = re.compile(r'^(\s*)([A-Za-z_][A-Za-z0-9_]*)\((.*)\);\s*(//.*)?$')
RE_V0_STR = re.compile(r'"((?:\\.|[^"\\])*)"')
RE_BPD_STRING = re.compile(r'^\s*"((?:\\.|[^"\\])*)"\s*$')

_BSS_MAPPING_CACHE = None
RE_FUNC_TOKEN = re.compile(r'^_?[A-Za-z][A-Za-z0-9_]*$')
DIALOG_CONTROL_SUFFIX_CHARS = '<>&.'

def _toolkit_root():
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def _split_dialog_message_suffix(text):
    value = '' if text is None else str(text)
    suffix_chars = []
    while value and value[-1] in DIALOG_CONTROL_SUFFIX_CHARS:
        suffix_chars.append(value[-1])
        value = value[:-1]
    suffix_chars.reverse()
    return value, ''.join(suffix_chars)

def _make_dialog_entry(
    *,
    name,
    message,
    name_line_index,
    message_line_index,
    is_select,
    name_arg_index=None,
    message_arg_index=None,
    call_line=None,
    user_func_name=None,
    entry_type=None
):
    visible_message, message_suffix = _split_dialog_message_suffix(message)
    if visible_message == "　" and not message_suffix:
        return None
    entry = {
        "name": name,
        "message": visible_message,
        "message_suffix": message_suffix,
        "name_line_index": name_line_index,
        "message_line_index": message_line_index,
        "is_select": is_select
    }
    if name_arg_index is not None:
        entry["name_arg_index"] = name_arg_index
    if message_arg_index is not None:
        entry["message_arg_index"] = message_arg_index
    if call_line is not None:
        entry["call_line"] = call_line
    if user_func_name is not None:
        entry["user_func_name"] = user_func_name
    if entry_type is not None:
        entry["entry_type"] = entry_type
    return entry

def _restore_message_suffix(text, suffix):
    value = '' if text is None else str(text)
    tail = '' if suffix is None else str(suffix)
    if tail and not value.endswith(tail):
        return value + tail
    return value

def _extract_push_string(line):
    match = RE_PUSH_STRING.match(line.rstrip('\r\n'))
    if not match:
        return None
    return asdis.unescape(match.group(1))

def _replace_push_string(line, new_text):
    match = RE_PUSH_STRING.match(line.rstrip('\r\n'))
    if not match:
        return line
    indent_match = re.match(r'^(\s*)', line)
    indent = indent_match.group(1) if indent_match else ''
    suffix = '\n' if line.endswith('\n') else ''
    return f'{indent}push_string("{asdis.escape(new_text)}");{suffix}'

def _extract_bpd_string_line(line):
    match = RE_BPD_STRING.match(line.rstrip('\r\n'))
    if not match:
        return None
    return asdis.unescape(match.group(1))

def _replace_bpd_string_line(line, new_text):
    match = RE_BPD_STRING.match(line.rstrip('\r\n'))
    if not match:
        return line
    indent_match = re.match(r'^(\s*)', line)
    indent = indent_match.group(1) if indent_match else ''
    suffix = '\n' if line.endswith('\n') else ''
    return f'{indent}"{asdis.escape(new_text)}"{suffix}'

def _extract_v0_call(line):
    stripped = line.rstrip('\r\n')
    m = RE_V0_CALL.match(stripped)
    if not m:
        return None
    func = m.group(2)
    args_text = m.group(3)
    literals = []
    for mm in RE_V0_STR.finditer(args_text):
        literals.append({
            "span": mm.span(1),
            "text": asdis.unescape(mm.group(1))
        })
    return {
        "func": func,
        "args_text": args_text,
        "literals": literals
    }

def _load_bss_mapping():
    global _BSS_MAPPING_CACHE
    if _BSS_MAPPING_CACHE is not None:
        return _BSS_MAPPING_CACHE
    mapping_path = os.path.join(_toolkit_root(), "bss_mapping.json")
    try:
        with open(mapping_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        if isinstance(data, dict):
            _BSS_MAPPING_CACHE = {str(k): str(v) for k, v in data.items()}
        else:
            _BSS_MAPPING_CACHE = {}
    except Exception:
        _BSS_MAPPING_CACHE = {}
    return _BSS_MAPPING_CACHE

def _split_qualified_name(name):
    text = str(name or '').strip()
    if not text:
        return '', ''
    if '::' in text:
        prefix, base = text.rsplit('::', 1)
        return prefix, base
    return '', text

def _function_aliases(*names):
    mapping = _load_bss_mapping()
    reverse = {}
    for raw_name, mapped_name in mapping.items():
        reverse.setdefault(mapped_name, set()).add(raw_name)
    aliases = set()
    stack = [str(name).strip() for name in names if str(name).strip()]
    while stack:
        current = stack.pop()
        if current in aliases:
            continue
        aliases.add(current)
        _, base = _split_qualified_name(current)
        if base and base not in aliases:
            stack.append(base)
        mapped = mapping.get(base)
        if mapped and mapped not in aliases:
            stack.append(mapped)
        for raw_name in reverse.get(base, ()):
            if raw_name not in aliases:
                stack.append(raw_name)
    return aliases

def _extract_call_name(line):
    m = RE_CALL_NAME.match(line.strip())
    if not m:
        return None
    return m.group(1)

def _is_noarg_call_line(line):
    return _extract_call_name(line) is not None

def _matches_function_name(name, aliases):
    if not name:
        return False
    _, base = _split_qualified_name(name)
    return name in aliases or base in aliases

def _looks_like_function_token(text):
    value = str(text or '').strip()
    if not value:
        return False
    return RE_FUNC_TOKEN.match(value) is not None

def _replace_v0_call_string(line, literal_index, new_text):
    suffix = '\n' if line.endswith('\n') else ''
    stripped = line.rstrip('\r\n')
    m = RE_V0_CALL.match(stripped)
    if not m:
        return line
    indent = m.group(1) or ""
    func = m.group(2)
    args_text = m.group(3)
    literals = list(RE_V0_STR.finditer(args_text))
    if not literals:
        return line
    idx = literal_index
    if idx < 0:
        idx = len(literals) + idx
    if idx < 0 or idx >= len(literals):
        return line
    target = literals[idx]
    start, end = target.span(1)
    new_args = args_text[:start] + asdis.escape(new_text) + args_text[end:]
    rebuilt = f"{indent}{func}({new_args});"
    comment = m.group(4)
    if comment:
        rebuilt += f" {comment}"
    return rebuilt + suffix

def _extract_push_dword(line):
    match = RE_PUSH_DWORD.match(line.rstrip('\r\n'))
    if not match:
        return None
    return int(match.group(1))

def _is_push_dword_line(line):
    return _extract_push_dword(line) is not None

def _extract_push_base_offset(line):
    match = RE_PUSH_BASE_OFFSET.match(line.rstrip('\r\n'))
    if not match:
        return None
    return int(match.group(1))

def _extract_move_arity(line):
    match = RE_MOVE.match(line.rstrip('\r\n'))
    if not match:
        return None
    return int(match.group(1))

def _extract_nargs(line):
    match = RE_NARGS.match(line.rstrip('\r\n'))
    if not match:
        return None
    return int(match.group(1))

def _is_add(line):
    return RE_ADD.match(line.rstrip('\r\n')) is not None

def _is_mul(line):
    return RE_MUL.match(line.rstrip('\r\n')) is not None

def normalize_user_function_names(user_function_names):
    if not user_function_names:
        return []
    if isinstance(user_function_names, str):
        raw_items = re.split(r'[\r\n,;]+', user_function_names)
    else:
        raw_items = user_function_names
    names = []
    seen = set()
    for item in raw_items:
        text = str(item).strip()
        if not text or text in seen:
            continue
        names.append(text)
        seen.add(text)
    return names

def _normalize_user_function_name_for_match(name):
    text = str(name or '').strip()
    if text.startswith('_'):
        return text[1:]
    return text

def _prev_effective_line(lines, index):
    i = index - 1
    while i >= 0:
        stripped = lines[i].strip()
        if stripped and not stripped.startswith('//'):
            return i
        i -= 1
    return None

def _prev_effective_non_nargs_line(lines, index):
    i = _prev_effective_line(lines, index)
    while i is not None and _extract_nargs(lines[i]) is not None:
        i = _prev_effective_line(lines, i)
    return i

def _parse_dialog_call_modern(lines, call_idx):
    msg_idx = _prev_effective_non_nargs_line(lines, call_idx)
    if msg_idx is None:
        return None
    message = _extract_push_string(lines[msg_idx])
    if message is None:
        return None
    prev_idx = _prev_effective_line(lines, msg_idx)
    if prev_idx is None:
        return None
    name = None
    name_idx = None
    v = _extract_push_string(lines[prev_idx])
    if v is not None:
        name = v
        name_idx = prev_idx
        flag0_idx = _prev_effective_line(lines, prev_idx)
        if flag0_idx is None or not _is_push_dword_line(lines[flag0_idx]):
            return None
        f1_idx = _prev_effective_line(lines, flag0_idx)
        f2_idx = _prev_effective_line(lines, f1_idx) if f1_idx is not None else None
        if f1_idx is None or f2_idx is None:
            return None
        if not _is_push_dword_line(lines[f1_idx]) or not _is_push_dword_line(lines[f2_idx]):
            return None
    else:
        anon_mark = _extract_push_dword(lines[prev_idx])
        if anon_mark is None:
            return None
        flag0_idx = _prev_effective_line(lines, prev_idx)
        if flag0_idx is None or not _is_push_dword_line(lines[flag0_idx]):
            return None
        f1_idx = _prev_effective_line(lines, flag0_idx)
        f2_idx = _prev_effective_line(lines, f1_idx) if f1_idx is not None else None
        if f1_idx is None or f2_idx is None:
            return None
        if not _is_push_dword_line(lines[f1_idx]) or not _is_push_dword_line(lines[f2_idx]):
            return None
    return _make_dialog_entry(
        name=name,
        message=message,
        name_line_index=name_idx,
        message_line_index=msg_idx,
        is_select=False
    )

def _parse_dialog_call_legacy(lines, call_idx):
    cursor = _prev_effective_non_nargs_line(lines, call_idx)
    if cursor is None:
        return None
    dwords = []
    while cursor is not None:
        dv = _extract_push_dword(lines[cursor])
        if dv is None:
            break
        dwords.append(dv)
        cursor = _prev_effective_line(lines, cursor)
    if len(dwords) < 3:
        return None
    if cursor is None:
        return None
    first_str = _extract_push_string(lines[cursor])
    if first_str is None:
        return None
    prev_idx = _prev_effective_line(lines, cursor)
    second_str = _extract_push_string(lines[prev_idx]) if prev_idx is not None else None
    if second_str is not None:
        return _make_dialog_entry(
            name=first_str,
            message=second_str,
            name_line_index=cursor,
            message_line_index=prev_idx,
            is_select=False
        )
    return _make_dialog_entry(
        name=None,
        message=first_str,
        name_line_index=None,
        message_line_index=cursor,
        is_select=False
    )

def _is_dialog_call(lines, call_idx):
    top_arg_idx = _prev_effective_non_nargs_line(lines, call_idx)
    if top_arg_idx is None:
        return None
    if _extract_push_string(lines[top_arg_idx]) is not None:
        parsed = _parse_dialog_call_modern(lines, call_idx)
        if parsed:
            return parsed
        return _parse_dialog_call_legacy(lines, call_idx)
    if _extract_push_dword(lines[top_arg_idx]) is not None:
        parsed = _parse_dialog_call_legacy(lines, call_idx)
        if parsed:
            return parsed
        return _parse_dialog_call_modern(lines, call_idx)
    parsed = _parse_dialog_call_modern(lines, call_idx)
    if parsed:
        return parsed
    return _parse_dialog_call_legacy(lines, call_idx)

def _is_print_message_call(line):
    return _matches_function_name(_extract_call_name(line), _function_aliases("f_PrintMessage", "f_140", "grp_::f_140"))

def _looks_like_user_function_call(lines, call_idx):
    cursor = _prev_effective_non_nargs_line(lines, call_idx)
    if cursor is None:
        return False
    arg_indices = []
    while cursor is not None and _is_user_func_arg_line(lines, cursor):
        arg_indices.append(cursor)
        cursor = _prev_effective_line(lines, cursor)
    for idx in arg_indices:
        value = _extract_push_string(lines[idx])
        if value is not None:
            return _looks_like_function_token(value)
    return False

def _extract_print_message_entry(lines, call_idx):
    if not _is_print_message_call(lines[call_idx]):
        return None
    return _is_dialog_call(lines, call_idx)

def _is_user_func_helper_call(lines, index):
    call_name = _extract_call_name(lines[index])
    if not call_name:
        return False
    prev_idx = _prev_effective_line(lines, index)
    if prev_idx is None:
        return False
    nargs = _extract_nargs(lines[prev_idx])
    if nargs is None:
        return False
    arg_idx = _prev_effective_line(lines, prev_idx)
    if arg_idx is None:
        return False
    stripped = lines[arg_idx].strip()
    return stripped.startswith('push_')

def _is_user_func_call(line):
    call_name = _extract_call_name(line)
    if not call_name:
        return False
    _, base = _split_qualified_name(call_name)
    return base == "f_01c"

def _is_user_func_arg_line(lines, index):
    stripped = lines[index].strip()
    if not stripped or stripped.startswith('//'):
        return False
    if _extract_nargs(lines[index]) is not None:
        return True
    # Treat helper calls structurally instead of relying on mapped function names.
    if _is_user_func_helper_call(lines, index):
        return True
    return stripped.startswith('push_')

def _extract_user_func_entries(lines, call_idx, user_function_names):
    if not user_function_names or not _is_user_func_call(lines[call_idx]):
        return []
    target_names = set(_normalize_user_function_name_for_match(x) for x in user_function_names)
    cursor = _prev_effective_non_nargs_line(lines, call_idx)
    if cursor is None:
        return []
    arg_indices = []
    while cursor is not None and _is_user_func_arg_line(lines, cursor):
        arg_indices.append(cursor)
        cursor = _prev_effective_line(lines, cursor)
    func_name = None
    func_name_idx = None
    for idx in arg_indices:
        value = _extract_push_string(lines[idx])
        if value is not None:
            func_name = value
            func_name_idx = idx
            break
    if _normalize_user_function_name_for_match(func_name) not in target_names:
        return []
    entries = []
    for idx in reversed(arg_indices):
        if idx == func_name_idx:
            continue
        value = _extract_push_string(lines[idx])
        if value is None:
            continue
        entry = _make_dialog_entry(
            name=None,
            message=value,
            name_line_index=None,
            message_line_index=idx,
            is_select=True,
            call_line=call_idx + 1,
            user_func_name=func_name
        )
        if entry:
            entries.append(entry)
    return entries


def _next_effective_line(lines, index):
    i = index + 1
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped and not stripped.startswith('//'):
            return i
        i += 1
    return None

def _is_label_line(line):
    return asdis.re_label.match(asdis.remove_comment(line.strip())) is not None

def _is_v1_select_call_name(name):
    """BGI V1 的选择 UI 通常落在 slct::f_160 ~ slct::f_17f。"""
    if not name:
        return False
    prefix, base = _split_qualified_name(name)
    m = re.fullmatch(r'f_([0-9A-Fa-f]{3})', base)
    if not m:
        return False
    op = int(m.group(1), 16)
    return prefix == 'slct' and 0x160 <= op < 0x180

def _is_select_related_call(line):
    """判断当前调用是否可能是选项注册/显示调用。"""
    call_name = _extract_call_name(line)
    if not call_name:
        return False
    if _matches_function_name(call_name, _function_aliases(
        "f_Select",
        "f_0b0",
        "f_SetNextSelectingJumping",
        "f_0a9",
    )):
        return True
    return _is_v1_select_call_name(call_name)

def _looks_like_visible_select_text(text):
    """过滤明显不是选项文字的字符串，例如函数名、资源名、标签名。"""
    value = str(text or '').strip()
    if not value:
        return False
    if value.lower().startswith('@hex:'):
        return False
    if _looks_like_function_token(value):
        return False
    # 资源名/文件名通常是纯 ASCII token，避免把图像、音频、标签名误当选项。
    if re.fullmatch(r'[A-Za-z0-9_./\\:\-]+', value):
        return False
    return True

def _is_select_stack_arg_line(lines, index):
    """选项注册调用前的参数栈行。只向前穿过 push/简单运算/nargs，不跨越其他业务调用。"""
    stripped = lines[index].strip()
    if not stripped or stripped.startswith('//'):
        return False
    if _extract_nargs(lines[index]) is not None:
        return True
    if stripped.startswith('push_'):
        return True
    if _is_add(lines[index]) or _is_mul(lines[index]):
        return True
    return False

def _extract_select_entries_from_stack_call(lines, call_idx):
    """从 BGI V1/V0 的选项注册调用前回溯 push_string 参数。

    旧逻辑只处理 push_string 后紧跟 move(2) 的形式；实际选项按钮常见形式是：
        push_offset(Lxxxxx);
        push_string("選択肢");
        slct::f_16x(); / f_SetNextSelectingJumping();

    因此这里以选择系统调用为锚点，向前收集同一参数块中的 push_string。
    """
    if not _is_select_related_call(lines[call_idx]):
        return []

    cursor = _prev_effective_non_nargs_line(lines, call_idx)
    arg_indices = []
    scanned = 0
    while cursor is not None and scanned < 64:
        if _is_label_line(lines[cursor]):
            break
        if not _is_select_stack_arg_line(lines, cursor):
            break
        arg_indices.append(cursor)
        cursor = _prev_effective_line(lines, cursor)
        scanned += 1

    entries = []
    for idx in reversed(arg_indices):
        value = _extract_push_string(lines[idx])
        if value is None:
            continue
        if not _looks_like_visible_select_text(value):
            continue
        entry = _make_dialog_entry(
            name=None,
            message=value,
            name_line_index=None,
            message_line_index=idx,
            is_select=True,
            call_line=call_idx + 1
        )
        if entry:
            entries.append(entry)
    return entries


def _is_notify_line_count_call(line):
    call_name = _extract_call_name(line)
    if not call_name:
        return False
    if _matches_function_name(call_name, _function_aliases("f_NotifyOfLineCount", "f_0fe")):
        return True
    _, base = _split_qualified_name(call_name)
    return base == "f_0fe"

def _extract_selectex_func_name_before_user_call(lines, call_idx):
    """f_01c(); の直前に push_string("SelectEx"/_"SelectEx") があるかを見る。"""
    if not _is_user_func_call(lines[call_idx]):
        return None, None
    func_idx = _prev_effective_line(lines, call_idx)
    if func_idx is None:
        return None, None
    func_name = _extract_push_string(lines[func_idx])
    if _normalize_user_function_name_for_match(func_name) != "SelectEx":
        return None, None
    return func_idx, func_name

def _find_selectex_argument_window_start(lines, func_name_idx):
    """SelectEx 呼び出し直前の選択肢構築ブロックの開始位置を探す。

    実サンプルでは直前の f_0fe(); が「元スクリプト行通知」で、
    その直後から push_string(選択肢...) / flag / jump id が積まれる。
    f_0fe を越えて遡るとファイルパス、台詞、リソース名まで拾うので、ここを境界にする。
    """
    cursor = _prev_effective_line(lines, func_name_idx)
    fallback_start = max(0, func_name_idx - 96)
    while cursor is not None and func_name_idx - cursor <= 160:
        if _is_label_line(lines[cursor]):
            return cursor + 1
        if _is_notify_line_count_call(lines[cursor]):
            return cursor + 1
        call_name = _extract_call_name(lines[cursor])
        if call_name and _is_v1_select_call_name(call_name):
            return cursor + 1
        cursor = _prev_effective_line(lines, cursor)
    return fallback_start

def _has_move_style_choices(lines, start_idx, end_idx):
    for idx in range(start_idx, end_idx):
        if _extract_push_string(lines[idx]) is None:
            continue
        next_idx = _next_effective_line(lines, idx)
        if next_idx is not None and _extract_move_arity(lines[next_idx]) == 2:
            return True
    return False

def _extract_direct_selectex_entries(lines, call_idx):
    """BGI V1 の f_01c("SelectEx") 直前ブロックから直接選択肢を拾う。

    例：
        f_0fe();
        push_string("左の方のカードを引く");
        push_string("右の方のカードを引く");
        push_string("真ん中のカードを引く");
        push_dword(1); ...
        f_0e5(); ...
        push_string("SelectEx");
        f_01c();

    旧形式は push_string(...); move(2); なので、同じ窓内に move(2) 型選択肢がある場合は
    既存の _is_select_option_string に任せ、ここでは重複抽出しない。
    """
    func_name_idx, func_name = _extract_selectex_func_name_before_user_call(lines, call_idx)
    if func_name_idx is None:
        return []
    start_idx = _find_selectex_argument_window_start(lines, func_name_idx)
    if _has_move_style_choices(lines, start_idx, func_name_idx):
        return []

    entries = []
    for idx in range(start_idx, func_name_idx):
        value = _extract_push_string(lines[idx])
        if value is None:
            continue
        if _normalize_user_function_name_for_match(value) == "SelectEx":
            continue
        if not _looks_like_visible_select_text(value):
            continue
        entry = _make_dialog_entry(
            name=None,
            message=value,
            name_line_index=None,
            message_line_index=idx,
            is_select=True,
            call_line=call_idx + 1,
            user_func_name=func_name,
        )
        if entry:
            entries.append(entry)
    # SelectEx の直接選択肢は通常 2 個以上。1 個だけならプロンプト等の誤検出リスクが高い。
    if len(entries) < 2:
        return []
    return entries


def _extract_selectex_prompt_entry(lines, call_idx):
    """提取 _SelectEx/SelectEx 的提示标题。实际样本中旧式 move(2) 选项后可能紧接：

        push_string("見たいシーンを選択してください");
        push_string("_SelectEx");
        f_01c();

    选项本身由旧式 move(2) 逻辑提取，这里只补最靠近 SelectEx 函数名之前的提示文本。
    """
    func_name_idx, func_name = _extract_selectex_func_name_before_user_call(lines, call_idx)
    if func_name_idx is None:
        return None
    prompt_idx = _prev_effective_line(lines, func_name_idx)
    if prompt_idx is None:
        return None
    value = _extract_push_string(lines[prompt_idx])
    if value is None:
        return None
    if _normalize_user_function_name_for_match(value) == "SelectEx":
        return None
    if not _looks_like_visible_select_text(value):
        return None
    # 如果这行本身就是旧式 choice 的 push_string(...); move(2)，它不是提示标题。
    next_idx = _next_effective_line(lines, prompt_idx)
    if next_idx is not None and _extract_move_arity(lines[next_idx]) == 2:
        return None
    return _make_dialog_entry(
        name=None,
        message=value,
        name_line_index=None,
        message_line_index=prompt_idx,
        is_select=False,
        call_line=call_idx + 1,
        user_func_name=func_name,
        entry_type="ui",
    )


def _extract_named_user_func_single_arg_entry(lines, call_idx, target_func_names, entry_type="ui"):
    """提取 f_01c("FuncName") 前面的单个可见字符串参数。

    用于 AutoSaveRange 这种运行时 UI/章节标题字符串：
        push_string("プロローグ");
        push_string("AutoSaveRange");
        f_01c();
    """
    if not _is_user_func_call(lines[call_idx]):
        return None
    func_idx = _prev_effective_line(lines, call_idx)
    if func_idx is None:
        return None
    func_name = _extract_push_string(lines[func_idx])
    norm_func = _normalize_user_function_name_for_match(func_name)
    targets = {_normalize_user_function_name_for_match(x) for x in target_func_names}
    if norm_func not in targets:
        return None
    arg_idx = _prev_effective_line(lines, func_idx)
    if arg_idx is None:
        return None
    value = _extract_push_string(lines[arg_idx])
    if value is None:
        return None
    if not _looks_like_visible_select_text(value):
        return None
    return _make_dialog_entry(
        name=None,
        message=value,
        name_line_index=None,
        message_line_index=arg_idx,
        is_select=False,
        call_line=call_idx + 1,
        user_func_name=func_name,
        entry_type=entry_type,
    )


def _is_sys_set_caption_call(line):
    call_name = _extract_call_name(line)
    if not call_name:
        return False
    _, base = _split_qualified_name(call_name)
    return base == "f_11e"


def _extract_sys_set_caption_entry(lines, call_idx):
    """提取 sys_::f_11e 的标题文本：push_string("おまけ"); nargs(1); sys_::f_11e();"""
    if not _is_sys_set_caption_call(lines[call_idx]):
        return None
    arg_idx = _prev_effective_non_nargs_line(lines, call_idx)
    if arg_idx is None:
        return None
    value = _extract_push_string(lines[arg_idx])
    if value is None:
        return None
    if not _looks_like_visible_select_text(value):
        return None
    return _make_dialog_entry(
        name=None,
        message=value,
        name_line_index=None,
        message_line_index=arg_idx,
        is_select=False,
        call_line=call_idx + 1,
        user_func_name="sys_::f_11e",
        entry_type="ui",
    )


def _is_call_base(line, base_name, prefix=None):
    call_name = _extract_call_name(line)
    if not call_name:
        return False
    pfx, base = _split_qualified_name(call_name)
    if base != base_name:
        return False
    if prefix is not None and pfx != prefix:
        return False
    return True


def _arg_window_before_call(lines, call_idx, max_scan=96):
    """Return effective line indices belonging to the argument block before a no-arg call.

    This handles the common V1 form:
        push_string(...)
        push_dword(...)
        nargs(n)
        some_::f_xxx();
    and stops at the previous runtime line notification / label / business call.
    """
    cursor = _prev_effective_line(lines, call_idx)
    arg_indices = []
    scanned = 0
    while cursor is not None and scanned < max_scan:
        if _is_label_line(lines[cursor]):
            break
        stripped = lines[cursor].strip()
        if not stripped or stripped.startswith('//'):
            cursor = _prev_effective_line(lines, cursor)
            scanned += 1
            continue
        if _is_notify_line_count_call(lines[cursor]):
            break
        call_name = _extract_call_name(lines[cursor])
        if call_name and not _extract_nargs(lines[cursor]):
            break
        if not (stripped.startswith('push_') or _extract_nargs(lines[cursor]) is not None or _is_add(lines[cursor]) or _is_mul(lines[cursor])):
            break
        arg_indices.append(cursor)
        cursor = _prev_effective_line(lines, cursor)
        scanned += 1
    return list(reversed(arg_indices))


def _extract_visible_string_entries_from_call(lines, call_idx, *, call_bases, prefix=None, entry_type='ui', is_select=False, skip_first_ascii_resource=True):
    call_name = _extract_call_name(lines[call_idx])
    if not call_name:
        return []
    pfx, base = _split_qualified_name(call_name)
    if base not in set(call_bases):
        return []
    if prefix is not None and pfx != prefix:
        return []
    entries = []
    for idx in _arg_window_before_call(lines, call_idx):
        value = _extract_push_string(lines[idx])
        if value is None:
            continue
        if not _looks_like_visible_select_text(value):
            continue
        entry = _make_dialog_entry(
            name=None,
            message=value,
            name_line_index=None,
            message_line_index=idx,
            is_select=is_select,
            call_line=call_idx + 1,
            user_func_name=call_name,
            entry_type=entry_type,
        )
        if entry:
            entries.append(entry)
    return entries


def _extract_msg143_entry(lines, call_idx):
    """Extract msg_::f_143(), used by this title for simple one/two string messages.

    Observed forms:
        push_string("name"); push_string("message"); nargs(2); msg_::f_143();
        push_dword(0); push_string("message"); nargs(2); msg_::f_143();
    """
    if not _is_call_base(lines[call_idx], 'f_143', prefix='msg_') and not _is_call_base(lines[call_idx], 'f_143', prefix='msg'):
        return None
    msg_idx = _prev_effective_non_nargs_line(lines, call_idx)
    if msg_idx is None:
        return None
    message = _extract_push_string(lines[msg_idx])
    if message is None:
        return None
    prev_idx = _prev_effective_line(lines, msg_idx)
    name = None
    name_idx = None
    if prev_idx is not None:
        maybe_name = _extract_push_string(lines[prev_idx])
        if maybe_name is not None:
            name = maybe_name
            name_idx = prev_idx
    return _make_dialog_entry(
        name=name,
        message=message,
        name_line_index=name_idx,
        message_line_index=msg_idx,
        is_select=False,
        call_line=call_idx + 1,
        user_func_name='msg_::f_143',
    )


def _extract_preview_text_entry(lines, call_idx):
    # setup.bsc SetPreviewText: push_string(...); nargs(1); grp_::f_451();
    return _extract_visible_string_entries_from_call(
        lines,
        call_idx,
        call_bases={'f_451'},
        prefix='grp_',
        entry_type='ui',
    )


def _extract_default_caption_entry(lines, call_idx):
    # setup.bsc SetDefaultCaption: push_string(...); nargs(1); sys_::f_13f();
    return _extract_visible_string_entries_from_call(
        lines,
        call_idx,
        call_bases={'f_13f'},
        prefix='sys_',
        entry_type='ui',
    )


def _extract_name_setup_entries(lines, call_idx):
    """Extract visible strings from name/config registration calls in setup.bsc.

    These are not dialogue lines, but they are visible default names / random name pools / line-icon names.
    Resource arguments such as line_icon_NA are filtered by _looks_like_visible_select_text().
    """
    entries = []
    entries.extend(_extract_visible_string_entries_from_call(lines, call_idx, call_bases={'f_108', 'f_109'}, prefix='sys_', entry_type='ui'))
    entries.extend(_extract_visible_string_entries_from_call(lines, call_idx, call_bases={'f_448', 'f_449', 'f_469'}, prefix='grp_', entry_type='ui'))
    return entries


def _extract_function_concat_literal_entry(lines, call_idx):
    # function.bsc display fragments: push_string("　ナスカ:"); push_base_offset(...); ...; f_091();
    if not _is_call_base(lines[call_idx], 'f_091'):
        return None
    cursor = _prev_effective_line(lines, call_idx)
    scanned = 0
    while cursor is not None and scanned < 8:
        value = _extract_push_string(lines[cursor])
        if value is not None:
            if not _looks_like_visible_select_text(value):
                return None
            return _make_dialog_entry(
                name=None,
                message=value,
                name_line_index=None,
                message_line_index=cursor,
                is_select=False,
                call_line=call_idx + 1,
                user_func_name='f_091',
                entry_type='ui',
            )
        stripped = lines[cursor].strip()
        if not stripped.startswith('push_'):
            break
        cursor = _prev_effective_line(lines, cursor)
        scanned += 1
    return None

def _entry_dedupe_key(entry):
    return (
        entry.get("message_line_index"),
        entry.get("message_arg_index"),
        entry.get("call_line"),
        entry.get("message"),
        bool(entry.get("is_select")),
    )

def _is_select_option_string(lines, string_idx):
    message = _extract_push_string(lines[string_idx])
    if message is None:
        return None
    next_idx = string_idx + 1
    while next_idx < len(lines):
        stripped = lines[next_idx].strip()
        if stripped and not stripped.startswith('//'):
            break
        next_idx += 1
    if next_idx >= len(lines) or _extract_move_arity(lines[next_idx]) != 2:
        return None
    return _make_dialog_entry(
        name=None,
        message=message,
        name_line_index=None,
        message_line_index=string_idx,
        is_select=True
    )

def extract_dialog_entries(lines, user_function_names=None):
    entries = []
    seen_entries = set()
    normalized_user_functions = normalize_user_function_names(user_function_names)
    pending_name = None
    pending_name_idx = None
    pending_name_arg_index = None

    def add_entry(entry):
        if not entry:
            return
        key = _entry_dedupe_key(entry)
        if key in seen_entries:
            return
        seen_entries.add(key)
        entries.append(entry)

    for i, line in enumerate(lines):
        parsed = _extract_print_message_entry(lines, i)
        if parsed:
            add_entry(parsed)

        # V1/V0 选项：以选择系统调用为锚点，回溯参数块中的 push_string。
        for parsed_option in _extract_select_entries_from_stack_call(lines, i):
            add_entry(parsed_option)

        # BGI V1 常见封装：push_string("SelectEx"); f_01c();
        # 选项文本不直接挂在 slct::f_16x，而是位于 SelectEx 之前的参数构造块中。
        for parsed_option in _extract_direct_selectex_entries(lines, i):
            add_entry(parsed_option)

        # _SelectEx/SelectEx 的提示标题，例如「見たいシーンを選択してください」。
        add_entry(_extract_selectex_prompt_entry(lines, i))

        # AutoSaveRange 的章节/回想标题，例如「プロローグ」「共通・第一話（前編）」。
        add_entry(_extract_named_user_func_single_arg_entry(lines, i, {"AutoSaveRange"}, entry_type="ui"))

        # sys_::f_11e 的窗口/场景标题，例如「おまけ」。
        add_entry(_extract_sys_set_caption_entry(lines, i))

        # msg_::f_143 的简化对话/系统消息形态。
        add_entry(_extract_msg143_entry(lines, i))

        # setup/function 里的可见 UI、默认名、随机名池、预览文本等。
        for setup_entry in _extract_default_caption_entry(lines, i):
            add_entry(setup_entry)
        for setup_entry in _extract_preview_text_entry(lines, i):
            add_entry(setup_entry)
        for setup_entry in _extract_name_setup_entries(lines, i):
            add_entry(setup_entry)
        add_entry(_extract_function_concat_literal_entry(lines, i))

        # 兼容旧样式：push_string("選択肢"); move(2);
        parsed_option = _is_select_option_string(lines, i)
        if parsed_option:
            add_entry(parsed_option)

        user_func_entries = _extract_user_func_entries(lines, i, normalized_user_functions)
        if user_func_entries:
            for entry in user_func_entries:
                add_entry(entry)

        v0_call = _extract_v0_call(line)
        if not v0_call:
            continue
        func = v0_call["func"]
        literals = v0_call["literals"]
        if _matches_function_name(func, _function_aliases("f_SetName", "f_014")):
            if literals:
                pending_name = literals[0]["text"]
                pending_name_idx = i
                pending_name_arg_index = 0
            continue
        if _matches_function_name(func, _function_aliases("f_PrintMessage", "f_010")):
            if not literals:
                continue
            msg_arg_index = len(literals) - 1
            entry = _make_dialog_entry(
                name=pending_name,
                message=literals[msg_arg_index]["text"],
                name_line_index=pending_name_idx,
                message_line_index=i,
                name_arg_index=pending_name_arg_index,
                message_arg_index=msg_arg_index,
                is_select=False,
                call_line=i + 1
            )
            add_entry(entry)
            pending_name = None
            pending_name_idx = None
            pending_name_arg_index = None
            continue
        if _matches_function_name(func, _function_aliases("f_SetNextSelectingJumping", "f_0a9")):
            for idx_lit, lit in enumerate(literals):
                if not _looks_like_visible_select_text(lit["text"]):
                    continue
                entry = _make_dialog_entry(
                    name=None,
                    message=lit["text"],
                    name_line_index=None,
                    message_line_index=i,
                    message_arg_index=idx_lit,
                    is_select=True,
                    call_line=i + 1
                )
                add_entry(entry)
            continue
        if _matches_function_name(func, _function_aliases("f_Select", "f_0b0")):
            for idx_lit, lit in enumerate(literals):
                if not _looks_like_visible_select_text(lit["text"]):
                    continue
                entry = _make_dialog_entry(
                    name=None,
                    message=lit["text"],
                    name_line_index=None,
                    message_line_index=i,
                    message_arg_index=idx_lit,
                    is_select=True,
                    call_line=i + 1
                )
                add_entry(entry)
    return entries

def extract_dialog_json_from_bsd(input_path, output_json, user_function_names=None):
    with open(input_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    entries = extract_dialog_entries(lines, user_function_names=user_function_names)
    payload = []
    for e in entries:
        if e["name"] is not None and e["name"] != "":
            item = {"name": e["name"], "message": e["message"]}
        else:
            item = {"message": e["message"]}
        payload.append(item)
    out_dir = os.path.dirname(output_json)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    with open(output_json, 'w', encoding='utf-8') as f:
        json.dump(payload, f, ensure_ascii=False, indent=2)
    return len(entries)

def extract_push_string_entries_from_bpd(lines):
    push_entries = []
    string_entries = []
    in_strings = False
    for idx, line in enumerate(lines):
        stripped = line.strip()
        if stripped == '#strings':
            in_strings = True
            continue
        if not in_strings:
            text = _extract_push_string(line)
            if text is not None:
                push_entries.append({
                    "text": text,
                    "line_index": idx
                })
            continue
        if stripped.startswith('#strdata'):
            continue
        text = _extract_bpd_string_line(line)
        if text is not None:
            string_entries.append({
                "text": text,
                "line_index": idx
            })
    return push_entries, string_entries

def extract_push_string_json_from_bpd(input_bpd, output_json):
    with open(input_bpd, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    entries, _ = extract_push_string_entries_from_bpd(lines)
    payload = [{"text": entry["text"]} for entry in entries]
    out_dir = os.path.dirname(output_json)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    with open(output_json, 'w', encoding='utf-8') as f:
        json.dump(payload, f, ensure_ascii=False, indent=2)
    return len(entries)

def import_dialog_json_to_bsd(input_bsd, input_json, output_bsd, user_function_names=None):
    with open(input_bsd, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    with open(input_json, 'r', encoding='utf-8') as f:
        items = json.load(f)
    if not isinstance(items, list):
        raise Exception("JSON 格式错误: 需要数组格式")
    entries = extract_dialog_entries(lines, user_function_names=user_function_names)
    if len(items) != len(entries):
        raise Exception(f"JSON 条目数与对话条目数不一致: json={len(items)} bsd={len(entries)}")
    applied = 0
    for idx, item in enumerate(items):
        if not isinstance(item, dict):
            raise Exception(f"第 {idx} 项不是对象")
        unknown_keys = set(item.keys()) - {"name", "message"}
        if unknown_keys:
            raise Exception(f"第 {idx} 项包含非法字段: {sorted(unknown_keys)}")
        if "message" not in item:
            raise Exception(f"第 {idx} 项缺少 message")
        entry = entries[idx]
        if "name" in item and entry["name_line_index"] is not None:
            if entry.get("name_arg_index") is None:
                lines[entry["name_line_index"]] = _replace_push_string(lines[entry["name_line_index"]], str(item["name"]))
            else:
                lines[entry["name_line_index"]] = _replace_v0_call_string(
                    lines[entry["name_line_index"]],
                    int(entry["name_arg_index"]),
                    str(item["name"])
                )
            applied += 1
        message_text = _restore_message_suffix(item["message"], entry.get("message_suffix"))
        if entry.get("message_arg_index") is None:
            lines[entry["message_line_index"]] = _replace_push_string(lines[entry["message_line_index"]], message_text)
        else:
            lines[entry["message_line_index"]] = _replace_v0_call_string(
                lines[entry["message_line_index"]],
                int(entry["message_arg_index"]),
                message_text
            )
        applied += 1
    out_dir = os.path.dirname(output_bsd)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    with open(output_bsd, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    return len(entries), applied

def import_push_string_json_to_bpd(input_bpd, input_json, output_bpd):
    with open(input_bpd, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    with open(input_json, 'r', encoding='utf-8') as f:
        items = json.load(f)
    if not isinstance(items, list):
        raise Exception("JSON 格式错误: 需要数组格式")
    entries, string_entries = extract_push_string_entries_from_bpd(lines)
    if len(items) != len(entries):
        raise Exception(f"JSON 条目数与 push_string 条目数不一致: json={len(items)} bpd={len(entries)}")
    if string_entries and len(string_entries) != len(entries):
        raise Exception(f"BPD 字符串区条目数与 push_string 条目数不一致: strings={len(string_entries)} push={len(entries)}")
    applied = 0
    for idx, item in enumerate(items):
        if not isinstance(item, dict):
            raise Exception(f"第 {idx} 项不是对象")
        unknown_keys = set(item.keys()) - {"text"}
        if unknown_keys:
            raise Exception(f"第 {idx} 项包含非法字段: {sorted(unknown_keys)}")
        if "text" not in item:
            raise Exception(f"第 {idx} 项缺少 text")
        new_text = '' if item["text"] is None else str(item["text"])
        lines[entries[idx]["line_index"]] = _replace_push_string(lines[entries[idx]["line_index"]], new_text)
        if string_entries:
            lines[string_entries[idx]["line_index"]] = _replace_bpd_string_line(lines[string_entries[idx]["line_index"]], new_text)
        applied += 1
    out_dir = os.path.dirname(output_bpd)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
    with open(output_bpd, 'w', encoding='utf-8') as f:
        f.writelines(lines)
    return len(entries), applied

def extract_dialog_json_from_script(input_script, output_json, encoding='shift_jis', fallback_encoding='gbk', user_function_names=None):
    import bgidis
    with tempfile.TemporaryDirectory(prefix='bgi_json_extract_') as td:
        temp_bsd = os.path.join(td, os.path.basename(input_script) + '.bsd')
        bgidis.dis(input_script, encoding=encoding, fallback_encoding=fallback_encoding, output_path=temp_bsd)
        return extract_dialog_json_from_bsd(temp_bsd, output_json, user_function_names=user_function_names)

def import_dialog_json_to_script(
    input_script,
    input_json,
    output_script,
    encoding='shift_jis',
    fallback_encoding='gbk',
    source_encoding=None,
    source_fallback_encoding=None,
    user_function_names=None
):
    import bgidis
    import bgias
    dis_encoding = source_encoding or encoding
    dis_fallback = source_fallback_encoding or fallback_encoding
    with tempfile.TemporaryDirectory(prefix='bgi_json_import_') as td:
        base_name = os.path.basename(input_script)
        temp_src_bsd = os.path.join(td, base_name + '.src.bsd')
        temp_out_bsd = os.path.join(td, base_name + '.out.bsd')
        bgidis.dis(input_script, encoding=dis_encoding, fallback_encoding=dis_fallback, output_path=temp_src_bsd)
        count, applied = import_dialog_json_to_bsd(
            temp_src_bsd,
            input_json,
            temp_out_bsd,
            user_function_names=user_function_names
        )
        bgias.asm(temp_out_bsd, encoding=encoding, fallback_encoding=fallback_encoding, output_path=output_script)
        return count, applied
