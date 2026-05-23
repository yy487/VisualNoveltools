#!/usr/bin/env python3

# BGI script file disassembler

import glob
import os
import re
import struct
import sys
import base64

import asdis
import bgiop

_decode_fallback_logged = set()
_decode_replace_logged = set()
_decode_fallback_count = 0
_decode_fallback_by_encoding = {}
_decode_replace_primary_count = 0
_decode_replace_utf8_count = 0
RE_SYMBOL_NAME = re.compile(r'^[A-Za-z_][A-Za-z0-9_]*$')

def reset_decode_fallback_stats():
    global _decode_fallback_count, _decode_fallback_by_encoding, _decode_replace_primary_count, _decode_replace_utf8_count
    _decode_fallback_logged.clear()
    _decode_replace_logged.clear()
    _decode_fallback_count = 0
    _decode_fallback_by_encoding = {}
    _decode_replace_primary_count = 0
    _decode_replace_utf8_count = 0

def get_decode_fallback_stats():
    return {
        'fallback_count': _decode_fallback_count,
        'fallback_by_encoding': dict(_decode_fallback_by_encoding),
        'replace_primary_count': _decode_replace_primary_count,
        'replace_utf8_count': _decode_replace_utf8_count
    }

def _looks_like_symbol_name(text):
    return RE_SYMBOL_NAME.match(str(text or '')) is not None

def _looks_suspicious_import_name(text):
    value = str(text or '')
    if not value:
        return True
    if _looks_like_symbol_name(value):
        return False
    return len(value) == 1

def _export_offsets_look_reasonable(exports):
    if not exports:
        return False
    offsets = [val for _, val in exports]
    if any((not isinstance(val, int)) or val < 0 or val >= 0x40000000 for val in offsets):
        return False
    return offsets == sorted(offsets)

def get_code_end(data):
    pos = -1
    while 1:
        res = data.find(b'\xF4\x00\x00\x00', pos+1)
        if res == -1:
            break
        pos = res
    return pos + 4
    
def _get_encoding_list(primary, fallback=None):
    encodings = []
    if primary:
        encodings.append(primary)
    if fallback and fallback not in encodings:
        encodings.append(fallback)
    for enc in ('cp932', 'gbk', 'utf-8'):
        if enc not in encodings:
            encodings.append(enc)
    return encodings

def _record_decode_stats(data, primary, meta):
    global _decode_fallback_count, _decode_replace_primary_count, _decode_replace_utf8_count
    if not meta:
        return
    kind = meta.get('kind')
    if kind == 'fallback':
        enc = meta.get('encoding')
        tried = meta.get('tried', [])
        _decode_fallback_count += 1
        _decode_fallback_by_encoding[enc] = _decode_fallback_by_encoding.get(enc, 0) + 1
        key = (tuple(tried), enc)
        if key not in _decode_fallback_logged:
            _decode_fallback_logged.add(key)
            preview = data[:24].hex()
            tail = '...' if len(data) > 24 else ''
            print(f"[解码回退] 字符串解码失败，尝试编码 {tried}，改用 {enc} 成功，bytes={preview}{tail}")
    elif kind == 'replace_primary':
        _decode_replace_primary_count += 1
        key = ('replace', primary)
        if key not in _decode_replace_logged:
            _decode_replace_logged.add(key)
            print(f"[解码回退] 所有编码严格解码失败，使用 {primary} 的十六进制占位模式")
    elif kind == 'replace_utf8':
        _decode_replace_utf8_count += 1
        if ('replace', 'utf-8') not in _decode_replace_logged:
            _decode_replace_logged.add(('replace', 'utf-8'))
            print("[解码回退] 主编码占位解码失败，改用 utf-8 的十六进制占位模式")

def _decode_bytes_core(data, primary, fallback=None):
    tried = []
    encodings = _get_encoding_list(primary, fallback)
    for idx, enc in enumerate(encodings):
        try:
            text = asdis.sanitize_decoded_text(data.decode(enc), enc)
            meta = None
            if idx > 0:
                meta = {
                    'kind': 'fallback',
                    'encoding': enc,
                    'tried': list(tried)
                }
            return text, meta
        except Exception:
            tried.append(enc)
            continue
    try:
        return asdis.decode_with_placeholders(data, primary), {'kind': 'replace_primary'}
    except Exception:
        return asdis.decode_with_placeholders(data, 'utf-8'), {'kind': 'replace_utf8'}

