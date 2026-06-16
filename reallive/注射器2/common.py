# -*- coding: utf-8 -*-
from __future__ import annotations

import json, re, struct
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Iterator

DEFAULT_ENCODING = 'cp932'
SEEN_INDEX_COUNT = 10000
SEEN_INDEX_SIZE = SEEN_INDEX_COUNT * 8
SEEN_MAGIC = 10002
XOR_KEY_VA = 0x596984


def read_u32(data: bytes, off: int) -> int:
    return struct.unpack_from('<I', data, off)[0]

def write_u32(buf: bytearray, off: int, val: int) -> None:
    struct.pack_into('<I', buf, off, val & 0xffffffff)

def is_sjis_lead(b: int) -> bool:
    return (0x80 <= b < 0xA0) or (0xDF < b < 0xFE)

def decode_text(raw: bytes, encoding: str = DEFAULT_ENCODING) -> str:
    return raw.decode(encoding, errors='replace')

def encode_text(s: str, encoding: str = DEFAULT_ENCODING) -> bytes:
    return s.encode(encoding)

def load_static_xor_key() -> bytes:
    """Return the bundled 256-byte XOR key for this RealLive build."""
    try:
        from crypt_template import STATIC_XOR_KEY
    except Exception as e:
        raise RuntimeError('static crypt template not found; pass an IDA export directory instead') from e
    if len(STATIC_XOR_KEY) != 256:
        raise RuntimeError(f'STATIC_XOR_KEY must be 256 bytes, got {len(STATIC_XOR_KEY)}')
    return STATIC_XOR_KEY


def load_xor_key_from_export(export_dir: str | Path | None = None) -> bytes:
    """Read byte_596984 from IDA export memory/*.txt, or use static template.

    The decompile shows sub_4071D0 XORing compressed SEEN code with
    byte_596984[i % 256].  For this project the table is bundled in
    crypt_template.py so the normal workflow no longer depends on
    RealLive.exe_export_for_ai.  Passing export_dir keeps compatibility and is
    useful when adapting the tool to another RealLive executable.
    """
    if export_dir is None or str(export_dir) == '':
        return load_static_xor_key()
    export_dir = Path(export_dir)
    if not (export_dir / 'memory').is_dir():
        return load_static_xor_key()
    bmap: dict[int, int] = {}
    for fname in (export_dir / 'memory').glob('*.txt'):
        for line in fname.read_text(errors='ignore').splitlines():
            if '|' not in line or line.startswith('#'):
                continue
            parts = line.split('|')
            if len(parts) < 3:
                continue
            try:
                addr = int(parts[0].strip(), 16)
                bs = bytes.fromhex(parts[1].replace(' ', ''))
            except Exception:
                continue
            for i, b in enumerate(bs):
                bmap[addr + i] = b
    try:
        return bytes(bmap[XOR_KEY_VA + i] for i in range(256))
    except KeyError as e:
        raise RuntimeError(f'cannot read 256-byte XOR key at VA 0x{XOR_KEY_VA:X} from {export_dir}/memory') from e

def xor_crypt(data: bytes | bytearray, key: bytes) -> bytes:
    return bytes((b ^ key[i & 0xff]) for i, b in enumerate(data))

def lz_decompress(buf: bytes) -> bytes:
    """RealLive LZ decoder: sub_40A240.

    input layout: u32 packed_size, u32 unpacked_size, then flag/literal/backref stream.
    A 0 flag bit reads a 16-bit copy token: count=(token&0xf)+2, distance=token>>4.
    """
    if len(buf) < 8:
        raise ValueError('compressed stream too short')
    out_len = read_u32(buf, 4)
    src = 8
    out = bytearray()
    while len(out) < out_len:
        if src >= len(buf):
            raise ValueError('truncated LZ flag stream')
        flags = buf[src]
        src += 1
        for _ in range(8):
            if len(out) >= out_len:
                break
            if flags & 1:
                if src >= len(buf):
                    raise ValueError('truncated LZ literal')
                out.append(buf[src]); src += 1
            else:
                if src + 2 > len(buf):
                    raise ValueError('truncated LZ backref')
                token = buf[src] | (buf[src+1] << 8); src += 2
                count = (token & 0x0f) + 2
                dist = token >> 4
                if dist == 0 or dist > len(out):
                    raise ValueError(f'invalid LZ backref dist={dist} at src=0x{src:x}')
                for _ in range(count):
                    out.append(out[-dist])
            flags >>= 1
    return bytes(out)

