#!/usr/bin/env python3

import struct, json, sys
from pathlib import Path

def align_up(v, align):
    return (v + align - 1) & ~(align - 1)

def inject(dll_path, json_path, output_path):
    with open(dll_path, 'rb') as f:
        data = bytearray(f.read())

    with open(json_path, 'r', encoding='utf-8') as f:
        entries = json.load(f)

    e_lfanew = struct.unpack_from('<I', data, 0x3C)[0]
    magic = struct.unpack_from('<H', data, e_lfanew + 24)[0]
    is_64 = (magic == 0x20B)
    
    img_base_off = e_lfanew + 24 + (24 if is_64 else 28)
    image_base = struct.unpack_from('<Q' if is_64 else '<I', data, img_base_off)[0]

    num_sec_off = e_lfanew + 6
    num_sections = struct.unpack_from('<H', data, num_sec_off)[0]
    opt_hdr_size = struct.unpack_from('<H', data, e_lfanew + 20)[0]
    sec_start = e_lfanew + 24 + opt_hdr_size

    file_align = struct.unpack_from('<I', data, e_lfanew + 24 + 36)[0]
    sect_align = struct.unpack_from('<I', data, e_lfanew + 24 + 32)[0]
    size_of_headers = struct.unpack_from('<I', data, e_lfanew + 24 + 60)[0]

    sections = []
    for i in range(num_sections):
        off = sec_start + i * 40
        name = data[off:off+8].decode('ascii', errors='ignore').rstrip('\x00')
        sections.append({
            'name': name,
            'vaddr': struct.unpack_from('<I', data, off + 12)[0],
            'vsize': struct.unpack_from('<I', data, off + 8)[0],
            'roff':  struct.unpack_from('<I', data, off + 20)[0],
            'rsize': struct.unpack_from('<I', data, off + 16)[0]
        })

    new_sec_off = sec_start + num_sections * 40
    if new_sec_off + 40 > size_of_headers:
        sys.exit("Error: SizeOfHeaders space is insufficient.")

    to_inject = [e for e in entries if e.get('message') and (e.get('_va', 0) or e.get('_msg_va', 0))]
    if not to_inject:
        return

    current_rva = max(s['vaddr'] + align_up(s['vsize'], sect_align) for s in sections)
    new_section_va = image_base + current_rva
    va_map = {}
    new_section_data = bytearray()

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
        new_section_data.extend(encoded)
        current_rva += len(encoded)

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
            new_section_data.extend(n_enc)
            current_rva += len(n_enc)

    while len(new_section_data) % file_align:
        new_section_data.append(0)

    new_section_size = len(new_section_data)
    new_section_rva = new_section_va - image_base

    new_section_raw = max(s['roff'] + s['rsize'] for s in sections)
    new_section_raw = align_up(new_section_raw, file_align)
    data = data[:new_section_raw]
    
    if len(data) < new_section_raw:
        data.extend(b'\x00' * (new_section_raw - len(data)))

    data.extend(new_section_data)

    patched = 0
    for e in to_inject:
        old_va = e.get('_va', 0) or e.get('_msg_va', 0)
        new_va = va_map.get(old_va, 0)
        if not new_va: 
            continue
        for fixup_off in e.get('_fixups', []):
            if fixup_off > 0 and fixup_off + 4 < len(data):
                struct.pack_into('<I', data, fixup_off + 1, new_va)
                patched += 1

    new_header = bytearray()
    new_header.extend(b'.dat2\x00\x00\x00')
    new_header.extend(struct.pack('<I', new_section_size))
    new_header.extend(struct.pack('<I', new_section_rva))
    new_header.extend(struct.pack('<I', new_section_size))
    new_header.extend(struct.pack('<I', new_section_raw))
    new_header.extend(b'\x00' * 12)
    new_header.extend(struct.pack('<I', 0xC0000040))
    data[new_sec_off : new_sec_off + 40] = new_header

    struct.pack_into('<H', data, num_sec_off, num_sections + 1)
    new_image_size = align_up(new_section_rva + new_section_size, sect_align)
    struct.pack_into('<I', data, e_lfanew + 24 + 56, new_image_size)

    checksum_off = e_lfanew + 24 + (68 if is_64 else 64)
    struct.pack_into('<I', data, checksum_off, 0)

    dd_start = e_lfanew + 24 + (112 if is_64 else 96)
    sec_dir_off = dd_start + 4 * 8
    if sec_dir_off + 8 <= e_lfanew + 24 + opt_hdr_size:
        struct.pack_into('<I', data, sec_dir_off, 0)
        struct.pack_into('<I', data, sec_dir_off + 4, 0)

    with open(output_path, 'wb') as f:
        f.write(data)

    print(f"Inject success -> {output_path}")
    print(f"Patched ptrs : {patched}")

if __name__ == '__main__':
    sc00 = Path(__file__).parent / 'dll' / 'sc00.dll'
    json_path = Path(__file__).parent / 'sc00_dialogue.json'
    output = Path(__file__).parent / 'dll' / 'sc00_patched.dll'

    if len(sys.argv) > 1: sc00 = Path(sys.argv[1])
    if len(sys.argv) > 2: json_path = Path(sys.argv[2])
    if len(sys.argv) > 3: output = Path(sys.argv[3])

    if not sc00.exists(): sys.exit(f"Error: {sc00} not found.")
    if not json_path.exists(): sys.exit(f"Error: {json_path} not found.")

    inject(str(sc00), str(json_path), str(output))