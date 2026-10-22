#!/usr/bin/env python3
"""
AI5WIN 字库追加/替换工具 (Font Injection Tool)
================================================
支持两种模式:
  1. replace  - 替换已有 CHS 版 EXE 中的 FNT/TBL 数据
  2. append   - 将 FNT/TBL 追加到原始 EXE 末尾，创建新段

用法:
  python font_inject.py replace <exe_path> [--tbl <tbl_file>] [--fnt <fnt_file>] [-o <output>]
  python font_inject.py append  <exe_path> [--tbl <tbl_file>] [--fnt <fnt_file>] [-o <output>]
  python font_inject.py info    <exe_path>

拖放: 将 EXE 拖放到此脚本上，自动检测模式并处理。
"""

import sys
import os
import struct
import argparse
from pathlib import Path


# ── PE 常量 ───────────────────────────────────────────────────────
IMAGE_FILE_MACHINE_I386 = 0x014C
IMAGE_SCN_MEM_EXECUTE    = 0x20000000
IMAGE_SCN_MEM_READ       = 0x40000000
IMAGE_SCN_MEM_WRITE      = 0x80000000
IMAGE_SCN_CNT_CODE       = 0x00000020
IMAGE_SCN_CNT_INIT_DATA  = 0x00000040
IMAGE_SCN_CNT_UNINIT_DATA = 0x00000080


def align_up(value, alignment):
    return (value + alignment - 1) & ~(alignment - 1)


class PEFile:
    def __init__(self, data):
        self.data = bytearray(data)
        self._parse()

    def _parse(self):
        d = self.data
        # DOS header
        self.dos_magic = d[0:2].decode('ascii', errors='replace')
        if self.dos_magic != 'MZ':
            raise ValueError("Not a valid MZ executable")
        self.pe_offset = struct.unpack_from('<I', d, 0x3C)[0]

        # NT header
        sig = d[self.pe_offset:self.pe_offset+4]
        if sig != b'PE\x00\x00':
            raise ValueError("Not a valid PE file")
        self.machine = struct.unpack_from('<H', d, self.pe_offset+4)[0]
        self.num_sections = struct.unpack_from('<H', d, self.pe_offset+6)[0]
        self.size_of_optional_header = struct.unpack_from('<H', d, self.pe_offset+20)[0]

        # Optional header
        self.opt_start = self.pe_offset + 24
        self.magic_opt = struct.unpack_from('<H', d, self.opt_start)[0]  # PE32=0x10B, PE32+=0x20B
        self.entry_rva = struct.unpack_from('<I', d, self.opt_start+16)[0]
        self.image_base = struct.unpack_from('<I', d, self.opt_start+28)[0]
        self.section_alignment = struct.unpack_from('<I', d, self.opt_start+32)[0]
        self.file_alignment = struct.unpack_from('<I', d, self.opt_start+36)[0]
        self.size_of_image = struct.unpack_from('<I', d, self.opt_start+56)[0]
        self.size_of_headers = struct.unpack_from('<I', d, self.opt_start+60)[0]
        self.data_dir_count = struct.unpack_from('<I', d, self.opt_start+92)[0]

        # Sections
        self.section_start = self.opt_start + self.size_of_optional_header
        self.sections = []
        for i in range(self.num_sections):
            so = self.section_start + i * 40
            name = d[so:so+8].rstrip(b'\x00').decode('ascii', errors='replace')
            vsize = struct.unpack_from('<I', d, so+8)[0]
            vaddr = struct.unpack_from('<I', d, so+12)[0]
            rsize = struct.unpack_from('<I', d, so+16)[0]
            roff  = struct.unpack_from('<I', d, so+20)[0]
            flags = struct.unpack_from('<I', d, so+36)[0]
            self.sections.append({
                'name': name, 'vsize': vsize, 'vaddr': vaddr,
                'rsize': rsize, 'roff': roff, 'flags': flags,
                'section_offset': so,
            })

    def get_last_section_end(self):
        """Return the end of the last section's raw data in file."""
        last = self.sections[-1]
        return last['roff'] + last['rsize']

    def add_section(self, name, data, characteristics):
        """Add a new section to the PE and return its VA."""
        # Align sizes
        vsize = align_up(len(data), self.section_alignment)
        rsize = align_up(len(data), self.file_alignment)

        # Find VA for new section (after last section, aligned)
        last = self.sections[-1]
        vaddr = align_up(last['vaddr'] + last['vsize'], self.section_alignment)

        # Find file offset for new section (after last section raw data, aligned)
        roff = align_up(last['roff'] + last['rsize'], self.file_alignment)

        # Pad file to roff
        while len(self.data) < roff:
            self.data.append(0)

        # Add section data
        padded = bytearray(data)
        while len(padded) < rsize:
            padded.append(0)
        self.data[roff:roff] = padded

        # Create section header
        sec = {
            'name': name[:8],
            'vsize': vsize,
            'vaddr': vaddr,
            'rsize': rsize,
            'roff': roff,
            'flags': characteristics,
            'section_offset': 0,  # Will be set
        }
        self.sections.append(sec)
        self.num_sections += 1

        # Insert section header at end of section table
        insert_pos = self.section_start + len(self.sections) * 40 - 40  # already +1
        # Actually we add at the end
        insert_pos = self.section_start + (self.num_sections - 1) * 40

        # Ensure there's space for new section header in the header region
        # The section headers are at the end of the optional header
        # We need enough space in size_of_headers
        required_header_end = insert_pos + 40
        if required_header_end > self.size_of_headers:
            # Need to expand headers
            # Actually, the original headers should have space because
            # section headers are followed by padding to file_alignment
            # Let's check
            pass

        sec['section_offset'] = insert_pos

        # Write section header
        name_bytes = name[:8].encode('ascii').ljust(8, b'\x00')
        self.data[insert_pos:insert_pos+8] = name_bytes
        struct.pack_into('<I', self.data, insert_pos+8,  vsize)
        struct.pack_into('<I', self.data, insert_pos+12, vaddr)
        struct.pack_into('<I', self.data, insert_pos+16, rsize)
        struct.pack_into('<I', self.data, insert_pos+20, roff)
        # Pointer to relocations (0), line numbers (0)
        struct.pack_into('<I', self.data, insert_pos+24, 0)
        struct.pack_into('<I', self.data, insert_pos+28, 0)
        # Number of relocations (0), line numbers (0)
        struct.pack_into('<H', self.data, insert_pos+32, 0)
        struct.pack_into('<H', self.data, insert_pos+34, 0)
        struct.pack_into('<I', self.data, insert_pos+36, characteristics)

        # Update num_sections
        struct.pack_into('<H', self.data, self.pe_offset+6, self.num_sections)

        # Update size_of_image
        new_size_of_image = vaddr + vsize
        struct.pack_into('<I', self.data, self.opt_start+56, new_size_of_image)
        self.size_of_image = new_size_of_image

        return vaddr

    def write(self, path):
        with open(path, 'wb') as f:
            f.write(self.data)