def _decode_bytes(data, primary, fallback=None, track_stats=True):
    text, meta = _decode_bytes_core(data, primary, fallback)
    if track_stats:
        _record_decode_stats(data, primary, meta)
    return text

def _encode_plain_text(text, primary, fallback=None):
    for enc in _get_encoding_list(primary, fallback):
        try:
            return text.encode(enc)
        except Exception:
            continue
    try:
        return text.encode(primary, errors='replace')
    except Exception:
        return text.encode('utf-8', errors='replace')

def _encode_text(text, primary, fallback=None):
    return asdis.encode_with_placeholders(
        text,
        lambda chunk: _encode_plain_text(chunk, primary, fallback)
    )

def _best_op_chain_score(code, pos, depth, memo):
    key = (pos, depth)
    if key in memo:
        return memo[key]
    if depth <= 0:
        return 0
    if pos + 4 > len(code):
        return -1000000
    cur, = struct.unpack('<I', code[pos:pos+4])
    if cur not in bgiop.ops:
        return -1000000
    fmt = bgiop.ops[cur][0]
    sizes = [4 + struct.calcsize(fmt)]
    if cur == 0x016 and fmt == '<i':
        sizes.append(4)
    best = -1000000
    for step in sizes:
        nxt = pos + step
        if nxt <= len(code):
            score = 1 + _best_op_chain_score(code, nxt, depth - 1, memo)
            if score > best:
                best = score
    memo[key] = best
    return best

def _resolve_op_entry(code, op_addr, op):
    fmt, pfmt, fcn = bgiop.ops[op]
    if op == 0x016 and fmt == '<i':
        noarg_pos = op_addr + 4
        witharg_pos = op_addr + 8
        if witharg_pos > len(code):
            return '', 'f_016()', None
        memo = {}
        noarg_score = _best_op_chain_score(code, noarg_pos, 24, memo)
        witharg_score = _best_op_chain_score(code, witharg_pos, 24, memo)
        if noarg_score >= witharg_score:
            return '', 'f_016()', None
    return fmt, pfmt, fcn

