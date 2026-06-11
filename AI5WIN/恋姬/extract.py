# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from ai5mes_common import (
    DEFAULT_ENCODING,
    RUNTIME_NAME_CALL_RAW,
    RUNTIME_NAME_TEXT,
    Instruction,
    MesProgram,
    TextSlot,
    decode_raw,
    is_text_file,
    json_dump,
    parse_mes,
    slot_message,
    slot_scr,
)


def first_opcode_text_slot(inst: Instruction) -> TextSlot | None:
    for s in inst.slots:
        if s.kind == "opcode_text":
            return s
    return None


def text_of(inst: Instruction | None, *, mode: str = "message") -> str | None:
    if inst is None:
        return None
    s = first_opcode_text_slot(inst)
    if s is None:
        return None
    return decode_raw(s.raw, mode=mode)


def is_newline_zero(inst: Instruction | None) -> bool:
    return inst is not None and inst.op == 0x11 and inst.meta.get("newline_arg") == 0


def is_msg_terminator(inst: Instruction) -> bool:
    # 样本中 0B 13 FF 00 是一条文本显示段的主要收束/等待调用。
    return inst.op == 0x0B and inst.meta.get("func_id") == 0x13


def collect_body_slots(program: MesProgram, start_i: int, *, max_scan: int = 80) -> tuple[list[TextSlot], int, list[dict[str, Any]]]:
    """从正文起点收集到文本收束调用前的 0x01 文本槽。

    返回：slots, end_i, inline_controls。
    inline_controls 记录夹在多个文本槽之间的运行时调用；用于在 scr_msg/message 预览中放占位符。
    """
    insts = program.instructions
    slots: list[TextSlot] = []
    controls: list[dict[str, Any]] = []
    i = start_i
    scanned = 0
    while i < len(insts) and scanned < max_scan:
        inst = insts[i]
        if is_msg_terminator(inst):
            return slots, i + 1, controls
        if inst.op in (0x0E, 0x13):
            # 菜单/等待选择不是普通正文。
            return slots, i, controls
        s = first_opcode_text_slot(inst)
        if inst.op == 0x01 and s is not None:
            slots.append(s)
        elif inst.op == 0x0F and slots:
            raw_bytes = program.data[inst.offset + 1:inst.end]
            raw = raw_bytes.hex(" ").upper()
            # 只有运行时男主名调用在文本中写死为“小十郎”；其它 0F 内联调用不是翻译文本，
            # 不再把 {{CALL:0F}} 之类占位符暴露到 message/scr_msg。原始 raw 仍保留在 _inline_controls。
            placeholder = RUNTIME_NAME_TEXT if raw_bytes == RUNTIME_NAME_CALL_RAW else ""
            controls.append({"after_part": len(slots) - 1, "op": "0F", "raw": raw, "placeholder": placeholder})
        i += 1
        scanned += 1
    return slots, i, controls


def join_slots_with_controls(slots: list[TextSlot], controls: list[dict[str, Any]], *, mode: str) -> str:
    parts: list[str] = []
    by_after: dict[int, list[str]] = {}
    for c in controls:
        by_after.setdefault(int(c.get("after_part", -1)), []).append(str(c.get("placeholder", "{{CALL}}")))
    for idx, slot in enumerate(slots):
        parts.append(slot_scr(slot) if mode == "scr" else slot_message(slot))
        for ph in by_after.get(idx, []):
            parts.append(ph)
    return "".join(parts)


