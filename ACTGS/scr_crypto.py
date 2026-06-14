#!/usr/bin/env python3
"""
ACTGS 引擎 - 加解密与档案处理核心模块
被 scr_extract.py 和 scr_inject.py 共用

提供:
  - auto_find_key(exe_path)                    从 ACTGS.exe 自动搜索 XOR 密钥
  - xor_cycle(data, key)                       循环 XOR (索引从 1 开始)
  - decrypt_script(raw, key) / encrypt_script  单个脚本的加解密 (首字节 0x58<->0x4E)
  - parse_archive(arc_path, key)               解析 arc.scr, 返回 (scripts, header, header_encrypted, gaps, trailing)
  - build_archive(header, scripts_data, key, header_encrypted, gaps, trailing)   重建 arc.scr

密钥搜索原理 (多层回退):
  1. 特征码搜索: 在 EXE .text 段搜索 cmp eax, 0x58 及后续 mov [ebp-xx], key_addr
  2. 附加模式: cmp al, 0x58 / mov eax, offset key / push offset key
  3. 已知密钥环回退: 对旧版 ACTGS 引擎使用硬编码密钥尝试解密索引
"""

import struct


# ============================================================
# 已知密钥环 (来自 crass ACTGS 插件)
# ============================================================
_KNOWN_KEY_RING = [
    b'"3A\x11\x05T\x16',                 # 新版 ACTGS (Delphi 编译)
    b'ACTGS ACTRESS Game System',         # 旧版 ACTGS 文本密钥
    b'\x10\xD3\x27\x53\x10\xD3\x27\x53\xC6\x08\x33\x12\x51\x88\x19\x21\xA1',  # 旧版二进制密钥
]


# ============================================================
# PE 解析
# ============================================================
def parse_pe_sections(data):
    """解析 PE 节表, 返回 (image_base, [(name, va, rawsize, rawoff), ...])"""
    pe_off = struct.unpack('<I', data[0x3C:0x40])[0]
    coff = data[pe_off+4 : pe_off+4+20]
    num_sec = struct.unpack('<H', coff[2:4])[0]
    opt_size = struct.unpack('<H', coff[16:18])[0]
    opt_hdr = data[pe_off+4+20 : pe_off+4+20+opt_size]
    image_base = struct.unpack('<I', opt_hdr[28:32])[0]

    sec_start = pe_off + 4 + 20 + opt_size
    sections = []
    for i in range(num_sec):
        sec = data[sec_start + i*40 : sec_start + (i+1)*40]
        name = sec[:8].rstrip(b'\x00').decode('ascii', 'replace')
        va      = struct.unpack('<I', sec[12:16])[0]
        rawsize = struct.unpack('<I', sec[16:20])[0]
        rawoff  = struct.unpack('<I', sec[20:24])[0]
        sections.append((name, va, rawsize, rawoff))
    return image_base, sections


def _va_to_fileoff(va_addr, image_base, sections):
    rva = va_addr - image_base
    for _, sva, srawsize, srawoff in sections:
        if sva <= rva < sva + srawsize:
            return srawoff + (rva - sva)
    return None


def _read_key_at_fileoff(data, foff):
    """从文件偏移读取以 null 结尾的密钥, 返回 bytes 或 None"""
    if not foff or foff >= len(data):
        return None
    end_idx = data.index(0, foff) if 0 in data[foff:foff+64] else foff + 64
    key = data[foff:end_idx]
    if 4 <= len(key) <= 32:
        return key
    return None