def parse_hdr(hdr, encoding, fallback_encoding=None):
    def _text_badness(text):
        bad = 0
        for ch in text:
            if ch == '\ufffd':
                bad += 3
            elif ord(ch) < 32 and ch not in ('\t', '\n', '\r'):
                bad += 2
            elif not ch.isprintable():
                bad += 1
        return bad

    def _decode_candidate(data):
        text, meta = _decode_bytes_core(data, encoding, fallback_encoding)
        return {'text': text, 'meta': meta, 'data': data}

    def _commit_candidate_stats(items):
        for item in items:
            _record_decode_stats(item['data'], encoding, item['meta'])

    def _looks_like_padding(data):
        return not data or all(b == 0 for b in data)

    hdrtext = _decode_bytes(hdr[:0x1C].rstrip(b'\x00'), encoding, fallback_encoding)
    imports = []
    entries, = struct.unpack('<I', hdr[0x20:0x24])
    pos = 0x24
    for k in range(entries):
        pos1 = hdr.find(b'\x00', pos)
        name = _decode_bytes(hdr[pos:pos1], encoding, fallback_encoding)
        imports.append(name)
        pos = pos1 + 1
        
    extra_imports = []
    rest = hdr[pos:]
    if len(rest) >= 4:
        possible_count, = struct.unpack('<I', rest[:4])
        if 0 < possible_count < 1000:
            temp_pos_list = 4
            valid_list = True
            structure = []
            structure_meta = []
            try:
                for k in range(possible_count):
                    gap_size = 0
                    while temp_pos_list < len(rest) and rest[temp_pos_list] == 0:
                        gap_size += 1
                        temp_pos_list += 1
                    if gap_size > 0:
                        structure.append(gap_size)
                    pos1 = rest.find(b'\x00', temp_pos_list)
                    if pos1 == -1:
                        valid_list = False
                        break
                    decoded = _decode_candidate(rest[temp_pos_list:pos1])
                    structure.append(decoded['text'])
                    structure_meta.append(decoded)
                    temp_pos_list = pos1 + 1
            except:
                valid_list = False

            list_strings = [x for x in structure if isinstance(x, str)]
            valid_list = valid_list and len(list_strings) == possible_count
            final_extras = []
            if valid_list:
                for item in structure:
                    if isinstance(item, int):
                        final_extras.append(f"__GAP:{item}__")
                    else:
                        final_extras.append(item)

            temp_pos_exp = 4
            temp_exports = []
            export_meta = []
            valid_exports = True
            try:
                for k in range(possible_count):
                    while temp_pos_exp < len(rest) and rest[temp_pos_exp] == 0:
                        temp_pos_exp += 1
                    pos1 = rest.find(b'\x00', temp_pos_exp)
                    if pos1 == -1:
                        valid_exports = False
                        break
                    decoded = _decode_candidate(rest[temp_pos_exp:pos1])
                    name = decoded['text']
                    temp_pos_exp = pos1 + 1
                    if temp_pos_exp + 4 > len(rest):
                        valid_exports = False
                        break
                    val, = struct.unpack('<I', rest[temp_pos_exp:temp_pos_exp+4])
                    temp_pos_exp += 4
                    temp_exports.append((name, val))
                    export_meta.append(decoded)
            except:
                valid_exports = False

            if valid_list and valid_exports:
                list_bad = sum(_text_badness(x) for x in list_strings)
                exp_bad = sum(_text_badness(name) for name, _ in temp_exports)
                list_weird_short = sum(
                    1 for x in list_strings
                    if len(x) <= 4 and any((ch == '\ufffd') or (ord(ch) < 32 and ch not in ('\t', '\n', '\r')) for ch in x)
                )
                suspicious_imports = sum(1 for x in list_strings if _looks_suspicious_import_name(x))
                export_symbol_names = sum(1 for name, _ in temp_exports if _looks_like_symbol_name(name))
                exports_reasonable = _export_offsets_look_reasonable(temp_exports)
                list_padding = rest[temp_pos_list:]
                export_padding = rest[temp_pos_exp:]
                prefer_exports = (
                    exports_reasonable and export_symbol_names == possible_count and (
                        suspicious_imports > 0
                        or exp_bad < list_bad
                        or list_weird_short * 2 >= len(list_strings)
                        or (not _looks_like_padding(list_padding) and _looks_like_padding(export_padding))
                    )
                )
                if prefer_exports:
                    _commit_candidate_stats(export_meta)
                    pos += temp_pos_exp
                    return hdrtext, imports, temp_exports, hdr[pos:]
                _commit_candidate_stats(structure_meta)
                pos += temp_pos_list
                return hdrtext, imports, final_extras, hdr[pos:]

            if valid_list:
                _commit_candidate_stats(structure_meta)
                pos += temp_pos_list
                return hdrtext, imports, final_extras, hdr[pos:]
            if valid_exports:
                _commit_candidate_stats(export_meta)
                pos += temp_pos_exp
                return hdrtext, imports, temp_exports, hdr[pos:]

    return hdrtext, imports, extra_imports, hdr[pos:]

