# -*- coding: utf-8 -*-
"""解析和重建 YU-RIS YSTB(v5) 二进制脚本。

本模块按 YuRis_Tool 的结构实现：
- Header: 0x20 bytes
- Command 区：每条 4 字节，u8 command_id / u8 expr_count / u16 label_id
- CommandExpression 区：每条 12 字节，u8 expr_id / u8 flag / u8 arg_load_fn / u8 arg_load_op / i32 size / i32 offset
- CommandData 区：表达式指令数据，可被多个 expression 共享片段
- LineIndex 区：每条 command 一个 i32 行号

注入采用安全 append 策略：不重排旧 CommandData，只把新表达式数据追加到末尾，
然后修改对应 expression 的 size / offset，并重建 header 与分区。
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

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
PUSH_STRING = 0x4D  # 'M'


@dataclass
class YstbHeader:
    version: int
    command_count: int
    command_size: int
    cmd_expr_size: int
    cmd_data_size: int
    line_idx_size: int
    reserved: bytes = b"\x00\x00\x00\x00"

    @classmethod
    def read(cls, data: bytes | bytearray) -> "YstbHeader":
        if data[:4] != YSTB_MAGIC:
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
        buf = bytearray(0x20)
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

    def to_bytes(self) -> bytes:
        buf = bytearray(12)
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


class Ystb:
    def __init__(self, header: YstbHeader, commands: list[Command], cmd_data: bytearray, line_idx: bytearray):
        self.header = header
        self.commands = commands
        self.cmd_data = cmd_data
        self.line_idx = line_idx

    @property
    def segments(self) -> list[Segment]:
        cmd_addr = 0x20
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
    def read_file(
        cls,
        path: str | Path,
        *,
        key: bytes | None = None,
        xor_mode: str = "segment",
    ) -> "Ystb":
        return cls.read(Path(path).read_bytes(), key=key, xor_mode=xor_mode)

    @classmethod
    def read(cls, data: bytes, *, key: bytes | None = None, xor_mode: str = "segment") -> "Ystb":
        buf = bytearray(data)
        header = YstbHeader.read(buf)
        if key:
            if xor_mode == "segment":
                tmp = cls._dummy_for_segments(header)
                xor_segments(buf, tmp.segments, key)
            elif xor_mode == "flat":
                xor_flat_after_header(buf, key)
            else:
                raise ValueError(f"未知 xor_mode: {xor_mode}")
            header = YstbHeader.read(buf)

        cmd_addr = 0x20
        expr_addr = cmd_addr + header.command_size
        data_addr = expr_addr + header.cmd_expr_size
        line_addr = data_addr + header.cmd_data_size
        end = line_addr + header.line_idx_size
        if end > len(buf):
            raise ValueError(f"YSTB 分区大小超出文件长度: need={end}, file={len(buf)}")

        commands: list[Command] = []
        pos = cmd_addr
        for i in range(header.command_count):
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
                pos += 12
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

        buf = bytearray()
        buf += self.header.to_bytes()
        buf += command_blob
        buf += expr_blob
        buf += self.cmd_data
        buf += self.line_idx

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
        # 保留旧 CommandData，避免破坏其他 expression 的共享片段。
        new_off = len(self.cmd_data)
        self.cmd_data.extend(new_data)
        expr.instruction_offset = new_off
        expr.instruction_size = len(new_data)

    def iter_text_refs(
        self,
        yscm: Yscm,
        *,
        command_names: Iterable[str] = ("WORD",),
        encoding: str = DEFAULT_ENCODING,
        include_raw_candidates: bool = False,
        errors: str = "strict",
    ) -> Iterable[TextRef]:
        command_ids = {cid for name in command_names if (cid := yscm.find_command_id(name)) is not None}
        for cmd in self.commands:
            command_name = yscm.command_name(cmd.command_id)
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
                if cmd.command_id in command_ids or include_raw_candidates:
                    yield TextRef(text, text_format, data, cmd, expr, expr_index)

    def find_text_ref_by_position(
        self,
        yscm: Yscm,
        cmd_index: int,
        expr_index: int,
        *,
        encoding: str = DEFAULT_ENCODING,
        errors: str = "strict",
    ) -> TextRef | None:
        if not (0 <= cmd_index < len(self.commands)):
            return None
        cmd = self.commands[cmd_index]
        if not (0 <= expr_index < len(cmd.expressions)):
            return None
        expr = cmd.expressions[expr_index]
        expr_info = yscm.get_expr_info(cmd.command_id, expr.expr_id)
        data = self.get_expr_data(expr)
        parsed = decode_text_expr(data, encoding=encoding, errors=errors, raw=bool(expr_info and expr_info.is_raw))
        if not parsed:
            return None
        text, text_format = parsed
        return TextRef(text, text_format, data, cmd, expr, expr_index)


def decode_text_expr(
    data: bytes,
    *,
    encoding: str = DEFAULT_ENCODING,
    errors: str = "strict",
    raw: bool = False,
) -> tuple[str, str] | None:
    """尝试把 expression data 识别为文本。

    raw=True 时按 YSCM 的 Raw 表达式直接解码；否则优先识别 PushString，
    再保守识别无 0 字节的裸字符串。
    """
    if not data:
        return None
    if raw:
        return decode_text(_strip_one_nul(data), encoding, errors=errors), "raw"
    if data[0] == PUSH_STRING and len(data) >= 3:
        size = int.from_bytes(data[1:3], "little", signed=False)
        if 3 + size <= len(data):
            payload = data[3:3 + size]
            return decode_text(payload, encoding, errors=errors), "push_string"
    # 某些 WORD 的 expression data 直接是文本；包含 0 字节通常是二进制表达式，跳过。
    if b"\x00" not in data and not _looks_like_binary_expr(data):
        return decode_text(data, encoding, errors=errors), "raw"
    return None


def encode_text_expr(text: str, text_format: str, *, encoding: str = DEFAULT_ENCODING) -> bytes:
    payload = encode_text(text, encoding)
    if text_format == "push_string":
        if len(payload) > 0xFFFF:
            raise ValueError("PushString 文本超过 65535 字节")
        return bytes([PUSH_STRING]) + len(payload).to_bytes(2, "little") + payload
    if text_format == "raw":
        return payload
    raise ValueError(f"未知 _text_format: {text_format}")


def _strip_one_nul(data: bytes) -> bytes:
    # 兼容 YURIS_TOOLS 追加时留下的尾 0；正常 v5 expression_size 一般不含尾 0。
    if data.endswith(b"\x00"):
        return data[:-1]
    return data


def _looks_like_binary_expr(data: bytes) -> bool:
    if len(data) >= 3 and data[0] in {0x42, 0x46, 0x48, 0x49, 0x4C, 0x56, 0x57, 0x76}:
        return True
    return False


def _looks_like_translatable(text: str) -> bool:
    if not text:
        return False
    if text.strip() == "":
        return False
    # 过滤明显资源名/路径。这里保持保守，不做强过滤，避免漏提正文。
    lowered = text.lower()
    if lowered.endswith((".ogg", ".wav", ".png", ".bmp", ".jpg", ".jpeg", ".ybn")):
        return False
    return True