def _search_pattern_cmp_eax_58(data, text_start, text_end, image_base, sections):
    """
    搜索模式: cmp eax, 0x58  →  后续 mov [ebp-xx], imm32
    对应 Delphi/C++Builder 编译的 ACTGS 脚本解密函数.
    返回 key bytes 或 None.
    """
    pattern = bytes([0x83, 0xF8, 0x58])  # cmp eax, 0x58
    pos = text_start
    while pos < text_end:
        pos = data.find(pattern, pos, text_end)
        if pos < 0:
            break
        # 在后续 25 字节内搜索 C7 45 xx [dword] (mov [ebp-xx], imm32)
        window = data[pos:pos + 30]
        for j in range(3, 25):
            if j + 6 < len(window) and window[j] == 0xC7 and window[j+1] == 0x45:
                addr = struct.unpack('<I', window[j+3:j+7])[0]
                foff = _va_to_fileoff(addr, image_base, sections)
                key = _read_key_at_fileoff(data, foff)
                if key:
                    return key
        pos += 1
    return None


def _search_pattern_cmp_al_58(data, text_start, text_end, image_base, sections):
    """
    搜索模式: cmp al, 0x58 (3C 58)  →  后续 mov [ebp-xx], imm32
    某些编译器将 char 比较优化为 8 位操作.
    返回 key bytes 或 None.
    """
    pattern = bytes([0x3C, 0x58])  # cmp al, 0x58
    pos = text_start
    while pos < text_end:
        pos = data.find(pattern, pos, text_end)
        if pos < 0:
            break
        window = data[pos:pos + 30]
        for j in range(2, 25):
            if j + 6 < len(window) and window[j] == 0xC7 and window[j+1] == 0x45:
                addr = struct.unpack('<I', window[j+3:j+7])[0]
                foff = _va_to_fileoff(addr, image_base, sections)
                key = _read_key_at_fileoff(data, foff)
                if key:
                    return key
        pos += 1
    return None


def _search_pattern_push_offset(data, text_start, text_end, image_base, sections):
    """
    搜索模式: 68 xx xx xx xx (push imm32) 紧跟在 cmp eax/byte, 0x58 之后.
    某些编译器将密钥地址通过 push 传递 (如 call strlen 前).
    返回 key bytes 或 None.
    """
    # 搜索 cmp eax, 0x58 或 cmp byte, 0x58 后紧跟 push imm32
    for pattern, pat_len in [(bytes([0x83, 0xF8, 0x58]), 3), (bytes([0x80, 0xF8, 0x58]), 3), (bytes([0x3C, 0x58]), 2)]:
        pos = text_start
        while pos < text_end:
            pos = data.find(pattern, pos, text_end)
            if pos < 0:
                break
            window = data[pos + pat_len:pos + pat_len + 30]
            for j in range(0, min(25, len(window) - 5)):
                if window[j] == 0x68:  # push imm32
                    addr = struct.unpack('<I', window[j+1:j+5])[0]
                    foff = _va_to_fileoff(addr, image_base, sections)
                    key = _read_key_at_fileoff(data, foff)
                    if key:
                        return key
            pos += 1
    return None


def _search_pattern_mov_eax_offset(data, text_start, text_end, image_base, sections):
    """
    搜索模式: B8 xx xx xx xx (mov eax, imm32) 紧跟在 cmp 0x58 之后.
    某些编译器优化为 mov eax, offset key 后调用 strlen.
    返回 key bytes 或 None.
    """
    for pattern, pat_len in [(bytes([0x83, 0xF8, 0x58]), 3), (bytes([0x3C, 0x58]), 2)]:
        pos = text_start
        while pos < text_end:
            pos = data.find(pattern, pos, text_end)
            if pos < 0:
                break
            window = data[pos + pat_len:pos + pat_len + 30]
            for j in range(0, min(25, len(window) - 5)):
                if window[j] == 0xB8:  # mov eax, imm32
                    addr = struct.unpack('<I', window[j+1:j+5])[0]
                    foff = _va_to_fileoff(addr, image_base, sections)
                    key = _read_key_at_fileoff(data, foff)
                    if key:
                        return key
            pos += 1
    return None


