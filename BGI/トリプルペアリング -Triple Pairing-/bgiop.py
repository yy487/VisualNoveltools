# BGI script file opcode table

import asdis
import os
import re

re_fcn = re.compile(r'([A-Za-z_][A-Za-z0-9_:]*)\(.*\)')

offsets = set()
_string_encoding = 'shift_jis'
_string_fallback = 'gbk'
_decode_fallback_logged = set()
_decode_replace_logged = set()
_decode_fallback_count = 0
_decode_fallback_by_encoding = {}
_decode_replace_primary_count = 0
_decode_replace_utf8_count = 0

def normalize_encoding(encoding):
	if not encoding:
		return encoding
	enc = encoding.strip().lower()
	if enc in ('sjis', 'shift-jis', 'shift_jis', 's_jis'):
		return 'shift_jis'
	if enc in ('utf8', 'utf_8'):
		return 'utf-8'
	if enc in ('big-5', 'big5'):
		return 'big5'
	return enc

def set_string_encodings(primary, fallback=None):
	global _string_encoding, _string_fallback
	if primary:
		_string_encoding = normalize_encoding(primary)
	if fallback:
		_string_fallback = normalize_encoding(fallback)

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

def _get_encoding_list():
	encodings = []
	if _string_encoding:
		encodings.append(normalize_encoding(_string_encoding))
	if _string_fallback and _string_fallback not in encodings:
		encodings.append(normalize_encoding(_string_fallback))
	for enc in ('cp932', 'gbk', 'utf-8'):
		if enc not in encodings:
			encodings.append(enc)
	return encodings

def _decode_bytes(data):
	global _decode_fallback_count, _decode_replace_primary_count, _decode_replace_utf8_count
	tried = []
	encodings = _get_encoding_list()
	for idx, enc in enumerate(encodings):
		try:
			text = asdis.sanitize_decoded_text(data.decode(enc), enc)
			if idx > 0:
				_decode_fallback_count += 1
				_decode_fallback_by_encoding[enc] = _decode_fallback_by_encoding.get(enc, 0) + 1
				key = (tuple(tried), enc)
				if key not in _decode_fallback_logged:
					_decode_fallback_logged.add(key)
					preview = data[:24].hex()
					tail = '...' if len(data) > 24 else ''
					print(f"[解码回退] 字符串解码失败，尝试编码 {tried}，改用 {enc} 成功，bytes={preview}{tail}")
			return text
		except Exception:
			tried.append(enc)
			continue
	try:
		_decode_replace_primary_count += 1
		key = ('replace', _string_encoding)
		if key not in _decode_replace_logged:
			_decode_replace_logged.add(key)
			print(f"[解码回退] 所有编码严格解码失败，使用 {_string_encoding} 的十六进制占位模式")
		return asdis.decode_with_placeholders(data, _string_encoding)
	except Exception:
		_decode_replace_utf8_count += 1
		if ('replace', 'utf-8') not in _decode_replace_logged:
			_decode_replace_logged.add(('replace', 'utf-8'))
			print("[解码回退] 主编码占位解码失败，改用 utf-8 的十六进制占位模式")
		return asdis.decode_with_placeholders(data, 'utf-8')

def clear_offsets():
	offsets.clear()

def get_string_ptr(code, addr, defines, *args):
	offset = args[0]
	pos0 = args[0]
	try:
		pos1 = code.find(b'\x00', pos0)
		string = _decode_bytes(code[pos0:pos1])
		string = asdis.escape(string)
	except:
		string = "<bad>"
	return offset, string

def get_file_ptr(code, addr, defines, *args):
	offset = args[0]
	lno = args[1]
	pos0 = args[0]
	try:
		pos1 = code.find(b'\x00', pos0)
		string = _decode_bytes(code[pos0:pos1])
		string = asdis.escape(string)
	except:
		string = "<bad>"
	return offset, lno, string
	
def get_string_content(code, addr, defines, *args):
	offset = args[0]
	pos0 = args[0]
	try:
		pos1 = code.find(b'\x00', pos0)
		string = _decode_bytes(code[pos0:pos1])
		string = asdis.escape(string)
	except:
		string = "<bad>"
	return (string,)

def get_line_content(code, addr, defines, *args):
	offset = args[0]
	lno = args[1]
	pos0 = args[0]
	try:
		pos1 = code.find(b'\x00', pos0)
		string = _decode_bytes(code[pos0:pos1])
		string = asdis.escape(string)
	except:
		string = "<bad>"
	return string, lno
	
def get_offset(code, addr, defines, *args):
	offset = args[0]
	offsets.add(offset)
	if offset in defines:
		offset_s = defines[offset]
	else:
		offset_s = 'L%05x' % offset
	return offset_s
	
