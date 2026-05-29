# -*- coding: utf-8 -*-
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ENCODING = "cp932"
XOR_KEY = 0x7F
HEADER_SIZE = 0x14
IGNORE_PARAM = 0xFFFFFFFF

# 0 = do not treat as string reference
# -1 = string reference, rebuild/fix offset, but do not export
# 1 = string reference, rebuild/fix offset, export as translatable text
# 参数序号按 Command.params，从指令 offset+4 开始每 4 字节一个 u32。
STR_CODE_CONFIG: dict[int, list[int]] = {
    0x62: [0, -1, 1],      # ADV text: param1 voice/resource, param2 body/name+body
    0x6A: [0, 1],          # choice text: param1
    0xA5: [1],             # system confirm/prompt text

    # 已在样本中确认大量出现的资源/脚本/音频/图像字符串引用。
    # -1 参数全部按“探测式字符串引用”处理：命中字符串区 offset 才参与重建；
    # 未命中时认为是普通数值/flag/jump/id，原样保留，避免 0xD2 param0=0x5 这类误报。
    0x70: [-1],
    0x75: [-1],
    0x76: [-1, 0],
    0x77: [-1],
    0xA6: [-1],
    0xA7: [-1],
    0xA9: [-1],
    0xAF: [-1],
    0xD1: [-1, 0, -1, -1],
    0xD2: [-1, -1],      # mixed numeric/string args; detect only if value hits string offset
    0xD3: [-1, 0],
    0xD4: [0, -1],
    0xD5: [0, -1],
    0xDA: [-1],
    0xDB: [0, -1],
}

# 只有这些“实际导出文本”的参数必须严格命中字符串区；资源/控制参数一律保守探测。
STRICT_TEXT_REFS: set[tuple[int, int]] = {
    (0x62, 2),
    (0x6A, 1),
    (0xA5, 0),
}

# pasted code 里的 fixOrig 正向表：半角/私用字符 -> 正常显示字符。
FIX_KEY = '!?\uf8f0｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ'
FIX_VALUE = '！？　。「」、…をぁぃぅぇぉゃゅょっーあいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわん゛゜'
FIX_TO_NORMAL = str.maketrans(FIX_KEY, FIX_VALUE)
# 反向表只用于你选择 --fix-orig 提取后，又希望把全角假名直接注回原 SCR 编码风格的情况。
FIX_TO_RAW = str.maketrans(FIX_VALUE, FIX_KEY)


def read_u32(data: bytes | bytearray, off: int) -> int:
    return int.from_bytes(data[off:off + 4], "little")


def write_u32(buf: bytearray, off: int, value: int) -> None:
    buf[off:off + 4] = int(value).to_bytes(4, "little", signed=False)


def xor_7f(data: bytes) -> bytes:
    return bytes(b ^ XOR_KEY for b in data)


def decode_text(raw: bytes) -> str:
    return raw.decode(ENCODING, errors="strict")


def encode_text(text: str) -> bytes:
    return text.encode(ENCODING, errors="strict")


def make_addr_list(str_list: list[bytes]) -> list[int]:
    out: list[int] = []
    addr = 0
    for s in str_list:
        out.append(addr)
        addr += len(s) + 1
    return out


def ensure_json_list(path: Path) -> list[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"JSON root must be a list: {path}")
    return data


def save_json(path: Path, entries: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)
        f.write("\n")


@dataclass
class Command:
    offset: int
    code: int
    length: int
    data: bytearray
    params: list[int]
    ref_indices: dict[int, int]

    @classmethod
    def read_from(cls, cmd_sec: bytes, pos: int, addr_to_index: dict[int, int], strict: bool = True) -> "Command":
        if pos + 2 > len(cmd_sec):
            raise ValueError(f"truncated command header at cmd+0x{pos:X}")
        code = cmd_sec[pos]
        length = cmd_sec[pos + 1]
        if length < 4 or pos + length > len(cmd_sec):
            raise ValueError(f"bad command length at cmd+0x{pos:X}: code=0x{code:02X} len=0x{length:X}")
        data = bytearray(cmd_sec[pos:pos + length])
        param_count = length // 4 - 1
        params = [read_u32(data, 4 + i * 4) for i in range(param_count)]
        ref_indices: dict[int, int] = {}
        cfg = STR_CODE_CONFIG.get(code)
        if cfg:
            for i, flag in enumerate(cfg[:param_count]):
                if flag == 0:
                    continue
                p = params[i]
                if p == IGNORE_PARAM:
                    continue
                if p not in addr_to_index:
                    # 资源/控制 OP 里混有普通数值、flag、jump/id；不要因为 -1 参数未命中就报错。
                    # 只有确认的正文/选项/系统文本参数需要严格命中，否则提取结果本身不可信。
                    if strict and (code, i) in STRICT_TEXT_REFS:
                        raise ValueError(
                            f"string offset not found: cmd+0x{pos:X} op=0x{code:02X} param{i}=0x{p:X}"
                        )
                    continue
                ref_indices[i] = addr_to_index[p]
        return cls(pos, code, length, data, params, ref_indices)

    def write_back_refs(self, addr_list: list[int]) -> bytes:
        cfg = STR_CODE_CONFIG.get(self.code)
        if cfg:
            for param_i, str_i in self.ref_indices.items():
                if str_i < 0 or str_i >= len(addr_list):
                    raise ValueError(
                        f"bad string index in cmd+0x{self.offset:X} op=0x{self.code:02X}: {str_i}"
                    )
                value = addr_list[str_i]
                self.params[param_i] = value
                write_u32(self.data, 4 + param_i * 4, value)
        return bytes(self.data)


