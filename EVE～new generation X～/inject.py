#!/usr/bin/env python3
"""
inject.py - Inject translated text back into EVE sc00.dll

Reads sc00_dialogue.json (produced by extract.py), applies translations
from the 'message' field, and writes a patched DLL.

How it works:
1. Appends a new .dat2 section to the DLL with translated text
2. Patches all push-imm32 instructions to point to new text addresses
3. Updates PE header (section count, image size)

Usage:
  python inject.py [sc00.dll] [dialogue.json] [output.dll]

The JSON 'message' field should contain the translated text.
Entries with message == Japanese original are skipped (not translated).
"""

import struct, json, sys
from pathlib import Path

# ── PE helpers ─────────────────────────────────────────────────
def align_up(v, align=0x1000):
    return (v + align - 1) & ~(align - 1)

def parse_pe(data):
    e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
    image_base = struct.unpack_from('<I', data, e_lfanew + 24 + 28)[0]
    num_sections = struct.unpack_from('<H', data, e_lfanew + 6)[0]
    opt_hdr_size = struct.unpack_from('<H', data, e_lfanew + 20)[0]
    file_align = struct.unpack_from('<I', data, e_lfanew + 24 + 36)[0]
    sect_align = struct.unpack_from('<I', data, e_lfanew + 24 + 32)[0]
    sec_start = e_lfanew + 24 + opt_hdr_size

    sections = []
    for i in range(num_sections):
        off = sec_start + i * 40
        name = data[off:off+8].decode('ascii').rstrip('\x00')
        sections.append({
            'name': name,
            'vaddr': struct.unpack_from('<I', data, off + 12)[0],
            'vsize': struct.unpack_from('<I', data, off + 8)[0],
            'roff':  struct.unpack_from('<I', data, off + 20)[0],
            'rsize': struct.unpack_from('<I', data, off + 16)[0],
            'flags': struct.unpack_from('<I', data, off + 36)[0],
            'sec_off': off,
        })

    return {
        'image_base': image_base,
        'e_lfanew': e_lfanew,
        'num_sections': num_sections,
        'opt_hdr_size': opt_hdr_size,
        'file_align': file_align,
        'sect_align': sect_align,
        'sec_start': sec_start,
        'sections': sections,
    }

def last_section(pe):
    """Return the section with the highest raw offset."""
    return max(pe['sections'], key=lambda s: s['roff'] + s['rsize'])

def next_rva(pe):
    """Return the next available RVA after all sections."""
    return max(s['vaddr'] + align_up(s['vsize'], pe['sect_align'])
               for s in pe['sections'])

def va_to_file(va, pe):
    rva = va - pe['image_base']
    for s in pe['sections']:
        if s['vaddr'] <= rva < s['vaddr'] + s['vsize']:
            return rva - s['vaddr'] + s['roff']
    return None