def parse(code, hdr, encoding, fallback_encoding=None):
    padding = b''
    extra_imports = []
    if hdr:
        hdrtext, imports, extra_imports, padding = parse_hdr(hdr, encoding, fallback_encoding)
    else:
        hdrtext = None
        imports = []
    defines = {}
    
    for item in extra_imports:
        if isinstance(item, tuple):
            name, val = item
            defines[val] = name
            
    bgiop.clear_offsets()
    inst = {}
    
    code_size = len(code)
    pos = 0
    
    # Determine code end by scanning opcodes and first string references
    scan_pos = 0
    min_str_offset = len(code)
    last_terminator_end = 0
    
    while scan_pos < min_str_offset:
        if scan_pos + 4 > len(code):
            break
            
        op, = struct.unpack('<I', code[scan_pos:scan_pos+4])
        scan_pos += 4
        
        if op not in bgiop.ops:
            print(f"Unknown op {hex(op)} at {hex(scan_pos-4)}")
            if last_terminator_end > 0:
                 min_str_offset = last_terminator_end
            else:
                 min_str_offset = scan_pos - 4
            break
            
        fmt, pfmt, fcn = _resolve_op_entry(code, scan_pos - 4, op)
        
        if op == 0x1B or op == 0xF4:
            last_terminator_end = scan_pos

        if fmt:
            n = struct.calcsize(fmt)
            if scan_pos + n > len(code):
                break
                
            args = struct.unpack(fmt, code[scan_pos:scan_pos+n])
            
            if op == 0x03:
                offset = args[0]
                if offset < min_str_offset:
                    min_str_offset = offset
            elif op == 0x7F:
                offset = args[0]
                if offset < min_str_offset:
                    min_str_offset = offset
            
            scan_pos += n
            
    size = min_str_offset
    print(f"DEBUG: File size={len(code)}, min_str_offset={min_str_offset}, scan_pos={scan_pos}")

    string_refs = {}
    code_refs_order = []
    
    while pos < size:
        addr = pos
        op, = struct.unpack('<I', code[addr:addr+4])
        if op not in bgiop.ops:
            raise Exception(('size unknown for op %02x @ offset %05x' % (op, addr)))
        pos += 4
        fmt, pfmt, fcn = _resolve_op_entry(code, addr, op)
        if fmt:
            n = struct.calcsize(fmt)
            raw_args = struct.unpack(fmt, code[pos:pos+n])
            args = raw_args
            if fcn:
                args = fcn(code, addr, defines, *raw_args)
            
            if op == 0x03:
                offset = raw_args[0]
                string = args[0]
                string_refs[offset] = string
                code_refs_order.append(offset)
            elif op == 0x7F:
                offset = raw_args[0]
                string = args[0]
                string_refs[offset] = string
                code_refs_order.append(offset)
                
            inst[addr] = pfmt % args
            pos += n
        else:
            inst[addr] = pfmt
    if size < len(code):
        str_data = code[size:]
    else:
        str_data = b''
        
    offsets = bgiop.offsets.copy()
    
    return inst, offsets, hdrtext, imports, extra_imports, defines, str_data, padding, string_refs, size, code_refs_order
    
def out_strings(fo, str_data, string_refs, str_data_offset):
    pass

def out_smart_strdata(fo, str_data, string_refs, str_data_offset, code_refs_order, encoding, fallback_encoding=None, force_output=False):
    # Standard layout matches code reference order with no gaps
    is_standard = True
    
    valid_refs_map = {k:v for k,v in string_refs.items() if 0 <= k - str_data_offset < len(str_data)}
    
    current_calc_pos = 0
    seen_offsets = set()
    
    for abs_offset in code_refs_order:
        if abs_offset not in valid_refs_map: continue
        if abs_offset in seen_offsets: continue
        
        seen_offsets.add(abs_offset)
        rel_offset = abs_offset - str_data_offset
        
        if rel_offset != current_calc_pos:
            is_standard = False
            break
            
        try:
            val = valid_refs_map[abs_offset]
            encoded_len = len(_encode_text(asdis.unescape(val), encoding, fallback_encoding)) + 1
        except Exception as e:
            is_standard = False
            break
            
        current_calc_pos += encoded_len

    if is_standard:
        if current_calc_pos != len(str_data):
            is_standard = False
            
    if is_standard and len(seen_offsets) != len(valid_refs_map):
         is_standard = False

    if is_standard and not force_output:
        return

    # Non-standard layout: emit explicit strings and gaps
    current_pos = 0
    str_data_len = len(str_data)
    
    rel_refs = {k - str_data_offset: v for k,v in valid_refs_map.items()}
    sorted_rel_offsets = sorted(rel_refs.keys())
    
    while current_pos < str_data_len:
        if current_pos in rel_refs:
            string = rel_refs[current_pos]
            fo.write('"%s"\n' % string)
            current_pos += len(_encode_text(asdis.unescape(string), encoding, fallback_encoding)) + 1
        else:
            # Emit gap bytes as #strdata for exact reconstruction
            next_ref_pos = str_data_len
            for off in sorted_rel_offsets:
                if off > current_pos:
                    next_ref_pos = off
                    break
            
            gap_len = next_ref_pos - current_pos
            gap_data = str_data[current_pos : next_ref_pos]
            
            decoded_success = False
            if any(b != 0 for b in gap_data):
                try:
                    null_pos = gap_data.find(b'\x00')
                    if null_pos != -1:
                        candidate_data = gap_data[:null_pos]
                        if len(candidate_data) > 0:
                            candidate_str = _decode_bytes(candidate_data, encoding, fallback_encoding)
                            full_chunk = gap_data[:null_pos+1]
                            fo.write('#strdata "%s" // Garbage: "%s"\n' % (full_chunk.hex(), asdis.escape(candidate_str)))
                            current_pos += null_pos + 1
                            decoded_success = True
                            continue
                except:
                    pass
                
                if not decoded_success:
                    hex_str = gap_data.hex()
                    if len(hex_str) > 128:
                        for i in range(0, len(hex_str), 128):
                            fo.write('#strdata "%s"\n' % hex_str[i:i+128])
                    else:
                        fo.write('#strdata "%s"\n' % hex_str)
            else:
                fo.write('#strdata "%s"\n' % gap_data.hex())
            
            current_pos += gap_len