@dataclass
class SeenEntry:
    seen_no: int
    offset: int
    size: int

@dataclass
class SeenHeader:
    header_size: int
    magic: int
    line_table_off: int
    line_count: int
    line_table_size: int
    string_table_off: int
    string_count: int
    string_table_size: int
    code_off: int
    unpacked_size: int
    packed_size: int
    flags: int

    @classmethod
    def from_bytes(cls, data: bytes) -> 'SeenHeader':
        if len(data) < 48:
            raise ValueError('SEEN chunk too short for header')
        return cls(*struct.unpack_from('<12I', data, 0))

@dataclass
class DecodedSeen:
    entry: SeenEntry
    header: SeenHeader
    raw_chunk: bytes
    decoded_chunk: bytes
    code: bytes
    strings: list[tuple[int, str, bytes]]


def iter_seen_entries(seen_data: bytes) -> Iterator[SeenEntry]:
    if len(seen_data) < SEEN_INDEX_SIZE:
        raise ValueError('Seen.txt shorter than 0x13880 index table')
    for i in range(SEEN_INDEX_COUNT):
        off, size = struct.unpack_from('<II', seen_data, i * 8)
        if off or size:
            yield SeenEntry(i, off, size)

def parse_string_table(chunk: bytes, hdr: SeenHeader) -> list[tuple[int, str, bytes]]:
    out: list[tuple[int, str, bytes]] = []
    pos = hdr.string_table_off
    end = hdr.string_table_off + hdr.string_table_size
    for _ in range(hdr.string_count):
        if pos + 4 > end:
            break
        n = read_u32(chunk, pos)
        raw = chunk[pos+4:pos+4+n]
        out.append((pos, decode_text(raw.rstrip(b'\x00')), raw))
        pos += 4 + n
    return out

def decode_seen_chunk(entry: SeenEntry, seen_data: bytes, key: bytes) -> DecodedSeen:
    chunk = seen_data[entry.offset:entry.offset+entry.size]
    hdr = SeenHeader.from_bytes(chunk)
    if hdr.magic != SEEN_MAGIC:
        raise ValueError(f'SEEN{entry.seen_no:04d}: bad magic {hdr.magic}')
    packed_enc = chunk[hdr.code_off:hdr.code_off+hdr.packed_size]
    packed = xor_crypt(packed_enc, key)
    if read_u32(packed, 0) != hdr.packed_size or read_u32(packed, 4) != hdr.unpacked_size:
        raise ValueError(f'SEEN{entry.seen_no:04d}: decoded packed header mismatch')
    code = lz_decompress(packed)
    decoded_chunk = chunk[:hdr.code_off] + code
    return DecodedSeen(entry, hdr, chunk, decoded_chunk, code, parse_string_table(chunk, hdr))

@dataclass
class TextSpan:
    code_off: int
    size: int
    raw_hex: str

@dataclass
class TextEntry:
    scr_msg: str
    message: str
    _file: str
    _index: int
    _seen_no: int
    _line: int | None
    _code_offset: int
    _size: int
    name: str | None = None
    _scr_name: str | None = None
    _type: str = 'message'
    _encoding: str = DEFAULT_ENCODING
    _policy: str = 'relocate'
    _text_spans: list[dict[str, Any]] | None = None
    _name_source: str | None = None

    def to_json(self) -> dict[str, Any]:
        # Keep translator-facing fields in the project order: name, scr_msg, message.
        out: dict[str, Any] = {}
        if self.name is not None:
            out['name'] = self.name
        out['scr_msg'] = self.scr_msg
        out['message'] = self.message
        for k, v in asdict(self).items():
            if k in ('name', 'scr_msg', 'message') or v is None:
                continue
            out[k] = v
        return out


