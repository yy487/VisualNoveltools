#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Meguri CIV Translation Tool
============================
UTF-16LE text extraction, full rebuild with VM offset relocation.

Based on reverse engineering of Yuki ScriptEngine.

Usage:
  python civ_tool.py extract  <in.civ> [out.json]
  python civ_tool.py rebuild  <in.civ> <trans.json> <out.civ>
  python civ_tool.py auto     <in.civ> [out.civ]
  python civ_tool.py verify   <in.civ>
"""
import sys, struct, json, io, os, bisect
from typing import Dict, List, Tuple, Optional
from collections import Counter
from dataclasses import dataclass

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

ENCODING = 'utf-16le'
HEADER_SIZE = 0x40
MAGIC = b'CFLR'

# ── Nibble ─────────────────────────────────────────────────────────

def nibble_swap(data: bytes) -> bytes:
    return bytes(((b << 4) | (b >> 4)) & 0xFF for b in data)

# ── Little/Big Endian ──────────────────────────────────────────────

def u32le(b: bytes, off: int) -> int:
    return int.from_bytes(b[off:off+4], 'little')

def p32le(v: int) -> bytes:
    return (v & 0xFFFFFFFF).to_bytes(4, 'little')

def u32be(b: bytes, off: int) -> int:
    return int.from_bytes(b[off:off+4], 'big')

def p32be(v: int) -> bytes:
    return (v & 0xFFFFFFFF).to_bytes(4, 'big')

def fmt_off(v: int) -> str:
    return f"0x{v:08X}"

# ── Text Slot Scanner ──────────────────────────────────────────────

@dataclass(frozen=True)
class TextSlot:
    index: int
    op_offset: int      # byte offset of opcode 0x40
    text_id: int         # u32be at op_offset+1
    text_start: int      # first byte of UTF-16LE text
    text_end: int        # after NUL terminator (exclusive)
    text: str            # decoded text

RESOURCE_PREFIXES = ('BGM_', 'se_', 'vo_', 'ev_', 'bg_', 'ED_', 'OP', 'movie', 'Movie')
JP_RANGES = [(0x3040, 0x30FF), (0x3400, 0x9FFF)]
JP_SIGNAL = set('、。「」『』！？ー─…\n\r')

def has_japanese(s: str) -> bool:
    for c in s:
        o = ord(c)
        if any(lo <= o <= hi for lo, hi in JP_RANGES) or c in JP_SIGNAL:
            return True
    return False

def is_resource_like(s: str) -> bool:
    t = s.strip('\x00\r\n\t ')
    if not t: return True
    if any(t.startswith(p) for p in RESOURCE_PREFIXES): return True
    if all(ord(c) < 0x80 and (c.isalnum() or c in '_.-+:/\\') for c in t): return True
    return False

def scan_text_slots(body_dec: bytes) -> List[TextSlot]:
    """Scan for opcode 0x40 text slots: 40 <id:u32be> 7F <utf16le_z>."""
    slots = []
    i = 0
    n = len(body_dec)
    while i < n - 8:
        if body_dec[i] == 0x40 and body_dec[i + 5] == 0x7F:
            text_id = u32be(body_dec, i + 1)
            if 0 <= text_id < 100000:
                p = i + 6
                while p + 1 < n and not (body_dec[p] == 0 and body_dec[p + 1] == 0):
                    p += 2
                if p + 1 < n:
                    raw = body_dec[i + 6:p]
                    try:
                        text = raw.decode(ENCODING)
                    except UnicodeDecodeError:
                        text = ''
                    if text and (has_japanese(text) or not is_resource_like(text)):
                        slots.append(TextSlot(
                            index=len(slots), op_offset=i, text_id=text_id,
                            text_start=i + 6, text_end=p + 2, text=text))
                        i = p + 2
                        continue
        i += 1
    return slots

# ── 0x4A Reference Scanner ─────────────────────────────────────────

@dataclass(frozen=True)
class FourARef:
    op_offset: int
    operand_offsets: List[int]
    targets: List[int]
    end_offset: int

def scan_4a_refs(body_dec: bytes, inside_mask: bytearray) -> List[FourARef]:
    refs = []
    n = len(body_dec)
    for i in range(n - 5):
        if inside_mask[i] or body_dec[i] != 0x4A:
            continue
        p = i + 1
        vals = []; ops = []
        ok = False
        for _ in range(256):
            if p + 4 > n: break
            v = u32be(body_dec, p); cur = p; p += 4
            if v == 0xFFFFFFFF: ok = True; break
            pos = v & 0x7FFFFFFF
            if pos >= n: break
            vals.append(v); ops.append(cur)
        if ok and vals:
            refs.append(FourARef(i, ops, vals, p))
    return refs

def build_inside_mask(size: int, slots: List[TextSlot]) -> bytearray:
    m = bytearray(size)
    for s in slots:
        m[s.text_start:s.text_end] = b'\x01' * (s.text_end - s.text_start)
    return m

# ── VM Jump Scanner ────────────────────────────────────────────────

@dataclass(frozen=True)
class JumpRef:
    op_offset: int
    opcode: int
    operand_offset: int
    target: int

# Simplified operand size: handles 01/02/04/05 with 4-byte BE target,
# plus 0x5A choice control with count byte at +1 and target at +4.
def scan_jumps(body_dec: bytes, slots: List[TextSlot],
               foura: List[FourARef], index_entry_offsets: List[int]) -> List[JumpRef]:
    """Conservative jump scanner from known VM boundaries."""
    n = len(body_dec)
    starts: set = {0}
    starts.update(index_entry_offsets)
    for s in slots:
        starts.add(s.op_offset); starts.add(s.text_end)
    for r in foura:
        starts.add(r.op_offset); starts.add(r.end_offset)
        for v in r.targets:
            t = v & 0x7FFFFFFF
            if 0 <= t < n: starts.add(t)

    refs: Dict[Tuple[int,int,int], JumpRef] = {}
    changed = True; rounds = 0
    while changed and rounds < 8:
        rounds += 1; changed = False
        for i in sorted(starts):
            if not (0 <= i + 5 <= n): continue
            op = body_dec[i]
            if op not in (0x01, 0x02, 0x04, 0x05): continue
            target = u32be(body_dec, i + 1)
            if not (0 <= target < n): continue
            key = (i, op, target)
            if key not in refs:
                refs[key] = JumpRef(i, op, i + 1, target); changed = True
            for add in (i + 5, target):
                if 0 <= add < n and add not in starts:
                    starts.add(add); changed = True

    # 0x5A choice control
    for i in range(1, n - 8):
        if body_dec[i] == 0x5A and body_dec[i - 1] == 0x58:
            target = u32be(body_dec, i + 4)
            if 0 <= target < n:
                key = (i, 0x5A, target)
                if key not in refs:
                    refs[key] = JumpRef(i, 0x5A, i + 4, target)

    return sorted(refs.values(), key=lambda r: r.op_offset)

# ── Index Parser ───────────────────────────────────────────────────

@dataclass(frozen=True)
class IndexEntry:
    index: int; name: str; value: int; value_off: int

def parse_index(raw_index: bytes) -> Tuple[List[IndexEntry], int]:
    entries = []
    if len(raw_index) < 4: return entries, 0
    n_entry = u32le(raw_index, 0)
    pos = 4
    for i in range(n_entry):
        if pos + 8 > len(raw_index): break
        name_len = u32le(raw_index, pos)
        value_off = pos + 4
        value = u32le(raw_index, value_off)
        pos += 8
        name_raw = raw_index[pos:pos + name_len]; pos += name_len
        name = nibble_swap(name_raw).decode('cp932', errors='replace').rstrip('\x00')
        entries.append(IndexEntry(i, name, value, value_off))
    return entries, pos

# ── Ruby / Linebreak / Name ────────────────────────────────────────

RUBY_H = '\\rh'; RUBY_R = '\\rr'; RUBY_T = '\\rt'

def strip_ruby(text: str) -> str:
    out = []; pos = 0
    while True:
        a = text.find(RUBY_H, pos)
        if a < 0: out.append(text[pos:]); break
        b = text.find(RUBY_R, a + 3)
        c = text.find(RUBY_T, (b + 3) if b >= 0 else a + 3)
        if b < 0 or c < 0: out.append(text[pos:]); break
        out.append(text[pos:a]); out.append(text[a + 3:b]); pos = c + 3
    return ''.join(out)

def strip_linebreaks(text: str) -> str:
    return text.replace('\\n', '').replace('\\r', '').replace('\r', '').replace('\n', '')

def apply_custom_name(text: str, last='麻生', first='こま') -> str:
    full = f'{last} {first}'; compact = last + first
    for sep in ('　', ' ', ''):
        text = text.replace(f'\\lastname{sep}\\firstname', full if sep else compact)
    text = text.replace('\\lastname', last)
    text = text.replace('\\firstname', first)
    return text

def clean_message(text: str) -> str:
    return apply_custom_name(strip_linebreaks(strip_ruby(text)))

# ── CFLR Load/Save ─────────────────────────────────────────────────

class CFLRFile:
    def __init__(self):
        self.header = bytearray()
        self.raw_index = b''
        self.raw_body = b''
        self.body_dec = b''
        self.slots: List[TextSlot] = []
        self.foura: List[FourARef] = []
        self.jumps: List[JumpRef] = []
        self.index_entries: List[IndexEntry] = []

    @classmethod
    def load(cls, path: str) -> 'CFLRFile':
        data = open(path, 'rb').read()
        c = cls()
        c.header = bytearray(data[:HEADER_SIZE])
        idx_size = u32le(data, 0x34)
        body_size = u32le(data, 0x0C)
        c.raw_index = data[HEADER_SIZE:HEADER_SIZE + idx_size]
        c.raw_body = data[HEADER_SIZE + idx_size:]
        c.body_dec = nibble_swap(c.raw_body)

        c.index_entries, _ = parse_index(c.raw_index)
        c.slots = scan_text_slots(c.body_dec)
        inside = build_inside_mask(len(c.body_dec), c.slots)
        c.foura = scan_4a_refs(c.body_dec, inside)
        idx_offs = [e.value for e in c.index_entries]
        c.jumps = scan_jumps(c.body_dec, c.slots, c.foura, idx_offs)
        return c

    def save(self, path: str, new_body_dec: bytes, index_patches: Dict[str,int]):
        raw_body = nibble_swap(new_body_dec)
        new_index = bytearray(self.raw_index)
        for e in self.index_entries:
            if e.name in index_patches:
                new_index[e.value_off:e.value_off+4] = p32le(index_patches[e.name])

        h = self.header
        h[0x0C:0x10] = p32le(len(raw_body))
        h[0x34:0x38] = p32le(len(new_index))
        payload_sum = (sum(new_index) + sum(raw_body)) & 0xFFFFFFFF
        h[0x38:0x3C] = p32le(payload_sum)
        h[0x3C:0x40] = p32le(payload_sum)
        checksum = sum(h[0x0C:0x40]) & 0xFFFFFFFF
        h[0x08:0x0C] = p32le(checksum)

        os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
        with open(path, 'wb') as f:
            f.write(bytes(h)); f.write(bytes(new_index)); f.write(raw_body)

# ── Rebuild Engine ─────────────────────────────────────────────────

@dataclass
class Replacement:
    start: int; end: int; new_data: bytes; slot: TextSlot

class OffsetMapper:
    def __init__(self, old_size: int, replacements: List[Replacement]):
        self.old_size = old_size
        self.replacements = sorted(replacements, key=lambda r: r.start)
        self.prefix_delta: List[int] = []
        delta = 0
        for r in self.replacements:
            delta += len(r.new_data) - (r.end - r.start)
            self.prefix_delta.append(delta)

    def map(self, pos: int) -> int:
        if not (0 <= pos <= self.old_size): return pos
        for k, r in enumerate(self.replacements):
            if r.start <= pos < r.end:
                if pos == r.start: return pos + (self.prefix_delta[k-1] if k > 0 else 0)
                rel = pos - r.start
                return r.start + (self.prefix_delta[k-1] if k > 0 else 0) + min(rel, max(len(r.new_data)-1, 0))
        delta = 0
        for k, r in enumerate(self.replacements):
            if pos < r.start: break
            delta = self.prefix_delta[k]
        return pos + delta

def rebuild_body(old_body: bytes, replacements: List[Replacement]) -> bytes:
    out = bytearray(); pos = 0
    for r in sorted(replacements, key=lambda x: x.start):
        if r.start < pos: raise ValueError('overlap')
        out += old_body[pos:r.start]; out += r.new_data; pos = r.end
    out += old_body[pos:]
    return bytes(out)

def patch_body_offsets(new_body: bytearray, mapper: OffsetMapper,
                       foura: List[FourARef], jumps: List[JumpRef]) -> dict:
    stats = {'4a': 0, 'jumps': 0}
    for r in foura:
        new_op = mapper.map(r.op_offset)
        delta = new_op - r.op_offset
        for off, val in zip(r.operand_offsets, r.targets):
            flag = val & 0x80000000
            t = mapper.map(val & 0x7FFFFFFF)
            new_body[off + delta:off + delta + 4] = p32be(flag | t)
            stats['4a'] += 1
    for j in jumps:
        no = mapper.map(j.op_offset)
        nt = mapper.map(j.target)
        if 0 <= j.operand_offset + 4 <= len(new_body) and new_body[no] == j.opcode:
            new_off = mapper.map(j.operand_offset)
            new_body[new_off:new_off + 4] = p32be(nt)
            stats['jumps'] += 1
    return stats

# ── Extract ────────────────────────────────────────────────────────

def cmd_extract(inp: str, out_json: str):
    print(f'[Extract] {inp}')
    cflr = CFLRFile.load(inp)
    print(f'  Text slots: {len(cflr.slots)}')
    print(f'  4A refs: {len(cflr.foura)}')
    print(f'  Jumps: {len(cflr.jumps)}')
    print(f'  Index entries: {len(cflr.index_entries)}')

    entries = []
    for s in cflr.slots:
        if is_resource_like(s.text) and not has_japanese(s.text):
            continue
        typ = 'choice' if _near_choice(cflr.body_dec, s) else 'text'
        entries.append({
            'scr_msg': s.text,
            'message': clean_message(s.text),
            '_index': len(entries),
            '_text_id': s.text_id,
            '_op_offset': fmt_off(s.op_offset),
            '_offset': fmt_off(s.text_start),
            '_end': fmt_off(s.text_end),
            '_size': s.text_end - s.text_start - 2,
            '_type': typ,
        })

    print(f'  Translatable: {len(entries)}')
    for e in entries[:10]:
        print(f'    [{e["_type"]}] id={e["_text_id"]} {e["scr_msg"][:60]}')

    with open(out_json, 'w', encoding='utf-8') as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)
    print(f'[Extract] → {out_json}')

def _near_choice(body_dec: bytes, slot: TextSlot) -> bool:
    lo = max(0, slot.op_offset - 0x80)
    hi = min(len(body_dec), slot.op_offset + 0x80)
    return b'\x44\x58\x5A' in body_dec[lo:hi] or b'\x58\x5A' in body_dec[lo:hi]

# ── Rebuild ────────────────────────────────────────────────────────

def cmd_rebuild(inp: str, trans_json: str, out: str):
    print(f'[Rebuild] {inp}')
    cflr = CFLRFile.load(inp)
    with open(trans_json, 'r', encoding='utf-8') as f:
        entries = json.load(f)

    # Match by text_id or op_offset, verify with scr_msg
    by_id = {s.text_id: s for s in cflr.slots}
    by_op = {s.op_offset: s for s in cflr.slots}
    replacements = []
    changed = 0
    for e in entries:
        scr = e.get('scr_msg', '')
        msg = e.get('message', '')
        if not msg or msg == scr:
            continue  # skip unchanged
        slot = by_id.get(e.get('_text_id'))
        if not slot:
            op = int(e.get('_op_offset', '0x0'), 16)
            slot = by_op.get(op)
        if not slot or slot.text != scr:
            print(f'  WARN: mismatch for id={e.get("_text_id")}')
            continue
        # Apply same cleanup to message before encoding
        clean = apply_custom_name(strip_linebreaks(strip_ruby(msg)))
        new_data = clean.encode(ENCODING) + b'\x00\x00'
        old_data = cflr.body_dec[slot.text_start:slot.text_end]
        if new_data != old_data:
            replacements.append(Replacement(slot.text_start, slot.text_end, new_data, slot))
            changed += 1

    if not replacements:
        print('  No changes. Copying original.')
        import shutil; shutil.copy(inp, out); return

    mapper = OffsetMapper(len(cflr.body_dec), replacements)
    new_body = bytearray(rebuild_body(cflr.body_dec, replacements))

    # Patch offsets
    stats = patch_body_offsets(new_body, mapper, cflr.foura, cflr.jumps)
    print(f'  Replacements: {changed}  4A patched: {stats["4a"]}  Jumps patched: {stats["jumps"]}')

    # Update index entrypoints
    index_patches = {}
    for e in cflr.index_entries:
        index_patches[e.name] = mapper.map(e.value)

    cflr.save(out, new_body, index_patches)
    print(f'  Size: {len(cflr.body_dec):,} → {len(new_body):,} ({len(new_body)-len(cflr.body_dec):+,})')

    # Verify
    v = CFLRFile.load(out)
    h_ok = u32le(v.header, 0x08) == sum(v.header[0x0C:0x40]) & 0xFFFFFFFF
    print(f'[Rebuild] → {out}  Header OK: {h_ok}  Slots: {len(v.slots)}')

# ── Auto ───────────────────────────────────────────────────────────

def cmd_auto(inp: str, out: str):
    print(f'[Auto] {inp}')
    cflr = CFLRFile.load(inp)
    replacements = []
    for s in cflr.slots:
        clean = clean_message(s.text)
        if clean == s.text:
            continue  # no change needed
        new_data = clean.encode(ENCODING) + b'\x00\x00'
        old_data = cflr.body_dec[s.text_start:s.text_end]
        if new_data != old_data:
            replacements.append(Replacement(s.text_start, s.text_end, new_data, s))

    if not replacements:
        print('  No changes needed.'); return

    mapper = OffsetMapper(len(cflr.body_dec), replacements)
    new_body = bytearray(rebuild_body(cflr.body_dec, replacements))

    stats = patch_body_offsets(new_body, mapper, cflr.foura, cflr.jumps)
    rh = RUBY_H
    print(f'  Ruby stripped: {sum(1 for r in replacements if rh in r.slot.text)}')
    lm = '\\lastname'
    fm = '\\firstname'
    print(f'  Names replaced: {sum(1 for r in replacements if lm in r.slot.text or fm in r.slot.text)}')
    lb = '\\n'
    print(f'  Linebreaks rm: {sum(1 for r in replacements if lb in r.slot.text)}')
    print(f'  4A patched: {stats["4a"]}  Jumps patched: {stats["jumps"]}')
    print(f'  Size: {len(cflr.body_dec):,} → {len(new_body):,} ({len(new_body)-len(cflr.body_dec):+,})')

    index_patches = {e.name: mapper.map(e.value) for e in cflr.index_entries}
    cflr.save(out, new_body, index_patches)

    v = CFLRFile.load(out)
    h_ok = u32le(v.header, 0x08) == sum(v.header[0x0C:0x40]) & 0xFFFFFFFF
    print(f'[Auto] → {out}  Header OK: {h_ok}  Slots: {len(v.slots)}')

# ── Verify ─────────────────────────────────────────────────────────

def cmd_verify(inp: str):
    print(f'[Verify] {inp}')
    cflr = CFLRFile.load(inp)
    h = cflr.header
    h_ok = u32le(h, 0x08) == sum(h[0x0C:0x40]) & 0xFFFFFFFF
    p_ok = u32le(h, 0x38) == (sum(cflr.raw_index) + sum(cflr.raw_body)) & 0xFFFFFFFF

    bad_4a = 0
    for r in cflr.foura:
        for v in r.targets:
            if not (0 <= (v & 0x7FFFFFFF) < len(cflr.body_dec)): bad_4a += 1
    bad_jumps = sum(1 for j in cflr.jumps if not (0 <= j.target < len(cflr.body_dec)))

    print(f'  Header: {"OK" if h_ok else "FAIL"}')
    print(f'  Payload: {"OK" if p_ok else "FAIL"}')
    print(f'  Text slots: {len(cflr.slots)}')
    print(f'  4A refs: {len(cflr.foura)}  (bad: {bad_4a})')
    print(f'  Jumps: {len(cflr.jumps)}  (bad: {bad_jumps})')
    print(f'  OK: {h_ok and p_ok and bad_4a == 0 and bad_jumps == 0}')

# ── Main ───────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print(__doc__); return
    cmd = sys.argv[1]
    base = r'H:\IDA-PRO-MCP\小萝莉\tool\scenario.civ'

    if cmd == 'extract':
        inp = sys.argv[2] if len(sys.argv) > 2 else base
        out = sys.argv[3] if len(sys.argv) > 3 else os.path.splitext(inp)[0] + '_texts.json'
        cmd_extract(inp, out)
    elif cmd == 'rebuild':
        inp = sys.argv[2] if len(sys.argv) > 2 else base
        jsn = sys.argv[3] if len(sys.argv) > 3 else os.path.splitext(inp)[0] + '_texts.json'
        out = sys.argv[4] if len(sys.argv) > 4 else inp.replace('.civ', '_chs.civ')
        cmd_rebuild(inp, jsn, out)
    elif cmd == 'auto':
        inp = sys.argv[2] if len(sys.argv) > 2 else base
        out = sys.argv[3] if len(sys.argv) > 3 else inp.replace('.civ', '_clean.civ')
        cmd_auto(inp, out)
    elif cmd == 'verify':
        inp = sys.argv[2] if len(sys.argv) > 2 else base
        cmd_verify(inp)
    else:
        print(f'Unknown: {cmd}')

if __name__ == '__main__':
    main()
