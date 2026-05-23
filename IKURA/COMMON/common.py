# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable

DEFAULT_ENCODING = "cp932"
TEXT_OPS = {0x2B, 0x2C}
CONTROL_AFTER_ZERO = {0x00, 0x01, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x10, 0x11, 0xFF}

# IKURA/ISF 的 0x00-0x7E 单字节压缩假名字典。
IKUAR_KANA = bytes([
    0x81, 0x40, 0x81, 0x40, 0x81, 0x41, 0x81, 0x42, 0x81, 0x45, 0x81, 0x48, 0x81, 0x49, 0x81, 0x69,
    0x81, 0x6a, 0x81, 0x75, 0x81, 0x76, 0x82, 0x4f, 0x82, 0x50, 0x82, 0x51, 0x82, 0x52, 0x82, 0x53,
    0x82, 0x54, 0x82, 0x55, 0x82, 0x56, 0x82, 0x57, 0x82, 0x58, 0x82, 0xa0, 0x82, 0xa2, 0x82, 0xa4,
    0x82, 0xa6, 0x82, 0xa8, 0x82, 0xa9, 0x82, 0xaa, 0x82, 0xab, 0x82, 0xac, 0x82, 0xad, 0x82, 0xae,
    0x81, 0x40, 0x82, 0xb0, 0x82, 0xb1, 0x82, 0xb2, 0x82, 0xb3, 0x82, 0xb4, 0x82, 0xb5, 0x82, 0xb6,
    0x82, 0xb7, 0x82, 0xb8, 0x82, 0xb9, 0x82, 0xba, 0x82, 0xbb, 0x82, 0xbc, 0x82, 0xbd, 0x82, 0xbe,
    0x82, 0xbf, 0x82, 0xc0, 0x82, 0xc1, 0x82, 0xc2, 0x82, 0xc3, 0x82, 0xc4, 0x82, 0xc5, 0x82, 0xc6,
    0x82, 0xc7, 0x82, 0xc8, 0x82, 0xc9, 0x82, 0xca, 0x82, 0xcb, 0x82, 0xcc, 0x82, 0xcd, 0x82, 0xce,
    0x82, 0xd0, 0x82, 0xd1, 0x82, 0xd3, 0x82, 0xd4, 0x82, 0xd6, 0x82, 0xd7, 0x82, 0xd9, 0x82, 0xda,
    0x82, 0xdc, 0x82, 0xdd, 0x82, 0xde, 0x82, 0xdf, 0x82, 0xe0, 0x82, 0xe1, 0x82, 0xe2, 0x82, 0xe3,
    0x82, 0xe4, 0x82, 0xe5, 0x82, 0xe6, 0x82, 0xe7, 0x82, 0xe8, 0x82, 0xe9, 0x82, 0xea, 0x82, 0xeb,
    0x82, 0xed, 0x82, 0xf0, 0x82, 0xf1, 0x83, 0x41, 0x83, 0x43, 0x83, 0x45, 0x83, 0x47, 0x83, 0x49,
    0x83, 0x4a, 0x83, 0x4c, 0x83, 0x4e, 0x83, 0x50, 0x83, 0x52, 0x83, 0x54, 0x83, 0x56, 0x83, 0x58,
    0x83, 0x5a, 0x83, 0x5c, 0x83, 0x5e, 0x83, 0x60, 0x83, 0x62, 0x83, 0x63, 0x83, 0x65, 0x83, 0x67,
    0x83, 0x69, 0x83, 0x6a, 0x82, 0xaf, 0x83, 0x6c, 0x83, 0x6d, 0x83, 0x6e, 0x83, 0x71, 0x83, 0x74,
    0x83, 0x77, 0x83, 0x7a, 0x83, 0x7d, 0x83, 0x7e, 0x83, 0x80, 0x83, 0x81, 0x83, 0x82, 0x83, 0x84,
])

# 反向表只用于可选压缩编码；默认注入仍优先写标准 Shift-JIS 双字节，降低重建复杂度。
KANA_REVERSE: dict[bytes, int] = {
    IKUAR_KANA[i:i + 2]: i // 2 for i in range(0, len(IKUAR_KANA) - 1, 2)
}

NAME_LINE_RE = re.compile(r"^【([^】\r\n]{1,32})】$")
EMBEDDED_NAME_RE = re.compile(r"^【([^】\r\n]{1,32})】(.+)$", re.S)


