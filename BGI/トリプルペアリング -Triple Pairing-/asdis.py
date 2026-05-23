# Common assembler/disassembler routines

import re

comma_replace     = '@@@z@@Q@@@'
quote_replace     = '$$$q$$H$$$'
backslash_replace = '###g##V###'

re_label = re.compile(r'([A-Za-z_][A-Za-z0-9_]+):$')
re_instr = re.compile(r'([A-Za-z_][A-Za-z0-9_:]*)\((.*)\);$')
re_header = re.compile(r'#header\s+"(.*)"')
re_header_padding = re.compile(r'#header_padding\s+"(.*)"')
re_import = re.compile(r'#import\s+"(.*)"')
re_extra_import = re.compile(r'#extra_import\s+"(.*)"')
re_strdata = re.compile(r'#strdata\s+"(.*)"')
re_define = re.compile(r'#define\s+([A-Za-z0-9_]+)\s+([A-Za-z0-9_]+)')

re_strings_start = re.compile(r'^#strings$')
re_gap = re.compile(r'^#gap\s+"(.+)"')
re_hex_placeholder = re.compile(r'\{\{([0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2})*)\}\}')

class QuoteMismatch(Exception):
	pass
	
class InvalidInstructionFormat(Exception):
	pass
	
class InvalidFunction(Exception):
	pass

def escape(text):
	text = text.replace('\\', '\\\\')
	text = text.replace('\a', '\\a')
	text = text.replace('\b', '\\b')
	text = text.replace('\t', '\\t')
	text = text.replace('\n', '\\n')
	text = text.replace('\v', '\\v')
	text = text.replace('\f', '\\f')
	text = text.replace('\r', '\\r')
	text = text.replace('"', '\\"')
	return text
	
def unescape(text):
	text = text.replace('\\\\', backslash_replace)
	text = text.replace('\\a', '\a')
	text = text.replace('\\b', '\b')
	text = text.replace('\\t', '\t')
	text = text.replace('\\n', '\n')
	text = text.replace('\\v', '\v')
	text = text.replace('\\f', '\f')
	text = text.replace('\\r', '\r')
	text = text.replace('\\"', '"')
	text = text.replace(backslash_replace, '\\')
	return text

def format_hex_placeholder(raw):
	return '{{' + ':'.join(['%02X' % b for b in raw]) + '}}'

def placeholder_to_bytes(text):
	return bytes.fromhex(text.replace(':', ''))

def _is_private_use(ch):
	code = ord(ch)
	return (
		0xE000 <= code <= 0xF8FF
		or 0xF0000 <= code <= 0xFFFFD
		or 0x100000 <= code <= 0x10FFFD
	)

def _is_control_char(ch):
	if ch in '\a\b\t\n\v\f\r':
		return False
	code = ord(ch)
	return (0x00 <= code <= 0x1F) or (0x7F <= code <= 0x9F)

def is_hex_placeholder_candidate(ch):
	return _is_private_use(ch) or _is_control_char(ch)

def sanitize_decoded_text(text, encoding):
	out = []
	i = 0
	while i < len(text):
		if not is_hex_placeholder_candidate(text[i]):
			out.append(text[i])
			i += 1
			continue
		j = i + 1
		while j < len(text) and is_hex_placeholder_candidate(text[j]):
			j += 1
		chunk = text[i:j]
		try:
			raw = chunk.encode(encoding)
			out.append(format_hex_placeholder(raw))
		except Exception:
			for ch in chunk:
				try:
					out.append(format_hex_placeholder(ch.encode(encoding)))
				except Exception:
					out.append(ch)
		i = j
	return ''.join(out)

def decode_with_placeholders(data, encoding):
	try:
		return sanitize_decoded_text(data.decode(encoding), encoding)
	except UnicodeDecodeError:
		pass
	out = []
	pos = 0
	while pos < len(data):
		try:
			out.append(sanitize_decoded_text(data[pos:].decode(encoding), encoding))
			break
		except UnicodeDecodeError as e:
			if e.start > 0:
				prefix = data[pos:pos + e.start]
				out.append(sanitize_decoded_text(prefix.decode(encoding), encoding))
			bad_start = pos + e.start
			bad_end = pos + max(e.end, e.start + 1)
			out.append(format_hex_placeholder(data[bad_start:bad_end]))
			pos = bad_end
	return ''.join(out)

def encode_with_placeholders(text, encode_plain_text):
	data = bytearray()
	pos = 0
	for m in re_hex_placeholder.finditer(text):
		if m.start() > pos:
			data.extend(encode_plain_text(text[pos:m.start()]))
		data.extend(placeholder_to_bytes(m.group(1)))
		pos = m.end()
	if pos < len(text):
		data.extend(encode_plain_text(text[pos:]))
	return bytes(data)

def remove_comment(line):
	cpos = 0
	while True:
		cpos = line.find('//', cpos)
		if cpos == -1:
			return line.rstrip()
		qcount = line[:cpos].count('"') - line[:cpos].count('\\"')
		if qcount % 2 == 0:
			break
		cpos += 1
	line = line[:cpos]
	return line.rstrip()
	
def get_quotes(line, n):
	pos = 0
	quotes = []
	while True:
		pos = line.find('"', pos)
		if pos == -1:
			break
		quotes.append(pos)
		pos += 1
	return quotes
	
def replace_quote_commas(line, quotes):
	pos = 0
	commas = []
	while True:
		pos = line.find(',', pos)
		if pos == -1:
			break
		for squote, equote in zip(quotes[::2], quotes[1::2]):
			if squote < pos < equote:
				commas.append(pos)
				break
		pos += 1
	commas.reverse()
	for pos in commas:
		line = line[:pos] + comma_replace + line[pos+1:]
	return line
