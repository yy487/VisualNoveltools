#!/usr/bin/env python3
"""
extract.py - Extract ALL game dialogue text from EVE sc00.dll

Two-phase:
  1. Scan .data for ALL Shift-JIS dialogue strings (content-based, 100% coverage)
  2. Instruction-flow analysis to label: say_1str, say_2str, prepareQuestion, setName

Output JSON:
  {scr_msg, message}         — no name
  {name, scr_msg, message}   — name from setName()

Usage: python extract.py [sc00.dll] [output.json]
"""

import struct, json, sys, re
from pathlib import Path
from collections import defaultdict

# ═══ PE ═══════════════════════════════════════════════════════
def parse_pe(data):
    e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
    img = struct.unpack_from('<I', data, e_lfanew + 24 + 28)[0]
    nsec = struct.unpack_from('<H', data, e_lfanew + 6)[0]
    opt = struct.unpack_from('<H', data, e_lfanew + 20)[0]
    sstart = e_lfanew + 24 + opt
    sec = {}
    for i in range(nsec):
        o = sstart + i * 40
        nm = data[o:o+8].decode('ascii').rstrip('\x00')
        sec[nm] = {
            'vaddr': struct.unpack_from('<I', data, o + 12)[0],
            'vsize': struct.unpack_from('<I', data, o + 8)[0],
            'roff':  struct.unpack_from('<I', data, o + 20)[0],
        }
    return img, sec

def read_str(data, img, sec, va):
    rva = va - img
    for s in sec.values():
        if s['vaddr'] <= rva < s['vaddr'] + s['vsize']:
            off = rva - s['vaddr'] + s['roff']
            end = data.find(b'\x00', off)
            if end > off + 1:
                try: return data[off:end].decode('cp932')
                except: return None
    return None

# ═══ Content filters ═════════════════════════════════════════
def has_jp(s):   return any('　' <= c <= '鿿' for c in s)
def has_kana(s): return any('ぁ' <= c <= 'ん' or 'ァ' <= c <= 'ン' for c in s)

def is_resource(s):
    if not s: return True
    if re.match(r'^\.\w{3,4}$', s): return True
    if re.match(r'^[A-Z]{2,6}\d{2,4}$', s): return True
    if s.startswith('res\\') or s.startswith('view.'): return True
    if re.search(r'\\[gs]\\', s): return True
    if re.search(r'\.(dlt|gyu|dat|wav|ogg|bmp|png|dll|exe)$', s, re.I): return True
    return False

def is_system(s):
    if not s: return True
    if '??' in s and '@' in s: return True
    if re.match(r'^[\d,.\-+*/\\=<>!@#$%^&()\[\]{}|;:]+$', s): return True
    return False

def is_dialogue(s):
    if not s or len(s) < 1: return False
    if is_resource(s) or is_system(s): return False
    if has_jp(s) or has_kana(s): return True
    if all(c in '…。、！？…〜～・」「『』（）．―－…□■▲△▼▽◆◇○●☆★♂♀　 ' for c in s): return True
    return False

# ═══ Instruction flow ═════════════════════════════════════════
# (IAT addr, func_name, [text_param_indices])
# index 0 = last push = first functional param (stdcall push order)
TEXT_FUNCS = {
    0x103AD254: ('say_1str',       [1]),
    0x103AD42C: ('say_2str',       [1, 2]),
    0x103AD20C: ('prepareQuestion', [0]),
    0x103AD338: ('initQuestion',    [0]),
    0x103AD298: ('changeRequest',   [0]),
    0x103AD2A4: ('setMoveScene',    [2]),
    0x103AD2E8: ('defDialog',       [0]),
    0x103AD2CC: ('dlgInfo',         [0, 1]),
    0x103AD248: ('viewDate',        [0]),
    0x103AD23C: ('setSaveTitle',    [0]),
    0x103AD2D4: ('setName',         list(range(9))),
}