def out(fo, inst, offsets, hdrtext, imports, extra_imports, defines, str_data, padding, string_refs, str_data_offset, code_refs_order, exact_mode=False, strout_file=None, encoding='cp932', fallback_encoding=None):
    def _split_inline_comment(inst_line):
        in_quote = False
        escape = False
        for i, ch in enumerate(inst_line):
            if escape:
                escape = False
                continue
            if ch == '\\':
                escape = True
                continue
            if ch == '"':
                in_quote = not in_quote
                continue
            if not in_quote and ch == '/' and i + 1 < len(inst_line) and inst_line[i + 1] == '/':
                return inst_line[:i].rstrip(), inst_line[i + 2:].lstrip()
        return inst_line, None

    if hdrtext:
        fo.write('#header "%s"\n' % asdis.escape(hdrtext))
        if padding is not None:
            # Try to annotate padding with readable chunks
            comment = ""
            try:
                def _decode_padding_chunk(chunk):
                    candidates = []
                    for enc in (encoding, fallback_encoding, 'cp932'):
                        if enc and enc not in candidates:
                            candidates.append(enc)
                    for enc in candidates:
                        try:
                            return chunk.decode(enc)
                        except Exception:
                            continue
                    return None

                parts = []
                candidates = padding.split(b'\x00')
                for c in candidates:
                    if len(c) > 0:
                        decoded = _decode_padding_chunk(c)
                        if decoded and len(decoded) > 1 and decoded.isprintable():
                            parts.append(decoded)
                if parts:
                    comment = " // " + ", ".join(parts)
            except:
                pass
            fo.write('#header_padding "%s"%s\n' % (padding.hex(), comment))
        fo.write('\n')
    if imports:
        for name in imports:
            fo.write('#import "%s"\n' % asdis.escape(name))
        fo.write('\n')
        
    if extra_imports:
        for item in extra_imports:
            if isinstance(item, tuple):
                name, val = item
                fo.write('#export "%s" %d\n' % (asdis.escape(name), val))
            elif item.startswith('__GAP:'):
                size = int(item.split(':')[1].replace('__', ''))
                fo.write('#extra_import_gap %d\n' % size)
            else:
                fo.write('#extra_import "%s"\n' % asdis.escape(item))
        fo.write('\n')
    
    is_standard = True
    if defines:
        for offset in sorted(defines):
            fo.write('#define %s L%05x\n' % (defines[offset], offset))
        fo.write('\n')
    for addr in sorted(inst):
        if inst[addr].startswith('line('):
            fo.write('\n')
        if addr in offsets or addr in defines:
            if addr in defines:
                fo.write('\n%s:\n' % defines[addr])
            else:
                fo.write('\nL%05x:\n' % addr)
        inst_str = inst[addr]
        code_part, comment_part = _split_inline_comment(inst_str)
        if comment_part is not None:
            fo.write('\t%s; //%s\n' % (code_part, comment_part))
        else:
            fo.write('\t%s;\n' % inst_str)

    if str_data:
        if strout_file:
            strout_filename = os.path.basename(strout_file.name)
            fo.write('\n#string_table "%s"\n' % strout_filename)
            out_smart_strdata(strout_file, str_data, string_refs, str_data_offset, code_refs_order, encoding, fallback_encoding)
        elif exact_mode:
            # Explicit table only for non-standard layout
            fo.write('\n// Explicit string table (for binary exactness)\n')
            fo.write('// Moved to end for readability\n')
            out_smart_strdata(fo, str_data, string_refs, str_data_offset, code_refs_order, encoding, fallback_encoding, True)

        
