# -*- coding: utf-8 -*-
"""解析和重建 YU-RIS YSTB 二进制脚本。

本模块同时保留两套互补逻辑：
- 结构化逻辑：按 YuRis_Tool 的 Command / CommandExpression / CommandData / LineIndex 解析，
  适合通过 YSCM 定位 WORD 等命令。
- args 扫描逻辑：参考 YU-RIS-Script-Editor，直接扫描 12 字节参数项，
  可补出 ES.SEL.SET 后面的选项文本、部分非 WORD raw 文本，并兼容 YSTB v2。

注入统一采用 append 策略：不重排旧数据区，只把新表达式/参数数据追加到末尾，
然后修改对应 size / offset，最大限度避免破坏共享片段、跳转和行号。
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Any

from .common import (
    DEFAULT_ENCODING,
    decode_text,
    encode_text,
    read_i32_le,
    read_u16_le,
    read_u32_le,
    write_i32_le,
    write_u16_le,
    write_u32_le,
)
from .crypto import Segment, xor_flat_after_header, xor_segments
from .yscm import Yscm

YSTB_MAGIC = b"YSTB"
HEADER_SIZE = 0x20
PUSH_STRING = 0x4D  # 'M'
ARGS_ENTRY_SIZE = 12
RUBY_MARKER = b"\x87\x55"
SJIS_TO_GBK_REPLACE = {"♪": "", "〜": "~"}
SJIS_KEEP_CHARS = ["＠", "＃"]
SEL_SET_MARKER = b"\x4D\x0C\x00\x22\x45\x53\x2E\x53\x45\x4C\x2E\x53\x45\x54\x22"


@dataclass
class YstbHeader:
    version: int
    command_count: int
    command_size: int
    cmd_expr_size: int
    cmd_data_size: int
    line_idx_size: int
    reserved: bytes = b"\x00\x00\x00\x00"

    @property
    def is_v2(self) -> bool:
        return 200 < self.version < 300

    @classmethod
    def read(cls, data: bytes | bytearray) -> "YstbHeader":
        if len(data) < HEADER_SIZE or data[:4] != YSTB_MAGIC:
            raise ValueError("不是 YSTB 文件")
        return cls(
            version=read_u32_le(data, 0x04),
            command_count=read_u32_le(data, 0x08),
            command_size=read_u32_le(data, 0x0C),
            cmd_expr_size=read_u32_le(data, 0x10),
            cmd_data_size=read_u32_le(data, 0x14),
            line_idx_size=read_u32_le(data, 0x18),
            reserved=bytes(data[0x1C:0x20]),
        )

    def to_bytes(self) -> bytes:
        buf = bytearray(HEADER_SIZE)
        buf[0:4] = YSTB_MAGIC
        write_u32_le(buf, 0x04, self.version)
        write_u32_le(buf, 0x08, self.command_count)
        write_u32_le(buf, 0x0C, self.command_size)
        write_u32_le(buf, 0x10, self.cmd_expr_size)
        write_u32_le(buf, 0x14, self.cmd_data_size)
        write_u32_le(buf, 0x18, self.line_idx_size)
        buf[0x1C:0x20] = self.reserved[:4].ljust(4, b"\x00")
        return bytes(buf)


@dataclass
class CommandExpression:
    expr_id: int
    flag: int
    arg_load_fn: int
    arg_load_op: int
    instruction_size: int
    instruction_offset: int
    index_in_command: int = 0
    abs_expr_offset: int = 0

    @property
    def compat_arg_id(self) -> int:
        # YU-RIS-Script-Editor 把前 4 字节解释成 u16 arg_id + u16 arg_type。
        return self.expr_id | (self.flag << 8)

    @property
    def compat_arg_type(self) -> int:
        return self.arg_load_fn | (self.arg_load_op << 8)

    def to_bytes(self) -> bytes:
        buf = bytearray(ARGS_ENTRY_SIZE)
        buf[0] = self.expr_id & 0xFF
        buf[1] = self.flag & 0xFF
        buf[2] = self.arg_load_fn & 0xFF
        buf[3] = self.arg_load_op & 0xFF
        write_i32_le(buf, 4, self.instruction_size)
        write_i32_le(buf, 8, self.instruction_offset)
        return bytes(buf)


@dataclass
class Command:
    command_id: int
    expr_count: int
    label_id: int
    index: int
    abs_offset: int
    line_number: int | None = None
    expressions: list[CommandExpression] = field(default_factory=list)

    def to_bytes(self) -> bytes:
        buf = bytearray(4)
        buf[0] = self.command_id & 0xFF
        buf[1] = len(self.expressions) & 0xFF
        write_u16_le(buf, 2, self.label_id)
        return bytes(buf)


@dataclass
class TextRef:
    text: str
    text_format: str
    raw_data: bytes
    command: Command
    expression: CommandExpression
    expr_index: int
    source: str = "command"
    is_option: bool = False


class Ystb:
    def __init__(self, header: YstbHeader, commands: list[Command], cmd_data: bytearray, line_idx: bytearray):
        self.header = header
        self.commands = commands
        self.cmd_data = cmd_data
        self.line_idx = line_idx

    @property
    def segments(self) -> list[Segment]:
        cmd_addr = HEADER_SIZE
        expr_addr = cmd_addr + self.header.command_size
        data_addr = expr_addr + self.header.cmd_expr_size
        line_addr = data_addr + self.header.cmd_data_size
        return [
            Segment(cmd_addr, self.header.command_size),
            Segment(expr_addr, self.header.cmd_expr_size),
            Segment(data_addr, self.header.cmd_data_size),
            Segment(line_addr, self.header.line_idx_size),
        ]

    @classmethod
    def read_file(cls, path: str | Path, *, key: bytes | None = None, xor_mode: str = "segment") -> "Ystb | YstbV2":
        return cls.read(Path(path).read_bytes(), key=key, xor_mode=xor_mode)

    @classmethod
    def read(cls, data: bytes, *, key: bytes | None = None, xor_mode: str = "segment") -> "Ystb | YstbV2":
        if len(data) < HEADER_SIZE or data[:4] != YSTB_MAGIC:
            raise ValueError("不是 YSTB 文件")
        raw_header = YstbHeader.read(data)
        if raw_header.is_v2:
            return YstbV2.read(data, key=key, xor_mode=xor_mode)

        buf = bytearray(data)
        header = raw_header
        if key:
            if xor_mode == "segment":
                tmp = cls._dummy_for_segments(header)
                xor_segments(buf, tmp.segments, key)
            elif xor_mode == "flat":
                xor_flat_after_header(buf, key)
            else:
                raise ValueError(f"未知 xor_mode: {xor_mode}")
            header = YstbHeader.read(buf)

        cmd_addr = HEADER_SIZE
        expr_addr = cmd_addr + header.command_size
        data_addr = expr_addr + header.cmd_expr_size
        line_addr = data_addr + header.cmd_data_size
        end = line_addr + header.line_idx_size
        if end > len(buf):
            raise ValueError(f"YSTB 分区大小超出文件长度: need={end}, file={len(buf)}")

        commands: list[Command] = []
        pos = cmd_addr
        for i in range(header.command_count):
            if pos + 4 > expr_addr:
                raise ValueError("Command 区截断")
            cmd = Command(
                command_id=buf[pos],
                expr_count=buf[pos + 1],
                label_id=read_u16_le(buf, pos + 2),
                index=i,
                abs_offset=pos,
            )
            commands.append(cmd)
            pos += 4
        if pos != cmd_addr + header.command_size:
            raise ValueError("Command 区大小和 command_count 不一致")

        pos = expr_addr
        for cmd in commands:
            for expr_idx in range(cmd.expr_count):
                if pos + ARGS_ENTRY_SIZE > data_addr:
                    raise ValueError("CommandExpression 区截断")
                expr = CommandExpression(
                    expr_id=buf[pos],
                    flag=buf[pos + 1],
                    arg_load_fn=buf[pos + 2],
                    arg_load_op=buf[pos + 3],
                    instruction_size=read_i32_le(buf, pos + 4),
                    instruction_offset=read_i32_le(buf, pos + 8),
                    index_in_command=expr_idx,
                    abs_expr_offset=pos,
                )
                cmd.expressions.append(expr)
                pos += ARGS_ENTRY_SIZE
        if pos != expr_addr + header.cmd_expr_size:
            raise ValueError("CommandExpression 区大小和 expr_count 不一致")

        pos = line_addr
        for cmd in commands:
            if pos + 4 <= line_addr + header.line_idx_size:
                cmd.line_number = read_i32_le(buf, pos)
            pos += 4

        cmd_data = bytearray(buf[data_addr:data_addr + header.cmd_data_size])
        line_idx = bytearray(buf[line_addr:line_addr + header.line_idx_size])
        return cls(header, commands, cmd_data, line_idx)

    @classmethod
    def _dummy_for_segments(cls, header: YstbHeader) -> "Ystb":
        return cls(header, [], bytearray(), bytearray())

    def build(self, *, key: bytes | None = None, xor_mode: str = "segment") -> bytes:
        command_blob = b"".join(cmd.to_bytes() for cmd in self.commands)
        expr_blob = b"".join(expr.to_bytes() for cmd in self.commands for expr in cmd.expressions)
        self.header.command_count = len(self.commands)
        self.header.command_size = len(command_blob)
        self.header.cmd_expr_size = len(expr_blob)
        self.header.cmd_data_size = len(self.cmd_data)
        self.header.line_idx_size = len(self.line_idx)
        buf = bytearray(self.header.to_bytes() + command_blob + expr_blob + self.cmd_data + self.line_idx)
        if key:
            if xor_mode == "segment":
                xor_segments(buf, self.segments, key)
            elif xor_mode == "flat":
                xor_flat_after_header(buf, key)
            else:
                raise ValueError(f"未知 xor_mode: {xor_mode}")
        return bytes(buf)

    def write_file(self, path: str | Path, *, key: bytes | None = None, xor_mode: str = "segment") -> None:
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(self.build(key=key, xor_mode=xor_mode))

    def get_expr_data(self, expr: CommandExpression) -> bytes:
        off = expr.instruction_offset
        size = expr.instruction_size
        if off < 0 or size < 0 or off + size > len(self.cmd_data):
            raise ValueError(f"表达式数据越界: off={off}, size={size}, cmd_data={len(self.cmd_data)}")
        return bytes(self.cmd_data[off:off + size])

    def set_expr_data_append(self, expr: CommandExpression, new_data: bytes) -> None:
        new_off = len(self.cmd_data)
        self.cmd_data.extend(new_data)
        expr.instruction_offset = new_off
        expr.instruction_size = len(new_data)

    def iter_text_refs(
        self,
        yscm: Yscm | None,
        *,
        command_names: Iterable[str] = ("WORD",),
        encoding: str = DEFAULT_ENCODING,
        include_raw_candidates: bool = False,
        errors: str = "strict",
    ) -> Iterable[TextRef]:
        if yscm is None:
            return
        command_ids = {cid for name in command_names if (cid := yscm.find_command_id(name)) is not None}
        for cmd in self.commands:
            if command_ids and cmd.command_id not in command_ids:
                if not include_raw_candidates:
                    continue
            for expr_index, expr in enumerate(cmd.expressions):
                expr_info = yscm.get_expr_info(cmd.command_id, expr.expr_id)
                try:
                    data = self.get_expr_data(expr)
                except ValueError:
                    continue
                parsed = decode_text_expr(data, encoding=encoding, errors=errors, raw=bool(expr_info and expr_info.is_raw))
                if not parsed:
                    continue
                text, text_format = parsed
                if not _looks_like_translatable(text):
                    continue
                yield TextRef(text, text_format, data, cmd, expr, expr_index, source="command", is_option=False)

    def iter_args_scan_text_refs(self, *, encoding: str = DEFAULT_ENCODING, errors: str = "replace") -> Iterable[TextRef]:
        """参考 YU-RIS-Script-Editor 的 args_index 扫描逻辑。

        这套逻辑不依赖 YSCM：
        - arg_id == 0 且 arg_type == 0 的 raw 文本候选；
        - arg_type == 3 且内容为 ES.SEL.SET 后，下一条 PushString 视为选项。
        """
        opt_flag = False
        for cmd in self.commands:
            for expr_index, expr in enumerate(cmd.expressions):
                size = expr.instruction_size
                if size <= 0 or size > 4096:
                    opt_flag = False
                    continue
                try:
                    data = self.get_expr_data(expr)
                except ValueError:
                    opt_flag = False
                    continue

                if opt_flag:
                    parsed = decode_option_push_string(data, encoding=encoding, errors=errors)
                    if parsed:
                        text, text_format = parsed
                        if _looks_like_translatable(text):
                            yield TextRef(text, text_format, data, cmd, expr, expr_index, source="args_scan", is_option=True)
                        opt_flag = False
                        continue
                    opt_flag = False

                if expr.compat_arg_type == 3 and data == SEL_SET_MARKER:
                    opt_flag = True
                    continue

                if expr.compat_arg_id == 0 and expr.compat_arg_type == 0:
                    if not data:
                        continue
                    if data[0] == PUSH_STRING or data[:2] == b"H\x03" or b"\x00" in data or b"cg" in data:
                        continue
                    clean = data.replace(RUBY_MARKER, b"")
                    if not clean:
                        continue
                    try:
                        text = decode_text(clean, encoding, errors=errors)
                    except Exception:
                        continue
                    if _looks_like_translatable(text):
                        yield TextRef(text, "args_raw", data, cmd, expr, expr_index, source="args_scan", is_option=False)

    def find_text_ref_by_position(
        self,
        yscm: Yscm | None,
        cmd_index: int,
        expr_index: int,
        *,
        encoding: str = DEFAULT_ENCODING,
        errors: str = "strict",
        allow_args_scan: bool = True,
    ) -> TextRef | None:
        if not (0 <= cmd_index < len(self.commands)):
            return None
        cmd = self.commands[cmd_index]
        if not (0 <= expr_index < len(cmd.expressions)):
            return None
        expr = cmd.expressions[expr_index]
        data = self.get_expr_data(expr)
        expr_info = yscm.get_expr_info(cmd.command_id, expr.expr_id) if yscm else None
        parsed = decode_text_expr(data, encoding=encoding, errors=errors, raw=bool(expr_info and expr_info.is_raw))
        if parsed:
            text, text_format = parsed
            return TextRef(text, text_format, data, cmd, expr, expr_index, source="command")
        if allow_args_scan:
            # 位置已知时，直接按补充扫描规则尝试解析。
            if expr.compat_arg_id == 0 and expr.compat_arg_type == 0:
                clean = data.replace(RUBY_MARKER, b"")
                if clean and data[0] != PUSH_STRING and b"\x00" not in data:
                    try:
                        return TextRef(decode_text(clean, encoding, errors=errors), "args_raw", data, cmd, expr, expr_index, source="args_scan")
                    except Exception:
                        pass
            parsed_opt = decode_option_push_string(data, encoding=encoding, errors=errors)
            if parsed_opt:
                text, text_format = parsed_opt
                return TextRef(text, text_format, data, cmd, expr, expr_index, source="args_scan", is_option=True)
        return None


class YstbV2:
    """兼容 YU-RIS-Script-Editor 中的 YSTB v2 扫描/追加策略。"""

    def __init__(self, header_raw: bytearray, code_segment: bytearray, args_segment: bytes, args_seg_offset: int):
        self.header_raw = header_raw
        self.code_segment = code_segment
        self.args_segment = args_segment
        self.args_seg_offset = args_seg_offset
        self.append_region = bytearray()
        self.header = YstbHeader.read(header_raw)
        self.commands: list[Command] = []

    @property
    def segments(self) -> list[Segment]:
        code_size = read_u32_le(self.header_raw, 0x08)
        args_size = read_u32_le(self.header_raw, 0x0C)
        return [Segment(HEADER_SIZE, code_size), Segment(HEADER_SIZE + code_size, args_size)]

    @classmethod
    def read(cls, data: bytes, *, key: bytes | None = None, xor_mode: str = "segment") -> "YstbV2":
        buf = bytearray(data)
        header_raw = bytearray(buf[:HEADER_SIZE])
        if key:
            code_size = read_u32_le(header_raw, 0x08)
            args_size = read_u32_le(header_raw, 0x0C)
            if xor_mode == "segment":
                xor_segments(buf, [Segment(HEADER_SIZE, code_size), Segment(HEADER_SIZE + code_size, args_size)], key)
            elif xor_mode == "flat":
                xor_flat_after_header(buf, key)
            else:
                raise ValueError(f"未知 xor_mode: {xor_mode}")
            header_raw = bytearray(buf[:HEADER_SIZE])
        code_size = read_u32_le(header_raw, 0x08)
        args_size = read_u32_le(header_raw, 0x0C)
        args_seg_offset = read_u32_le(header_raw, 0x10)
        off = HEADER_SIZE
        code = bytearray(buf[off:off + code_size])
        off += code_size
        args = bytes(buf[off:off + args_size])
        return cls(header_raw, code, args, args_seg_offset)

    def build(self, *, key: bytes | None = None, xor_mode: str = "segment") -> bytes:
        header = bytearray(self.header_raw)
        write_u32_le(header, 0x08, len(self.code_segment))
        write_u32_le(header, 0x0C, len(self.args_segment) + len(self.append_region))
        buf = bytearray(header + self.code_segment + self.args_segment + self.append_region)
        if key:
            if xor_mode == "segment":
                xor_segments(buf, [Segment(HEADER_SIZE, len(self.code_segment)), Segment(HEADER_SIZE + len(self.code_segment), len(self.args_segment) + len(self.append_region))], key)
            elif xor_mode == "flat":
                xor_flat_after_header(buf, key)
            else:
                raise ValueError(f"未知 xor_mode: {xor_mode}")
        return bytes(buf)

    def write_file(self, path: str | Path, *, key: bytes | None = None, xor_mode: str = "segment") -> None:
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(self.build(key=key, xor_mode=xor_mode))

    def get_expr_data(self, expr: CommandExpression) -> bytes:
        off = expr.instruction_offset
        size = expr.instruction_size
        total = self.args_segment + bytes(self.append_region)
        if off < 0 or size < 0 or off + size > len(total):
            raise ValueError(f"v2 参数数据越界: off={off}, size={size}, args={len(total)}")
        return bytes(total[off:off + size])

    def set_expr_data_append(self, expr: CommandExpression, new_data: bytes) -> None:
        new_off = len(self.args_segment) + len(self.append_region)
        self.append_region.extend(new_data)
        # v2 的 size / offset 位于 code_segment 的参数项 entry_offset + 4 / + 8。
        rel = expr.abs_expr_offset
        write_u32_le(self.code_segment, rel + 4, len(new_data))
        write_u32_le(self.code_segment, rel + 8, new_off)
        expr.instruction_size = len(new_data)
        expr.instruction_offset = new_off

    def iter_text_refs(self, yscm: Yscm | None = None, *, command_names: Iterable[str] = ("WORD",), encoding: str = DEFAULT_ENCODING, include_raw_candidates: bool = False, errors: str = "replace") -> Iterable[TextRef]:
        # v2 中当前只按 YU-RIS-Script-Editor 逻辑提取 op=0x54 的第一参数。
        yield from self.iter_args_scan_text_refs(encoding=encoding, errors=errors)

    def iter_args_scan_text_refs(self, *, encoding: str = DEFAULT_ENCODING, errors: str = "replace") -> Iterable[TextRef]:
        pos = 0
        cmd_index = 0
        code = self.code_segment
        while pos < len(code):
            if pos + 2 > len(code):
                break
            op = code[pos]
            argc = code[pos + 1]
            if op == 0x38:
                pos += 0xA
                cmd_index += 1
                continue
            block_size = argc * ARGS_ENTRY_SIZE + 6
            if block_size <= 0 or pos + block_size > len(code):
                break
            if op == 0x54 and argc >= 1:
                entry_offset = pos + 6
                size = read_u32_le(code, entry_offset + 4)
                off = read_u32_le(code, entry_offset + 8)
                if size > 0 and off + size <= len(self.args_segment) + len(self.append_region):
                    expr = CommandExpression(0, 0, 0, 0, size, off, 0, entry_offset)
                    cmd = Command(op, argc, 0, cmd_index, pos, expressions=[expr])
                    data = self.get_expr_data(expr)
                    try:
                        text = decode_text(data.replace(RUBY_MARKER, b""), encoding, errors=errors)
                    except Exception:
                        text = ""
                    if _looks_like_translatable(text):
                        yield TextRef(text, "v2_raw", data, cmd, expr, 0, source="v2_args_scan")
            pos += block_size
            cmd_index += 1

    def find_text_ref_by_position(self, yscm: Yscm | None, cmd_index: int, expr_index: int, *, encoding: str = DEFAULT_ENCODING, errors: str = "replace", allow_args_scan: bool = True) -> TextRef | None:
        for ref in self.iter_args_scan_text_refs(encoding=encoding, errors=errors):
            if ref.command.index == cmd_index and ref.expr_index == expr_index:
                return ref
        return None


def decode_text_expr(data: bytes, *, encoding: str = DEFAULT_ENCODING, errors: str = "strict", raw: bool = False) -> tuple[str, str] | None:
    """尝试把 expression data 识别为文本。

    支持三种格式：
    - raw：裸文本；
    - push_string：0x4D + u16长度 + payload；
    - push_string_quoted：0x4D + u16长度 + '"' + payload + '"'。
    """
    if not data:
        return None
    if raw:
        return decode_text(_strip_one_nul(data).replace(RUBY_MARKER, b""), encoding, errors=errors), "raw"
    if data[0] == PUSH_STRING and len(data) >= 3:
        size = int.from_bytes(data[1:3], "little", signed=False)
        if 3 + size <= len(data):
            payload = data[3:3 + size]
            if len(payload) >= 2 and payload[0] == 0x22 and payload[-1] == 0x22:
                return decode_text(payload[1:-1].replace(RUBY_MARKER, b""), encoding, errors=errors), "push_string_quoted"
            return decode_text(payload.replace(RUBY_MARKER, b""), encoding, errors=errors), "push_string"
    if b"\x00" not in data and not _looks_like_binary_expr(data):
        return decode_text(data.replace(RUBY_MARKER, b""), encoding, errors=errors), "raw"
    return None


def decode_option_push_string(data: bytes, *, encoding: str = DEFAULT_ENCODING, errors: str = "replace") -> tuple[str, str] | None:
    if not data or data[0] != PUSH_STRING or len(data) < 5:
        return None
    size = int.from_bytes(data[1:3], "little", signed=False)
    if 3 + size <= len(data):
        payload = data[3:3 + size]
    else:
        payload = data[3:]
    if len(payload) >= 2 and payload[0] == 0x22 and payload[-1] == 0x22:
        payload = payload[1:-1]
        fmt = "option_push_string_quoted"
    else:
        # 兼容 YU-RIS-Script-Editor 的宽松切片 data[4:-1]。
        payload = data[4:-1] if len(data) > 5 and data[3] == 0x22 else payload
        fmt = "option_push_string"
    if not payload:
        return None
    return decode_text(payload.replace(RUBY_MARKER, b""), encoding, errors=errors), fmt


def encode_text_expr(text: str, text_format: str, *, encoding: str = DEFAULT_ENCODING) -> bytes:
    payload = _encode_game_text(text, encoding)
    if text_format in {"push_string", "option_push_string"}:
        if len(payload) > 0xFFFF:
            raise ValueError("PushString 文本超过 65535 字节")
        return bytes([PUSH_STRING]) + len(payload).to_bytes(2, "little") + payload
    if text_format in {"push_string_quoted", "option_push_string_quoted"}:
        if len(payload) + 2 > 0xFFFF:
            raise ValueError("PushString 文本超过 65535 字节")
        return bytes([PUSH_STRING]) + (len(payload) + 2).to_bytes(2, "little") + b'"' + payload + b'"'
    if text_format in {"raw", "args_raw", "v2_raw"}:
        return payload
    raise ValueError(f"未知 _text_format: {text_format}")



def _encode_game_text(text: str, encoding: str) -> bytes:
    """编码写回文本。

    cp932/shift_jis 走严格编码；GBK/cp936 兼容 YU-RIS-Script-Editor 的少量替换规则，
    避免 ♪、〜 等原 SJIS 字符在 GBK 补丁模式下直接报错。
    """
    norm = encoding.lower().replace("-", "_")
    if norm in {"gbk", "cp936"}:
        for old, new in SJIS_TO_GBK_REPLACE.items():
            text = text.replace(old, new)
        placeholders: dict[bytes, bytes] = {}
        for i, ch in enumerate(SJIS_KEEP_CHARS):
            ph = f"\x01KEEP{i}\x01"
            text = text.replace(ch, ph)
            placeholders[ph.encode("gbk", errors="replace")] = ch.encode("cp932", errors="replace")
        encoded = text.encode("gbk", errors="replace")
        for ph, raw in placeholders.items():
            encoded = encoded.replace(ph, raw)
        return encoded
    if norm in {"shift_jis", "sjis"}:
        encoding = "cp932"
    return encode_text(text, encoding)

def _strip_one_nul(data: bytes) -> bytes:
    if data.endswith(b"\x00"):
        return data[:-1]
    return data


def _looks_like_binary_expr(data: bytes) -> bool:
    if len(data) >= 3 and data[0] in {0x42, 0x46, 0x48, 0x49, 0x4C, 0x56, 0x57, 0x76}:
        return True
    return False


def _looks_like_translatable(text: str) -> bool:
    if not text or text.strip() == "":
        return False
    lowered = text.lower()
    if lowered.endswith((".ogg", ".wav", ".png", ".bmp", ".jpg", ".jpeg", ".ybn", ".ypf")):
        return False
    return True
