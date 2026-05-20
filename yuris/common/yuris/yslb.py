# -*- coding: utf-8 -*-
"""解析 YU-RIS ysl.ybn 标签表。"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .common import DEFAULT_ENCODING, read_u16_le, read_u32_le

YSLB_MAGIC = b"YSLB"


@dataclass
class Label:
    name: str
    name_hash: int
    command_index: int
    script_id: int
    unk2: int
    unk3: int


class Yslb:
    def __init__(self, labels: list[Label], version: int = 0):
        self.labels = labels
        self.version = version
        self._by_pos: dict[tuple[int, int], list[Label]] = {}
        for lab in labels:
            self._by_pos.setdefault((lab.script_id, lab.command_index), []).append(lab)

    @classmethod
    def read(cls, path: str | Path, encoding: str = DEFAULT_ENCODING) -> "Yslb":
        data = Path(path).read_bytes()
        if data[:4] != YSLB_MAGIC:
            raise ValueError(f"不是 YSLB 文件: {path}")
        pos = 4
        version = read_u32_le(data, pos); pos += 4
        count = read_u32_le(data, pos); pos += 4
        pos += 256 * 4
        labels: list[Label] = []
        for _ in range(count):
            size = data[pos]; pos += 1
            name = data[pos:pos + size].decode(encoding); pos += size
            name_hash = read_u32_le(data, pos); pos += 4
            command_index = read_u32_le(data, pos); pos += 4
            script_id = read_u16_le(data, pos); pos += 2
            unk2 = data[pos]; unk3 = data[pos + 1]; pos += 2
            labels.append(Label(name, name_hash, command_index, script_id, unk2, unk3))
        return cls(labels, version)

    def find(self, script_id: int, command_index: int) -> list[Label]:
        return self._by_pos.get((script_id, command_index), [])