def from_bytes(data: bytes) -> int:
    return int.from_bytes(data, "little", signed=False)


def to_bytes(value: int, length: int) -> bytes:
    return value.to_bytes(length, "little", signed=False)


def align_up(value: int, align: int) -> int:
    return (value + align - 1) // align * align


def load_json_entries(path: str | Path) -> list[dict[str, Any]]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be a list: {path}")
    return data


def save_json_entries(path: str | Path, entries: list[dict[str, Any]]) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)


def decode_ikuar_text(byte_data: bytes) -> str:
    """解码 IKURA/ISF 文本。

    兼容原脚本中的 0x7F ASCII 转义、0x00-0x7E 假名单字节压缩、标准 Shift-JIS 双字节。
    原工具对 0x5C 分支在遇到后续 Shift-JIS 高字节时会越界；这里做了容错：
    只有 0x5C 后接低值压缩索引时才按特殊组合处理，否则把 0x5C 当普通压缩字典项。
    """
    decoded = bytearray()
    i = 0
    n = len(byte_data)
    while i < n:
        b = byte_data[i]
        if b == 0x7F:
            if i + 1 < n:
                decoded.append(byte_data[i + 1])
            i += 2
            continue

        if b == 0x5C:
            # 特殊组合：0x5C + 小于 0x80 的压缩索引，组合成 0x83 xx。
            if i + 1 < n and byte_data[i + 1] < 0x80 and byte_data[i + 1] * 2 + 1 < len(IKUAR_KANA):
                decoded.append(IKUAR_KANA[0xB8])
                decoded.append(IKUAR_KANA[byte_data[i + 1] * 2 + 1])
                i += 2
            else:
                # 容错：按普通压缩表的 0x5C 处理，避免把后续 SJIS 高字节吞掉。
                pos = b * 2
                if pos + 1 < len(IKUAR_KANA):
                    decoded.extend(IKUAR_KANA[pos:pos + 2])
                i += 1
            continue

        if b > 0x7F:
            decoded.append(b)
            if i + 1 < n:
                decoded.append(byte_data[i + 1])
            i += 2
        else:
            pos = b * 2
            if pos + 1 < len(IKUAR_KANA):
                decoded.extend(IKUAR_KANA[pos:pos + 2])
            i += 1
    return decoded.decode(DEFAULT_ENCODING, errors="ignore")


def decode_drs_text(byte_data: bytes) -> str:
    decoded = bytearray()
    i = 0
    n = len(byte_data)
    while i < n:
        b = byte_data[i]
        if b == 0x7F:
            if i + 1 < n:
                decoded.append(byte_data[i + 1])
            i += 2
            continue
        if b > 0x7F:
            decoded.append(b)
            if i + 1 < n:
                decoded.append(byte_data[i + 1])
            i += 2
        else:
            pos = b * 2
            if pos + 1 < len(IKUAR_KANA):
                decoded.extend(IKUAR_KANA[pos:pos + 2])
            i += 1
    return decoded.decode(DEFAULT_ENCODING, errors="ignore")


def decode_text_by_engine(data: bytes, engine: str) -> str:
    if engine.upper() == "DRS":
        return decode_drs_text(data)
    return decode_ikuar_text(data)


def encode_ikuar_text(text: str) -> bytes:
    """把 Unicode 文本编码为 ISF 可接受的字节。

    - 可编码为双字节 cp932 的字符直接写 Shift-JIS；
    - 单字节 ASCII 需要写成 0x7F + byte，否则引擎会按假名字典解码；
    - 目标文本中的中文应先通过 cn_jp/subs 映射保证 cp932 可编码。
    """
    raw = text.encode(DEFAULT_ENCODING)
    out = bytearray()
    i = 0
    while i < len(raw):
        b = raw[i]
        if b <= 0x7E:
            out.append(0x7F)
            out.append(b)
            i += 1
        else:
            out.append(b)
            if i + 1 < len(raw):
                out.append(raw[i + 1])
            i += 2
    return bytes(out)


def encode_text_by_engine(text: str, engine: str) -> bytes:
    if engine.upper() == "DRS":
        # DRS 同样支持 0x7F ASCII 转义，日文/映射中文直接 cp932。
        return encode_ikuar_text(text)
    return encode_ikuar_text(text)