# ── CHS 版结构常量 ────────────────────────────────────────────────
# 这些是 AI5WIN_chs.exe 中的已知偏移
CHS_TBL_OFFSET = 0x18C600
CHS_TBL_SIZE   = 0x3800
CHS_FNT_OFFSET = 0x18FE00
CHS_FNT_SIZE   = 0x2C0400

# 代码补丁偏移（在 CHS 版中的文件偏移）
CHS_PATCHES_TBL = [0x4898E, 0x489A0, 0x489B2]  # mov eax, 0xE69800
CHS_PATCHES_FNT = [0x492F9, 0x4930B, 0x4931D]  # mov esi, 0xE6D000
CHS_PATCH_TBL_VA = 0xE69800
CHS_PATCH_FNT_VA = 0xE6D000


def detect_mode(exe_path):
    """Detect if this is a CHS-style modified EXE or original."""
    with open(exe_path, 'rb') as f:
        data = f.read()

    # Check for CHS section names
    pe = PEFile(data)
    section_names = [s['name'] for s in pe.sections]

    is_chs = any(n in section_names for n in ['.always', '.smile', '.2019'])

    # Check if there's a section 8 with FNT data
    has_appended = False
    if len(pe.sections) >= 8:
        sec8 = pe.sections[7]
        if sec8['rsize'] > 0x100000:  # Large section likely contains FNT
            has_appended = True

    return 'chs' if (is_chs or has_appended) else 'original'


def cmd_info(exe_path):
    """Print PE structure info."""
    with open(exe_path, 'rb') as f:
        data = f.read()
    pe = PEFile(data)

    print(f"File: {exe_path}")
    print(f"Size: {len(data):,} bytes (0x{len(data):X})")
    print(f"Image Base: 0x{pe.image_base:08X}")
    print(f"Entry RVA: 0x{pe.entry_rva:08X}")
    print(f"Entry VA: 0x{pe.image_base + pe.entry_rva:08X}")
    print(f"SizeOfImage: 0x{pe.size_of_image:08X}")
    print(f"SectionAlignment: 0x{pe.section_alignment:X}")
    print(f"FileAlignment: 0x{pe.file_alignment:X}")
    print(f"NumSections: {pe.num_sections}")
    print(f"Mode detected: {detect_mode(exe_path)}")
    print()
    print(f"{'Name':10s} {'VAddr':>10s} {'VSize':>10s} {'ROff':>10s} {'RSize':>10s} {'Flags':>10s}")
    print("-" * 62)
    for s in pe.sections:
        print(f"{s['name']:10s} 0x{s['vaddr']:08X} 0x{s['vsize']:08X} 0x{s['roff']:08X} 0x{s['rsize']:08X} 0x{s['flags']:08X}")

    if detect_mode(exe_path) == 'chs':
        print()
        print("CHS-style EXE detected. Appended data:")
        print(f"  TBL: file offset 0x{CHS_TBL_OFFSET:06X}, VA 0x{CHS_PATCH_TBL_VA:08X}")
        print(f"  FNT: file offset 0x{CHS_FNT_OFFSET:06X}, VA 0x{CHS_PATCH_FNT_VA:08X}")