def split_bracket_name(text: str) -> tuple[str | None, str, str | None]:
    """Split RealLive inline speaker prefix: 【葵】本文 -> (葵, 本文).

    The VM stores the speaker name and message as one rendered text span in this
    title.  JSON exposes them separately, while injection reconstructs the same
    inline form before writing the span back.
    """
    m = re.match(r'^【([^】]{1,40})】(.+)$', text, flags=re.S)
    if not m:
        return None, text, None
    name = m.group(1)
    body = m.group(2)
    if not name.strip() or not body:
        return None, text, None
    return name, body, 'bracket_prefix'


def compose_entry_text(entry: dict[str, Any], body: str, *, source_field: str = 'message') -> str:
    """Compose the actual VM text written/read for a JSON entry.

    For bracket-prefix dialogue the VM stores one inline string:
    `【speaker】message`.  JSON exposes `name` as the editable/target name and
    `scr_msg` as the immutable source body.  Validation must therefore use the
    original speaker name, not the possibly translated `name` field.  Newer
    extracted JSON stores that original speaker in `_scr_name`; older JSON does
    not, so injection also has a body-only fallback in validate_entry().
    """
    if entry.get('_name_source') == 'bracket_prefix':
        if source_field == 'scr_msg':
            nm = entry.get('_scr_name')
            if isinstance(nm, str):
                return f'【{nm}】{body}'
        nm = entry.get('name')
        if isinstance(nm, str):
            return f'【{nm}】{body}'
    return body


def skip_dollar_expr(code: bytes, pos: int) -> int:
    """Skip one RealLive expression token beginning with '$'.

    This follows the byte stream behaviour used by sub_465940/sub_43C6A0 well
    enough for structural text extraction: '$ FF <u32>' is an immediate; other
    '$id[...]' expressions are skipped until a top-level delimiter.  The body is
    expression data, so bytes such as '(' or '#' inside an immediate must never
    be interpreted as script syntax.
    """
    start = pos
    pos += 1
    if pos >= len(code):
        return pos
    if code[pos] == 0xFF:
        return min(pos + 5, len(code))
    depth = 0
    quoted = False
    while pos < len(code):
        b = code[pos]
        if quoted:
            if b == ord('\\'):
                pos += 2; continue
            if b == ord('"'):
                quoted = False; pos += 1; continue
            if is_sjis_lead(b):
                pos += 2; continue
            pos += 1; continue
        if b == ord('"'):
            quoted = True; pos += 1; continue
        if is_sjis_lead(b):
            pos += 2; continue
        if b in (0, 0x0A, ord(','), ord(')')) and depth <= 0:
            break
        if b in (ord('['), ord('('), ord('{')):
            depth += 1
        elif b in (ord(']'), ord(')'), ord('}')):
            if depth <= 0:
                break
            depth -= 1
        pos += 1
        if pos - start > 4096:
            break
    return pos


def skip_bare_arg(code: bytes, pos: int) -> int:
    """Skip a non-$ command argument: bare resource name or quoted literal."""
    quoted = False
    while pos < len(code):
        b = code[pos]
        if quoted:
            if b == ord('\\'):
                pos += 2; continue
            if b == ord('"'):
                quoted = False; pos += 1; continue
            if is_sjis_lead(b):
                pos += 2; continue
            pos += 1; continue
        if b == ord('"'):
            quoted = True; pos += 1; continue
        if is_sjis_lead(b):
            pos += 2; continue
        if b in (ord(','), ord(')')):
            break
        pos += 1
    return pos