def is_cp932_encodable(text: str) -> bool:
    try:
        text.encode(DEFAULT_ENCODING)
        return True
    except UnicodeEncodeError:
        return False


def ikuar_decrypt(data: bytes) -> bytes:
    data = bytearray(data)
    if len(data) < 8:
        return bytes(data)
    version = int.from_bytes(data[4:6], "little")
    key = data[6]
    if version == 0x9795:
        for i in range(8, len(data)):
            b = data[i]
            data[i] = (b >> 2) | ((b << 6) & 0xFF)
    elif version == 0xD197:
        for i in range(8, len(data)):
            data[i] = (~data[i]) & 0xFF
    elif version == 0xCE89:
        for i in range(8, len(data)):
            data[i] ^= key
    return bytes(data)


def ikuar_encrypt(data: bytes) -> bytes:
    data = bytearray(data)
    if len(data) < 8:
        return bytes(data)
    version = int.from_bytes(data[4:6], "little")
    key = data[6]
    if version == 0x9795:
        for i in range(8, len(data)):
            b = data[i]
            data[i] = ((b << 2) & 0xFF) | (b >> 6)
    elif version == 0xD197:
        for i in range(8, len(data)):
            data[i] = (~data[i]) & 0xFF
    elif version == 0xCE89:
        for i in range(8, len(data)):
            data[i] ^= key
    return bytes(data)


@dataclass
class TextRecord:
    raw_index: int
    op_index: int
    op_offset: int
    opcode: int
    start: int
    end: int
    text: str
    raw_text: bytes
    kind: str = "dialogue"
    has_name_br: bool = False
    embedded_name: str | None = None
    embedded_prefix: str | None = None


