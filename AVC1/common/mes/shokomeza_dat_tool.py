# -*- coding: utf-8 -*-
from __future__ import annotations
import argparse, json, re, struct, zlib
from dataclasses import dataclass, asdict
from pathlib import Path

COUNT_XOR_LEGACY=0x26ACA46E
CRC64_ECMA_POLY=0x42F0E1EBA9EA3693
DEFAULT_GAME_TITLE='少女たちは荒野を目指す'
ENTRY_SIZE=21

@dataclass
class Entry:
    index:int; key_lo:int; key_hi:int; flag:int; offset:int; packed_size:int; out_capacity:int
    unpacked_size:int|None=None; name:str|None=None

def u32(b,o): return struct.unpack_from('<I',b,o)[0]
def p32(v): return struct.pack('<I',v&0xffffffff)

def crc64_ecma_msb(data:bytes)->int:
    crc=0xffffffffffffffff
    for b in data:
        idx=((crc>>56)^b)&0xff
        c=idx<<56
        for _ in range(8):
            c=(((c<<1)^CRC64_ECMA_POLY)&0xffffffffffffffff) if (c&0x8000000000000000) else ((c<<1)&0xffffffffffffffff)
        crc=((crc<<8)&0xffffffffffffffff)^c
    return (~crc)&0xffffffffffffffff

def title_key(title:str)->tuple[int,int]:
    c=crc64_ecma_msb(title.encode('cp932')); return c,c&0xffffffff

def parse(data:bytes):
    count=u32(data,0)^COUNT_XOR_LEGACY; pos=4; out=[]
    if count>100000: raise ValueError(f'entry count异常: {count}')
    for i in range(count):
        kl=u32(data,pos); kh=u32(data,pos+4); pos+=8
        flag=data[pos]^(kl&0xff); pos+=1
        off=u32(data,pos)^kl; pos+=4
        ps=u32(data,pos)^kl; pos+=4
        cap=u32(data,pos)^kl; pos+=4
        if off+ps>len(data): raise ValueError(f'entry {i} 越界')
        out.append(Entry(i,kl,kh,flag,off,ps,cap))
    return out,pos

def xordwords(blob:bytes,key:int)->bytes:
    b=bytearray(blob)
    for i in range(len(b)//4): struct.pack_into('<I',b,i*4,u32(b,i*4)^key)
    return bytes(b)

def unpack_blob(data:bytes,e:Entry,keylow:int)->bytes:
    return zlib.decompress(xordwords(data[e.offset:e.offset+e.packed_size],keylow^e.key_lo))

def safe(s):
    s=re.sub(r'[\\/:*?"<>|\x00-\x1f]+','_',s.strip())
    return s[:80] or 'entry'

def guess(i,raw):
    t=raw.decode('cp932',errors='replace')
    for line in t.splitlines():
        line=line.strip()
        if line.startswith('*') and len(line)>1: return f'{i:03d}_{safe(line[1:].split()[0])}.txt'
    return f'{i:03d}_entry.txt'

def unpack(dat:Path,outdir:Path,title:str):
    data=dat.read_bytes(); es,index_end=parse(data); crc,key=title_key(title)
    data_base=min(e.offset for e in es); gap=data[index_end:data_base]
    outdir.mkdir(parents=True,exist_ok=True); used=set()
    for e in es:
        raw=unpack_blob(data,e,key); e.unpacked_size=len(raw); name=guess(e.index,raw)
        base=name; n=1
        while name.lower() in used:
            p=Path(base); name=f'{p.stem}_{n}{p.suffix}'; n+=1
        used.add(name.lower()); e.name=name; (outdir/name).write_bytes(raw)
    man={'format':'ACV legacy DAT','game_title':title,'crc64':f'0x{crc:016X}','key_low32':f'0x{key:08X}','entry_count':len(es),'index_end':index_end,'data_base':data_base,'header_gap_hex':gap.hex(),'entries':[asdict(e) for e in es]}
    (outdir/'manifest.json').write_text(json.dumps(man,ensure_ascii=False,indent=2),encoding='utf-8')
    return es,index_end,data_base,key

def pack(indir:Path,out:Path,title:str|None):
    man=json.loads((indir/'manifest.json').read_text(encoding='utf-8')); title=title or man['game_title']; _,key=title_key(title)
    es=[Entry(**e) for e in man['entries']]; gap=bytes.fromhex(man.get('header_gap_hex','')); pos=4+len(es)*ENTRY_SIZE+len(gap); chunks=[]; nes=[]
    for e in es:
        raw=(indir/e.name).read_bytes(); comp=zlib.compress(raw,9); enc=xordwords(comp,key^e.key_lo)
        ne=Entry(e.index,e.key_lo,e.key_hi,e.flag,pos,len(enc),max(e.out_capacity,len(raw)),len(raw),e.name); nes.append(ne); chunks.append(enc); pos+=len(enc)
    b=bytearray(); b+=p32(len(nes)^COUNT_XOR_LEGACY)
    for e in nes:
        b+=p32(e.key_lo)+p32(e.key_hi); b.append(e.flag^(e.key_lo&0xff)); b+=p32(e.offset^e.key_lo)+p32(e.packed_size^e.key_lo)+p32(e.out_capacity^e.key_lo)
    b+=gap
    for c in chunks:b+=c
    out.write_bytes(b)

def verify(dat:Path,title:str):
    data=dat.read_bytes(); es,ie=parse(data); _,key=title_key(title); total=0
    for e in es: total+=len(unpack_blob(data,e,key))
    print(f'[verify] entries={len(es)} ok={len(es)} unpacked={total} index_end=0x{ie:X} data_base=0x{min(e.offset for e in es):X} gap={min(e.offset for e in es)-ie} key=0x{key:08X}')

def main():
    ap=argparse.ArgumentParser(); sp=ap.add_subparsers(dest='cmd',required=True)
    a=sp.add_parser('unpack'); a.add_argument('input',type=Path); a.add_argument('output',type=Path); a.add_argument('--game-title',default=DEFAULT_GAME_TITLE)
    a=sp.add_parser('pack'); a.add_argument('input_dir',type=Path); a.add_argument('output',type=Path); a.add_argument('--game-title',default=None)
    a=sp.add_parser('verify'); a.add_argument('input',type=Path); a.add_argument('--game-title',default=DEFAULT_GAME_TITLE)
    x=ap.parse_args()
    if x.cmd=='unpack':
        es,ie,db,key=unpack(x.input,x.output,x.game_title); print(f'[unpack] entries={len(es)} unpacked={sum(e.unpacked_size or 0 for e in es)} key=0x{key:08X} output={x.output}')
    elif x.cmd=='pack': pack(x.input_dir,x.output,x.game_title); verify(x.output,x.game_title or json.loads((x.input_dir/'manifest.json').read_text(encoding='utf-8'))['game_title'])
    else: verify(x.input,x.game_title)
if __name__=='__main__': main()