def skip_command_args(code: bytes, pos: int, argc: int) -> int:
    """Skip one parenthesized command-argument block.

    RealLive expression bytecode uses many printable bytes as operators; a byte
    value 0x28 '(' inside an expression is not necessarily a nested source
    parenthesis.  The previous parser balanced these bytes and therefore
    skipped past the real command-argument terminator, missing the raw u32 jump
    target immediately after it.  Here we scan to the first real ')' while
    treating CP932 lead bytes, quoted strings, and `$FF <u32>` immediates as
    opaque atoms.
    """
    if pos >= len(code) or code[pos] != ord('('):
        return pos
    pos += 1
    quoted = False
    while pos < len(code):
        b = code[pos]
        if quoted:
            if b == ord('\\'):
                pos += 2; continue
            if b == ord('"'):
                quoted = False; pos += 1; continue
            if is_sjis_lead(b):
                pos += 2; continue
            pos += 1; continue
        if b == ord('"'):
            quoted = True; pos += 1; continue
        if is_sjis_lead(b):
            pos += 2; continue
        if b == ord('$') and pos + 1 < len(code) and code[pos + 1] == 0xFF:
            pos = min(pos + 6, len(code)); continue
        if b == ord(')'):
            return pos + 1
        pos += 1
    return pos

def skip_hash_command(code: bytes, pos: int) -> int:
    """Skip a RealLive '#' command and any inline flow payload.

    Header layout from sub_463E00:
      '#' cls:u8 grp:u8 op:u16 argc:i16 flag:u8

    Class 0 / group 1 flow commands embed VM-code offsets after the argument
    list.  These bytes are not text and must be skipped during extraction.
    """
    if pos + 8 > len(code):
        return len(code)
    cls = code[pos + 1]
    grp = code[pos + 2]
    op = code[pos + 3] | (code[pos + 4] << 8)
    argc = int.from_bytes(code[pos + 5:pos + 7], 'little', signed=True)
    cmd_pos = pos
    pos += 8
    pos = skip_command_args(code, pos, argc)
    n = len(code)

    if cls == 0 and grp == 1:
        if op in (0, 2, 5):
            if pos + 4 <= n:
                return pos + 4
            return pos
        if op in (3, 8):
            if pos < n and code[pos] == ord('{'):
                pos += 1
                count = max(argc, 0)
                done = 0
                while pos + 4 <= n and code[pos] != ord('}') and (count == 0 or done < count):
                    pos += 4
                    done += 1
                if pos < n and code[pos] == ord('}'):
                    pos += 1
            return pos
        if op == 4:
            if pos < n and code[pos] == ord('{'):
                pos += 1
                guard = 0
                while pos < n and code[pos] != ord('}') and guard < 100000:
                    guard += 1
                    if code[pos] == ord('('):
                        pos = skip_command_args(code, pos, 1)
                        if pos + 4 <= n:
                            pos += 4
                        continue
                    if code[pos] == ord('$') and pos + 1 < n and code[pos + 1] == 0xFF:
                        pos = min(pos + 6, n)
                        continue
                    if is_sjis_lead(code[pos]):
                        pos += 2
                    else:
                        pos += 1
                if pos < n and code[pos] == ord('}'):
                    pos += 1
            return pos
    return max(pos, cmd_pos + 1)