@dataclass
class ISFFile:
    engine: str = "MPX"
    head_len: int = 0
    version_info: bytes = b""
    offsetlist: list[int] = field(default_factory=list)
    ops: list[dict[str, Any]] = field(default_factory=list)
    old_offset_to_op_idx: dict[int, int] = field(default_factory=dict)
    op_offsets: list[int] = field(default_factory=list)
    is_minyan: bool = False

    def load_path(self, path: str | Path) -> None:
        self.load_bytes(Path(path).read_bytes())

    def load_bytes(self, data: bytes) -> None:
        self.is_minyan = data.startswith(b"MINYAN")
        if self.is_minyan:
            data = data[6:]
        data = ikuar_decrypt(data)
        if len(data) < 8:
            raise ValueError("ISF too small")
        self.head_len = from_bytes(data[:4])
        self.version_info = data[4:8]
        if self.head_len < 8 or self.head_len > len(data):
            raise ValueError(f"invalid ISF head_len=0x{self.head_len:X}")

        self.offsetlist = []
        offsetcounts = (self.head_len - 8) // 4
        for i in range(offsetcounts):
            self.offsetlist.append(from_bytes(data[8 + i * 4: 12 + i * 4]))

        body = data[self.head_len:]
        pos = 0
        idx = 0
        self.ops = []
        self.op_offsets = []
        self.old_offset_to_op_idx = {}
        while pos < len(body):
            start = pos
            self.old_offset_to_op_idx[start] = idx
            self.op_offsets.append(start)
            if pos + 2 > len(body):
                raise ValueError(f"truncated opcode at body offset 0x{pos:X}")
            op = body[pos]
            l = body[pos + 1]
            pos += 2
            if l < 0x80:
                length = l
                head_bytes = 2
            else:
                if pos >= len(body):
                    raise ValueError(f"truncated extended length at body offset 0x{start:X}")
                ext = body[pos]
                pos += 1
                length = ext + (l - 0x80) * 0x100
                head_bytes = 3
            content_len = length - head_bytes
            if content_len < 0:
                raise ValueError(f"invalid opcode length at body offset 0x{start:X}")
            content = body[pos: pos + content_len]
            if len(content) != content_len:
                raise ValueError(f"truncated opcode content at body offset 0x{start:X}")
            pos += content_len
            self.ops.append({"op": op, "content": content})
            idx += 1
        self.old_offset_to_op_idx[pos] = idx

    def _split_pm_text_segments(self, content: bytes) -> list[tuple[int, int, bool]]:
        if not content:
            return []
        res: list[tuple[int, int, bool]] = []
        offset = 1
        while offset < len(content):
            cmd = content[offset]
            offset += 1
            if cmd == 0x01:
                offset += 4
            elif cmd == 0x04:
                offset += 1
            elif cmd == 0x08:
                offset += 4
            elif cmd == 0x09:
                offset += 1
            elif cmd == 0x0A:
                offset += 4
            elif cmd in (0x0B, 0x0C, 0x10):
                offset += 2
            elif cmd == 0x11:
                offset += 4
            elif cmd == 0xFF:
                start = offset
                if self.engine.upper() == "DRS":
                    while offset < len(content) and content[offset] != 0x00:
                        offset += 1
                    end = offset
                    if offset < len(content):
                        offset += 1
                    if end > start:
                        res.append((start, end, False))
                    continue

                text_end = offset
                while text_end < len(content):
                    # 句中换页/换行标记；它属于文本 payload，注入时会按 has_name_br 恢复。
                    if text_end + 2 < len(content) and content[text_end:text_end + 3] == b"\x00\x06\xFF":
                        text_end += 3
                        continue
                    if content[text_end] == 0x00:
                        if text_end + 1 < len(content):
                            if content[text_end + 1] in CONTROL_AFTER_ZERO:
                                break
                        else:
                            break
                    elif content[text_end] == 0x03 and text_end + 1 == len(content):
                        break
                    text_end += 1
                if text_end > start:
                    raw = content[start:text_end]
                    res.append((start, text_end, b"\x81\x7A\x00\x06\xFF" in raw))
                # 原工具 MPX 分支默认一个 0x2B/0x2C 内只处理一个文本 payload。
                offset = len(content)
            else:
                # 未知短控制符按 1 字节命令本身处理，继续向后找 0xFF。
                pass
        return res

    def iter_text_records(self) -> list[TextRecord]:
        records: list[TextRecord] = []
        raw_index = 0
        for op_index, op in enumerate(self.ops):
            opcode = op["op"]
            content = op["content"]
            op_offset = self.op_offsets[op_index] if op_index < len(self.op_offsets) else -1
            if opcode in TEXT_OPS:
                for start, end, has_br in self._split_pm_text_segments(content):
                    raw = content[start:end]
                    clean = raw.replace(b"\x00\x06\xFF", b"").rstrip(b"\x00")
                    if not clean:
                        continue
                    text = decode_text_by_engine(clean, self.engine)
                    records.append(TextRecord(raw_index, op_index, op_offset, opcode, start, end, text, clean, "dialogue", has_br))
                    raw_index += 1
            elif opcode == 0x15 and len(content) > 0x12:
                end = len(content)
                while end > 0x12 and content[end - 1] == 0x00:
                    end -= 1
                if end > 0x12:
                    raw = content[0x12:end]
                    text = decode_text_by_engine(raw, self.engine)
                    records.append(TextRecord(raw_index, op_index, op_offset, opcode, 0x12, end, text, raw, "system"))
                    raw_index += 1
            elif opcode == 0x25 and len(content) > 0x02:
                end = len(content)
                while end > 0x02 and content[end - 1] == 0x00:
                    end -= 1
                if end > 0x02:
                    raw = content[0x02:end]
                    text = decode_text_by_engine(raw, self.engine)
                    records.append(TextRecord(raw_index, op_index, op_offset, opcode, 0x02, end, text, raw, "system"))
                    raw_index += 1
            elif opcode in (0xF7, 0xE0, 0xE1, 0xE2, 0xE3):
                fixed_starts = {0xF7: 0, 0xE0: 1, 0xE1: 2, 0xE2: 5, 0xE3: 5}
                start = fixed_starts[opcode]
                end = len(content)
                while end > start and content[end - 1] == 0x00:
                    end -= 1
                if end > start:
                    raw = content[start:end]
                    text = decode_text_by_engine(raw, self.engine)
                    records.append(TextRecord(raw_index, op_index, op_offset, opcode, start, end, text, raw, "ui"))
                    raw_index += 1
            elif opcode == 0xE4 and len(content) >= 4:
                count = content[0] + content[1]
                pos = 4
                for _ in range(count):
                    start = pos
                    while pos < len(content) and content[pos] != 0x00:
                        pos += 1
                    end = pos
                    if end > start:
                        raw = content[start:end]
                        text = decode_text_by_engine(raw, self.engine)
                        records.append(TextRecord(raw_index, op_index, op_offset, opcode, start, end, text, raw, "choice"))
                        raw_index += 1
                    if pos < len(content) and content[pos] == 0x00:
                        pos += 1
        return records

    def build_workflow_entries(self, file_name: str, include_ui: bool = True) -> list[dict[str, Any]]:
        """把底层 TextRecord 转成项目统一 JSON。

        修正点：独立的 `【角色】` 行不再作为正文导出，而是挂到后一个正文条目的 name 字段。
        注入时通过 `_raw_index` 找回底层文本槽，未导出的 name 槽会原样保留，因此不会造成 trans_iter 错位。
        """
        entries: list[dict[str, Any]] = []
        pending_name: str | None = None
        pending_raw_index: int | None = None
        export_index = 0
        for rec in self.iter_text_records():
            m_line = NAME_LINE_RE.match(rec.text.strip()) if rec.kind == "dialogue" else None
            if m_line:
                pending_name = m_line.group(1)
                pending_raw_index = rec.raw_index
                continue

            text = rec.text
            name: str | None = None
            embedded = EMBEDDED_NAME_RE.match(text.strip()) if rec.kind == "dialogue" else None
            embedded_prefix = None
            if embedded:
                name = embedded.group(1)
                embedded_prefix = f"【{name}】"
                text = embedded.group(2)
                rec.embedded_name = name
                rec.embedded_prefix = embedded_prefix
            elif pending_name and rec.kind == "dialogue":
                name = pending_name

            # 只对剧情/选项默认导出；UI 可以用 --no-ui 关掉。
            if rec.kind in {"ui", "system"} and not include_ui:
                pending_name = None
                pending_raw_index = None
                continue

            obj: dict[str, Any] = {}
            if name:
                # JSON 字段顺序按项目工作流输出：name 放在 scr_msg / message 前面。
                obj["name"] = name
            obj.update({
                "scr_msg": text,
                "message": text,
                "_file": file_name,
                "_index": export_index,
                "_raw_index": rec.raw_index,
                "_op_index": rec.op_index,
                "_inst_offset": rec.op_offset,
                "_offset": rec.start,
                "_size": len(rec.raw_text),
                "_type": rec.kind,
                "_opcode": f"0x{rec.opcode:02X}",
                "_encoding": DEFAULT_ENCODING,
                "_policy": "relocate",
            })
            if name:
                obj["_name_source"] = "embedded_bracket" if embedded else "previous_bracket_line"
                obj["_virtual_name"] = True
            if pending_raw_index is not None and not embedded:
                obj["_name_raw_index"] = pending_raw_index
            if rec.has_name_br:
                obj["_has_name_br"] = True
            if embedded_prefix:
                obj["_embedded_name"] = True
                obj["_embedded_prefix"] = embedded_prefix

            entries.append(obj)
            export_index += 1
            pending_name = None
            pending_raw_index = None
        return entries

    def apply_workflow_entries(self, entries: list[dict[str, Any]], strict: bool = True) -> dict[str, int]:
        records = self.iter_text_records()
        by_raw = {r.raw_index: r for r in records}
        entry_by_raw: dict[int, dict[str, Any]] = {}
        stats = {"json_entries": len(entries), "patched": 0, "kept": 0, "failed": 0, "warnings": 0}

        for entry in entries:
            raw_idx = entry.get("_raw_index")
            if not isinstance(raw_idx, int):
                stats["failed"] += 1
                print(f"[inject][warn] 缺少 _raw_index，跳过: {entry.get('scr_msg')}")
                continue
            if raw_idx in entry_by_raw:
                stats["failed"] += 1
                print(f"[inject][warn] 重复 _raw_index={raw_idx}，跳过: {entry.get('scr_msg')}")
                continue
            entry_by_raw[raw_idx] = entry

        replacements: dict[int, list[tuple[int, int, bytes]]] = {}
        for raw_idx, entry in entry_by_raw.items():
            rec = by_raw.get(raw_idx)
            if rec is None:
                stats["failed"] += 1
                print(f"[inject][warn] _raw_index={raw_idx} 在原脚本中不存在: {entry.get('scr_msg')}")
                continue
            scr_msg = entry.get("scr_msg")
            message = entry.get("message")
            if not isinstance(scr_msg, str) or not isinstance(message, str):
                stats["failed"] += 1
                print(f"[inject][warn] 缺少 scr_msg/message，raw_index={raw_idx}")
                continue
            compare_text = rec.text
            embedded_prefix = entry.get("_embedded_prefix")
            if entry.get("_embedded_name") and isinstance(embedded_prefix, str) and compare_text.startswith(embedded_prefix):
                compare_text = compare_text[len(embedded_prefix):]
            if compare_text != scr_msg:
                stats["warnings"] += 1
                msg = f"[inject][warn] scr_msg 校验失败 raw_index={raw_idx}: json={scr_msg!r}, file={compare_text!r}"
                print(msg)
                if strict:
                    stats["failed"] += 1
                    continue

            write_text = message
            if entry.get("_embedded_name"):
                prefix = entry.get("_embedded_prefix")
                if not isinstance(prefix, str):
                    nm = entry.get("name", "")
                    prefix = f"【{nm}】" if nm else ""
                write_text = prefix + message
            # 未修改的条目不重写，保留原来的单字节压缩写法，保证零修改回环尽量 byte-exact。
            if message == scr_msg:
                stats["kept"] += 1
                continue

            try:
                new_bytes = encode_text_by_engine(write_text, self.engine)
            except UnicodeEncodeError as e:
                stats["failed"] += 1
                print(f"[inject][warn] 编码失败 raw_index={raw_idx}: {e}")
                continue

            # 恢复原来的 name 后换页/换行标记。只处理 `】` 后的第一个位置。
            if entry.get("_has_name_br"):
                mark = "】".encode(DEFAULT_ENCODING)
                if mark in new_bytes:
                    new_bytes = new_bytes.replace(mark, mark + b"\x00\x06\xFF", 1)

            replacements.setdefault(rec.op_index, []).append((rec.start, rec.end, new_bytes))
            stats["patched"] += 1

        # 从每条 op 的末尾向前替换，避免变长后影响同 op 内靠后的切片。
        for op_index, reps in replacements.items():
            content = bytearray(self.ops[op_index]["content"])
            for start, end, new_bytes in sorted(reps, key=lambda x: x[0], reverse=True):
                content[start:end] = new_bytes
            self.ops[op_index]["content"] = bytes(content)
        stats["kept"] += max(0, len(records) - stats["patched"] - stats["kept"])
        return stats

    def _op_to_bytes(self, op: dict[str, Any]) -> bytes:
        content = op["content"]
        total_len = len(content) + 2
        out = bytearray([op["op"]])
        if total_len < 0x80:
            out.append(total_len)
        else:
            total_len += 1
            prefix = 0x80 + (total_len // 0x100)
            low = total_len % 0x100
            if prefix > 0xFF:
                raise ValueError(f"opcode too large after rebuild: {total_len}")
            out.append(prefix)
            out.append(low)
        out.extend(content)
        return bytes(out)

    def to_bytes(self, encrypt: bool = True) -> bytes:
        new_offsets: dict[int, int] = {}
        body = bytearray()
        for idx, op in enumerate(self.ops):
            new_offsets[idx] = len(body)
            body.extend(self._op_to_bytes(op))
        new_offsets[len(self.ops)] = len(body)

        off_table = bytearray()
        for old_off in self.offsetlist:
            op_idx = self.old_offset_to_op_idx.get(old_off)
            new_off = old_off if op_idx is None else new_offsets[op_idx]
            off_table.extend(to_bytes(new_off, 4))

        out = bytearray()
        out.extend(to_bytes(self.head_len, 4))
        out.extend(self.version_info)
        out.extend(off_table)
        if len(out) < self.head_len:
            out.extend(b"\x00" * (self.head_len - len(out)))
        out.extend(body)

        final = ikuar_encrypt(bytes(out)) if encrypt else bytes(out)
        if self.is_minyan:
            final = b"MINYAN" + final
        return final

    def save_path(self, path: str | Path, encrypt: bool = True) -> None:
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(self.to_bytes(encrypt=encrypt))


def detect_engine_from_isf(path: str | Path, default: str = "MPX") -> str:
    # 散 ISF 很难单独从头部区分 DRS/MPX；默认按新版 MPX。
    return default
