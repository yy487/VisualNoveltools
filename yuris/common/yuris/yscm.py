# -*- coding: utf-8 -*-
"""解析 YU-RIS ysc.ybn 命令表。"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .common import DEFAULT_ENCODING, read_u32_le

YSCM_MAGIC = b"YSCM"


@dataclass
class ExpressionInfo:
    keyword: str
    result_type: int
    validate_mode: int

    @property
    def is_raw(self) -> bool:
        # YuRis_Tool: ExprEvalResult.Raw == 3
        return self.result_type == 3


@dataclass
class CommandInfo:
    id: int
    name: str
    exprs: list[ExpressionInfo]


class Yscm:
    def __init__(self, commands: list[CommandInfo], version: int = 0):
        self.commands = commands
        self.version = version
        self._by_name = {c.name.upper(): c for c in commands}

    @classmethod
    def read(cls, path: str | Path, encoding: str = DEFAULT_ENCODING) -> "Yscm":
        data = Path(path).read_bytes()
        if data[:4] != YSCM_MAGIC:
            raise ValueError(f"不是 YSCM 文件: {path}")
        pos = 4
        version = read_u32_le(data, pos); pos += 4
        count = read_u32_le(data, pos); pos += 4
        pos += 4  # zero
        commands: list[CommandInfo] = []
        for cid in range(count):
            name, pos = _read_zstr(data, pos, encoding)
            expr_count = data[pos]; pos += 1
            exprs: list[ExpressionInfo] = []
            for _ in range(expr_count):
                keyword, pos = _read_zstr(data, pos, encoding)
                result_type = data[pos]; pos += 1
                validate_mode = data[pos]; pos += 1
                exprs.append(ExpressionInfo(keyword, result_type, validate_mode))
            commands.append(CommandInfo(cid, name, exprs))
        return cls(commands, version)

    def get_command(self, command_id: int) -> CommandInfo | None:
        if 0 <= command_id < len(self.commands):
            return self.commands[command_id]
        return None

    def find_command_id(self, name: str) -> int | None:
        info = self._by_name.get(name.upper())
        return None if info is None else info.id

    def command_name(self, command_id: int) -> str:
        info = self.get_command(command_id)
        return info.name if info else f"CMD_{command_id:02X}"

    def get_expr_info(self, command_id: int, expr_id: int) -> ExpressionInfo | None:
        info = self.get_command(command_id)
        if not info or not (0 <= expr_id < len(info.exprs)):
            return None
        return info.exprs[expr_id]


def _read_zstr(data: bytes, pos: int, encoding: str) -> tuple[str, int]:
    end = data.index(0, pos)
    return data[pos:end].decode(encoding), end + 1
