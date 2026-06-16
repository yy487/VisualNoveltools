# -*- coding: utf-8 -*-
from __future__ import annotations
import argparse, struct
from pathlib import Path
from common import *


def hexdump(b: bytes, maxn: int = 24) -> str:
    s = b[:maxn].hex(' ')
    if len(b) > maxn: s += ' ...'
    return s


def disasm_code(code: bytes) -> list[str]:
    lines=[]; pos=0; quote=False
    while pos < len(code):
        start=pos; b=code[pos]
        if b == 0:
            lines.append(f'{start:08X}: END 00'); pos+=1; continue
        if b == 0x0A and pos+2 < len(code):
            line=code[pos+1] | (code[pos+2]<<8)
            lines.append(f'{start:08X}: LINE {line}    ; {code[start:start+3].hex(" ")}')
            pos+=3; continue
        if b == ord('#'):
            end=skip_hash_command(code,pos)
            raw=code[pos:end]
            if len(raw)>=8:
                lines.append(f'{start:08X}: CMD_HASH cls={raw[1]:02X} grp={raw[2]:02X} op={raw[3] | (raw[4]<<8):04X} arg={int.from_bytes(raw[5:7],"little",signed=True)} flag={raw[7]:02X} size={end-start} ; {hexdump(raw)}')
            else:
                lines.append(f'{start:08X}: CMD_HASH_TRUNC size={end-start} ; {hexdump(raw)}')
            pos=end; continue
        if b == ord('$'):
            end=skip_dollar_expr(code,pos)
            lines.append(f'{start:08X}: EXPR size={end-start} ; {hexdump(code[start:end])}')
            pos=end; continue
        if b in (ord('@'), ord('!')) and pos+2 < len(code):
            val=code[pos+1] | (code[pos+2]<<8)
            lines.append(f'{start:08X}: CTRL_{chr(b)} {val} ; {code[start:start+3].hex(" ")}')
            pos+=3; continue
        if b == ord('"'):
            quote=not quote; lines.append(f'{start:08X}: QUOTE_TOGGLE {int(quote)}'); pos+=1; continue
        if is_sjis_lead(b) and pos+1 < len(code):
            raw=code[pos:pos+2]
            lines.append(f'{start:08X}: TEXT_CHAR {decode_text(raw)!r} ; {raw.hex(" ")}')
            pos+=2; continue
        if 0x20 <= b < 0x7f or 0xA0 <= b <= 0xDF:
            raw=bytes([b])
            lines.append(f'{start:08X}: TEXT_CHAR {decode_text(raw)!r} ; {raw.hex(" ")}')
            pos+=1; continue
        lines.append(f'{start:08X}: BYTE {b:02X}')
        pos+=1
    return lines


def write_decoded_seen(decoded: DecodedSeen, out_dir: Path) -> None:
    name=f'Seen{decoded.entry.seen_no:04d}'
    (out_dir/'decoded').mkdir(parents=True, exist_ok=True)
    (out_dir/'asm').mkdir(parents=True, exist_ok=True)
    (out_dir/'strings').mkdir(parents=True, exist_ok=True)
    (out_dir/'decoded'/f'{name}.dec').write_bytes(decoded.decoded_chunk)
    with open(out_dir/'strings'/f'{name}.strings.txt','w',encoding='utf-8',newline='\n') as f:
        f.write(f'# {name} string_count={decoded.header.string_count}\n')
        for idx,(off,txt,raw) in enumerate(decoded.strings):
            f.write(f'{idx:04d} off=0x{off:X} size={len(raw)} {txt}\n')
    with open(out_dir/'asm'/f'{name}.asm.txt','w',encoding='utf-8',newline='\n') as f:
        h=decoded.header
        f.write(f'# {name}\n')
        f.write(f'# raw_offset=0x{decoded.entry.offset:X} raw_size=0x{decoded.entry.size:X}\n')
        f.write(f'# header={h}\n')
        f.write('# ---- code ----\n')
        for line in disasm_code(decoded.code):
            f.write(line+'\n')


def main() -> None:
    ap = argparse.ArgumentParser(description='RealLive Seen.txt structural decoder/disassembler')
    ap.add_argument('seen_txt')
    ap.add_argument('arg2', help='out_dir, or legacy ida_export_dir when arg3 is supplied')
    ap.add_argument('arg3', nargs='?', help='legacy out_dir when ida_export_dir is supplied')
    ap.add_argument('--ida-export', help='optional IDA export dir; normally not needed because crypt_template.py is bundled')
    ap.add_argument('--seen', type=int, nargs='*', help='optional seen numbers to disassemble')
    args = ap.parse_args()
    if args.arg3 is None:
        export_dir = args.ida_export
        out_dir = args.arg2
    else:
        export_dir = args.ida_export or args.arg2
        out_dir = args.arg3
    seen_data = Path(args.seen_txt).read_bytes()
    key = load_xor_key_from_export(export_dir)
    want = set(args.seen or [])
    out = Path(out_dir); out.mkdir(parents=True, exist_ok=True)
    n = 0
    for entry in iter_seen_entries(seen_data):
        if want and entry.seen_no not in want:
            continue
        dec = decode_seen_chunk(entry, seen_data, key)
        write_decoded_seen(dec, out)
        n += 1
    src = 'static crypt_template.py' if not export_dir else str(export_dir)
    print(f'[disasm] decoded/disassembled scenes={n} out={out} key={src}')

if __name__ == '__main__':
    main()