def _find_key_from_exe(exe_path):
    """从 EXE 文件中搜索 XOR 密钥 (多层特征码匹配). 返回 bytes 或 None."""
    with open(exe_path, 'rb') as f:
        data = f.read()

    image_base, sections = parse_pe_sections(data)

    # 优先搜索 .text 段, 若不存在则搜索所有可执行段
    text_sec = next((s for s in sections if s[0] == '.text'), None)
    if text_sec:
        search_sections = [text_sec]
    else:
        # 回退: 搜索任何包含 "text" 或 "code" 的段, 或所有 RX 段
        search_sections = [s for s in sections if 'text' in s[0].lower() or 'code' in s[0].lower()]
        if not search_sections:
            return None

    searchers = [
        _search_pattern_cmp_eax_58,
        _search_pattern_cmp_al_58,
        _search_pattern_push_offset,
        _search_pattern_mov_eax_offset,
    ]

    for sec in search_sections:
        sec_start = sec[3]
        sec_end = sec[3] + sec[2]
        for searcher in searchers:
            key = searcher(data, sec_start, sec_end, image_base, sections)
            if key:
                return key

    return None


def _try_key_ring(arc_path):
    """使用已知密钥环尝试解密档案索引, 返回 (key, header_encrypted) 或 (None, None)."""
    with open(arc_path, 'rb') as f:
        data = f.read()

    _HDR_SIZE = 0x10
    _ENTRY_SIZE = 0x20
    _MAX_FILES = 100000

    for key in _KNOWN_KEY_RING:
        # 尝试明文头部
        try:
            file_count = struct.unpack('<I', data[:4])[0]
            if 0 < file_count <= _MAX_FILES and _HDR_SIZE + file_count * _ENTRY_SIZE <= len(data):
                # 尝试解密索引验证
                index_enc = data[_HDR_SIZE:_HDR_SIZE + file_count * _ENTRY_SIZE]
                index_dec = bytearray(xor_cycle(index_enc, key))
                first_name = index_dec[8:_ENTRY_SIZE].split(b'\x00')[0]
                if first_name and all(32 <= b < 127 for b in first_name):
                    # 验证偏移合理
                    first_offset = struct.unpack('<I', index_dec[0:4])[0]
                    expected = _HDR_SIZE + file_count * _ENTRY_SIZE
                    if first_offset >= expected:
                        return key, False
        except Exception:
            pass

        # 尝试加密头部
        try:
            header_raw = bytearray(xor_cycle(data[:_HDR_SIZE], key))
            file_count = struct.unpack('<I', header_raw[:4])[0]
            if 0 < file_count <= _MAX_FILES and _HDR_SIZE + file_count * _ENTRY_SIZE <= len(data):
                index_enc = data[_HDR_SIZE:_HDR_SIZE + file_count * _ENTRY_SIZE]
                index_dec = bytearray(xor_cycle(index_enc, key))
                first_name = index_dec[8:_ENTRY_SIZE].split(b'\x00')[0]
                if first_name and all(32 <= b < 127 for b in first_name):
                    first_offset = struct.unpack('<I', index_dec[0:4])[0]
                    expected = _HDR_SIZE + file_count * _ENTRY_SIZE
                    if first_offset >= expected:
                        return key, True
        except Exception:
            pass

    return None, None


def auto_find_key(exe_path):
    """
    从 ACTGS.exe 自动搜索 XOR 密钥.

    1. 优先通过特征码在 EXE 中搜索
    2. 若失败, 回退到已知密钥环

    返回 bytes 或 None.
    """
    # 步骤 1: 从 EXE 搜索
    key = _find_key_from_exe(exe_path)
    if key:
        return key

    # 步骤 2: 回退到已知密钥环 (无需 EXE, 但需要同一目录有 .dat/.scr)
    import os
    exe_dir = os.path.dirname(exe_path)
    for ext in ['.scr', '.dat']:
        for fname in os.listdir(exe_dir):
            if fname.lower().endswith(ext):
                arc_path = os.path.join(exe_dir, fname)
                key, _ = _try_key_ring(arc_path)
                if key:
                    return key

    return None


