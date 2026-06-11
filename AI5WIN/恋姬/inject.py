# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import bisect
import shutil
from pathlib import Path
from typing import Any

from ai5mes_common import (
    DEFAULT_ENCODING,
    RUNTIME_NAME_CALL_RAW,
    RUNTIME_NAME_TEXT,
    MesProgram,
    TextSlot,
    encode_message_text,
    is_text_file,
    json_load,
    parse_mes,
    slot_scr,
    write_u32_le,
)

FULLWIDTH_SPACE = "　"


def collect_json_entries(json_path: Path) -> dict[str, list[dict[str, Any]]]:
    result: dict[str, list[dict[str, Any]]] = {}
    if json_path.is_file():
        data = json_load(json_path)
        for e in data:
            fn = e.get("_file")
            if isinstance(fn, str):
                result.setdefault(fn, []).append(e)
        return result
    for jp in sorted(json_path.rglob("*.json")):
        data = json_load(jp)
        for e in data:
            fn = e.get("_file")
            if isinstance(fn, str):
                result.setdefault(fn, []).append(e)
    return result


def ruby_blank_replacements(program: MesProgram) -> dict[int, bytes]:
    repl: dict[int, bytes] = {}
    for inst in program.instructions:
        if inst.op == 0x0B and inst.meta.get("func_id") == 0x18:
            for slot in inst.slots:
                if slot.kind == "arg_text":
                    # 按字符数替换成全角空格；结构随整文件重建自动变长。
                    try:
                        chars = slot.raw.decode(DEFAULT_ENCODING)
                        count = len(chars)
                    except UnicodeDecodeError:
                        count = max(1, len(slot.raw) // 2)
                    repl[slot.slot_id] = (FULLWIDTH_SPACE * count).encode(DEFAULT_ENCODING)
    return repl




def runtime_name_call_replacements(program: MesProgram) -> list[tuple[int, int, bytes, str]]:
    """把 0F 02 00 FF 00 运行时名字调用直接替换为 TEXT "小十郎"。

    返回通用 span 替换：(old_start, old_end, new_bytes, label)。
    这不是尾部追加，而是在重建整文件时把原指令段替换成新的 TEXT 指令。
    """
    new_inst = b"\x01" + RUNTIME_NAME_TEXT.encode(DEFAULT_ENCODING) + b"\x00"
    spans: list[tuple[int, int, bytes, str]] = []
    for inst in program.instructions:
        if inst.op == 0x0F and program.data[inst.offset + 1:inst.end] == RUNTIME_NAME_CALL_RAW:
            spans.append((inst.offset, inst.end, new_inst, "runtime-name-call"))
    return spans


def derive_message_parts_from_inline_controls(e: dict[str, Any], slot_count: int) -> list[str] | None:
    """当正文中夹着已经写死的运行时名字（如 小十郎）时，优先按顶层 message 回拆。

    旧版 JSON 会同时带 message 和 message_parts。翻译者通常只改 message，
    如果仍然盲用 message_parts，就会出现“小十郎周围的译文没有注回”的问题。
    这里把 message 视作：part0 + placeholder + part1 + ...，再回拆给各 TEXT 槽。
    只有占位符足以作为分隔符时才返回；否则返回 None，走原 message_parts。
    """
    message = e.get("message")
    controls = e.get("_inline_controls")
    if not isinstance(message, str) or not isinstance(controls, list) or slot_count <= 0:
        return None

    by_after: dict[int, list[str]] = {}
    for c in controls:
        if not isinstance(c, dict):
            continue
        try:
            after = int(c.get("after_part"))
        except (TypeError, ValueError):
            continue
        ph = c.get("placeholder")
        # 只有明确写死到 message 里的占位文本才可用于回拆；空 placeholder 表示未知 0F，不能参与。
        if isinstance(ph, str) and ph:
            by_after.setdefault(after, []).append(ph)

    if not by_after:
        return None

    parts: list[str | None] = [None] * slot_count
    pos = 0
    used_any = False
    for idx in range(slot_count):
        ph = "".join(by_after.get(idx, []))
        if not ph:
            # 如果两个相邻 TEXT 槽之间没有可见分隔符，无法仅靠顶层 message 安全回拆。
            if idx < slot_count - 1 and idx not in by_after:
                return None
            continue
        hit = message.find(ph, pos)
        if hit < 0:
            return None
        parts[idx] = message[pos:hit]
        pos = hit + len(ph)
        used_any = True

    if not used_any:
        return None

    # 最后一个未赋值的槽吃掉剩余文本；通常就是“小十郎”后的正文。
    last_unset = None
    for i in range(slot_count - 1, -1, -1):
        if parts[i] is None:
            last_unset = i
            break
    if last_unset is None:
        if message[pos:]:
            return None
    else:
        parts[last_unset] = message[pos:]

    if any(p is None for p in parts):
        return None
    return [p for p in parts if p is not None]



def joined_message_from_parts_and_controls(e: dict[str, Any], msg_parts: list[Any]) -> str | None:
    """按导出时的规则把 message_parts + 可见 inline placeholder 拼回顶层 message。

    用于检测：翻译者只改了顶层 message，但当前条目无法安全回拆到多个 TEXT 槽。
    """
    if not all(isinstance(x, str) for x in msg_parts):
        return None
    controls = e.get("_inline_controls")
    by_after: dict[int, list[str]] = {}
    if isinstance(controls, list):
        for c in controls:
            if not isinstance(c, dict):
                continue
            try:
                after = int(c.get("after_part"))
            except (TypeError, ValueError):
                continue
            ph = c.get("placeholder", "")
            if isinstance(ph, str) and ph:
                by_after.setdefault(after, []).append(ph)
    out: list[str] = []
    for idx, part in enumerate(msg_parts):
        out.append(part)
        out.extend(by_after.get(idx, []))
    return "".join(out)

def build_replacements(program: MesProgram, entries: list[dict[str, Any]], *, inject_names: bool) -> tuple[dict[int, bytes], int, int, list[str]]:
    slot_map = program.slot_by_id
    replacements = ruby_blank_replacements(program)
    patched = 0
    failed = 0
    warnings: list[str] = []

    for e in entries:
        slot_ids_obj = e.get("_slot_ids")
        if isinstance(slot_ids_obj, list) and slot_ids_obj:
            slot_ids = slot_ids_obj
            scr_parts = e.get("scr_msg_parts")
            msg_parts = e.get("message_parts")
            if not isinstance(scr_parts, list) or not isinstance(msg_parts, list) or len(scr_parts) != len(slot_ids) or len(msg_parts) != len(slot_ids):
                failed += 1
                warnings.append(f"index={e.get('_index')} multi-part entry requires scr_msg_parts/message_parts with same length as _slot_ids")
                continue
            ok = True
            for sid, scr_part in zip(slot_ids, scr_parts):
                if not isinstance(sid, int) or sid not in slot_map or not isinstance(scr_part, str):
                    ok = False
                    warnings.append(f"index={e.get('_index')} invalid part slot/scr: {sid}")
                    break
                actual = slot_scr(slot_map[sid])
                if actual != scr_part:
                    ok = False
                    warnings.append(f"slot={sid} scr_msg_part mismatch\n  json={scr_part}\n  file={actual}")
                    break
            if not ok:
                failed += 1
                continue
            # 强制注入策略：
            # 1) 所有条目优先使用顶层 message，避免翻译者只改 message 时被旧的 message_parts 覆盖。
            # 2) 多槽且含可见占位符（小十郎）时，尽量按顶层 message 回拆，保留原有控制调用位置。
            # 3) 其它多槽无法可靠回拆时，强制把完整顶层 message 写入第一个 TEXT 槽，
            #    后续 TEXT 槽清空。这样不会静默写回原文，也不做尾部追加；代价是中间未知控制仍会原样执行。
            top_message = e.get("message")
            force_flattened = False
            if isinstance(top_message, str):
                if len(slot_ids) == 1:
                    msg_parts_to_write = [top_message]
                else:
                    derived_msg_parts = derive_message_parts_from_inline_controls(e, len(slot_ids))
                    if derived_msg_parts is not None:
                        msg_parts_to_write = derived_msg_parts
                    else:
                        msg_parts_to_write = [top_message] + [""] * (len(slot_ids) - 1)
                        force_flattened = True
            else:
                msg_parts_to_write = msg_parts

            try:
                for sid, msg_part in zip(slot_ids, msg_parts_to_write):
                    if not isinstance(msg_part, str):
                        raise TypeError("message_parts element must be string")
                    replacements[sid] = encode_message_text(msg_part)
            except (UnicodeEncodeError, TypeError) as ex:
                failed += 1
                warnings.append(f"index={e.get('_index')} encode failed: {ex}")
                continue
            # force_flattened 属于预期行为，不默认刷 warning；真正的校验/编码失败才报警。
            patched += 1
        else:
            slot_id = e.get("_slot_id")
            scr_msg = e.get("scr_msg")
            message = e.get("message")
            if not isinstance(slot_id, int) or slot_id not in slot_map:
                failed += 1
                warnings.append(f"index={e.get('_index')} missing/invalid _slot_id: {slot_id}")
                continue
            if not isinstance(scr_msg, str) or not isinstance(message, str):
                failed += 1
                warnings.append(f"slot={slot_id} missing scr_msg/message")
                continue
            slot = slot_map[slot_id]
            actual = slot_scr(slot)
            if actual != scr_msg:
                failed += 1
                warnings.append(
                    f"slot={slot_id} scr_msg mismatch\n  json={scr_msg}\n  file={actual}"
                )
                continue
            try:
                replacements[slot_id] = encode_message_text(message)
            except UnicodeEncodeError as ex:
                failed += 1
                warnings.append(f"slot={slot_id} encode failed: {ex}")
                continue
            patched += 1

        # 静态 name 行可选注回。动态 name 跳过。
        if inject_names and e.get("_name_source") == "static_text" and isinstance(e.get("_name_slot_id"), int):
            name_slot_id = e["_name_slot_id"]
            if name_slot_id in slot_map and isinstance(e.get("name"), str):
                name_slot = slot_map[name_slot_id]
                expected_name_scr = e.get("_name_scr")
                if isinstance(expected_name_scr, str) and slot_scr(name_slot) != expected_name_scr:
                    failed += 1
                    warnings.append(f"name slot={name_slot_id} scr mismatch; skip name injection")
                else:
                    try:
                        replacements[name_slot_id] = encode_message_text(f"【{e['name']}】")
                    except UnicodeEncodeError as ex:
                        failed += 1
                        warnings.append(f"name slot={name_slot_id} encode failed: {ex}")
    return replacements, patched, failed, warnings


def rebuild_program(program: MesProgram, replacements_by_slot: dict[int, bytes], extra_spans: list[tuple[int, int, bytes, str]] | None = None) -> tuple[bytes, list[str]]:
    data = program.data
    warnings: list[str] = []

    # raw span 替换表：只替换 C-string 内容，不替换 NUL。
    spans: list[tuple[int, int, bytes, str]] = []
    for slot_id, new_raw in replacements_by_slot.items():
        slot = program.slot_by_id.get(slot_id)
        if slot is None:
            warnings.append(f"replacement slot_id not found: {slot_id}")
            continue
        spans.append((slot.raw_start, slot.raw_end, new_raw, f"slot:{slot_id}"))
    if extra_spans:
        spans.extend(extra_spans)
    spans.sort(key=lambda x: x[0])

    # 检查重叠。
    last_end = -1
    for start, end, _raw, label in spans:
        if start < last_end:
            raise ValueError(f"overlapped replacement around {label}, offset=0x{start:X}")
        last_end = end

    out = bytearray()
    old_points: list[int] = []
    new_points: list[int] = []
    pos = 0
    for start, end, new_raw, _label in spans:
        out.extend(data[pos:start])
        old_points.append(start)
        new_points.append(len(out))
        out.extend(new_raw)
        pos = end
        old_points.append(end)
        new_points.append(len(out))
    out.extend(data[pos:])
    old_points.append(len(data))
    new_points.append(len(out))

    def old_to_new(old_off: int) -> int:
        # 对任意旧 offset，按它之前完成的替换累计 delta。
        idx = bisect.bisect_right(old_points, old_off) - 1
        if idx < 0:
            return old_off
        # 若正好处在记录点之后的普通区域，delta 恒定。
        delta = new_points[idx] - old_points[idx]
        return old_off + delta

    # 修正所有绝对跳转目标和 target 字段。
    for jr in program.jumps:
        new_operand = old_to_new(jr.operand_offset)
        new_target = old_to_new(jr.old_target)
        if new_operand < 0 or new_operand + 4 > len(out):
            raise ValueError(f"jump operand relocated out of range: old=0x{jr.operand_offset:X}, new=0x{new_operand:X}")
        write_u32_le(out, new_operand, new_target)

    return bytes(out), warnings


def inject_one_file(src: Path, out: Path, entries: list[dict[str, Any]], *, inject_names: bool) -> tuple[int, int, int, list[str]]:
    program = parse_mes(src)
    replacements, patched, failed, warnings = build_replacements(program, entries, inject_names=inject_names)
    extra_spans = runtime_name_call_replacements(program)
    rebuilt, rwarn = rebuild_program(program, replacements, extra_spans)
    warnings.extend(rwarn)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(rebuilt)
    return patched, failed, len(replacements), warnings


def main() -> None:
    ap = argparse.ArgumentParser(description="AI5WIN MES JSON 注入；整文件文本池/指令流重建，不做尾部追加")
    ap.add_argument("input", help="原始 MES 文件或目录")
    ap.add_argument("json", help="翻译 JSON 文件或目录")
    ap.add_argument("output", help="输出 MES 文件或目录")
    ap.add_argument("--no-inject-names", action="store_true", help="不把静态 name 字段注回 name 行")
    ap.add_argument("--copy-unmatched", action="store_true", help="目录模式下复制没有对应 JSON 的 MES 文件")
    args = ap.parse_args()

    input_path = Path(args.input)
    json_path = Path(args.json)
    output_path = Path(args.output)
    by_file = collect_json_entries(json_path)
    inject_names = not args.no_inject_names

    total_files = total_patched = total_failed = total_repl = total_warn = 0
    if input_path.is_file():
        entries = by_file.get(input_path.name, [])
        if not entries and json_path.is_file():
            entries = json_load(json_path)
        patched, failed, repl, warnings = inject_one_file(input_path, output_path, entries, inject_names=inject_names)
        total_files = 1
        total_patched += patched
        total_failed += failed
        total_repl += repl
        for w in warnings:
            print(f"[inject][warn] {input_path.name}: {w}")
        total_warn += len(warnings)
    else:
        files = sorted(p for p in input_path.rglob("*") if is_text_file(p))
        for src in files:
            rel = src.relative_to(input_path)
            out = output_path / rel
            entries = by_file.get(src.name, [])
            if entries:
                patched, failed, repl, warnings = inject_one_file(src, out, entries, inject_names=inject_names)
                total_patched += patched
                total_failed += failed
                total_repl += repl
                total_warn += len(warnings)
                for w in warnings:
                    print(f"[inject][warn] {src.name}: {w}")
            elif args.copy_unmatched:
                out.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, out)
            total_files += 1

    print(f"[inject] scanned_files={total_files}")
    print(f"[inject] patched_entries={total_patched}")
    print(f"[inject] failed_entries={total_failed}")
    print(f"[inject] rebuilt_slots={total_repl}")
    print(f"[inject] warnings={total_warn}")
    print(f"[inject] output={args.output}")


if __name__ == "__main__":
    main()
