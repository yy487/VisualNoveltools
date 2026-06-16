# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse, json, struct, sys
from pathlib import Path
from typing import Iterable

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    Image = ImageDraw = ImageFont = None

SJIS_RANGES = [
    (0x8140, 0x81FC, 0),
    (0x824F, 0x82F1, 189),
    (0x8340, 0x8396, 352),
    (0x8740, 0x879C, 439),
    (0x889F, 0x9872, 532),
    (0x989F, 0x9FFC, 4584),
    (0xE040, 0xEAA2, 6470),
]
HALF_COUNT = 160

class Fnt:
    def __init__(self, width:int, height:int, plain:bytearray):
        self.width=width; self.height=height; self.plain=plain
    @property
    def half_width(self): return self.width//2
    @property
    def half_stride(self): return self.height*((self.half_width+7)//8)
    @property
    def full_stride(self): return self.height*((self.width+7)//8)
    @property
    def full_base(self): return HALF_COUNT*self.half_stride
    @property
    def full_count(self): return (len(self.plain)-self.full_base)//self.full_stride

def bits_from_bytes(bs: bytes, nbits:int):
    for i in range(nbits):
        yield (bs[i//8] >> (i&7)) & 1

def decompress_fnt(data:bytes)->Fnt:
    if data[:3] != b'FNT': raise ValueError('bad FNT magic')
    w,h = struct.unpack_from('<HH', data, 3)
    out_size = struct.unpack_from('<I', data, 7)[0]
    flag_bits = struct.unpack_from('<I', data, 11)[0]
    pos=15
    flag_bytes=(flag_bits+7)//8
    flags=data[pos:pos+flag_bytes]; pos += flag_bytes
    comp_size=struct.unpack_from('<I', data, pos)[0]; pos += 4
    comp=data[pos:pos+comp_size]
    ip=0; out=bytearray()
    for flag in bits_from_bytes(flags, flag_bits):
        if len(out)>=out_size: break
        if flag==0:
            if ip>=len(comp): raise ValueError('literal past end')
            out.append(comp[ip]); ip+=1
        else:
            if ip+2>len(comp): raise ValueError('backref past end')
            v=comp[ip] | (comp[ip+1]<<8); ip+=2
            dist=(v>>5)+1; ln=(v&0x1f)+1
            for _ in range(ln):
                out.append(out[-dist])
                if len(out)>=out_size: break
    if len(out)!=out_size:
        raise ValueError(f'decompress size mismatch {len(out)} != {out_size}')
    return Fnt(w,h,out)

def compress_lzss_literal(plain:bytes)->bytes:
    # 最稳妥：全部 literal。引擎压缩格式允许，只是文件会变大。
    flag_bits=len(plain)
    flag_bytes=(flag_bits+7)//8
    flags=b'\x00'*flag_bytes
    return struct.pack('<I', flag_bits)+flags+struct.pack('<I', len(plain))+plain

def build_fnt_bytes(f:Fnt)->bytes:
    return b'FNT'+struct.pack('<HHI', f.width, f.height, len(f.plain))+compress_lzss_literal(bytes(f.plain))

def sjis_code(ch:str)->int:
    b=ch.encode('cp932')
    if len(b)!=2: raise ValueError(f'target must be full-width cp932 two-byte char: {ch!r} -> {b.hex()}')
    return (b[0]<<8)|b[1]

def sjis_full_index(ch:str)->int:
    code=sjis_code(ch)
    for a,b,base in SJIS_RANGES:
        if a <= code <= b:
            return base + (code-a)
    raise ValueError(f'target out of BBR drawable range: {ch!r} cp932={code:04X}')

def encode_bitmap_from_image(img, width:int, height:int)->bytes:
    rowbytes=(width+7)//8
    out=bytearray(height*rowbytes)
    pix=img.load()
    for y in range(height):
        for x in range(width):
            if pix[x,y] > 0:
                out[y*rowbytes + x//8] |= 1 << (x & 7)  # 与引擎 _bittest 低位优先匹配
    return bytes(out)

def render_char(ch:str, font_path:Path, width:int, height:int, *, ttc_index:int=0, size:int|None=None, xoff:int=0, yoff:int=0, threshold:int=96)->bytes:
    if Image is None:
        raise RuntimeError('Pillow not installed. Install with: pip install pillow')
    # 小字号点阵宁可略大再居中裁切，默认字号取 height。
    fs=size or height
    font=ImageFont.truetype(str(font_path), fs, index=ttc_index)
    canvas=Image.new('L', (width*4, height*4), 0)
    d=ImageDraw.Draw(canvas)
    bbox=d.textbbox((0,0), ch, font=font)
    tw,th=bbox[2]-bbox[0], bbox[3]-bbox[1]
    x=(canvas.width-tw)//2 - bbox[0] + xoff
    y=(canvas.height-th)//2 - bbox[1] + yoff
    d.text((x,y), ch, font=font, fill=255)
    # 裁中间 width x height
    left=(canvas.width-width)//2; top=(canvas.height-height)//2
    crop=canvas.crop((left,top,left+width,top+height))
    bw=crop.point(lambda p: 255 if p>=threshold else 0, 'L')
    return encode_bitmap_from_image(bw, width, height)

def load_map(path:Path)->dict[str,str]:
    m=json.loads(path.read_text('utf-8'))
    out={}
    for k,v in m.items():
        if not isinstance(k,str) or not isinstance(v,str) or len(k)!=1 or len(v)!=1:
            print(f'[warn] skip non-single mapping: {k!r}->{v!r}', file=sys.stderr); continue
        if v == '凜':
            print('[warn] mapping target 凜 is cp932 EAA3, outside BBR range; auto replace with 凛', file=sys.stderr)
            v='凛'
        out[k]=v
    return out

def patch_one(in_path:Path, out_path:Path, mapping:dict[str,str], font_path:Path, args)->tuple[int,int]:
    f=decompress_fnt(in_path.read_bytes())
    patched=0; skipped=0
    seen=set()
    for src,dst in mapping.items():
        try:
            idx=sjis_full_index(dst)
        except Exception as e:
            skipped+=1; print(f'[skip] {src}->{dst}: {e}', file=sys.stderr); continue
        if idx in seen:
            skipped+=1; print(f'[skip] duplicate target slot {dst} index={idx}', file=sys.stderr); continue
        seen.add(idx)
        if idx >= f.full_count:
            skipped+=1; print(f'[skip] {src}->{dst}: index {idx} >= full_count {f.full_count}', file=sys.stderr); continue
        glyph=render_char(src, font_path, f.width, f.height, ttc_index=args.ttc_index, size=args.size or f.height, xoff=args.xoff, yoff=args.yoff, threshold=args.threshold)
        off=f.full_base + idx*f.full_stride
        f.plain[off:off+f.full_stride]=glyph
        patched+=1
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(build_fnt_bytes(f))
    return patched, skipped

def main():
    ap=argparse.ArgumentParser(description='Rebuild BBR FNT bitmap fonts according to CN->CP932 mapping.')
    ap.add_argument('input_dir')
    ap.add_argument('map_json')
    ap.add_argument('output_dir')
    ap.add_argument('--ttf', required=True, help='Chinese capable .ttf/.ttc path, e.g. C:\\Windows\\Fonts\\simsun.ttc')
    ap.add_argument('--ttc-index', type=int, default=0)
    ap.add_argument('--size', type=int, default=0, help='font render size; 0 means current FNT height')
    ap.add_argument('--xoff', type=int, default=0)
    ap.add_argument('--yoff', type=int, default=0)
    ap.add_argument('--threshold', type=int, default=96)
    ap.add_argument('--files', nargs='*', default=['FNT10X10.FNT','FNT12X12.FNT','FNT14X14.FNT','hgo10x10.fnt','hgo12x12.fnt','hgo14x14.fnt'])
    args=ap.parse_args()
    inp=Path(args.input_dir); out=Path(args.output_dir); font=Path(args.ttf)
    mapping=load_map(Path(args.map_json))
    print(f'[map] entries={len(mapping)} font={font}')
    total_p=total_s=0
    for name in args.files:
        p=inp/name
        if not p.exists():
            print(f'[warn] missing {p}', file=sys.stderr); continue
        op=out/name
        patched, skipped=patch_one(p,op,mapping,font,args)
        total_p+=patched; total_s+=skipped
        print(f'[font] {name}: patched={patched} skipped={skipped} -> {op}')
    print(f'[done] patched_slots={total_p} skipped={total_s} out={out}')

if __name__=='__main__': main()