def iter_text_entries(decoded: DecodedSeen) -> list[TextEntry]:
    """Extract visible text by emulating the message-stream parser.

    This follows the control-byte dispatch in sub_465940: 0x0A is a line marker;
    @/! are renderer controls; # dispatches sub_463E00; $ is an expression; ','
    is a separator; '"' toggles quote mode; otherwise bytes are rendered as
    text.  Choice labels are recognized only after #00:02:0003 and the following
    brace block.  This is still structural VM parsing, not a naked SJIS scan.
    """
    code = decoded.code
    entries: list[TextEntry] = []
    pos = 0
    cur_line: int | None = None
    index = 0
    buf = bytearray()
    spans: list[TextSpan] = []
    quote_mode = False
    expect_choice_block = False
    choice_depth = 0

    def text_quality_ok(text: str) -> bool:
        if not text or text.isspace():
            return False
        has_kana = any('\u3040' <= c <= '\u30ff' for c in text)
        has_cjk = any('\u4e00' <= c <= '\u9fff' for c in text)
        has_jp_punct = any(c in '「」『』【】（）、。…！？ー～・♪♥' for c in text)
        if len(text.strip()) <= 1 and not (has_kana or has_jp_punct):
            return False
        if text.startswith(')') and not (has_kana or has_jp_punct):
            return False
        return has_kana or has_cjk or has_jp_punct

    def flush(until_pos: int, force_type: str | None = None) -> None:
        nonlocal index, buf, spans
        if not buf:
            return
        raw = bytes(buf)
        if raw:
            text = decode_text(raw).replace('\r', '')
            if text_quality_ok(text):
                etype = force_type or ('choice' if choice_depth > 0 else 'message')
                name, body, name_source = (None, text, None)
                if etype == 'message':
                    name, body, name_source = split_bracket_name(text)
                entries.append(TextEntry(
                    name=name,
                    _scr_name=name if name_source == 'bracket_prefix' else None,
                    scr_msg=body,
                    message=body,
                    _file=f'Seen{decoded.entry.seen_no:04d}',
                    _index=index,
                    _seen_no=decoded.entry.seen_no,
                    _line=cur_line,
                    _code_offset=spans[0].code_off if spans else until_pos,
                    _size=sum(s.size for s in spans),
                    _type=etype,
                    _text_spans=[asdict(s) for s in spans],
                    _name_source=name_source,
                ))
                index += 1
        buf.clear(); spans.clear()

    def append_bytes(start: int, raw: bytes) -> None:
        if raw:
            buf.extend(raw)
            spans.append(TextSpan(start, len(raw), raw.hex()))

    while pos < len(code):
        b = code[pos]

        if b == 0:
            flush(pos)
            quote_mode = False
            pos += 1
            continue

        if b == 0x0A and pos + 2 < len(code):
            flush(pos, 'choice' if choice_depth > 0 else None)
            quote_mode = False
            cur_line = code[pos + 1] | (code[pos + 2] << 8)
            pos += 3
            continue

        if b == ord('"'):
            quote_mode = not quote_mode
            pos += 1
            continue

        if not quote_mode:
            if b == ord('@') or b == ord('!'):
                if pos + 2 < len(code):
                    flush(pos)
                    pos += 3
                    continue
            if b == ord('#'):
                flush(pos)
                expect_choice_block = False
                if pos + 8 <= len(code):
                    cls = code[pos + 1]
                    grp = code[pos + 2]
                    op = code[pos + 3] | (code[pos + 4] << 8)
                    expect_choice_block = (cls == 0 and grp == 2 and op == 3)
                pos = skip_hash_command(code, pos)
                continue
            if b == ord('$'):
                flush(pos)
                pos = skip_dollar_expr(code, pos)
                continue
            if b == ord(','):
                flush(pos)
                pos += 1
                continue
            if b == ord('{') and expect_choice_block:
                flush(pos)
                choice_depth += 1
                expect_choice_block = False
                pos += 1
                continue
            if b == ord('}') and choice_depth > 0:
                flush(pos, 'choice')
                choice_depth -= 1
                pos += 1
                continue
            if b in (ord('{'), ord('}')):
                flush(pos)
                expect_choice_block = False
                pos += 1
                continue

        expect_choice_block = False
        if is_sjis_lead(b) and pos + 1 < len(code):
            append_bytes(pos, code[pos:pos + 2])
            pos += 2
            continue
        if 0x20 <= b < 0x7F or 0xA0 <= b <= 0xDF:
            append_bytes(pos, bytes([b]))
            pos += 1
            continue

        flush(pos)
        quote_mode = False
        pos += 1

    flush(pos)
    return entries



def lz_compress_literal(raw: bytes) -> bytes:
    """Build a valid RealLive packed stream using literal-only LZ packets.

    Kept as a fallback/debug compressor. It always works but can expand the
    stream to roughly 112.5% of decoded code size.
    """
    out = bytearray(b"\x00" * 8)
    pos = 0
    while pos < len(raw):
        n = min(8, len(raw) - pos)
        out.append((1 << n) - 1)
        out.extend(raw[pos:pos+n])
        pos += n
    write_u32(out, 0, len(out))
    write_u32(out, 4, len(raw))
    return bytes(out)


