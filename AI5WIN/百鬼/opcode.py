# -*- coding: utf-8 -*-
"""
Baigui MES opcode/common definitions.

This module is intentionally dependency-light because the required filename
`opcode.py` shadows Python's stdlib `opcode` module when scripts are run from
this directory.  Do not import dataclasses/inspect/dis here.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

DEFAULT_ENCODING = "cp932"

# ---------------------------------------------------------------------------
# AI5WIN/Okumura LZSS layer
# ---------------------------------------------------------------------------
LZSS_N = 4096
LZSS_F = 18
LZSS_INIT_POS = 0xFEE


def lzss_decompress(src: bytes, expected_size=None) -> bytes:
    text_buf = bytearray(LZSS_N)
    r = LZSS_INIT_POS
    flags = 0
    ip = 0
    out = bytearray()
    src = bytes(src)
    while ip < len(src):
        flags >>= 1
        if (flags & 0x100) == 0:
            if ip >= len(src):
                break
            flags = src[ip] | 0xFF00
            ip += 1
        if flags & 1:
            if ip >= len(src):
                break
            c = src[ip]
            ip += 1
            out.append(c)
            text_buf[r] = c
            r = (r + 1) & 0xFFF
        else:
            if ip + 1 >= len(src):
                break
            lo = src[ip]
            hi = src[ip + 1]
            ip += 2
            pos = lo | ((hi & 0xF0) << 4)
            length = (hi & 0x0F) + 3
            for k in range(length):
                c = text_buf[(pos + k) & 0xFFF]
                out.append(c)
                text_buf[r] = c
                r = (r + 1) & 0xFFF
                if expected_size is not None and len(out) >= expected_size:
                    return bytes(out[:expected_size])
        if expected_size is not None and len(out) >= expected_size:
            return bytes(out[:expected_size])
    return bytes(out)


def lzss_compress_literal(data: bytes) -> bytes:
    data = bytes(data)
    out = bytearray()
    for i in range(0, len(data), 8):
        chunk = data[i:i + 8]
        out.append((1 << len(chunk)) - 1)
        out.extend(chunk)
    return bytes(out)


def lzss_compress(data: bytes, max_candidates: int = 128) -> bytes:
    # Greedy compressor compatible with the game's decompressor.  It does not
    # reproduce the vendor compressed byte stream exactly, but decompression is
    # byte-exact on the plain MES layer.
    from collections import defaultdict, deque

    data = bytes(data)
    n = len(data)
    out = bytearray()
    pos = 0
    table = defaultdict(deque)

    def key_at(i):
        if i + 2 < n:
            return data[i:i + 3]
        return None

    def add_pos(i):
        k = key_at(i)
        if k is None:
            return
        dq = table[k]
        dq.append(i)
        while dq and i - dq[0] > LZSS_N:
            dq.popleft()
        while len(dq) > 256:
            dq.popleft()

    while pos < n:
        flag_pos = len(out)
        out.append(0)
        flags = 0
        for bit in range(8):
            if pos >= n:
                break
            best_len = 0
            best_abs = 0
            k = key_at(pos)
            dq = table.get(k) if k is not None else None
            if dq:
                while dq and pos - dq[0] > LZSS_N:
                    dq.popleft()
                checked = 0
                for cand in reversed(dq):
                    dist = pos - cand
                    if dist <= 0 or dist > LZSS_N:
                        continue
                    length = 0
                    while length < LZSS_F and pos + length < n and data[cand + length] == data[pos + length]:
                        length += 1
                    if length > best_len:
                        best_len = length
                        best_abs = cand
                        if length == LZSS_F:
                            break
                    checked += 1
                    if checked >= max_candidates:
                        break
            if best_len >= 3:
                ring_pos = (LZSS_INIT_POS + best_abs) & 0xFFF
                out.append(ring_pos & 0xFF)
                out.append(((ring_pos >> 4) & 0xF0) | (best_len - 3))
                for j in range(best_len):
                    add_pos(pos + j)
                pos += best_len
            else:
                flags |= 1 << bit
                out.append(data[pos])
                add_pos(pos)
                pos += 1
        out[flag_pos] = flags
    return bytes(out)


# ---------------------------------------------------------------------------
# VM summary definitions
# ---------------------------------------------------------------------------
EXPR_TERMINATOR = 0xFF
TEXT_STRING_CTRL = 0x01
TEXT_BLOCK_CTRL = 0x0B
TEXT_END_CTRL = 0x00

SJIS_LEAD_RANGES = ((0x81, 0x9F), (0xE0, 0xEF), (0xFA, 0xFC))
TEXT_CTRL_MNEMONICS = {
    0x00: "TEXT_END",
    0x01: "TEXT_STRING",
    0x0B: "TEXT_BLOCK_BOUNDARY",
    0x10: "TEXT_TIME_OR_PARAM",
    0x11: "TEXT_FLAG_CHECK",
}

VM_OPCODE_INFO = {
    0x00: {"mnemonic": "OP_00"},
    0x01: {"mnemonic": "TEXT_GATE", "handler": "sub_425AE0 -> sub_422F10"},
    0x02: {"mnemonic": "OP_02"},
    0x03: {"mnemonic": "NOP_OR_NULL"},
    0x04: {"mnemonic": "OP_04"},
    0x05: {"mnemonic": "OP_05"},
    0x06: {"mnemonic": "OP_06"},
    0x07: {"mnemonic": "OP_07"},
    0x08: {"mnemonic": "OP_08"},
    0x09: {"mnemonic": "OP_09"},
    0x0A: {"mnemonic": "OP_0A"},
    0x0B: {"mnemonic": "OP_0B"},
    0x0C: {"mnemonic": "OP_0C"},
    0x0D: {"mnemonic": "CALL_MES_CANDIDATE", "handler": "sub_427740"},
    0x0E: {"mnemonic": "OP_0E"},
    0x0F: {"mnemonic": "OP_0F"},
    0x10: {"mnemonic": "OP_10"},
    0x11: {"mnemonic": "OP_11"},
    0x12: {"mnemonic": "OP_12"},
    0x13: {"mnemonic": "OP_13"},
    0x14: {"mnemonic": "OP_14"},
    0x18: {"mnemonic": "CHOICE_MENU", "handler": "sub_428470"},
}

RESOURCE_SUFFIXES = (
    ".wav", ".mam", ".gpr", ".mes", ".bmp", ".png", ".jpg", ".jpeg",
    ".dat", ".avi", ".mpg", ".mpeg", ".mid", ".ogg",
    ".a6", ".a5", ".a4", ".an", ".bin",
)

# ---------------------------------------------------------------------------
# Encoding and placeholder helpers
# ---------------------------------------------------------------------------


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def decode_cp(data: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    return data.decode(encoding)


def encode_cp(text: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    return text.encode(encoding)


def is_sjis_lead(b: int) -> bool:
    return any(lo <= b <= hi for lo, hi in SJIS_LEAD_RANGES)


def is_probably_resource(text: str) -> bool:
    t = text.strip().lower().replace("\\", "/")
    if not t:
        return False
    if t.endswith(RESOURCE_SUFFIXES):
        return True
    if "/" in t and "." in t:
        return True
    # The game uses many ASCII identifiers in script commands, e.g.
    # bg26_1.a6, music_play, savedata keys.  They are not translatable text.
    if t.isascii():
        # Allow normal English prose only if it contains spaces/punctuation;
        # Baigui script command IDs are compact [a-z0-9_./-] tokens.
        compact = all((ch.isalnum() or ch in "_./-:") for ch in t)
        if compact:
            return True
    return False


def is_choice_label(text: str) -> bool:
    return text.startswith("選択肢") or text.startswith("選擇肢") or text.startswith("選択")


def is_display_string(text: str) -> bool:
    if not text:
        return False
    if is_probably_resource(text):
        return False
    # CP932 vendor/private-use bytes such as 0xF3 may decode to Unicode PUA;
    # if any such marker is present, this is control/binary data, not text.
    if any(0xE000 <= ord(ch) <= 0xF8FF for ch in text):
        return False
    if any(ord(ch) < 0x20 for ch in text):
        return False
    # Require at least one Japanese/fullwidth character for this game.
    has_jp = any(
        0x3040 <= ord(ch) <= 0x30FF or
        0x3400 <= ord(ch) <= 0x9FFF or
        0xFF00 <= ord(ch) <= 0xFFEF or
        ch in "「」『』（）、。！？…ー・"
        for ch in text
    )
    if not has_jp:
        return False
    return True


def escape_string_bytes(raw: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    """Render bytes into readable text plus {{XX}} placeholders.

    This avoids backslash escapes and avoids emitting raw undecodable bytes.
    """
    out = []
    i = 0
    while i < len(raw):
        b = raw[i]
        if b == 0:
            out.append("{{00}}")
            i += 1
            continue
        # Try two-byte CP932 first when lead byte matches.  The engine treats
        # 0x81-0x9F / 0xE0-0xEF / 0xFA-0xFC as lead bytes, so an orphan lead
        # at the end of a cstring is control data and must not be rendered as
        # Python's single-byte CP932 fallback such as Greek letters.
        if is_sjis_lead(b):
            if i + 1 < len(raw):
                pair = raw[i:i + 2]
                try:
                    out.append(pair.decode(encoding))
                    i += 2
                    continue
                except Exception:
                    pass
            out.append("{{%02X}}" % b)
            i += 1
            continue
        if 0x20 <= b <= 0x7E:
            ch = chr(b)
            # Avoid problematic quote/backslash escapes by placeholders.
            if ch in ['"', "\\", "{", "}"]:
                out.append("{{%02X}}" % b)
            else:
                out.append(ch)
            i += 1
            continue
        # Halfwidth kana and other cp932 single-byte chars.
        try:
            ch = bytes([b]).decode(encoding)
            if ch.isprintable() and ch not in ['"', "\\", "{", "}"]:
                out.append(ch)
            else:
                out.append("{{%02X}}" % b)
        except Exception:
            out.append("{{%02X}}" % b)
        i += 1
    return "".join(out)


def parse_placeholder_string(s: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    out = bytearray()
    i = 0
    while i < len(s):
        if s.startswith("{{", i):
            j = s.find("}}", i + 2)
            if j < 0:
                raise ValueError("unterminated placeholder")
            body = s[i + 2:j]
            parts = body.split(":")
            valid = True
            vals = []
            try:
                for p in parts:
                    if len(p) != 2:
                        valid = False
                        break
                    vals.append(int(p, 16))
            except Exception:
                valid = False
            if valid:
                out.extend(vals)
                i = j + 2
                continue
            # Robustness for old asm files where a literal '{' immediately
            # precedes a placeholder, e.g. "{{{0B}}" = "{" + "{{0B}}".
            if body.startswith("{"):
                out.extend("{".encode(encoding))
                i += 1
                continue
            raise ValueError("bad placeholder: {{%s}}" % body)
        else:
            # Encode one Unicode char at a time.
            out.extend(s[i].encode(encoding))
            i += 1
    return bytes(out)


def quote_asm_string(raw: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    return '"' + escape_string_bytes(raw, encoding) + '"'


def unquote_asm_string(token: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    token = token.strip()
    if len(token) < 2 or token[0] != '"' or token[-1] != '"':
        raise ValueError("expected quoted string")
    return parse_placeholder_string(token[1:-1], encoding)


def strip_comment(line: str) -> str:
    in_quote = False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == '"':
            in_quote = not in_quote
        elif ch == ';' and not in_quote:
            return line[:i]
        i += 1
    return line


def load_json(path):
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError("JSON root must be a list")
    return data


def save_json(path, obj):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2)