def dis(file, exact_mode=False, strout_mode=False, encoding='shift_jis', fallback_encoding='gbk', output_path=None):
    encoding = bgiop.normalize_encoding(encoding)
    fallback_encoding = bgiop.normalize_encoding(fallback_encoding)
    if output_path:
        ofile = output_path
    else:
        ofile = os.path.splitext(file)[0] + '.bsd'
    fi = open(file, 'rb')
    hdr_test = fi.read(0x20)
    if hdr_test.startswith(b'BurikoCompiledScriptVer1.00\x00'):
        hdrsize = 0x1C + struct.unpack('<I', hdr_test[0x1C:0x20])[0]
    else:
        hdrsize = 0
    fi.seek(0, 0)
    hdr = fi.read(hdrsize)
    code = fi.read()
    fi.close()
    bgiop.set_string_encodings(encoding, fallback_encoding)
    inst, offsets, hdrtext, imports, extra_imports, defines, str_data, padding, string_refs, size, code_refs_order = parse(code, hdr, encoding, fallback_encoding)
    
    strout_file = None
    if strout_mode:
        if output_path:
            strout_filename = os.path.splitext(output_path)[0] + '.strings.txt'
        else:
            strout_filename = os.path.splitext(file)[0] + '.strings.txt'
        strout_file = open(strout_filename, 'w', encoding='utf-8')
        
    fo = open(ofile, 'w', encoding='utf-8')
    out(fo, inst, offsets, hdrtext, imports, extra_imports, defines, str_data, padding, string_refs, size, code_refs_order, exact_mode, strout_file, encoding, fallback_encoding)
    fo.close()
    
    if strout_file:
        strout_file.close()

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print('Usage: bgidis.py <file(s)> [-e|--exact] [--strout] [-c|--encoding <enc>] [-f|--fallback-encoding <enc>]')
        print('(only extension-less files amongst <file(s)> will be processed)')
        print('  -e, --exact: Output explicit string table for binary exact reconstruction')
        print('  --strout: Output string table to independent .strings.txt file')
        sys.exit(1)
        
    args = []
    exact_mode = False
    strout_mode = False
    
    encoding = 'shift_jis'
    fallback_encoding = 'gbk'
    skip_next = False
    for i, arg in enumerate(sys.argv[1:]):
        if skip_next:
            skip_next = False
            continue
        if arg in ('-e', '--exact'):
            exact_mode = True
        elif arg == '--strout':
            strout_mode = True
        elif arg in ('-c', '--encoding') and i + 2 <= len(sys.argv[1:]):
            encoding = bgiop.normalize_encoding(sys.argv[1:][i+1])
            skip_next = True
        elif arg in ('-f', '--fallback-encoding') and i + 2 <= len(sys.argv[1:]):
            fallback_encoding = bgiop.normalize_encoding(sys.argv[1:][i+1])
            skip_next = True
        else:
            args.append(arg)
            
    for arg in args:
        for script in glob.glob(arg):
            base, ext = os.path.splitext(script)
            if (not ext or ext == '._bs') and os.path.isfile(script):
                print('Disassembling %s...' % script)
                dis(script, exact_mode, strout_mode, encoding, fallback_encoding)
