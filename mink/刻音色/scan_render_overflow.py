# -*- coding: utf-8 -*-
from __future__ import annotations
import argparse, json
from pathlib import Path
from keyinse_common import DEFAULT_ENCODING, TEXT_OP, CHOICE_OP, decode_text_instruction, iter_script_files, parse_instructions, renderer_layout_report

def main() -> None:
    ap = argparse.ArgumentParser(description='Scan 刻音色 .s scripts for text records that exceed the real 3-row renderer limit')
    ap.add_argument('input', help='.s file or script directory')
    ap.add_argument('--encoding', default=DEFAULT_ENCODING)
    ap.add_argument('--json', dest='json_out', help='write report JSON')
    args = ap.parse_args()
    rows=[]
    for p in iter_script_files(Path(args.input)):
        idx=0
        for inst in parse_instructions(p.read_bytes()):
            if inst.op == TEXT_OP:
                text=decode_text_instruction(inst,args.encoding)
                rep=renderer_layout_report(text,args.encoding)
                if rep.get('rows',0)>3:
                    rows.append({'file':p.name,'index':idx,'inst_offset':f'0x{inst.offset:X}','rows':rep['rows'],'segment_units':rep['segment_units'],'text':text})
                idx+=1
            elif inst.op == CHOICE_OP:
                idx+=1
    print(f'[scan] overflow_texts={len(rows)}')
    for r in rows[:50]:
        print(f"[overflow] {r['file']} index={r['index']} off={r['inst_offset']} rows={r['rows']} units={r['segment_units']} text={r['text'][:80]}")
    if args.json_out:
        Path(args.json_out).write_text(json.dumps(rows,ensure_ascii=False,indent=2),encoding='utf-8')
if __name__ == '__main__':
    main()