def _find_lz_match(raw: bytes, pos: int, last: dict[bytes, list[int]]) -> tuple[int, int]:
    """Return (distance, length) for the RealLive 12-bit-distance LZ token.

    Token layout is <distance:12><length_minus_2:4>; therefore length is 2..17
    and distance is 1..4095.  A small rolling dictionary is enough for this VM
    code because command/line markers repeat densely.
    """
    if pos + 2 > len(raw):
        return 0, 0
    key = raw[pos:pos + 2]
    best_dist = best_len = 0
    for prev in reversed(last.get(key, [])):
        dist = pos - prev
        if dist <= 0:
            continue
        if dist > 0x0FFF:
            break
        max_len = min(17, len(raw) - pos)
        n = 0
        while n < max_len and raw[prev + n] == raw[pos + n]:
            n += 1
        if n > best_len and n >= 2:
            best_len = n
            best_dist = dist
            if n == 17:
                break
    return best_dist, best_len


def lz_compress(raw: bytes, level: str = 'greedy') -> bytes:
    """RealLive-compatible LZ compressor.

    The decoder used by this title is the sub_40A240 stream: one flag byte, then
    eight items, LSB first; flag bit 1 means literal, flag bit 0 means a 16-bit
    back-reference token.  The output header stores packed_size and
    unpacked_size, both little-endian u32.
    """
    if level == 'literal':
        return lz_compress_literal(raw)

    out = bytearray(b"\x00" * 8)
    pos = 0
    recent: dict[bytes, list[int]] = {}

    def remember(at: int) -> None:
        if at + 2 > len(raw):
            return
        key = raw[at:at + 2]
        bucket = recent.setdefault(key, [])
        bucket.append(at)
        # Keep enough candidates for good compression without becoming quadratic.
        if len(bucket) > 64:
            del bucket[:-64]

    while pos < len(raw):
        flag_pos = len(out)
        out.append(0)
        flags = 0
        for bit in range(8):
            if pos >= len(raw):
                break
            dist, n = _find_lz_match(raw, pos, recent)
            if n >= 3:
                token = (dist << 4) | (n - 2)
                out.extend(token.to_bytes(2, 'little'))
                for i in range(n):
                    remember(pos + i)
                pos += n
            else:
                flags |= (1 << bit)
                out.append(raw[pos])
                remember(pos)
                pos += 1
        out[flag_pos] = flags

    write_u32(out, 0, len(out))
    write_u32(out, 4, len(raw))
    return bytes(out)


def build_offset_delta_map(replacements: list[tuple[int, int, int]]) -> list[tuple[int, int]]:
    """Return [(old_position_after_replacement, accumulated_delta), ...]."""
    acc = 0
    out: list[tuple[int, int]] = []
    for start, old_len, new_len in sorted(replacements):
        acc += new_len - old_len
        out.append((start + old_len, acc))
    return out


def remap_code_offset(old_off: int, delta_map: list[tuple[int, int]]) -> int:
    delta = 0
    for limit, acc in delta_map:
        if old_off >= limit:
            delta = acc
        else:
            break
    return old_off + delta