def cmd_replace(exe_path, tbl_path, fnt_path, output_path):
    """Replace FNT/TBL data in an existing CHS-style EXE."""
    with open(exe_path, 'rb') as f:
        data = bytearray(f.read())

    mode = detect_mode(exe_path)
    if mode != 'chs':
        print("WARNING: This doesn't look like a CHS-style EXE.")
        print("Use 'append' mode for original EXEs.")
        if input("Continue anyway? (y/N): ").lower() != 'y':
            return

    modified = False

    if tbl_path:
        if not os.path.exists(tbl_path):
            print(f"ERROR: TBL file not found: {tbl_path}")
            return
        with open(tbl_path, 'rb') as f:
            tbl_data = f.read()
        print(f"TBL file size: {len(tbl_data):,} bytes (0x{len(tbl_data):X})")

        if len(tbl_data) > CHS_TBL_SIZE:
            print(f"WARNING: TBL data ({len(tbl_data)} bytes) exceeds allocated space ({CHS_TBL_SIZE} bytes)!")
            print("This may corrupt the FNT data that follows.")
            if input("Continue? (y/N): ").lower() != 'y':
                return

        # Pad to allocated size
        padded = bytearray(tbl_data)
        while len(padded) < CHS_TBL_SIZE:
            padded.append(0)
        data[CHS_TBL_OFFSET:CHS_TBL_OFFSET + CHS_TBL_SIZE] = padded
        print(f"TBL written at offset 0x{CHS_TBL_OFFSET:06X} (size 0x{len(tbl_data):X})")
        modified = True

    if fnt_path:
        if not os.path.exists(fnt_path):
            print(f"ERROR: FNT file not found: {fnt_path}")
            return
        with open(fnt_path, 'rb') as f:
            fnt_data = f.read()
        print(f"FNT file size: {len(fnt_data):,} bytes (0x{len(fnt_data):X})")

        if len(fnt_data) > CHS_FNT_SIZE:
            print(f"WARNING: FNT data ({len(fnt_data)} bytes) exceeds allocated space ({CHS_FNT_SIZE} bytes)!")
            if input("Continue? (y/N): ").lower() != 'y':
                return

        # Pad
        padded = bytearray(fnt_data)
        while len(padded) < CHS_FNT_SIZE:
            padded.append(0)
        data[CHS_FNT_OFFSET:CHS_FNT_OFFSET + CHS_FNT_SIZE] = padded
        print(f"FNT written at offset 0x{CHS_FNT_OFFSET:06X} (size 0x{len(fnt_data):X})")
        modified = True

    if modified:
        with open(output_path, 'wb') as f:
            f.write(data)
        print(f"\nOutput written to: {output_path}")
        print("Done!")
    else:
        print("No changes made. Specify --tbl and/or --fnt.")


