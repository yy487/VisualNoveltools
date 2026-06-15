#!/usr/bin/env python3
"""
side-B メルトピア .dat 解包/封包工具 v2
算法: Rijndael-256/256 CBC (block=32B, key=32B, 14 rounds) + zlib
容器: AttacheCase v2 stripped header
密钥: 0x0047544F (LE) + 28 字节零 —— EXE 固定全局 key，不需要运行游戏提取

用法:
  martopia_dat_rebuild.py info <dat_file>
  martopia_dat_rebuild.py unpack <dat_file> <out_dir> --meta <meta_dir>
  martopia_dat_rebuild.py unpack-all <dat_dir> <out_root> --meta <meta_dir>
  martopia_dat_rebuild.py pack <unpacked_dir> <meta_json> <out_dat>
  martopia_dat_rebuild.py pack-all <unpacked_root> <out_dat_dir> <meta_dir>
  martopia_dat_rebuild.py selftest <dat_file>
"""
import os, sys, glob, time, struct, json, zlib, shutil, secrets, argparse

# ========================================================================
# Rijndael-256/256 核心
# ========================================================================

_SBOX_HEX = (
    "637c777bf26b6fc53001672bfed7ab76ca82c97dfa5947f0add4a2af9ca472c0"
    "b7fd9326363ff7cc34a5e5f171d8311504c723c31896059a071280e2eb27b275"
    "09832c1a1b6e5aa0523bd6b329e32f8453d100ed20fcb15b6acbbe394a4c58cf"
    "d0efaafb434d338545f9027f503c9fa851a3408f929d38f5bcb6da2110fff3d2"
    "cd0c13ec5f974417c4a77e3d645d197360814fdc222a908846eeb814de5e0bdb"
    "e0323a0a4906245cc2d3ac629195e479e7c8376d8dd54ea96c56f4ea657aae08"
    "ba78252e1ca6b4c6e8dd741f4bbd8b8a703eb5664803f60e613557b986c11d9e"
    "e1f8981169d98e949b1e87e9ce5528df8ca1890dbfe6426841992d0fb054bb16"
)
SBOX = list(bytes.fromhex(_SBOX_HEX))
INV_SBOX = [0] * 256
for i, v in enumerate(SBOX):
    INV_SBOX[v] = i

def _xtime(x):
    return ((x << 1) ^ 0x1b) & 0xff if x & 0x80 else (x << 1) & 0xff

def _gmul(a, b):
    p = 0
    for _ in range(8):
        if b & 1: p ^= a
        a = _xtime(a)
        b >>= 1
    return p & 0xff

RCON = [0, 1]
for _ in range(30):
    RCON.append(_xtime(RCON[-1]))

# 正向 T 表 (encrypt)
TE0 = [0] * 256
TE1 = [0] * 256
TE2 = [0] * 256
TE3 = [0] * 256
for i in range(256):
    s = SBOX[i]
    t2 = _gmul(s, 2)
    t3 = _gmul(s, 3)
    # Te0[i] = [2*S[i], 1*S[i], 1*S[i], 3*S[i]], Te1 = ROR(Te0, 8), ...
    w = (t2 << 24) | (s << 16) | (s << 8) | t3
    TE0[i] = w
    TE1[i] = ((w >> 8) | (w << 24)) & 0xffffffff
    TE2[i] = ((w >> 16) | (w << 16)) & 0xffffffff
    TE3[i] = ((w >> 24) | (w << 8)) & 0xffffffff

# 反向 T 表 (decrypt) —— Td1=ROR(Td0,8), Td2=ROR(Td0,16), Td3=ROR(Td0,24)
TD0 = [0] * 256
TD1 = [0] * 256
TD2 = [0] * 256
TD3 = [0] * 256
for i in range(256):
    s = INV_SBOX[i]
    e = _gmul(s, 0x0e)
    n = _gmul(s, 0x09)
    d = _gmul(s, 0x0d)
    b = _gmul(s, 0x0b)
    w = (e << 24) | (n << 16) | (d << 8) | b
    TD0[i] = w
    TD1[i] = ((w >> 8) | (w << 24)) & 0xffffffff
    TD2[i] = ((w >> 16) | (w << 16)) & 0xffffffff
    TD3[i] = ((w >> 24) | (w << 8)) & 0xffffffff

Nb, Nk, Nr = 8, 8, 14  # 256-bit block + 256-bit key → 14 rounds