def patch_line_table_for_new_code(chunk_prefix: bytearray, hdr: SeenHeader, old_code_size: int,
                                  delta_map: list[tuple[int, int]]) -> None:
    """Patch line-table values that look like code offsets.

    In this title each line-table element is a little-endian u32. Values inside
    the old code range are VM code offsets used by line/scenario jumps; sentinel
    values such as 1000000 are left untouched.
    """
    start = hdr.line_table_off
    count = min(hdr.line_count, hdr.line_table_size // 4)
    for i in range(count):
        off = start + i * 4
        val = read_u32(chunk_prefix, off)
        if 0 <= val < old_code_size:
            write_u32(chunk_prefix, off, remap_code_offset(val, delta_map))




def _range_contains(repls: list[tuple[int, int, int]], pos: int, size: int = 4) -> bool:
    end = pos + size
    for st, old_len, _new_len in repls:
        if pos < st + old_len and end > st:
            return True
    return False



def iter_flow_jump_field_offsets(code: bytes) -> Iterator[int]:
    """Yield byte offsets of VM-code absolute jump operands in #00:01 flow ops.

    RealLive does not encode all branch targets as `) <u32>` / `} <u32>`.
    Several flow commands store raw target offsets immediately after the 8-byte
    command header or inside a brace jump table.  If text relocation changes code
    length, every one of these operands must be remapped.

    Observed layouts in this title:
      #00:01:0000()       <u32 target>          unconditional branch
      #00:01:0002(expr)   <u32 target>          conditional branch
      #00:01:0005()       <u32 target>          branch/call-like flow
      #00:01:0003(expr)   { <u32>*argc }        selection/choice jump table
      #00:01:0008(expr)   { <u32>*argc }        multi-way jump table
      #00:01:0004(expr)   { (case) <u32> ... }  switch table

    The scanner is structural: it starts only from a `#` command header and uses
    the expression skipper for `$FF <u32>` immediates, so numeric constants are
    not treated as branch targets.
    """
    pos = 0
    n = len(code)
    while pos + 8 <= n:
        if code[pos] != ord('#'):
            pos += 1
            continue
        cmd_pos = pos
        cls = code[pos + 1]
        grp = code[pos + 2]
        op = code[pos + 3] | (code[pos + 4] << 8)
        argc = int.from_bytes(code[pos + 5:pos + 7], 'little', signed=True)
        pos += 8
        after_args = skip_command_args(code, pos, argc)

        if cls == 0 and grp == 1:
            # Simple branch forms: target immediately follows the arguments.
            if op in (0, 2, 5):
                if after_args + 4 <= n:
                    yield after_args
                pos = max(after_args + 4, cmd_pos + 1)
                continue

            # Jump table: raw u32 targets between braces.
            if op in (3, 8):
                p = after_args
                if p < n and code[p] == ord('{'):
                    p += 1
                    count = max(argc, 0)
                    done = 0
                    while p + 4 <= n and code[p] != ord('}') and (count == 0 or done < count):
                        yield p
                        p += 4
                        done += 1
                    pos = max(p + 1 if p < n and code[p] == ord('}') else p, cmd_pos + 1)
                    continue

            # Switch table: { (expr) target (expr) target ... }
            if op == 4:
                p = after_args
                if p < n and code[p] == ord('{'):
                    p += 1
                    while p < n and code[p] != ord('}'):
                        if code[p] == ord('('):
                            p = skip_command_args(code, p, 1)
                            if p + 4 <= n:
                                yield p
                                p += 4
                            continue
                        if code[p] == ord('$'):
                            p = skip_dollar_expr(code, p)
                            continue
                        p += 1
                    pos = max(p + 1 if p < n and code[p] == ord('}') else p, cmd_pos + 1)
                    continue

        # For non-flow commands, or flow commands with no raw target, continue
        # after the parsed argument block.  Always advance at least one byte to
        # avoid getting stuck on malformed data.
        pos = max(after_args, cmd_pos + 1)


def patch_inline_jump_targets(old_code: bytes, new_code: bytearray,
                              replacements: list[tuple[int, int, int]]) -> int:
    """Remap absolute VM-code jump targets after text relocation.

    Earlier versions patched only the very narrow pattern `) <u32>` / `} <u32>`.
    That misses the main RealLive flow command operands, especially
    `#00:01:0000 <u32>` and brace jump tables used by choices.  The result is a
    syntactically valid Seen.txt whose VM branches into the wrong byte offset and
    can freeze before the first scene.
    """
    if not replacements:
        return 0
    delta_map = build_offset_delta_map(replacements)
    patched = 0
    seen_fields: set[int] = set()

    for old_pos in iter_flow_jump_field_offsets(old_code):
        if old_pos in seen_fields:
            continue
        seen_fields.add(old_pos)
        if _range_contains(replacements, old_pos, 4):
            continue
        if old_pos + 4 > len(old_code):
            continue
        old_target = int.from_bytes(old_code[old_pos:old_pos + 4], 'little')
        # Target 0 is valid and remains 0 unless text before offset 0 existed.
        # Other values must point into the decoded VM code.
        if old_target >= len(old_code):
            continue
        new_pos = remap_code_offset(old_pos, delta_map)
        new_target = remap_code_offset(old_target, delta_map)
        if new_pos < 0 or new_pos + 4 > len(new_code):
            continue
        cur = int.from_bytes(new_code[new_pos:new_pos + 4], 'little')
        # If the value was already changed by another pass, do not overwrite it.
        if cur != old_target:
            continue
        new_code[new_pos:new_pos + 4] = new_target.to_bytes(4, 'little')
        patched += 1
    return patched

def rebuild_seen_chunk_with_code(decoded: DecodedSeen, new_code: bytes, key: bytes,
                                 replacements: list[tuple[int, int, int]]) -> bytes:
    """Rebuild one SEEN chunk after decoded VM code has changed."""
    hdr = decoded.header
    prefix = bytearray(decoded.raw_chunk[:hdr.code_off])
    # Header is 12 u32 fields; only code sizes and packed payload length change.
    new_code_buf = bytearray(new_code)
    patched_jumps = patch_inline_jump_targets(decoded.code, new_code_buf, replacements)
    new_code = bytes(new_code_buf)

    packed = lz_compress(new_code)
    packed_enc = xor_crypt(packed, key)
    write_u32(prefix, 9 * 4, len(new_code))
    write_u32(prefix, 10 * 4, len(packed_enc))
    # The line table in this title stores source/scenario line numbers, not VM
    # byte offsets; it is intentionally left unchanged.  Inline branch targets
    # in the code stream itself are remapped above.
    return bytes(prefix) + packed_enc


def replace_ranges(data: bytes, replacements: list[tuple[int, int, bytes]]) -> tuple[bytes, list[tuple[int, int, int]]]:
    """Apply non-overlapping replacements to data.

    replacements are (start, old_len, new_bytes). Returns new data and a compact
    delta list (start, old_len, new_len) for offset remapping.
    """
    replacements = sorted(replacements, key=lambda x: x[0])
    out = bytearray()
    cur = 0
    delta_recs: list[tuple[int, int, int]] = []
    for start, old_len, new_bytes in replacements:
        if start < cur:
            raise ValueError(f'overlapping text replacement at 0x{start:X}')
        out.extend(data[cur:start])
        out.extend(new_bytes)
        cur = start + old_len
        delta_recs.append((start, old_len, len(new_bytes)))
    out.extend(data[cur:])
    return bytes(out), delta_recs


def entry_text_range(entry: dict[str, Any]) -> tuple[int, int]:
    spans = entry.get('_text_spans') or []
    if not spans:
        start = entry.get('_code_offset')
        size = entry.get('_size')
        if not isinstance(start, int) or not isinstance(size, int):
            raise ValueError(f'entry lacks _text_spans/_code_offset/_size: index={entry.get("_index")}')
        return start, size
    starts: list[int] = []
    ends: list[int] = []
    for sp in spans:
        st = sp.get('code_off')
        sz = sp.get('size')
        if not isinstance(st, int) or not isinstance(sz, int):
            raise ValueError(f'bad _text_spans item in index={entry.get("_index")}')
        starts.append(st); ends.append(st + sz)
    return min(starts), max(ends) - min(starts)


def save_json(path: str | Path, entries: list[TextEntry | dict[str, Any]]) -> None:
    path = Path(path); path.parent.mkdir(parents=True, exist_ok=True)
    arr = [e.to_json() if isinstance(e, TextEntry) else e for e in entries]
    path.write_text(json.dumps(arr, ensure_ascii=False, indent=2), encoding='utf-8')
