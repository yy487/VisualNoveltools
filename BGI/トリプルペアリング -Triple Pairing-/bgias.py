#!/usr/bin/env python3

# BGI script file assembler

import glob
import os
import struct
import sys
import base64
import re

import asdis
import bgiop

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

def parse_instr(line, n):
    strings = []
    fcn, argstr = asdis.re_instr.match(line).groups()
    argstr = argstr.strip()
    if argstr:
        argstr = argstr.replace('\\\\', asdis.backslash_replace).replace('\\"', asdis.quote_replace)
        quotes = asdis.get_quotes(argstr, n)
        if len(quotes) % 2 != 0:
            raise asdis.QuoteMismatch('Mismatched quotes @ line %d' % n)
        argstr = asdis.replace_quote_commas(argstr, quotes)
        args = [x.strip().replace(asdis.comma_replace, ',').replace(asdis.quote_replace, '\\"').replace(asdis.backslash_replace, '\\\\') for x in argstr.split(',')]
        for arg in args:
            if arg and arg[0] == '"' and arg[-1] == '"':
                strings.append(arg)
    else:
        args = []
    return fcn, args, strings

def _resolve_asm_fmt(op, args, line_no):
    fmt = bgiop.ops[op][0]
    if op == 0x016 and fmt == '<i':
        if len(args) == 0:
            return ''
        if len(args) == 1:
            return '<i'
        raise ValueError(f'Invalid argument count for f_016 @ line {line_no}')
    return fmt