def make_entry(program: MesProgram, slots: list[TextSlot], index: int, typ: str, controls: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    if not slots:
        raise ValueError("make_entry requires at least one slot")
    controls = controls or []
    first = slots[0]
    scr_parts = [slot_scr(s) for s in slots]
    msg_parts = [slot_message(s) for s in slots]
    e: dict[str, Any] = {
        "scr_msg": join_slots_with_controls(slots, controls, mode="scr"),
        "message": join_slots_with_controls(slots, controls, mode="message"),
        "_file": program.path.name,
        "_index": index,
        "_type": typ,
        "_opcode": f"{first.op:02X}",
        "_slot_id": first.slot_id,
        "_inst_offset": first.inst_offset,
        "_offset": first.raw_start,
        "_size": first.size,
        "_encoding": DEFAULT_ENCODING,
        "_policy": "relocate",
    }
    if len(slots) > 1 or controls:
        e["scr_msg_parts"] = scr_parts
        e["message_parts"] = msg_parts
        e["_slot_ids"] = [s.slot_id for s in slots]
        e["_part_offsets"] = [s.raw_start for s in slots]
        e["_part_sizes"] = [s.size for s in slots]
        if controls:
            e["_inline_controls"] = controls
    return e


def extract_entries(program: MesProgram) -> list[dict[str, Any]]:
    insts = program.instructions
    consumed_slots: set[int] = set()
    entries: list[dict[str, Any]] = []

    def append_entry(slots: list[TextSlot], typ: str, controls: list[dict[str, Any]] | None = None) -> dict[str, Any]:
        e = make_entry(program, slots, len(entries), typ, controls)
        entries.append(e)
        for s in slots:
            consumed_slots.add(s.slot_id)
        return e

    i = 0
    while i < len(insts):
        inst = insts[i]

        # 选择支：0E ... target 后通常紧跟 01 choice_text。
        if inst.op == 0x0E and i + 1 < len(insts):
            nxt = insts[i + 1]
            slot = first_opcode_text_slot(nxt)
            if nxt.op == 0x01 and slot is not None:
                e = append_entry([slot], "choice")
                if inst.jumps:
                    e["_target"] = f"0x{inst.jumps[0].old_target:08X}"
                e["_choice_inst_offset"] = inst.offset
                i += 2
                continue

        # 动态男主名：01 "【" + 0F runtime-name + 01 "】" + 11 00 + body
        if (
            inst.op == 0x01 and text_of(inst) == "【"
            and i + 4 < len(insts)
            and insts[i + 1].op == 0x0F
            and insts[i + 2].op == 0x01 and text_of(insts[i + 2]) == "】"
            and is_newline_zero(insts[i + 3])
        ):
            body_slots, end_i, controls = collect_body_slots(program, i + 4)
            if body_slots:
                e = append_entry(body_slots, "dialogue", controls)
                e2 = {"name": RUNTIME_NAME_TEXT}
                e2.update(e)
                name_expr = program.data[insts[i + 1].offset + 1:insts[i + 1].end].hex(" ").upper()
                e2["_name_source"] = "runtime_call_hardcoded"
                e2["_name_call_op"] = "0F"
                e2["_name_call_raw"] = name_expr
                e2["_runtime_name_text"] = RUNTIME_NAME_TEXT
                e2["_virtual_name"] = False
                entries[-1] = e2
                for k in (i, i + 2):
                    s = first_opcode_text_slot(insts[k])
                    if s:
                        consumed_slots.add(s.slot_id)
                i = max(end_i, i + 5)
                continue

        # 静态 name 行：01 "【xxx】" + 11 00 + body
        slot = first_opcode_text_slot(inst)
        if (
            inst.op == 0x01 and slot is not None
            and i + 2 < len(insts)
            and is_newline_zero(insts[i + 1])
        ):
            t = slot_message(slot)
            if len(t) >= 3 and t.startswith("【") and t.endswith("】"):
                body_slots, end_i, controls = collect_body_slots(program, i + 2)
                if body_slots:
                    e = append_entry(body_slots, "dialogue", controls)
                    e2 = {"name": t[1:-1]}
                    e2.update(e)
                    e2["_name_source"] = "static_text"
                    e2["_name_slot_id"] = slot.slot_id
                    e2["_name_scr"] = slot_scr(slot)
                    e2["_name_message"] = t
                    entries[-1] = e2
                    consumed_slots.add(slot.slot_id)
                    i = max(end_i, i + 3)
                    continue

        i += 1

    # 其余 0x01 主文本作为旁白/无名正文。跳过孤立括号和静态 name 行本体。
    for inst in insts:
        if inst.op != 0x01:
            continue
        slot = first_opcode_text_slot(inst)
        if slot is None or slot.slot_id in consumed_slots:
            continue
        msg = slot_message(slot)
        if msg in {"", "【", "】"}:
            consumed_slots.add(slot.slot_id)
            continue
        if len(msg) >= 3 and msg.startswith("【") and msg.endswith("】"):
            consumed_slots.add(slot.slot_id)
            continue
        append_entry([slot], "monologue")

    for idx, e in enumerate(entries):
        e["_index"] = idx
    return entries


def extract_path(input_path: Path, output_path: Path) -> tuple[int, int, int]:
    files = [input_path] if input_path.is_file() else sorted(p for p in input_path.rglob("*") if is_text_file(p))
    total_entries = 0
    warnings = 0
    for file in files:
        program = parse_mes(file)
        entries = extract_entries(program)
        total_entries += len(entries)
        if input_path.is_dir():
            rel = file.relative_to(input_path).as_posix().replace("/", "__")
            out = output_path / f"{rel}.json"
        else:
            out = output_path if output_path.suffix.lower() == ".json" else output_path / f"{file.name}.json"
        json_dump(out, entries)
    return len(files), total_entries, warnings


def main() -> None:
    ap = argparse.ArgumentParser(description="AI5WIN MES 结构化文本提取，输出统一 JSON")
    ap.add_argument("input", help="MES 文件或 MES 目录")
    ap.add_argument("output", help="输出 JSON 文件或目录")
    args = ap.parse_args()
    files, entries, warnings = extract_path(Path(args.input), Path(args.output))
    print(f"[extract] scanned_files={files}")
    print(f"[extract] extracted_entries={entries}")
    print(f"[extract] warnings={warnings}")
    print(f"[extract] output={args.output}")


if __name__ == "__main__":
    main()
