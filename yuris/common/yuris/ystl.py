# -*- coding: utf-8 -*-
"""解析 YU-RIS yst_list.ybn 脚本列表。"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .common import DEFAULT_ENCODING, read_u32_le

YSTL_MAGIC = b"YSTL"


@dataclass
class ScriptInfo:
    id: int
    source: str


class Ystl:
    def __init__(self, scripts: list[ScriptInfo], version: int = 0):
        self.scripts = scripts
        self.version = version
        self.by_id = {s.id: s for s in scripts}

    @classmethod
    def read(cls, path: str | Path, encoding: str = DEFAULT_ENCODING) -> "Ystl":
        data = Path(path).read_bytes()
        if data[:4] != YSTL_MAGIC:
            raise ValueError(f"不是 YSTL 文件: {path}")
        pos = 4
        version = read_u32_le(data, pos); pos += 4
        count = read_u32_le(data, pos); pos += 4
        scripts: list[ScriptInfo] = []
        for _ in range(count):
            sid = read_u32_le(data, pos); pos += 4
            size = read_u32_le(data, pos); pos += 4
            source = data[pos:pos + size].decode(encoding); pos += size
            pos += 16
            if version > 462:
                pos += 4
            scripts.append(ScriptInfo(sid, source))
        return cls(scripts, version)

    def get_source(self, script_id: int) -> str | None:
        info = self.by_id.get(script_id)
        return info.source if info else None