def parse(asmtxt, search_dir='.', encoding='shift_jis', fallback_encoding='gbk'):
    instrs = []
    symbols = {}
    text_list = []
    text_seen = set()
    pos = 0
    hdrtext = None
    defines = {}
    imports = []
    extra_imports = []
    str_data_blob = b'' # Initialize earlier
    str_data_hex = []
    padding = None
    parsing_strings = False
    explicit_string_map = {}
    use_string_optimization = False
    
    def parse_str_line(line, current_blob):
        blob_chunk = b''
        is_entry = False
        if asdis.re_gap.match(line):
             hex_str, = asdis.re_gap.match(line).groups()
             blob_chunk = bytes.fromhex(hex_str)
             is_entry = True
        elif line.startswith('"') and line.endswith('"'):
             raw_str = asdis.unescape(line[1:-1])
             try:
                 encoded = _encode_text(raw_str, encoding, fallback_encoding) + b'\x00'
             except UnicodeEncodeError:
                 encoded = _encode_text(raw_str, fallback_encoding, None) + b'\x00'
             if line not in explicit_string_map:
                 explicit_string_map[line] = len(current_blob)
             blob_chunk = encoded
             is_entry = True
        elif asdis.re_strdata.match(line):
            hex_str, = asdis.re_strdata.match(line).groups()
            blob_chunk = bytes.fromhex(hex_str)
            is_entry = True
        return blob_chunk, is_entry

    # Pass 1: Scan for strings and definitions
    # This ensures that even if strings are at the end of the file, they are registered first.
    for id, line in enumerate(asmtxt.split('\n')):
        line = line.strip()
        line = asdis.remove_comment(line)
        if not line:
            continue
            
        if line.startswith('#string_table '):
            # Load external string table
            use_string_optimization = True
            fname_match = re.search(r'"([^"]+)"', line)
            if fname_match:
                fname = fname_match.group(1)
                fpath = os.path.join(search_dir, fname)
                if os.path.exists(fpath):
                    print(f"Loading string table from {fpath}")
                    try:
                        with open(fpath, 'r', encoding='utf-8') as fst:
                            for sline in fst:
                                sline = sline.strip()
                                sline = asdis.remove_comment(sline)
                                if not sline: continue
                                chunk, is_ent = parse_str_line(sline, str_data_blob)
                                if is_ent:
                                    str_data_blob += chunk
                    except Exception as e:
                        print(f"Error loading string table {fpath}: {e}")
                else:
                    print(f"Warning: String table {fpath} not found. Reconstructing logically.")
            continue

        if asdis.re_strings_start.match(line):
            parsing_strings = True
            continue
            
        if parsing_strings:
            chunk, is_ent = parse_str_line(line, str_data_blob)
            if is_ent:
                str_data_blob += chunk
                continue
            else:
                parsing_strings = False
                
        # Handle explicit #strdata anywhere (even outside #strings block)
        chunk, is_ent = parse_str_line(line, str_data_blob)
        if is_ent:
            str_data_blob += chunk

    # Pass 2: Parse instructions and other metadata
    parsing_strings = False # Reset
    pos = 0 # Reset pos tracking
    
    for id, line in enumerate(asmtxt.split('\n')):
        line = line.strip()
        line = asdis.remove_comment(line)
        if not line:
            continue
            
        if asdis.re_strings_start.match(line):
            parsing_strings = True
            continue
            
        if parsing_strings:
            # Skip lines handled in Pass 1
            if asdis.re_gap.match(line) or (line.startswith('"') and line.endswith('"')):
                continue
            else:
                parsing_strings = False

        if asdis.re_header.match(line):
            hdrtext, = asdis.re_header.match(line).groups()
            hdrtext = asdis.unescape(hdrtext)
        elif asdis.re_header_padding.match(line):
            hex_str, = asdis.re_header_padding.match(line).groups()
            padding = bytes.fromhex(hex_str)
            # print(f"DEBUG: Found padding {len(padding)} bytes")
        elif asdis.re_import.match(line):
            name, = asdis.re_import.match(line).groups()
            name = asdis.unescape(name)
            imports.append(name)
        elif asdis.re_extra_import.match(line):
            name, = asdis.re_extra_import.match(line).groups()
            name = asdis.unescape(name)
            extra_imports.append(name)
        elif line.startswith('#export '):
            m = re.match(r'#export\s+"([^"]+)"\s+(.+)', line)
            if m:
                name = asdis.unescape(m.group(1))
                val_str = m.group(2).strip()
                try:
                    val = int(val_str, 0)
                    extra_imports.append((name, val))
                except:
                    print(f"Warning: Invalid export value {val_str} at line {id+1}")
        elif line.startswith('#extra_import_gap '):
            # Hacky support for gaps
            try:
                size = int(line.split()[1])
                extra_imports.append(f"__GAP:{size}__")
            except:
                pass
        elif asdis.re_strdata.match(line):
            pass # Handled in Pass 1
        elif line.startswith('"') and line.endswith('"'):
            pass # Handled in Pass 1
        elif asdis.re_define.match(line):
            name, offset_s = asdis.re_define.match(line).groups()
            defines[name] = offset_s
        elif asdis.re_label.match(line):
            symbol, = asdis.re_label.match(line).groups()
            symbols[symbol] = pos
        elif asdis.re_instr.match(line):
            fcn, args, strings = parse_instr(line, id+1)    
            record = fcn, args, pos, id+1
            
            instrs.append(record)
            try:
                op = bgiop.rops[fcn]
            except KeyError:
                raise asdis.InvalidFunction('Invalid function @ line %d' % (id+1))
            
            # Verify arg count matches format
            fmt = _resolve_asm_fmt(op, args, id+1)
            # ... (validation omitted)
            
            pos += struct.calcsize(fmt) + 4
        else:
            # raise asdis.InvalidInstructionFormat('Invalid instruction format @ line %d' % (id+1))
            # Ignore unknown lines or warn?
            pass
            
    return instrs, symbols, text_list, hdrtext, imports, extra_imports, defines, str_data_blob, padding, explicit_string_map, use_string_optimization
    
