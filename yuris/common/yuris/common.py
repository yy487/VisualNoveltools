# -*- coding: utf-8 -*-
"""YU-RIS 文本提取/注入共用结构。"""
from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable

DEFAULT_ENCODING = "cp932"


@dataclass
class TextEntry:
    """统一文本条目。

    scr_msg 只保存原始脚本文本，用于注入定位和校验；message 是实际写回字段。
    下划线字段是工具内部定位字段，翻译时不要修改。
    """

    scr_msg: str
    message: str
    _file: str
    _index: int
    name: str | None = None
    _source: str | None = None
    _script_id: int | None = None
    _cmd_index: int | None = None
    _cmd_offset: int | None = None
    _expr_index: int | None = None
    _expr_id: int | None = None
    _expr_offset: int | None = None
    _expr_size: int | None = None
    _offset: int | None = None
    _is_option: bool | None = None
    _extract_source: str | None = None
    _text_format: str | None = None
    _opcode: str | None = None
    _opcode_id: int | None = None
    _type: str | None = None
    _label: str | None = None

    def to_json_obj(self) -> dict[str, Any]:
        obj = asdict(self)
        if obj.get("name") is None:
            obj.pop("name", None)
        return {k: v for k, v in obj.items() if v is not None}


def read_u16_le(data: bytes | bytearray | memoryview, off: int) -> int:
    return int.from_bytes(data[off:off + 2], "little", signed=False)


def read_i32_le(data: bytes | bytearray | memoryview, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little", signed=True)


def read_u32_le(data: bytes | bytearray | memoryview, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little", signed=False)


def write_u16_le(buf: bytearray, off: int, value: int) -> None:
    buf[off:off + 2] = int(value).to_bytes(2, "little", signed=False)


def write_i32_le(buf: bytearray, off: int, value: int) -> None:
    buf[off:off + 4] = int(value).to_bytes(4, "little", signed=True)


def write_u32_le(buf: bytearray, off: int, value: int) -> None:
    buf[off:off + 4] = int(value).to_bytes(4, "little", signed=False)


def decode_text(data: bytes, encoding: str = DEFAULT_ENCODING, *, errors: str = "strict") -> str:
    return data.decode(encoding, errors=errors)


def encode_text(text: str, encoding: str = DEFAULT_ENCODING, *, errors: str = "strict") -> bytes:
    return text.encode(encoding, errors=errors)


def is_encodable(text: str, encoding: str = DEFAULT_ENCODING) -> bool:
    try:
        text.encode(encoding)
        return True
    except UnicodeEncodeError:
        return False


def read_cstring(data: bytes | bytearray, off: int, encoding: str = DEFAULT_ENCODING) -> tuple[str, int]:
    end = off
    while end < len(data) and data[end] != 0:
        end += 1
    return bytes(data[off:end]).decode(encoding), end + 1


def load_json(path: str | Path) -> list[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON 顶层必须是 list: {path}")
    return data


def save_json(path: str | Path, entries: Iterable[TextEntry | dict[str, Any]]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    data: list[dict[str, Any]] = []
    for entry in entries:
        if isinstance(entry, TextEntry):
            data.append(entry.to_json_obj())
        else:
            data.append({k: v for k, v in entry.items() if v is not None})
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


def iter_ybn_files(path: str | Path) -> list[Path]:
    p = Path(path)
    if p.is_file():
        return [p]
    return sorted(x for x in p.rglob("*.ybn") if x.is_file())


def rel_file_name(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.name


def warn(msg: str) -> None:
    print(f"[warn] {msg}")