# ============================================================
# XOR 加解密
# ============================================================
def xor_cycle(data, key):
    """循环 XOR: ki = ki % klen + 1, 索引从 1 开始 (与引擎一致)"""
    out = bytearray(data)
    klen = len(key)
    ki = 0
    for i in range(len(out)):
        out[i] ^= key[ki % klen]
        ki = ki % klen + 1
    return bytes(out)


def decrypt_script(raw, key):
    """脚本解密: 首字节 0x58('X') → 0x4E('N'), 其余循环 XOR"""
    if not raw or raw[0] != 0x58:
        return raw
    dec = bytearray(raw)
    klen = len(key)
    ki = 0
    for j in range(1, len(dec)):
        dec[j] ^= key[ki % klen]
        ki = ki % klen + 1
    dec[0] = 0x4E
    return bytes(dec)


def encrypt_script(dec_data, key):
    """脚本加密: 首字节 → 0x58, 其余循环 XOR"""
    if not dec_data:
        return dec_data
    enc = bytearray(dec_data)
    enc[0] = 0x58
    klen = len(key)
    ki = 0
    for j in range(1, len(enc)):
        enc[j] ^= key[ki % klen]
        ki = ki % klen + 1
    return bytes(enc)


# ============================================================
# 档案解析 / 重建
# ============================================================
_HDR_SIZE   = 0x10
_ENTRY_SIZE = 0x20
_MAX_FILES  = 100000


def parse_archive(arc_path, key):
    """
    解析 arc.scr

    Returns:
        (scripts, header, header_encrypted, gaps, trailing)
          scripts:  [(name, decrypted_bytes), ...]
          header:   原始头部 16 字节 (未加密形式, 用于 build_archive 复用)
          header_encrypted: 原档案头部是否加密 (重建时需保持一致)
          gaps:     文件间空隙数据 [(file_index, gap_bytes), ...]
                    第 i 项是 scripts[i] 和 scripts[i+1] 之间的原始字节
          trailing: 最后一个文件之后的尾部字节

    旧版调用者若解包为 3 元组, trailing 和 gaps 被忽略即可.
    """
    with open(arc_path, 'rb') as f:
        data = f.read()

    header_encrypted = False
    header_raw = bytearray(data[:_HDR_SIZE])

    file_count = struct.unpack('<I', data[:4])[0]

    if file_count == 0 or file_count > _MAX_FILES or _HDR_SIZE + file_count * _ENTRY_SIZE > len(data):
        # 头部可能被加密
        header_raw = bytearray(xor_cycle(data[:_HDR_SIZE], key))
        file_count = struct.unpack('<I', header_raw[:4])[0]
        if file_count == 0 or file_count > _MAX_FILES or _HDR_SIZE + file_count * _ENTRY_SIZE > len(data):
            raise ValueError(f"无法解析档案: 文件数={file_count} 不合理")
        header_encrypted = True

    index_enc = data[_HDR_SIZE : _HDR_SIZE + file_count * _ENTRY_SIZE]
    index_dec = bytearray(xor_cycle(index_enc, key))

    first_name = index_dec[8:_ENTRY_SIZE].split(b'\x00')[0]
    if not first_name or not all(32 <= b < 127 for b in first_name):
        raise ValueError(f"索引解密失败: 首条目文件名非法 ({first_name.hex()})")

    scripts = []
    gaps = []       # [(after_index, gap_bytes), ...]
    prev_end = None

    for i in range(file_count):
        entry = index_dec[i * _ENTRY_SIZE : (i + 1) * _ENTRY_SIZE]
        offset = struct.unpack('<I', entry[0:4])[0]
        size   = struct.unpack('<I', entry[4:8])[0]
        name   = entry[8:_ENTRY_SIZE].split(b'\x00')[0].decode('ascii')
        scr    = decrypt_script(data[offset : offset + size], key)
        scripts.append((name, scr))

        # 捕获与前一个文件之间的空隙
        if prev_end is not None and offset > prev_end:
            gap_bytes = data[prev_end:offset]
            gaps.append((i - 1, gap_bytes))
        prev_end = offset + size

    # 捕获尾部数据
    trailing = data[prev_end:] if prev_end is not None else b''

    return scripts, bytes(header_raw), header_encrypted, gaps, trailing


