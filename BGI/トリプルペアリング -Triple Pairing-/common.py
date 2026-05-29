# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import os
import re
import tempfile
from pathlib import Path
from typing import Any, Iterable

import bgias
import bgidis
from bgi_dialog_json import (
    extract_dialog_entries,
    _replace_push_string,
    _replace_v0_call_string,
    _restore_message_suffix,
)

DEFAULT_ENCODING = "shift_jis"
DEFAULT_FALLBACK_ENCODING = "gbk"
JSON_SUFFIX = ".json"


def read_text_lines(path: str | Path) -> list[str]:
    with open(path, "r", encoding="utf-8") as f:
        return f.readlines()


def write_text_lines(path: str | Path, lines: list[str]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.writelines(lines)


def load_json_entries(path: str | Path) -> list[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON 格式错误，需要数组: {path}")
    return data


def save_json_entries(path: str | Path, entries: list[dict[str, Any]]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def normalize_rel_path(path: Path, root: Path | None = None) -> str:
    p = path if root is None else path.relative_to(root)
    return p.as_posix()


def json_path_for_source(source_path: Path, source_root: Path | None, json_root: Path) -> Path:
    rel = normalize_rel_path(source_path, source_root)
    return json_root / (rel + JSON_SUFFIX)


def output_path_for_source(source_path: Path, source_root: Path | None, output_root: Path) -> Path:
    rel = normalize_rel_path(source_path, source_root)
    return output_root / rel


def is_probably_bsd(path: Path) -> bool:
    return path.suffix.lower() == ".bsd"


def is_probably_script(path: Path) -> bool:
    # BGI V1 编译脚本常见为无扩展名；也兼容用户显式传入单文件。
    if path.is_dir():
        return False
    return path.suffix == ""


def iter_sources(input_path: str | Path, mode: str) -> tuple[Path | None, list[Path]]:
    root = Path(input_path)
    if root.is_file():
        return None, [root]
    if not root.is_dir():
        raise FileNotFoundError(str(root))

    mode = mode.lower()
    files: list[Path] = []
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        name = p.name
        if name.startswith("."):
            continue
        if mode == "bsd":
            if is_probably_bsd(p):
                files.append(p)
        elif mode == "script":
            if is_probably_script(p):
                files.append(p)
        elif mode == "auto":
            if is_probably_bsd(p) or is_probably_script(p):
                files.append(p)
        else:
            raise ValueError(f"未知 mode: {mode}")
    return root, files


def disassemble_script_to_bsd(
    script_path: str | Path,
    output_bsd: str | Path,
    *,
    encoding: str = DEFAULT_ENCODING,
    fallback_encoding: str = DEFAULT_FALLBACK_ENCODING,
) -> None:
    bgidis.dis(
        str(script_path),
        exact_mode=False,
        strout_mode=False,
        encoding=encoding,
        fallback_encoding=fallback_encoding,
        output_path=str(output_bsd),
    )


def assemble_bsd_to_script(
    bsd_path: str | Path,
    output_script: str | Path,
    *,
    encoding: str = DEFAULT_ENCODING,
    fallback_encoding: str = DEFAULT_FALLBACK_ENCODING,
) -> None:
    output_script = Path(output_script)
    output_script.parent.mkdir(parents=True, exist_ok=True)
    bgias.asm(
        str(bsd_path),
        encoding=encoding,
        fallback_encoding=fallback_encoding,
        output_path=str(output_script),
    )


def _entry_type(engine_entry: dict[str, Any]) -> str:
    explicit = engine_entry.get("entry_type")
    if explicit:
        return str(explicit)
    if engine_entry.get("is_select"):
        return "choice"
    if engine_entry.get("name"):
        return "dialogue"
    return "message"


def convert_engine_entries_to_workflow(
    engine_entries: list[dict[str, Any]],
    *,
    file_name: str,
) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for index, e in enumerate(engine_entries):
        text = str(e.get("message", ""))
        name = e.get("name")
        item: dict[str, Any] = {}
        if name is not None and name != "":
            item["name"] = str(name)
        item.update({
            "scr_msg": text,
            "message": text,
            "_file": file_name,
            "_index": index,
            "_type": _entry_type(e),
            "_line": int(e.get("message_line_index", -1)) + 1,
        })
        if e.get("name_line_index") is not None:
            item["_name_line"] = int(e["name_line_index"]) + 1
        if e.get("message_arg_index") is not None:
            item["_message_arg_index"] = int(e["message_arg_index"])
        if e.get("name_arg_index") is not None:
            item["_name_arg_index"] = int(e["name_arg_index"])
        if e.get("call_line") is not None:
            item["_call_line"] = int(e["call_line"])
        if e.get("user_func_name") is not None:
            item["_user_func"] = str(e["user_func_name"])
        suffix = e.get("message_suffix")
        if suffix:
            item["_message_suffix"] = str(suffix)
        out.append(item)
    return out


def extract_workflow_entries_from_bsd(
    bsd_path: str | Path,
    *,
    file_name: str | None = None,
    user_function_names: Iterable[str] | str | None = None,
) -> list[dict[str, Any]]:
    path = Path(bsd_path)
    lines = read_text_lines(path)
    engine_entries = extract_dialog_entries(lines, user_function_names=user_function_names)
    return convert_engine_entries_to_workflow(
        engine_entries,
        file_name=file_name or path.name,
    )


def extract_workflow_entries_from_script(
    script_path: str | Path,
    *,
    file_name: str | None = None,
    encoding: str = DEFAULT_ENCODING,
    fallback_encoding: str = DEFAULT_FALLBACK_ENCODING,
    user_function_names: Iterable[str] | str | None = None,
) -> list[dict[str, Any]]:
    with tempfile.TemporaryDirectory(prefix="bgi_v1_extract_") as td:
        temp_bsd = Path(td) / (Path(script_path).name + ".bsd")
        disassemble_script_to_bsd(
            script_path,
            temp_bsd,
            encoding=encoding,
            fallback_encoding=fallback_encoding,
        )
        return extract_workflow_entries_from_bsd(
            temp_bsd,
            file_name=file_name or Path(script_path).name,
            user_function_names=user_function_names,
        )


def _index_json_by_file(items: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = {}
    for item in items:
        if not isinstance(item, dict):
            continue
        file_name = str(item.get("_file", ""))
        grouped.setdefault(file_name, []).append(item)
    return grouped


def _single_file_items(items: list[dict[str, Any]], file_name: str) -> list[dict[str, Any]]:
    by_file = _index_json_by_file(items)
    if file_name in by_file:
        return by_file[file_name]
    # 兼容用户手动合并或旧 JSON：没有 _file 时认为全部属于当前文件。
    no_file = [x for x in items if isinstance(x, dict) and not x.get("_file")]
    return no_file if no_file else items


def _unique_scr_msg_fallback(engine_entries: list[dict[str, Any]]) -> dict[str, int | None]:
    pos: dict[str, int | None] = {}
    for idx, e in enumerate(engine_entries):
        msg = str(e.get("message", ""))
        if msg in pos:
            pos[msg] = None
        else:
            pos[msg] = idx
    return pos


def _replace_engine_entry_text(lines: list[str], engine_entry: dict[str, Any], new_text: str) -> bool:
    msg_line = engine_entry.get("message_line_index")
    if msg_line is None:
        return False
    message_text = _restore_message_suffix(new_text, engine_entry.get("message_suffix"))
    arg_idx = engine_entry.get("message_arg_index")
    if arg_idx is None:
        lines[int(msg_line)] = _replace_push_string(lines[int(msg_line)], message_text)
    else:
        lines[int(msg_line)] = _replace_v0_call_string(lines[int(msg_line)], int(arg_idx), message_text)
    return True


def _replace_engine_entry_name(lines: list[str], engine_entry: dict[str, Any], new_name: str) -> bool:
    name_line = engine_entry.get("name_line_index")
    if name_line is None:
        return False
    arg_idx = engine_entry.get("name_arg_index")
    if arg_idx is None:
        lines[int(name_line)] = _replace_push_string(lines[int(name_line)], new_name)
    else:
        lines[int(name_line)] = _replace_v0_call_string(lines[int(name_line)], int(arg_idx), new_name)
    return True


def patch_bsd_with_workflow_json(
    input_bsd: str | Path,
    json_path: str | Path,
    output_bsd: str | Path,
    *,
    file_name: str | None = None,
    user_function_names: Iterable[str] | str | None = None,
    strict: bool = False,
) -> dict[str, int]:
    path = Path(input_bsd)
    lines = read_text_lines(path)
    engine_entries = extract_dialog_entries(lines, user_function_names=user_function_names)
    json_items = load_json_entries(json_path)
    target_file = file_name or path.name
    items = _single_file_items(json_items, target_file)
    fallback_map = _unique_scr_msg_fallback(engine_entries)

    patched = 0
    patched_name = 0
    skipped = 0
    failed = 0
    warnings = 0

    used_targets: set[int] = set()
    for item in items:
        if not isinstance(item, dict):
            failed += 1
            continue
        if "message" not in item or "scr_msg" not in item:
            failed += 1
            warnings += 1
            print(f"[inject][warn] 缺少 scr_msg/message，跳过: {item}")
            if strict:
                raise ValueError("JSON 条目缺少 scr_msg/message")
            continue

        scr_msg = str(item["scr_msg"])
        new_msg = str(item["message"])
        index = item.get("_index")
        target_idx: int | None = None

        if isinstance(index, int) and 0 <= index < len(engine_entries):
            if str(engine_entries[index].get("message", "")) == scr_msg:
                target_idx = index
            else:
                warnings += 1
                print(
                    f"[inject][warn] _index 校验失败 file={target_file} index={index}\n"
                    f"  json scr_msg: {scr_msg}\n"
                    f"  file text : {engine_entries[index].get('message', '')}"
                )
        if target_idx is None:
            fb = fallback_map.get(scr_msg)
            if isinstance(fb, int):
                target_idx = fb
                warnings += 1
                print(f"[inject][warn] 使用同文件唯一 scr_msg fallback: file={target_file} index={fb}")

        if target_idx is None or target_idx in used_targets:
            failed += 1
            warnings += 1
            print(f"[inject][warn] 无法定位或重复定位，跳过 file={target_file}: {scr_msg}")
            if strict:
                raise ValueError(f"无法定位: {scr_msg}")
            continue
        used_targets.add(target_idx)
        engine_entry = engine_entries[target_idx]

        old_msg = str(engine_entry.get("message", ""))
        if new_msg == scr_msg and item.get("name") == engine_entry.get("name"):
            skipped += 1
            continue

        if new_msg != old_msg or new_msg != scr_msg:
            if _replace_engine_entry_text(lines, engine_entry, new_msg):
                patched += 1
            else:
                failed += 1
                warnings += 1
                print(f"[inject][warn] 目标 message 行无效 file={target_file} index={target_idx}")
                continue

        if "name" in item and engine_entry.get("name_line_index") is not None:
            new_name = str(item.get("name", ""))
            if new_name != str(engine_entry.get("name", "")):
                if _replace_engine_entry_name(lines, engine_entry, new_name):
                    patched_name += 1

    write_text_lines(output_bsd, lines)
    return {
        "entries": len(engine_entries),
        "json_items": len(items),
        "patched_message": patched,
        "patched_name": patched_name,
        "skipped": skipped,
        "failed": failed,
        "warnings": warnings,
    }


def patch_script_with_workflow_json(
    input_script: str | Path,
    json_path: str | Path,
    output_script: str | Path,
    *,
    file_name: str | None = None,
    encoding: str = DEFAULT_ENCODING,
    fallback_encoding: str = DEFAULT_FALLBACK_ENCODING,
    user_function_names: Iterable[str] | str | None = None,
    strict: bool = False,
) -> dict[str, int]:
    with tempfile.TemporaryDirectory(prefix="bgi_v1_inject_") as td:
        td_path = Path(td)
        temp_src = td_path / (Path(input_script).name + ".src.bsd")
        temp_out = td_path / (Path(input_script).name + ".out.bsd")
        disassemble_script_to_bsd(
            input_script,
            temp_src,
            encoding=encoding,
            fallback_encoding=fallback_encoding,
        )
        stats = patch_bsd_with_workflow_json(
            temp_src,
            json_path,
            temp_out,
            file_name=file_name or Path(input_script).name,
            user_function_names=user_function_names,
            strict=strict,
        )
        assemble_bsd_to_script(
            temp_out,
            output_script,
            encoding=encoding,
            fallback_encoding=fallback_encoding,
        )
        return stats


def parse_user_functions(raw: str | None) -> list[str]:
    if not raw:
        return []
    return [x.strip() for x in re.split(r"[\r\n,;]+", raw) if x.strip()]