class Scr0034:
    def __init__(self) -> None:
        self.header = b""
        self.cmd_sec = b""
        self.str_list: list[bytes] = []
        self.addr_list: list[int] = []
        self.commands: list[Command] = []

    @classmethod
    def read(cls, data: bytes, strict: bool = True) -> "Scr0034":
        if len(data) < HEADER_SIZE + 8:
            raise ValueError("file too small for SCR0034")
        obj = cls()
        obj.header = data[:HEADER_SIZE]
        if not obj.header.startswith(b"SCR:0034"):
            raise ValueError(f"not SCR:0034: magic={obj.header[:8]!r}")
        cmd_len = read_u32(data, HEADER_SIZE)
        cmd_start = HEADER_SIZE + 4
        cmd_end = cmd_start + cmd_len
        if cmd_end + 4 > len(data):
            raise ValueError("cmd section exceeds file size")
        obj.cmd_sec = data[cmd_start:cmd_end]
        str_len = read_u32(data, cmd_end)
        str_start = cmd_end + 4
        str_end = str_start + str_len
        if str_end > len(data):
            raise ValueError("string section exceeds file size")
        if str_end != len(data):
            # 当前样本应正好结束；保留严格报错，防止漏尾部未知数据。
            if strict:
                raise ValueError(f"trailing data exists: {len(data) - str_end} bytes")
        dec_str_sec = xor_7f(data[str_start:str_end])
        obj.str_list = dec_str_sec.split(b"\x00")
        obj.addr_list = make_addr_list(obj.str_list)
        addr_to_index = {addr: i for i, addr in enumerate(obj.addr_list)}
        pos = 0
        while pos < len(obj.cmd_sec):
            cmd = Command.read_from(obj.cmd_sec, pos, addr_to_index, strict=strict)
            obj.commands.append(cmd)
            pos += cmd.length
        if pos != len(obj.cmd_sec):
            raise ValueError("command parse did not end exactly")
        return obj

    def to_bytes(self, encrypt: bool = True) -> bytes:
        self.addr_list = make_addr_list(self.str_list)
        cmd_blob = b"".join(cmd.write_back_refs(self.addr_list) for cmd in self.commands)
        str_plain = b"\x00".join(self.str_list)
        str_blob = xor_7f(str_plain) if encrypt else str_plain
        out = bytearray()
        out.extend(self.header)
        out.extend(len(cmd_blob).to_bytes(4, "little"))
        out.extend(cmd_blob)
        out.extend(len(str_blob).to_bytes(4, "little"))
        out.extend(str_blob)
        return bytes(out)


def strip_one_trailing_lf(text: str) -> tuple[str, str]:
    if text.endswith("\n"):
        return text[:-1], "\n"
    return text, ""


def split_display_text(slot_text: str, opcode: int) -> tuple[str | None, str, str, str, str]:
    """Return (name, scr_msg, prefix, suffix, type). prefix+message+suffix rebuilds slot."""
    body, suffix = strip_one_trailing_lf(slot_text)
    if opcode == 0x62 and "\n" in body:
        maybe_name, maybe_msg = body.split("\n", 1)
        # SCR0034 的对话槽通常是 角色名\n｢正文｣\n；旁白槽没有这个前缀。
        if maybe_name and (maybe_msg.startswith("｢") or maybe_msg.startswith("「") or maybe_msg.startswith("\1") or maybe_msg.startswith("ﾞ")):
            return maybe_name, maybe_msg, maybe_name + "\n", suffix, "dialogue"
    if opcode == 0x6A:
        return None, body, "", suffix, "choice"
    if opcode == 0xA5:
        return None, body, "", suffix, "system"
    return None, body, "", suffix, "monologue"


def normalize_for_export(text: str, fix_orig: bool) -> str:
    return text.translate(FIX_TO_NORMAL) if fix_orig else text


def denormalize_for_import(text: str, fix_orig: bool) -> str:
    return text.translate(FIX_TO_RAW) if fix_orig else text