def out_hdr(fo, hdrtext, imports, extra_imports, padding, encoding='shift_jis', fallback_encoding='gbk'):
    def _encoded_name_len(text):
        return len(_encode_text(text, encoding, fallback_encoding))

    fo.write(_encode_text(hdrtext, encoding, fallback_encoding).ljust(0x1C, b'\x00'))
    entries = len(imports)
    
    body_size = 4 # entries count
    for name in imports:
        body_size += _encoded_name_len(name) + 1
        
    if extra_imports:
        body_size += 4 # extra entries count
        for item in extra_imports:
            if isinstance(item, tuple):
                name, val = item
                body_size += _encoded_name_len(name) + 1 + 4
            elif item.startswith('__GAP:'):
                size = int(item.split(':')[1].replace('__', ''))
                body_size += size
            else:
                body_size += _encoded_name_len(item) + 1
    
    if padding is not None:
        # Use explicit padding
        final_size_val = 4 + body_size + len(padding)
        fo.write(struct.pack('<II', final_size_val, entries))
        for name in imports:
            fo.write(_encode_text(name, encoding, fallback_encoding) + b'\x00')
            
        if extra_imports:
            # Filter out gaps for count?
            # NO, gaps are NOT entries in the count.
            # But we need to count how many ACTUAL imports there are.
            real_imports = [x for x in extra_imports if (isinstance(x, tuple) or not x.startswith('__GAP:'))]
            fo.write(struct.pack('<I', len(real_imports)))
            
            for item in extra_imports:
                if isinstance(item, tuple):
                    name, val = item
                    fo.write(_encode_text(name, encoding, fallback_encoding) + b'\x00')
                    fo.write(struct.pack('<I', val))
                elif item.startswith('__GAP:'):
                    size = int(item.split(':')[1].replace('__', ''))
                    fo.write(b'\x00' * size)
                else:
                    fo.write(_encode_text(item, encoding, fallback_encoding) + b'\x00')
                    
        fo.write(padding)
    else:
        # Align total header size to 32 bytes (0x20)
        # 0x1C (Signature) + 4 (Size Field) + body_size
        total_size = 0x1C + 4 + body_size
        pad_len = 0
        if total_size % 32 != 0:
            pad_len = 32 - (total_size % 32)
        
        # The value at 0x1C includes the Size field itself (4 bytes) + Body + Padding
        final_size_val = 4 + body_size + pad_len
        
        fo.write(struct.pack('<II', final_size_val, entries))
        for name in imports:
            fo.write(_encode_text(name, encoding, fallback_encoding) + b'\x00')
            
        if extra_imports:
            real_imports = [x for x in extra_imports if (isinstance(x, tuple) or not x.startswith('__GAP:'))]
            fo.write(struct.pack('<I', len(real_imports)))
            for item in extra_imports:
                if isinstance(item, tuple):
                    name, val = item
                    fo.write(_encode_text(name, encoding, fallback_encoding) + b'\x00')
                    fo.write(struct.pack('<I', val))
                elif isinstance(item, str) and item.startswith('__GAP:'):
                    # Gap in implicit mode? Should be allowed.
                    size = int(item.split(':')[1].replace('__', ''))
                    fo.write(b'\x00' * size)
                else:
                    fo.write(_encode_text(item, encoding, fallback_encoding) + b'\x00')
            
        fo.write(b'\x00'*pad_len)
    
def out(fo, instrs, symbols, texts, hdrtext, imports, extra_imports, defines, str_data_blob, padding, explicit_string_map, use_string_optimization=False, encoding='shift_jis', fallback_encoding='gbk'):
    if hdrtext:
        out_hdr(fo, hdrtext, imports, extra_imports, padding, encoding, fallback_encoding)
        
    # Pre-calculation of string offsets
    code_size = 0
    for fcn, args, pos, n in instrs:
        op = bgiop.rops[fcn]
        fmt = _resolve_asm_fmt(op, args, n)
        code_size += 4 + struct.calcsize(fmt)
        
    string_map = {}
    new_str_blob = b''
    
    # If we have explicit string map from #strings section, use it
    if explicit_string_map:
        # Adjust offsets to be absolute
        for s, off in explicit_string_map.items():
            string_map[s] = code_size + off
            
    # If no explicit strdata (and no #strings), build it from instructions
    elif not str_data_blob:
        current_offset = code_size
        for fcn, args, pos, n in instrs:
            for arg in args:
                if arg and arg.startswith('"') and arg.endswith('"'):
                    # Found a string literal
                    if arg not in string_map:
                        string_map[arg] = current_offset
                        raw_str = asdis.unescape(arg[1:-1])
                        encoded = _encode_text(raw_str, encoding, fallback_encoding) + b'\x00'
                        new_str_blob += encoded
                        current_offset += len(encoded)
        str_data_blob = new_str_blob

    final_blob = str_data_blob # Use local variable to accumulate

    for fcn, args, pos, n in instrs:
        op = bgiop.rops[fcn]
        fo.write(struct.pack('<I', op))
        
        fmt = _resolve_asm_fmt(op, args, n)
        if fmt:
            clean_args = []
            for arg in args:
                val = 0
                if arg and arg.startswith('"') and arg.endswith('"'):
                    if arg in string_map:
                        val = string_map[arg]
                    else:
                        # Auto-append missing string to the end of the blob
                        # This supports mixed explicit/implicit mode (Explicit Header + Implicit Tail)
                        
                        raw_str = asdis.unescape(arg[1:-1])
                        encoded = _encode_text(raw_str, encoding, fallback_encoding) + b'\x00'
                        
                        # Suffix/Deduplication Optimization:
                        # Check if the string already exists in the blob
                        # Only use this if we have an explicit string table (which might contain overlaps)
                        found_pos = -1
                        if use_string_optimization:
                            found_pos = final_blob.find(encoded)
                            if found_pos != -1:
                                pass # print(f"DEBUG: Optimized string '{arg}' -> found at {found_pos}")
                            else:
                                pass # print(f"DEBUG: String optimization failed for '{arg}' - appending")
                            
                        if found_pos != -1:
                            val = code_size + found_pos
                            string_map[arg] = val
                        else:
                            current_blob_offset = len(final_blob)
                            val = code_size + current_blob_offset
                            string_map[arg] = val
                            final_blob += encoded
                elif re.match(r'^L[0-9a-fA-F]{5,}$', arg):
                    if arg in symbols:
                        val = symbols[arg]
                    else:
                        try:
                            val = int(arg[1:], 16)
                        except:
                            raise ValueError(f'Undefined label: {arg} @ position {pos}')
                elif arg in symbols:
                    val = symbols[arg]
                elif arg.startswith('0x') or arg.startswith('-0x'):
                    val = int(arg, 16)
                elif arg:
                    try:
                        val = int(arg)
                    except ValueError:
                        if arg in symbols:
                            val = symbols[arg]
                        else:
                            raise ValueError(f'Invalid argument: {arg} @ position {pos}')
                clean_args.append(val)
            fo.write(struct.pack(fmt, *clean_args))

    if final_blob:
        print(f"DEBUG: Writing explicit string table size={len(final_blob)}")
        fo.write(final_blob)