def instruction_flow(data, img, sec):
    text_sec = sec['.text']
    data_sec = sec['.data']
    tr = text_sec['roff']
    raw = data[tr:tr + text_sec['vsize']]
    dvb = img + data_sec['vaddr']
    dve = dvb + data_sec['vsize']

    va_funcs = defaultdict(list)
    batch = []
    pos = 0
    while pos < len(raw):
        b = raw[pos]
        if b == 0x68 and pos + 5 <= len(raw):
            batch.append((tr + pos, 'i32', struct.unpack_from('<I', raw, pos+1)[0]))
            pos += 5
        elif b == 0x6A and pos + 1 <= len(raw):
            batch.append((tr + pos, 'i8', raw[pos+1]))
            pos += 2
        elif b in (0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57):
            batch.append((tr + pos, 'reg', b - 0x50))
            pos += 1
        elif b == 0xFF and pos + 6 <= len(raw) and raw[pos+1] == 0x15:
            iat = struct.unpack_from('<I', raw, pos+2)[0]
            fi = TEXT_FUNCS.get(iat)
            if fi:
                fn, idxs = fi
                rev = list(reversed(batch))
                for pi in idxs:
                    if pi < len(rev) and rev[pi][1] == 'i32' and dvb <= rev[pi][2] < dve:
                        va_funcs[rev[pi][2]].append((fn, rev[pi][0]))
            batch = []; pos += 6
        elif b in (0xE8,):  batch = []; pos += 5
        elif b in (0xC3,0xC2,0xE9): batch = []; pos += (5 if b==0xE9 else 1)
        elif b == 0xEB: batch = []; pos += 2
        else: pos += 1
    return va_funcs

# ═══ Main ═════════════════════════════════════════════════════
def extract(path_in, path_out):
    with open(path_in, 'rb') as f:
        data = f.read()

    img, sec = parse_pe(data)
    ts = sec['.text']; ds = sec['.data']
    raw_t = data[ts['roff']:ts['roff'] + ts['vsize']]
    dvb = img + ds['vaddr']
    dve = dvb + ds['vsize']

    # Phase 1: all dialogue strings
    print("Phase 1: scan .data...")
    section = data[ds['roff']:ds['roff'] + ds['vsize']]
    strings = {}
    i = 0
    while i < len(section):
        b = section[i]
        ok = (0x81 <= b <= 0x9F) or (0xE0 <= b <= 0xEF) or (0x20 <= b <= 0x7E)
        if ok:
            end = section.find(b'\x00', i)
            if end < 0: end = i + 500
            if end > i + 1:
                try: s = section[i:end].decode('cp932')
                except: s = ''
                if is_dialogue(s):
                    strings[dvb + i] = s
            i = end + 1
        else: i += 1
    print(f"  {len(strings)} strings")

    # Phase 2: fixups
    print("Phase 2: fixups...")
    fixups = defaultdict(list)
    pos = 0
    while pos < len(raw_t) - 5:
        if raw_t[pos] == 0x68:
            imm = struct.unpack_from('<I', raw_t, pos+1)[0]
            if imm in strings:
                fixups[imm].append(ts['roff'] + pos)
            pos += 5
        else: pos += 1

    # Phase 3: instruction flow
    print("Phase 3: instruction flow...")
    va_funcs = instruction_flow(data, img, sec)
    labeled = len(va_funcs)
    print(f"  {labeled} VAs labeled")

    # Phase 4: build output
    print("Phase 4: build JSON...")
    entries = []
    for va in sorted(strings.keys()):
        s = strings[va]
        f = fixups.get(va, [])
        if not f: continue

        funcs = list(set(fn for fn, _ in va_funcs.get(va, [])))

        # Filter system_text garbage
        if funcs and not any(fn in ('say_1str','say_2str','prepareQuestion','setName') for fn in funcs):
            if not has_kana(s) and not has_jp(s): continue
            if re.search(r'[A-Z\d]+-\*', s): continue
            if re.search(r'(opening|ending)', s, re.I): continue

        # Dedup
        if entries and s == entries[-1]['_msg']:
            for ff in f:
                if ff not in entries[-1]['_fixups']:
                    entries[-1]['_fixups'].append(ff)
            continue

        entries.append({'_msg': s, '_fixups': f, '_va': va, '_funcs': funcs})

    # Build clean JSON
    output = []
    for e in entries:
        has_name = 'setName' in e['_funcs']
        item = {
            'scr_msg': e['_msg'],
            'message': e['_msg'],
            '_fixups': e['_fixups'],
            '_va': e['_va'],
        }
        if has_name:
            item['name'] = e['_msg']  # the string itself IS the name
        output.append(item)

    with open(path_out, 'w', encoding='utf-8') as f:
        json.dump(output, f, ensure_ascii=False, indent=2)

    named = sum(1 for e in output if 'name' in e)
    print(f"\n{'='*50}")
    print(f"Total: {len(output)}  (with name: {named})")
    print(f"Saved to {path_out}")

if __name__ == '__main__':
    sc00 = Path(__file__).parent / 'dll' / 'sc00.dll'
    out  = Path(__file__).parent / 'sc00_dialogue.json'
    if len(sys.argv) > 1: sc00 = Path(sys.argv[1])
    if len(sys.argv) > 2: out  = Path(sys.argv[2])
    if not sc00.exists():
        print(f"Error: {sc00} not found"); sys.exit(1)
    extract(str(sc00), str(out))