def cmd_append(exe_path, tbl_path, fnt_path, output_path):
    """Append FNT/TBL data to a PE file as new sections."""
    with open(exe_path, 'rb') as f:
        data = bytearray(f.read())

    pe = PEFile(data)
    print(f"Original SizeOfImage: 0x{pe.size_of_image:08X}")
    print(f"Original sections: {pe.num_sections}")

    new_vas = {}

    if tbl_path:
        if not os.path.exists(tbl_path):
            print(f"ERROR: TBL file not found: {tbl_path}")
            return
        with open(tbl_path, 'rb') as f:
            tbl_data = f.read()

        tbl_va = pe.add_section(
            '.fnt_tbl',
            tbl_data,
            IMAGE_SCN_MEM_READ | IMAGE_SCN_CNT_INIT_DATA
        )
        new_vas['tbl'] = tbl_va
        print(f"TBL section added: VA=0x{tbl_va:08X}, size=0x{len(tbl_data):X}")

    if fnt_path:
        if not os.path.exists(fnt_path):
            print(f"ERROR: FNT file not found: {fnt_path}")
            return
        with open(fnt_path, 'rb') as f:
            fnt_data = f.read()

        fnt_va = pe.add_section(
            '.fnt_dat',
            fnt_data,
            IMAGE_SCN_MEM_READ | IMAGE_SCN_CNT_INIT_DATA
        )
        new_vas['fnt'] = fnt_va
        print(f"FNT section added: VA=0x{fnt_va:08X}, size=0x{len(fnt_data):X}")

    pe.write(output_path)
    print(f"\nOutput written to: {output_path}")
    print(f"New SizeOfImage: 0x{pe.size_of_image:08X}")
    print(f"New sections: {pe.num_sections}")

    # Print patching guide
    if new_vas:
        print()
        print("=" * 60)
        print("  CODE PATCHING GUIDE")
        print("=" * 60)
        print()
        print("You need to find and patch the TBL/FNT buffer references")
        print("in the game code. Here's how:")
        print()
        print("1. Run the game with a debugger (x32dbg/OllyDbg)")
        print("2. Set a read breakpoint on the original FNT/TBL data area")
        print("   (search memory for the original font file's byte pattern)")
        print("3. When the breakpoint hits, you'll find functions like:")
        print("   - A function that reads the TBL (character lookup table)")
        print("   - A function that reads the FNT (font bitmap data)")
        print()
        if 'tbl' in new_vas:
            print(f"4. Patch the TBL pointer(s) to use VA: 0x{new_vas['tbl']:08X}")
            print(f"   Look for 'mov reg, 0x{new_vas['tbl']:08X}' style instructions")
            print(f"   Or push-immediate arguments pointing to old TBL buffer")
        if 'fnt' in new_vas:
            print(f"5. Patch the FNT pointer(s) to use VA: 0x{new_vas['fnt']:08X}")
            print(f"   Look for 'mov reg, 0x{new_vas['fnt']:08X}' style instructions")
        print()
        print("6. The README mentions a specific approach:")
        print("   - Run the game, search memory for bytes matching your TBL/FNT")
        print("   - Set memory read breakpoints")
        print("   - Find the caller functions (e.g., sub_449570, sub_449EE0)")
        print("   - Patch the mov instructions inside to point to new VAs")
        print()
        print("NOTE: You may need to disable Windows ASLR:")
        print("  reg add \"HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management\" /v MoveImages /t REG_DWORD /d 0 /f")
        print()


def main():
    if len(sys.argv) == 2 and sys.argv[1] not in ('info', 'replace', 'append', '-h', '--help'):
        # Drag-and-drop mode: auto-detect
        exe_path = sys.argv[1]
        if not os.path.exists(exe_path):
            print(f"File not found: {exe_path}")
            sys.exit(1)
        mode = detect_mode(exe_path)
        print(f"Drag-and-drop mode. Detected: {mode}")
        if mode == 'chs':
            print("CHS-style EXE → Running 'info' mode. Use command line for replace/append.")
            cmd_info(exe_path)
        else:
            print("Original EXE → Please use command line to specify TBL/FNT files.")
            cmd_info(exe_path)
        return

    parser = argparse.ArgumentParser(
        description='AI5WIN Font Injection Tool - Append/Replace FNT/TBL in game EXE',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python font_inject.py info AI5WIN_chs.exe
  python font_inject.py replace AI5WIN_chs.exe --tbl FONT.TBL --fnt FONT.FNT -o new.exe
  python font_inject.py append AI5WIN.exe --tbl FONT.TBL --fnt FONT.FNT -o patched.exe
        """
    )
    parser.add_argument('command', nargs='?', choices=['info', 'replace', 'append'],
                        help='Operation mode')
    parser.add_argument('exe_path', nargs='?', help='Path to EXE file')
    parser.add_argument('--tbl', help='Path to TBL file')
    parser.add_argument('--fnt', help='Path to FNT file')
    parser.add_argument('-o', '--output', help='Output file path')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return

    if not args.exe_path:
        print("ERROR: EXE path required")
        return

    if not os.path.exists(args.exe_path):
        print(f"ERROR: File not found: {args.exe_path}")
        return

    if args.command == 'info':
        cmd_info(args.exe_path)
    elif args.command == 'replace':
        if not args.tbl and not args.fnt:
            print("ERROR: Specify at least --tbl or --fnt")
            return
        output = args.output or Path(args.exe_path).stem + '_new.exe'
        cmd_replace(args.exe_path, args.tbl, args.fnt, output)
    elif args.command == 'append':
        if not args.tbl and not args.fnt:
            print("ERROR: Specify at least --tbl or --fnt")
            return
        output = args.output or Path(args.exe_path).stem + '_patched.exe'
        cmd_append(args.exe_path, args.tbl, args.fnt, output)


if __name__ == '__main__':
    main()