# ── Main injection ────────────────────────────────────────────
def inject(dll_path, json_path, output_path):
    with open(dll_path, 'rb') as f:
        data = bytearray(f.read())

    with open(json_path, 'r', encoding='utf-8') as f:
        entries = json.load(f)

    pe = parse_pe(data)
    image_base = pe['image_base']

    # ── Step 1: Collect entries with text to inject ──
    to_inject = []
    for e in entries:
        msg = e.get('message', '')
        if not msg:
            continue
        text_va = e.get('_va', 0) or e.get('_msg_va', 0)
        if not text_va:
            continue
        to_inject.append(e)

    if not to_inject:
        print("No entries to inject.")
        return

    print(f"Entries to inject: {len(to_inject)}")

    # ── Step 2: Build new text section ──
    new_texts = []  # list of (rva, bytes)
    current_rva = next_rva(pe)
    new_section_va = image_base + current_rva

    va_map = {}  # old VA -> new VA

    for e in to_inject:
        old_va = e.get('_va', 0) or e.get('_msg_va', 0)
        text = e['message']
        try:
            encoded = text.encode('cp932') + b'\x00'
        except UnicodeEncodeError:
            encoded = text.encode('shift-jis', errors='replace') + b'\x00'

        if len(encoded) % 2:
            encoded += b'\x00'

        va_map[old_va] = image_base + current_rva
        new_texts.append((current_rva, encoded))
        current_rva += len(encoded)

        # Also handle name if present in old format
        name_va = e.get('_name_va', 0)
        name = e.get('name', '')
        if name_va and name_va != old_va and name:
            try:
                n_enc = name.encode('cp932') + b'\x00'
            except:
                n_enc = name.encode('shift-jis', errors='replace') + b'\x00'
            if len(n_enc) % 2:
                n_enc += b'\x00'
            va_map[name_va] = image_base + current_rva
            new_texts.append((current_rva, n_enc))
            current_rva += len(n_enc)

    if not va_map:
        print("No text to remap.")
        return

    print(f"Unique text remaps: {len(va_map)}")

    # ── Step 3: Write new section data ──
    new_section_data = bytearray()
    for rva, encoded in new_texts:
        new_section_data.extend(encoded)

    while len(new_section_data) % pe['file_align']:
        new_section_data.append(0)

    new_section_size = len(new_section_data)
    new_section_rva = current_rva
    new_section_raw = last_section(pe)['roff'] + last_section(pe)['rsize']
    new_section_raw = align_up(new_section_raw, pe['file_align'])

    # ── Step 4: Patch push-imm32 operands ──
    patched = 0
    for e in to_inject:
        old_va = e.get('_va', 0) or e.get('_msg_va', 0)
        new_va = va_map.get(old_va, 0)
        if not new_va:
            continue

        fixups = e.get('_fixups', [])
        for fixup_off in fixups:
            if fixup_off > 0 and fixup_off + 4 < len(data):
                old_imm = struct.unpack_from('<I', data, fixup_off + 1)[0]
                nv = va_map.get(old_imm, 0)
                if nv:
                    struct.pack_into('<I', data, fixup_off + 1, nv)
                    patched += 1

    print(f"Fixups patched: {patched}")

    # ── Step 5: Add new section header ──
    # Section name: .dat2
    new_sec_off = pe['sec_start'] + pe['num_sections'] * 40
    sec_name = b'.dat2\x00\x00\x00'
    data.extend(sec_name)
    data.extend(struct.pack('<I', new_section_size))  # VirtualSize
    data.extend(struct.pack('<I', new_section_rva))   # VirtualAddress
    data.extend(struct.pack('<I', new_section_size))  # SizeOfRawData
    data.extend(struct.pack('<I', new_section_raw))   # PointerToRawData
    data.extend(b'\x00' * 12)  # Reloc/LineNum/Reserved
    data.extend(struct.pack('<I', 0xC0000040))  # Flags = R/W + initialized

    # ── Step 6: Write section data ──
    data.extend(b'\x00' * (new_section_raw - len(data)))  # pad
    data[new_section_raw:new_section_raw] = new_section_data

    # ── Step 7: Update PE headers ──
    # Number of sections +1
    struct.pack_into('<H', data, pe['e_lfanew'] + 6, pe['num_sections'] + 1)
    # SizeOfImage
    new_image_size = align_up(new_section_rva + new_section_size, pe['sect_align'])
    struct.pack_into('<I', data, pe['e_lfanew'] + 24 + 56, new_image_size)

    # ── Step 8: Write output ──
    with open(output_path, 'wb') as f:
        f.write(data)

    size_mb = len(data) / (1024*1024)
    print(f"\nDone! Output: {output_path} ({size_mb:.1f} MB)")
    print(f"New section .dat2: RVA=0x{new_section_rva:X} size={new_section_size}")
    print(f"New text VA base: 0x{new_section_va:08X}")

    # Verify
    verify_mappings(data, pe, va_map, image_base)


def verify_mappings(data, pe, va_map, image_base):
    """Check a few mappings are correct."""
    count = 0
    for old_va, new_va in va_map.items():
        if count >= 3:
            break
        old_off = va_to_file(old_va, pe)
        new_off = va_to_file(new_va, pe) if va_to_file(new_va, pe) else -1
        old_str = data[old_off:data.find(b'\x00', old_off)].decode('cp932', errors='replace')[:60]
        if new_off > 0:
            new_str = data[new_off:data.find(b'\x00', new_off)].decode('cp932', errors='replace')[:60]
            print(f"  0x{old_va:08X} -> 0x{new_va:08X}: [{old_str}] -> [{new_str}]")
        count += 1


# ── CLI ────────────────────────────────────────────────────────
if __name__ == '__main__':
    sc00 = Path(__file__).parent / 'dll' / 'sc00.dll'
    json_path = Path(__file__).parent / 'sc00_dialogue.json'
    output = Path(__file__).parent / 'dll' / 'sc00_patched.dll'

    if len(sys.argv) > 1: sc00 = Path(sys.argv[1])
    if len(sys.argv) > 2: json_path = Path(sys.argv[2])
    if len(sys.argv) > 3: output = Path(sys.argv[3])

    for p, desc in [(sc00, 'DLL'), (json_path, 'JSON')]:
        if not p.exists():
            print(f"Error: {desc} not found: {p}")
            sys.exit(1)

    inject(str(sc00), str(json_path), str(output))