def _key_expand_enc(key):
    W = [struct.unpack('>I', key[4*i:4*i+4])[0] for i in range(Nk)]
    for i in range(Nk, Nb*(Nr+1)):
        t = W[i-1]
        if i % Nk == 0:
            t = ((t << 8) | (t >> 24)) & 0xffffffff
            t = (SBOX[(t>>24)&0xff]<<24)|(SBOX[(t>>16)&0xff]<<16)|(SBOX[(t>>8)&0xff]<<8)|SBOX[t&0xff]
            t ^= RCON[i//Nk] << 24
        elif Nk > 6 and i % Nk == 4:
            t = (SBOX[(t>>24)&0xff]<<24)|(SBOX[(t>>16)&0xff]<<16)|(SBOX[(t>>8)&0xff]<<8)|SBOX[t&0xff]
        W.append((W[i-Nk] ^ t) & 0xffffffff)
    return W  # 长度 = Nb*(Nr+1) = 120

def _inv_mix_word(w):
    a = [(w>>24)&0xff, (w>>16)&0xff, (w>>8)&0xff, w&0xff]
    b0 = _gmul(a[0],0x0e)^_gmul(a[1],0x0b)^_gmul(a[2],0x0d)^_gmul(a[3],0x09)
    b1 = _gmul(a[0],0x09)^_gmul(a[1],0x0e)^_gmul(a[2],0x0b)^_gmul(a[3],0x0d)
    b2 = _gmul(a[0],0x0d)^_gmul(a[1],0x09)^_gmul(a[2],0x0e)^_gmul(a[3],0x0b)
    b3 = _gmul(a[0],0x0b)^_gmul(a[1],0x0d)^_gmul(a[2],0x09)^_gmul(a[3],0x0e)
    return (b0<<24)|(b1<<16)|(b2<<8)|b3

def make_enc_schedule(key):
    return _key_expand_enc(key)

def make_dec_schedule(key):
    W = _key_expand_enc(key)
    dW = W[:]
    for r in range(1, Nr):
        for c in range(Nb):
            dW[r*Nb + c] = _inv_mix_word(W[r*Nb + c])
    return dW

# --- encrypt 单块 (Python int) ---
def encrypt_block(pt, W):
    # pt: 32 bytes → 8 big-endian words
    s = list(struct.unpack('>8I', pt))
    for c in range(8):
        s[c] ^= W[c]
    # 中间 Nr-1 轮
    for r in range(1, Nr):
        t = [0]*8
        base = r * Nb
        # 正向 ShiftRows: 输出 col c 取输入 col (c+shift) % 8
        for c in range(8):
            b0 = (s[(c + 0) & 7] >> 24) & 0xff
            b1 = (s[(c + 1) & 7] >> 16) & 0xff
            b2 = (s[(c + 3) & 7] >>  8) & 0xff
            b3 =  s[(c + 4) & 7]        & 0xff
            t[c] = (TE0[b0] ^ TE1[b1] ^ TE2[b2] ^ TE3[b3] ^ W[base + c]) & 0xffffffff
        s = t
    # 最后一轮: 只有 SubBytes + ShiftRows + AddRoundKey (无 MixColumns)
    t = [0]*8
    base = Nr * Nb
    for c in range(8):
        b0 = SBOX[(s[(c + 0) & 7] >> 24) & 0xff]
        b1 = SBOX[(s[(c + 1) & 7] >> 16) & 0xff]
        b2 = SBOX[(s[(c + 3) & 7] >>  8) & 0xff]
        b3 = SBOX[ s[(c + 4) & 7]        & 0xff]
        t[c] = ((b0<<24) | (b1<<16) | (b2<<8) | b3) ^ W[base + c]
    return struct.pack('>8I', *t)

def cbc_encrypt(key, iv, pt):
    assert len(key) == 32 and len(iv) == 32 and len(pt) % 32 == 0
    W = make_enc_schedule(key)
    out = bytearray()
    prev = iv
    for i in range(0, len(pt), 32):
        blk = bytes(a ^ b for a, b in zip(pt[i:i+32], prev))
        prev = encrypt_block(blk, W)
        out += prev
    return bytes(out)

# --- decrypt: 尝试用 numpy 加速，否则走纯 Python ---
try:
    import numpy as _np
    _USE_NUMPY = True
    _TD0_np = _np.array(TD0, dtype=_np.uint32)
    _TD1_np = _np.array(TD1, dtype=_np.uint32)
    _TD2_np = _np.array(TD2, dtype=_np.uint32)
    _TD3_np = _np.array(TD3, dtype=_np.uint32)
    _INV_SBOX_np = _np.array(INV_SBOX, dtype=_np.uint8)

    def cbc_decrypt(key, iv, ct):
        assert len(key) == 32 and len(iv) == 32 and len(ct) % 32 == 0
        if not ct:
            return b''
        dW_list = make_dec_schedule(key)
        dW = _np.array(dW_list, dtype=_np.uint32).reshape(Nr+1, Nb)
        n = len(ct) // 32
        arr = _np.frombuffer(ct, dtype=_np.uint8).reshape(n, 32)
        cols = (arr[:, 0::4].astype(_np.uint32) << 24) | \
               (arr[:, 1::4].astype(_np.uint32) << 16) | \
               (arr[:, 2::4].astype(_np.uint32) <<  8) | \
               (arr[:, 3::4].astype(_np.uint32))
        state = cols ^ dW[Nr]
        for r in range(Nr-1, 0, -1):
            c_idx = _np.arange(8)
            src0 = state[:, (c_idx - 0) % 8]
            src1 = state[:, (c_idx - 1) % 8]
            src2 = state[:, (c_idx - 3) % 8]
            src3 = state[:, (c_idx - 4) % 8]
            b0 = ((src0 >> 24) & 0xff).astype(_np.uint8)
            b1 = ((src1 >> 16) & 0xff).astype(_np.uint8)
            b2 = ((src2 >>  8) & 0xff).astype(_np.uint8)
            b3 = ( src3        & 0xff).astype(_np.uint8)
            state = _TD0_np[b0] ^ _TD1_np[b1] ^ _TD2_np[b2] ^ _TD3_np[b3] ^ dW[r]
        c_idx = _np.arange(8)
        src0 = state[:, (c_idx - 0) % 8]
        src1 = state[:, (c_idx - 1) % 8]
        src2 = state[:, (c_idx - 3) % 8]
        src3 = state[:, (c_idx - 4) % 8]
        b0 = _INV_SBOX_np[((src0 >> 24) & 0xff).astype(_np.uint8)].astype(_np.uint32)
        b1 = _INV_SBOX_np[((src1 >> 16) & 0xff).astype(_np.uint8)].astype(_np.uint32)
        b2 = _INV_SBOX_np[((src2 >>  8) & 0xff).astype(_np.uint8)].astype(_np.uint32)
        b3 = _INV_SBOX_np[( src3        & 0xff).astype(_np.uint8)].astype(_np.uint32)
        state = ((b0 << 24) | (b1 << 16) | (b2 << 8) | b3) ^ dW[0]
        out = _np.zeros((n, 32), dtype=_np.uint8)
        out[:, 0::4] = ((state >> 24) & 0xff).astype(_np.uint8)
        out[:, 1::4] = ((state >> 16) & 0xff).astype(_np.uint8)
        out[:, 2::4] = ((state >>  8) & 0xff).astype(_np.uint8)
        out[:, 3::4] = ( state        & 0xff).astype(_np.uint8)
        raw_dec = out.tobytes()
        ct_arr = _np.frombuffer(ct, dtype=_np.uint8)
        prev = _np.concatenate([_np.frombuffer(iv, dtype=_np.uint8), ct_arr[:-32]])
        return (_np.frombuffer(raw_dec, dtype=_np.uint8) ^ prev).tobytes()
except ImportError:
    _USE_NUMPY = False
    def decrypt_block(ct, dW):
        s = list(struct.unpack('>8I', ct))
        for c in range(8):
            s[c] ^= dW[Nr*Nb + c]
        for r in range(Nr-1, 0, -1):
            t = [0]*8
            base = r * Nb
            for c in range(8):
                b0 = (s[(c - 0) & 7] >> 24) & 0xff
                b1 = (s[(c - 1) & 7] >> 16) & 0xff
                b2 = (s[(c - 3) & 7] >>  8) & 0xff
                b3 =  s[(c - 4) & 7]        & 0xff
                t[c] = TD0[b0] ^ TD1[b1] ^ TD2[b2] ^ TD3[b3] ^ dW[base + c]
            s = t
        t = [0]*8
        for c in range(8):
            b0 = INV_SBOX[(s[(c - 0) & 7] >> 24) & 0xff]
            b1 = INV_SBOX[(s[(c - 1) & 7] >> 16) & 0xff]
            b2 = INV_SBOX[(s[(c - 3) & 7] >>  8) & 0xff]
            b3 = INV_SBOX[ s[(c - 4) & 7]        & 0xff]
            t[c] = ((b0<<24) | (b1<<16) | (b2<<8) | b3) ^ dW[c]
        return struct.pack('>8I', *t)

    def cbc_decrypt(key, iv, ct):
        dW = make_dec_schedule(key)
        out = bytearray()
        prev = iv
        for i in range(0, len(ct), 32):
            c = ct[i:i+32]
            out += bytes(a ^ b for a, b in zip(decrypt_block(c, dW), prev))
            prev = c
        return bytes(out)

# ========================================================================
# 容器格式
# ========================================================================

GLOBAL_KEY = struct.pack('<I', 0x0047544F) + b'\x00' * 28
MAGIC_PLACEHOLDER = b'\xff' * 16   # 写回时的 16 字节 magic 占位

# 文件布局:
#   [0..3]                subver = 0x00030006
#   [4..19]               magic 占位 (FF*16 或 00*16)
#   [20..23]              task_code = 105 (0x69)
#   [24..27]              data_version = 1
#   [28..31]              TOC 字节数 N (32 对齐)
#   [32..63]              TOC 初始 IV
#   [64..64+N)            TOC 密文 (CBC 32B 块)
#   [64+N..64+N+31]       Payload 初始 IV
#   [64+N+32..EOF]        Payload 密文 (CBC → zlib)

def is_attachecase(d):
    if len(d) < 32: return False
    return (struct.unpack('<I', d[0:4])[0] == 0x00030006
            and struct.unpack('<I', d[20:24])[0] == 105
            and struct.unpack('<I', d[24:28])[0] == 1)

def read_dat_header(d):
    """读取 DAT 外层头。d 可以是完整文件 bytes，也可以只传前 32 字节。"""
    if len(d) < 32:
        raise ValueError('DAT header too short')
    return {
        'subver': struct.unpack('<I', d[0:4])[0],
        'marker_hex': d[4:20].hex(),
        'task_code': struct.unpack('<I', d[20:24])[0],
        'data_version': struct.unpack('<I', d[24:28])[0],
        'toc_size': struct.unpack('<I', d[28:32])[0],
    }

def _lenient_inflate(data):
    try:
        return zlib.decompress(data)
    except zlib.error:
        obj = zlib.decompressobj()
        out = obj.decompress(data)
        try:
            out += obj.flush()
        except zlib.error:
            pass
        return out

def decrypt_dat(path):
    with open(path, 'rb') as f:
        d = f.read()
    if not is_attachecase(d):
        raise ValueError(f"{path}: not an AttacheCase-encrypted file")

    hdr = read_dat_header(d[:32])
    toc_size = hdr['toc_size']
    if toc_size % 32 != 0:
        raise ValueError(f"{path}: toc_size {toc_size} is not 32-byte aligned")
    if 64 + toc_size + 32 > len(d):
        raise ValueError(f"{path}: toc_size {toc_size} out of range (size {len(d)})")

    toc_iv = d[32:64]
    toc_ct = d[64:64+toc_size]
    payload_iv = d[64+toc_size:64+toc_size+32]
    payload_ct = d[64+toc_size+32:]
    if len(payload_ct) % 32 != 0:
        raise ValueError(f"{path}: payload cipher size {len(payload_ct)} is not 32-byte aligned")

    toc_pt = cbc_decrypt(GLOBAL_KEY, toc_iv, toc_ct)
    payload_pt = cbc_decrypt(GLOBAL_KEY, payload_iv, payload_ct)
    blob = _lenient_inflate(payload_pt)
    hdr.update(dict(payload_cipher_size=len(payload_ct), blob_size=len(blob)))
    return toc_pt, blob, hdr

def parse_toc(toc_pt):
    """解析 TOC 明文。返回 (header_raw, entries)。
    header_raw: 从 TOC 开头到第一个 'Fn_' 之前的原始字节（保留 \\n\\r\\n 分隔符）
    entries: Fn_ 段条目列表
    """
    text = toc_pt.rstrip(b'\x00')
    # 定位第一个 Fn_ 作为 header 结束
    fn_pos = text.find(b'Fn_')
    header_raw = text[:fn_pos] if fn_pos >= 0 else b''

    # 切出 Fn_ 段直到 U_ 段（或结束）
    u_pos = text.find(b'U_')
    fn_section = text[fn_pos:u_pos] if u_pos >= 0 else text[fn_pos:]
    # 去掉 UTF-8 BOM（若紧接在 Fn_ 段末尾）
    fn_section = fn_section.rstrip(b'\xef\xbb\xbf').rstrip()
    entries = []
    for ln in fn_section.replace(b'\r\n', b'\n').split(b'\n'):
        if not ln.startswith(b'Fn_'):
            continue
        colon = ln.index(b':')
        fields = ln[colon+1:].split(b'\t')
        if len(fields) < 2:
            continue
        try:
            name = fields[0].decode('shift-jis', errors='replace')
        except Exception:
            name = fields[0].decode('latin-1')
        size_field = fields[1].decode('ascii', errors='replace') if len(fields) > 1 else '0'
        try:
            size = int(size_field)
            is_dir = False
        except ValueError:
            size = 0
            is_dir = True
        extra = [f.decode('ascii', errors='replace') for f in fields[2:]]
        entries.append(dict(name=name, size=size, is_dir=is_dir, extra=extra,
                             prefix=ln[:colon+1].decode('ascii', errors='replace')))
    return header_raw, entries

# ========================================================================
# Unpack
# ========================================================================

def cmd_unpack(dat_path, out_dir, meta_dir=None):
    toc_pt, blob, meta_hdr = decrypt_dat(dat_path)
    header_raw, entries = parse_toc(toc_pt)
    cur = 0
    for e in entries:
        e['offset'] = cur
        if not e['is_dir']:
            cur += e['size']
    if cur > len(blob):
        raise ValueError(f"{dat_path}: TOC file sizes exceed blob size ({cur} > {len(blob)})")
    trailing = blob[cur:]

    os.makedirs(out_dir, exist_ok=True)
    extracted = 0
    for e in entries:
        if e['is_dir']:
            continue
        data = blob[e['offset']:e['offset']+e['size']]
        outp = os.path.join(out_dir, e['name'].replace('\\', '/'))
        parent = os.path.dirname(outp)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(outp, 'wb') as f:
            f.write(data)
        extracted += 1

    if meta_dir is not None:
        os.makedirs(meta_dir, exist_ok=True)
        name_noext = os.path.splitext(os.path.basename(dat_path))[0]
        with open(os.path.join(meta_dir, name_noext + '.json'),
                  'w', encoding='utf-8') as f:
            json.dump({
                'tool': 'martopia_dat_rebuild_v2',
                'source_dat': os.path.basename(dat_path),
                'header_raw_hex': header_raw.hex(),
                'entries': entries,
                'trailing_size': len(trailing),
                'trailing_hex': trailing.hex(),
                'meta': meta_hdr,
            }, f, ensure_ascii=False, indent=1)
    return extracted, len(blob)

def cmd_unpack_all(dat_dir, out_root, meta_dir=None):
    dats = sorted(glob.glob(os.path.join(dat_dir, '*.dat')))
    print(f'[+] {len(dats)} .dat files found in {dat_dir}')
    os.makedirs(out_root, exist_ok=True)
    if meta_dir:
        os.makedirs(meta_dir, exist_ok=True)
    total_entries = 0
    total_bytes = 0
    t0 = time.time()
    raw_dir = os.path.join(out_root, '_raw')
    for i, p in enumerate(dats):
        name_noext = os.path.splitext(os.path.basename(p))[0]
        try:
            with open(p, 'rb') as f:
                head = f.read(32)
            if not is_attachecase(head):
                os.makedirs(raw_dir, exist_ok=True)
                shutil.copy(p, os.path.join(raw_dir, os.path.basename(p)))
                total_bytes += os.path.getsize(p)
                continue
            n, b = cmd_unpack(p, os.path.join(out_root, name_noext), meta_dir)
            total_entries += n
            total_bytes += b
        except Exception as e:
            print(f'  [FAIL] {p}: {e}')
        if (i+1) % 50 == 0 or i == len(dats)-1:
            el = time.time() - t0
            rate = total_bytes/el/1024/1024 if el > 0 else 0
            print(f'  [{i+1:4d}/{len(dats)}] entries={total_entries} '
                  f'blob={total_bytes/1024/1024:.1f}MB '
                  f'elapsed={el:.1f}s ({rate:.1f} MB/s)')
    el = time.time() - t0
    print(f'[+] DONE. {len(dats)} files, {total_entries} entries, '
          f'{total_bytes/1024/1024:.1f}MB in {el:.1f}s')

# ========================================================================
# Pack
# ========================================================================

def _pad_to_block(data, block=32):
    """用随机字节填充到 block 对齐（AttacheCase 风格）"""
    if len(data) % block == 0:
        return data
    pad = block - (len(data) % block)
    return data + secrets.token_bytes(pad)

def _build_toc_text(entries, header_raw=None, encode_utf8_block=True):
    """重建 TOC 明文（字节）。header_raw 为原始 header 字节块（保留 \\n\\r\\n 分隔符）"""
    if header_raw is None:
        now = time.strftime('%Y/%m/%d %H:%M:%S')
        header_raw = (b'Passcode:AttacheCase\n\r\n'
                      b'LastDateTime:' + now.encode('ascii') + b'\n\r\n')
    parts = [header_raw]
    for i, e in enumerate(entries):
        name = e['name'].encode('shift-jis', errors='replace')
        size_str = ('*' if e.get('is_dir') else str(e['size'])).encode('ascii')
        extras = b'\t'.join(f.encode('ascii', errors='replace')
                            for f in e.get('extra', []))
        prefix = e.get('prefix', f"Fn_{i}:").encode('ascii')
        line = prefix + name + b'\t' + size_str
        if extras:
            line += b'\t' + extras
        line += b'\r\n'
        parts.append(line)
    if encode_utf8_block:
        # UTF-8 段：BOM + 同样的 header（UTF-8 编码）+ U_N 条目
        parts.append(b'\xef\xbb\xbf')
        # header_raw 本身是 shift-jis，对 ASCII 部分直接复用即可
        try:
            header_txt = header_raw.decode('shift-jis', errors='replace')
        except Exception:
            header_txt = header_raw.decode('latin-1')
        parts.append(header_txt.encode('utf-8'))
        for i, e in enumerate(entries):
            name = e['name'].encode('utf-8', errors='replace')
            size_str = ('*' if e.get('is_dir') else str(e['size'])).encode('ascii')
            extras = b'\t'.join(f.encode('ascii', errors='replace')
                                for f in e.get('extra', []))
            prefix = f"U_{i}:".encode('ascii')
            line = prefix + name + b'\t' + size_str
            if extras:
                line += b'\t' + extras
            line += b'\r\n'
            parts.append(line)
    return b''.join(parts)

def cmd_pack(unpacked_dir, meta_json_path, out_dat):
    """
    unpacked_dir: 该 dat 的解包目录 (对应 unpacked/NNNN/)
    meta_json_path: meta/NNNN.json (unpack 时生成)
    out_dat: 输出的 .dat 路径
    """
    with open(meta_json_path, 'r', encoding='utf-8') as f:
        meta = json.load(f)
    entries = meta['entries']
    header_raw = bytes.fromhex(meta['header_raw_hex']) if 'header_raw_hex' in meta else None
    outer_meta = meta.get('meta', {})
    marker = bytes.fromhex(outer_meta.get('marker_hex', MAGIC_PLACEHOLDER.hex()))
    if len(marker) != 16:
        marker = MAGIC_PLACEHOLDER
    trailing = bytes.fromhex(meta.get('trailing_hex', ''))

    # 1) 按 entries 顺序读取文件，重算 size；目录条目只保留 TOC，不参与 blob
    blob_parts = []
    for e in entries:
        if e['is_dir']:
            continue
        inner = os.path.join(unpacked_dir, e['name'].replace('\\', '/'))
        if not os.path.exists(inner):
            raise FileNotFoundError(f"missing: {inner}")
        with open(inner, 'rb') as f:
            data = f.read()
        e['size'] = len(data)
        blob_parts.append(data)
    blob = b''.join(blob_parts) + trailing

    # 2) zlib 压缩
    payload_pt = zlib.compress(blob, level=9)
    payload_pt_padded = _pad_to_block(payload_pt)
    payload_iv = secrets.token_bytes(32)
    payload_ct = cbc_encrypt(GLOBAL_KEY, payload_iv, payload_pt_padded)

    # 3) 重建 TOC 文本 + 加密
    toc_text = _build_toc_text(entries, header_raw)
    toc_pt = _pad_to_block(toc_text + b'\x00' * (32 - (len(toc_text) % 32 or 32)))
    # 上面两步可能多加 32，简化:
    rem = len(toc_text) % 32
    if rem == 0:
        toc_pt = toc_text
    else:
        toc_pt = toc_text + b'\x00' * (32 - rem)
    toc_size = len(toc_pt)
    toc_iv = secrets.token_bytes(32)
    toc_ct = cbc_encrypt(GLOBAL_KEY, toc_iv, toc_pt)

    # 4) 写头部。保留原 marker；其余字段按反汇编固定值写回。
    header = struct.pack('<I', 0x00030006) + marker + \
             struct.pack('<I', 105) + struct.pack('<I', 1) + \
             struct.pack('<I', toc_size)
    assert len(header) == 32

    out = header + toc_iv + toc_ct + payload_iv + payload_ct
    os.makedirs(os.path.dirname(out_dat) or '.', exist_ok=True)
    with open(out_dat, 'wb') as f:
        f.write(out)
    return len(out), len(blob)

def cmd_pack_all(unpacked_root, out_dir, meta_dir):
    metas = sorted(glob.glob(os.path.join(meta_dir, '*.json')))
    print(f'[+] {len(metas)} meta files in {meta_dir}')
    os.makedirs(out_dir, exist_ok=True)
    t0 = time.time()
    total_in = 0
    total_out = 0
    # 同时把 _raw 目录里的非加密文件直通复制回去
    raw_src = os.path.join(unpacked_root, '_raw')
    if os.path.isdir(raw_src):
        for fn in os.listdir(raw_src):
            shutil.copy(os.path.join(raw_src, fn), os.path.join(out_dir, fn))
    for i, m in enumerate(metas):
        name = os.path.splitext(os.path.basename(m))[0]
        src_dir = os.path.join(unpacked_root, name)
        out_dat = os.path.join(out_dir, name + '.dat')
        try:
            out_sz, blob_sz = cmd_pack(src_dir, m, out_dat)
            total_in += blob_sz
            total_out += out_sz
        except Exception as e:
            print(f'  [FAIL] {name}: {e}')
        if (i+1) % 20 == 0 or i == len(metas)-1:
            el = time.time() - t0
            rate = total_in/el/1024/1024 if el > 0 else 0
            print(f'  [{i+1:4d}/{len(metas)}] in={total_in/1024/1024:.1f}MB '
                  f'out={total_out/1024/1024:.1f}MB elapsed={el:.1f}s ({rate:.2f} MB/s)')
    el = time.time() - t0
    print(f'[+] DONE pack-all in {el:.1f}s')


# ========================================================================
# Info / EXE mapping
# ========================================================================

def cmd_info(dat_path, list_entries=False):
    """打印 DAT 外层结构和 TOC 摘要。"""
    with open(dat_path, 'rb') as f:
        d = f.read()
    print(f'file: {dat_path}')
    print(f'size: {len(d)} bytes')
    if not is_attachecase(d):
        print('type: raw / not AttacheCase DAT')
        return
    hdr = read_dat_header(d[:32])
    print('type: AttacheCase v2 stripped DAT')
    print(f"subver: 0x{hdr['subver']:08x}")
    print(f"marker: {hdr['marker_hex']}")
    print(f"task_code: {hdr['task_code']}")
    print(f"data_version: {hdr['data_version']}")
    print(f"toc_size: {hdr['toc_size']} bytes")
    print(f"payload_cipher_size: {len(d) - 64 - hdr['toc_size'] - 32} bytes")
    toc_pt, blob, meta_hdr = decrypt_dat(dat_path)
    header_raw, entries = parse_toc(toc_pt)
    total = sum(e['size'] for e in entries if not e['is_dir'])
    print(f"blob_size: {len(blob)} bytes")
    print(f"entries: {len(entries)} total, {sum(1 for e in entries if not e['is_dir'])} files")
    print(f"file_bytes_sum: {total} bytes")
    print(f"trailing_blob: {len(blob) - total} bytes")
    try:
        print('toc_header: ' + header_raw.decode('shift-jis', errors='replace').replace('\r', '\\r').replace('\n', '\\n'))
    except Exception:
        pass
    if list_entries:
        cur = 0
        for i, e in enumerate(entries):
            off = cur
            if not e['is_dir']:
                cur += e['size']
            kind = 'DIR ' if e['is_dir'] else 'FILE'
            print(f"  [{i:04d}] {kind} off={off:08x} size={e['size']:8d} {e['name']}")


def _pe_rva_to_offset(exe, sections, rva):
    for va, vs, raw, raw_size in sections:
        span = max(vs, raw_size)
        if va <= rva < va + span:
            return raw + (rva - va)
    return None


def cmd_map_exe(exe_path, out_path=None):
    """从 Martopia.exe 的静态资源表导出 dat -> Resource 路径映射。"""
    with open(exe_path, 'rb') as f:
        exe = f.read()
    if len(exe) < 0x100 or exe[:2] != b'MZ':
        raise ValueError(f'{exe_path}: not a PE file')
    pe = struct.unpack('<I', exe[0x3C:0x40])[0]
    if exe[pe:pe+4] != b'PE\0\0':
        raise ValueError(f'{exe_path}: invalid PE signature')
    nsec = struct.unpack('<H', exe[pe+6:pe+8])[0]
    opt_size = struct.unpack('<H', exe[pe+20:pe+22])[0]
    sec0 = pe + 24 + opt_size
    sections = []
    for i in range(nsec):
        s = exe[sec0 + i*40: sec0 + (i+1)*40]
        vs = struct.unpack('<I', s[8:12])[0]
        va = struct.unpack('<I', s[12:16])[0]
        raw_size = struct.unpack('<I', s[16:20])[0]
        raw = struct.unpack('<I', s[20:24])[0]
        sections.append((va, vs, raw, raw_size))
    base = 0x400000
    start_ida, end_ida = 0x5671F4, 0x5689AC
    start = _pe_rva_to_offset(exe, sections, start_ida - base)
    end = _pe_rva_to_offset(exe, sections, end_ida - base)
    if start is None or end is None:
        raise ValueError('mapping table RVA not found in PE sections')
    entries = []
    for off in range(start, end, 8):
        p1, p2 = struct.unpack('<II', exe[off:off+8])
        if p1 == 0 or p2 == 0:
            continue
        o1 = _pe_rva_to_offset(exe, sections, p1 - base)
        o2 = _pe_rva_to_offset(exe, sections, p2 - base)
        if o1 is None or o2 is None:
            continue
        e1 = exe.find(b'\x00', o1)
        e2 = exe.find(b'\x00', o2)
        if e1 < 0 or e2 < 0:
            continue
        s1 = exe[o1:e1].decode('shift_jis', errors='replace')
        s2 = exe[o2:e2].decode('shift_jis', errors='replace')
        entries.append((s1, s2))
    lines = ['# DAT file mapping extracted from Martopia.exe', f'# Total entries: {len(entries)}', '# Format: DAT_FILENAME -> RESOURCE_PATH', '']
    lines += [f'{a} -> {b}' for a, b in entries]
    text = '\n'.join(lines) + '\n'
    if out_path:
        with open(out_path, 'w', encoding='utf-8', newline='\n') as f:
            f.write(text)
        print(f'[+] {len(entries)} mappings -> {out_path}')
    else:
        print(text, end='')
    return entries

# ========================================================================
# Roundtrip self-test
# ========================================================================

def cmd_selftest(dat_path):
    """内存中做一次 unpack → pack → unpack 往返，比对内部文件字节相等"""
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        out1 = os.path.join(tmp, 'u1')
        meta_dir = os.path.join(tmp, 'meta')
        cmd_unpack(dat_path, out1, meta_dir)
        name = os.path.splitext(os.path.basename(dat_path))[0]
        packed = os.path.join(tmp, name + '_rp.dat')
        cmd_pack(out1, os.path.join(meta_dir, name + '.json'), packed)
        out2 = os.path.join(tmp, 'u2')
        cmd_unpack(packed, out2)
        # 比对两次解包的所有文件
        def walk(root):
            res = {}
            for dp, _, fns in os.walk(root):
                for fn in fns:
                    full = os.path.join(dp, fn)
                    rel = os.path.relpath(full, root).replace('\\', '/')
                    res[rel] = open(full, 'rb').read()
            return res
        a, b = walk(out1), walk(out2)
        if set(a.keys()) != set(b.keys()):
            print(f'[FAIL] file set mismatch: only_a={set(a)-set(b)} only_b={set(b)-set(a)}')
            return False
        diffs = [k for k in a if a[k] != b[k]]
        if diffs:
            print(f'[FAIL] content diff in: {diffs[:5]}')
            return False
        print(f'[OK] roundtrip {dat_path}: {len(a)} files match')
        return True

# ========================================================================
# CLI
# ========================================================================

def main():
    ap = argparse.ArgumentParser(description='side-B メルトピア DAT 解包/封包工具 v2')
    sub = ap.add_subparsers(dest='cmd', required=True)

    p_i = sub.add_parser('info', help='显示 DAT 结构和 TOC 摘要')
    p_i.add_argument('dat')
    p_i.add_argument('--list', action='store_true', help='列出 TOC 条目')

    p_m = sub.add_parser('map-exe', help='从 Martopia.exe 导出 dat 映射表')
    p_m.add_argument('exe')
    p_m.add_argument('-o', '--out', help='输出 dat_mapping.txt', default=None)

    p_u = sub.add_parser('unpack', help='解包单个 dat')
    p_u.add_argument('dat')
    p_u.add_argument('out_dir')
    p_u.add_argument('--meta', help='meta 目录（用于后续打包）', default=None)

    p_ua = sub.add_parser('unpack-all', help='批量解包目录下所有 dat')
    p_ua.add_argument('dat_dir')
    p_ua.add_argument('out_dir')
    p_ua.add_argument('--meta', help='meta 输出目录', default=None)

    p_p = sub.add_parser('pack', help='打包单个目录到 dat')
    p_p.add_argument('unpacked_dir')
    p_p.add_argument('meta_json')
    p_p.add_argument('out_dat')

    p_pa = sub.add_parser('pack-all', help='批量打包；参数顺序: unpacked_root out_dir meta_dir')
    p_pa.add_argument('unpacked_root')
    p_pa.add_argument('out_dir')
    p_pa.add_argument('meta_dir')

    p_st = sub.add_parser('selftest', help='对单个 dat 做 unpack → pack → unpack 往返校验')
    p_st.add_argument('dat')

    args = ap.parse_args()
    if args.cmd == 'info':
        cmd_info(args.dat, args.list)
    elif args.cmd == 'map-exe':
        cmd_map_exe(args.exe, args.out)
    elif args.cmd == 'unpack':
        n, sz = cmd_unpack(args.dat, args.out_dir, args.meta)
        print(f'[+] {args.dat}: {n} files, blob={sz} bytes → {args.out_dir}')
    elif args.cmd == 'unpack-all':
        cmd_unpack_all(args.dat_dir, args.out_dir, args.meta)
    elif args.cmd == 'pack':
        out_sz, blob_sz = cmd_pack(args.unpacked_dir, args.meta_json, args.out_dat)
        print(f'[+] {args.out_dat}: blob={blob_sz} bytes → dat={out_sz} bytes')
    elif args.cmd == 'pack-all':
        cmd_pack_all(args.unpacked_root, args.out_dir, args.meta_dir)
    elif args.cmd == 'selftest':
        ok = cmd_selftest(args.dat)
        sys.exit(0 if ok else 1)

if __name__ == '__main__':
    main()