ops = {
0x000: ('<i', 'push_dword(%d)', None),
0x001: ('<I', 'push_offset(%s)', get_offset),
0x002: ('<i', 'push_base_offset(%d)', None),
0x003: ('<I', 'push_string("%s")', get_string_content),
0x004: ('<i', 'f_004(%d)', None),
0x005: ('<i', 'f_005(%d)', None),
0x006: ('<i', 'f_006(%d)', None),
0x007: ('<i', 'f_007(%d)', None),
0x008: ('<i', 'load(%d)', None),
0x009: ('<i', 'move(%d)', None),
0x00A: ('<i', 'move_arg(%d)', None),
0x00B: ('<i', 'f_00b(%d)', None),
0x00C: ('<i', 'f_00c(%d)', None),
0x00D: ('<i', 'f_00d(%d)', None),
0x00E: ('<i', 'f_00e(%d)', None),
0x00F: ('<i', 'f_00f(%d)', None),
0x010: ('', 'load_base()', None),
0x011: ('', 'store_base()', None),
0x012: ('<i', 'f_012(%d)', None),
0x013: ('<i', 'f_013(%d)', None),
0x014: ('<i', 'f_014(%d)', None),
0x015: ('<i', 'f_015(%d)', None),
0x016: ('<i', 'f_016(%d)', None),
0x017: ('<i', 'f_017(%d)', None),
0x018: ('', 'jmp()', None),
0x019: ('<I', 'jc(%#x)', None),
0x01A: ('', 'call()', None),
0x01B: ('', 'ret()', None),
0x01E: ('', 'reg_exception_handler()', None),
0x01F: ('', 'unreg_exception_handler()', None),
0x020: ('', 'add()', None),
0x021: ('', 'sub()', None),
0x022: ('', 'mul()', None),
0x023: ('', 'div()', None),
0x024: ('', 'mod()', None),
0x025: ('', 'and()', None),
0x026: ('', 'or()', None),
0x027: ('', 'xor()', None),
0x028: ('', 'not()', None),
0x029: ('', 'shl()', None),
0x02A: ('', 'shr()', None),
0x02B: ('', 'sar()', None),
0x030: ('', 'eq()', None),
0x031: ('', 'neq()', None),
0x032: ('', 'leq()', None),
0x033: ('', 'geq()', None),
0x034: ('', 'lt()', None),
0x035: ('', 'gt()', None),
0x038: ('', 'bool_and()', None),
0x039: ('', 'bool_or()', None),
0x03A: ('', 'bool_zero()', None),
0x03F: ('<i', 'nargs(%d)', None),
0x07B: ('<Iii', 'f_07b(0x%x, %d, %d)', None),
0x07E: ('<I', 'f_07e(%s)', get_offset),
0x07F: ('<Ii', 'line("%s", %d)', get_line_content),
}

rops = {}
_base_rops = {}

def make_ops():
	for op in range(0x600):
		if op not in ops:
			if op < 0x100:
				ops[op] = ('', 'f_%03x()' % op, None)
			elif 0x100 <= op < 0x140:
				ops[op] = ('', 'sys_::f_%03x()' % op, None)
			elif 0x140 <= op < 0x160:
				ops[op] = ('', 'msg_::f_%03x()' % op, None)
			elif 0x160 <= op < 0x180:
				ops[op] = ('', 'slct::f_%03x()' % op, None)
			elif 0x180 <= op < 0x200:
				ops[op] = ('', 'snd_::f_%03x()' % op, None)
			elif 0x200 <= op < 0x600:
				ops[op] = ('', 'grp_::f_%03x()' % op, None)
				
def make_rops():
	rops.clear()
	for op in ops:
		fcn, = re_fcn.match(ops[op][1]).groups()
		rops[fcn] = op
	for name, op in _base_rops.items():
		if name not in rops:
			rops[name] = op
	
import json

def _toolkit_root():
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def load_mapping():
    mapping_path = os.path.join(_toolkit_root(), 'bss_mapping.json')
    try:
        with open(mapping_path, 'r', encoding='utf-8') as f:
            mapping = json.load(f)
        for key, name in mapping.items():
            try:
                if key.startswith('@'):
                    op = int(key[1:], 16)
                elif '_' in key:
                    op = int(key.split('_')[1], 16)
                else:
                    continue
            except (ValueError, IndexError):
                continue
                
            if op in ops:
                fmt, pfmt, fcn = ops[op]
                # Replace function name in pfmt
                # pfmt is like "f_083(%d)" or "sys_::f_100()"
                # We want "f_CompareMemory(%d)"
                
                # Check if pfmt has arguments
                paren_index = pfmt.find('(')
                if paren_index != -1:
                    args_part = pfmt[paren_index:]
                    new_pfmt = name + args_part
                    ops[op] = (fmt, new_pfmt, fcn)
    except FileNotFoundError:
        print(f"Warning: bss_mapping.json not found: {mapping_path}")
    except Exception as e:
        print(f"Error loading mapping: {e}")

make_ops()
for op in ops:
	fcn, = re_fcn.match(ops[op][1]).groups()
	_base_rops[fcn] = op
load_mapping()
make_rops()