def build_archive(header, scripts_data, key, header_encrypted=False,
                  gaps=None, trailing=b''):
    """
    重建 arc.scr

    Parameters:
        header:           parse_archive 返回的未加密头部
        scripts_data:     [(name, already_encrypted_bytes), ...]
        header_encrypted: 是否加密头部 (默认 False)
        gaps:             文件间空隙 [(after_index, gap_bytes), ...]
                          来自 parse_archive, None 表示忽略空隙 (重建为连续布局)
        trailing:         尾部字节 (来自 parse_archive), 默认为空

    若提供 gaps/trailing, 则原样还原空隙和尾部以实现零突变往返.
    否则以连续布局重建 (适用于注入翻译后的新档案).
    """
    file_count = len(scripts_data)
    data_start = _HDR_SIZE + file_count * _ENTRY_SIZE
    index_entries = bytearray()
    file_data = bytearray()

    if gaps:
        # 精确模式: 按原始偏移重建, 保留空隙
        # 从 gaps 重建每个文件的偏移
        file_offsets = []
        current_offset = data_start
        for i, (name, enc_scr) in enumerate(scripts_data):
            file_offsets.append((current_offset, name, enc_scr))
            current_offset += len(enc_scr)
            # 插入空隙
            gap_for_i = [g for g in gaps if g[0] == i]
            if gap_for_i:
                current_offset += len(gap_for_i[0][1])

        # 生成索引
        for i, (offset, name, enc_scr) in enumerate(file_offsets):
            entry = bytearray(_ENTRY_SIZE)
            struct.pack_into('<I', entry, 0, offset)
            struct.pack_into('<I', entry, 4, len(enc_scr))
            name_bytes = name.encode('ascii')
            entry[8:8+len(name_bytes)] = name_bytes
            index_entries.extend(entry)

        # 生成文件数据区 (含空隙)
        for i, (offset, name, enc_scr) in enumerate(file_offsets):
            file_data.extend(enc_scr)
            # 写入空隙
            gap_for_i = [g for g in gaps if g[0] == i]
            if gap_for_i:
                file_data.extend(gap_for_i[0][1])
        # 追加尾部
        file_data.extend(trailing)
    else:
        # 连续模式 (默认): 文件紧密排列, 无空隙
        index_entries_list = []
        current_offset = data_start
        file_parts = []
        for name, enc_scr in scripts_data:
            entry = bytearray(_ENTRY_SIZE)
            struct.pack_into('<I', entry, 0, current_offset)
            struct.pack_into('<I', entry, 4, len(enc_scr))
            name_bytes = name.encode('ascii')
            entry[8:8+len(name_bytes)] = name_bytes
            index_entries_list.append(entry)
            file_parts.append(enc_scr)
            current_offset += len(enc_scr)

        for entry in index_entries_list:
            index_entries.extend(entry)
        for part in file_parts:
            file_data.extend(part)

    index_enc = xor_cycle(bytes(index_entries), key)

    out = bytearray(header)
    struct.pack_into('<I', out, 0, file_count)
    if header_encrypted:
        out = bytearray(xor_cycle(bytes(out), key))

    out.extend(index_enc)
    out.extend(file_data)
    return bytes(out)


# ============================================================
# CLI 便捷入口: 直接运行可仅提取密钥
# ============================================================
if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print(f"用法: {sys.argv[0]} <ACTGS.exe>")
        print(f"  从 EXE 中搜索并打印 XOR 密钥")
        sys.exit(1)
    key = auto_find_key(sys.argv[1])
    if not key:
        print("错误: 未找到密钥")
        sys.exit(1)
    print(f"密钥 (hex): {key.hex()}")
    print(f"密钥 (raw): {key!r}")
    print(f"长度: {len(key)}")