def asm(file, encoding='shift_jis', fallback_encoding='gbk', output_path=None):
    encoding = bgiop.normalize_encoding(encoding)
    fallback_encoding = bgiop.normalize_encoding(fallback_encoding)
    if not os.path.exists(file):
        print(f"Error: File {file} not found.")
        return

    if output_path:
        ofile = output_path
    else:
        ofile = os.path.splitext(file)[0]
    asmtxt = open(file, 'r', encoding='utf-8').read()
    
    search_dir = os.path.dirname(os.path.abspath(file))

    try:
        instrs, symbols, texts, hdrtext, imports, extra_imports, defines, str_data_blob, padding, explicit_string_map, use_string_optimization = parse(asmtxt, search_dir, encoding, fallback_encoding)
        out(open(ofile, 'wb'), instrs, symbols, texts, hdrtext, imports, extra_imports, defines, str_data_blob, padding, explicit_string_map, use_string_optimization, encoding, fallback_encoding)
    except asdis.InvalidFunction as e:
        msg = (
            f"{e}\n"
            "可能原因: bss_mapping.json 不存在或与反汇编时不同，导致函数名无法识别。\n"
            "请使用相同的 bss_mapping.json 重新反汇编/汇编，或将函数名改回 f_XXX 形式。"
        )
        print(msg)
        raise Exception(msg)
    except Exception as e:
        print(e)
        import traceback
        traceback.print_exc()
        raise
    
if __name__ == '__main__':
    if len(sys.argv) < 2:
        print('Usage: bgias.py <file(s)> [-c|--encoding <enc>] [-f|--fallback-encoding <enc>]')
        print('(only .bsd files amongst <file(s)> will be processed)')
        sys.exit(1)
    encoding = 'shift_jis'
    fallback_encoding = 'gbk'
    args = []
    skip_next = False
    for i, arg in enumerate(sys.argv[1:]):
        if skip_next:
            skip_next = False
            continue
        if arg in ('-c', '--encoding') and i + 2 <= len(sys.argv[1:]):
            encoding = bgiop.normalize_encoding(sys.argv[1:][i+1])
            skip_next = True
        elif arg in ('-f', '--fallback-encoding') and i + 2 <= len(sys.argv[1:]):
            fallback_encoding = bgiop.normalize_encoding(sys.argv[1:][i+1])
            skip_next = True
        else:
            args.append(arg)
    try:
        for arg in args:
            for script in glob.glob(arg):
                base, ext = os.path.splitext(script)
                if ext == '.bsd':
                    print('Assembling %s...' % script)
                    asm(script, encoding, fallback_encoding)
    except Exception:
        sys.exit(1)
