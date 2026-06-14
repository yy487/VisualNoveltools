#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""
cdt_tool.py  —  18radio 引擎 (.cdt) 封包  解包 / 回装 / 内容解码 工具

================================================================================
封包格式（从 18radio.exe 逆向 + 全部样本逐字节验证确认）
================================================================================

  ┌─ Header ──────────────────────────────────────────────────────┐
  │ [0x00] u16   file_count (N)                                     │
  │ [0x02] u32 × N   index[N]   每项 = 对应文件“记录”的绝对文件偏移 │
  └───────────────────────────────────────────────────────────────┘
        头部大小 = 2 + 4*N ，且 index[0] 恒等于该值

  ┌─ Record（位于 index[i]，长度可变）────────────────────────────┐
  │ [+0x00] u32  data_offset   文件数据的【绝对】偏移              │
  │ [+0x04] u32  orig_size     文件【解压后】字节数（见下）        │
  │ [+0x08] u32  mtime         修改时间 (Unix time_t)              │
  │ [+0x0C] u16  flag          内容编码方式 0/1/2/3（见下）        │
  │ [+0x0E] u16  name_len      文件名字节数（引擎校验 <= 0x3F=63） │
  │ [+0x10] u8 × name_len  name  文件名，逐字节 XOR 0xBD           │
  └───────────────────────────────────────────────────────────────┘

  ┌─ Data ────────────────────────────────────────────────────────┐
  │ 各文件【存储字节】按 index 顺序紧接记录区连续排布；            │
  │ 无对齐、无填充、无间隙。                                       │
  │ 某文件的【存储大小】= 下一文件 data_offset − 本文件 data_offset│
  │ （最后一个文件 = 文件总长 − data_offset）。                    │
  └───────────────────────────────────────────────────────────────┘

  ★ 关键：记录里的 orig_size 是“解压后大小”。
    - flag==1（多为图像，未压缩）：存储大小 == orig_size
    - flag==2（脚本，已压缩）    ：存储大小 <  orig_size
    所以判断越界必须用“存储大小”，绝不能用 orig_size。

内容编码（flag，来自 exe 的读取分发 sub_4536B0）：
  flag=0  原样（不变换）
  flag=1  XOR 0x5AA55AA5（小端按 4 字节循环，即重复密钥 A5 5A A5 5A）
  flag=2  LZSS 解压：存储字节 = [u32 大端 解压后大小][MSB-first 位流]
            位=1 → 读16位窗口绝对偏移 + 8位长度(值+4，范围4..259)，从64KB环形窗拷贝
            位=0 → 读8位字面量
            窗口 65536 字节，写指针自 65277 起回绕（前259字节镜像到+65536处理越界）
  flag=3  另一种解压（sub_453F50，本工具暂未实现；样本中未出现）

本工具：
  · unpack / pack / verify / list 处理“封包层”，按【存储字节】逐字节还原。
  · decode 处理“内容层”，按 flag 还原文件真实内容（解密/解压），供查看/翻译，
    不可直接回封（回封请用 unpack 得到的原始目录）。

================================================================================
用法
================================================================================
  拖放：
    · 把 xxx.cdt 拖到本脚本  → 解包到 xxx_unpacked\（原始存储字节，可回封）
    · 把 解包目录 拖到本脚本 → 按目录内 _cdt_manifest.json 回封

  命令行：
    python cdt_tool.py list   <xxx.cdt>
    python cdt_tool.py unpack <xxx.cdt> [输出目录]        # 原始字节，保证回封
    python cdt_tool.py decode <xxx.cdt> [输出目录]        # 解密/解压后的真实内容
    python cdt_tool.py pack   <解包目录> [输出.cdt]
    python cdt_tool.py verify <xxx.cdt>                   # 内存中 解包→回封 比对 MD5
"""

import sys
import os
import struct
import json
import hashlib
import tempfile

NAME_XOR = 0xBD
CONTENT_XOR = (0xA5, 0x5A, 0xA5, 0x5A)   # flag==1: 0x5AA55AA5 (LE)
MANIFEST = "_cdt_manifest.json"
FLAG_DESC = {0: "stored", 1: "xor", 2: "lzss", 3: "lz_escape"}


# ----------------------------------------------------------------------------
# 解析
# ----------------------------------------------------------------------------
def _decode_name(raw: bytes) -> bytes:
    return bytes(b ^ NAME_XOR for b in raw)


def parse(data: bytes):
    """返回 (count, entries[])。每个 entry 含 stored_size（存储大小，由间隙推得）。"""
    if len(data) < 2:
        raise ValueError("文件过小，不是有效的 cdt 封包")
    count = struct.unpack_from("<H", data, 0)[0]
    idx_end = 2 + count * 4
    if count == 0 or idx_end > len(data):
        raise ValueError("索引区越界，可能不是 cdt 封包（或文件已损坏）")

    index = list(struct.unpack_from("<%dI" % count, data, 2))
    entries = []
    for i, rec_off in enumerate(index):
        if rec_off + 16 > len(data):
            raise ValueError(f"记录 #{i} 偏移 {rec_off} 越界")
        data_off, orig_size, mtime, flag, nlen = struct.unpack_from("<IIIHH", data, rec_off)
        name_enc = data[rec_off + 16 : rec_off + 16 + nlen]
        name = _decode_name(name_enc).decode("cp932", "replace")
        entries.append(
            {
                "index": i,
                "rec_off": rec_off,
                "data_off": data_off,
                "orig_size": orig_size,   # 记录里的字段 = 解压后大小
                "mtime": mtime,
                "flag": flag,
                "name": name,
                "name_enc": name_enc,
                "stored_size": None,      # 稍后由间隙推得
            }
        )

    # 用“到下一文件 data_offset 的间隙”推存储大小（按偏移排序，最后一个到文件尾）
    order = sorted(range(count), key=lambda k: entries[k]["data_off"])
    for j, k in enumerate(order):
        cur = entries[k]["data_off"]
        nxt = entries[order[j + 1]]["data_off"] if j + 1 < count else len(data)
        ss = nxt - cur
        if ss < 0:
            raise ValueError(f"记录 #{entries[k]['index']}（{entries[k]['name']}）存储大小为负，布局异常")
        if cur + ss > len(data):
            raise ValueError(f"记录 #{entries[k]['index']}（{entries[k]['name']}）数据区越界")
        entries[k]["stored_size"] = ss
    return count, entries


# ----------------------------------------------------------------------------
# 内容解码（flag）
# ----------------------------------------------------------------------------
class _BitReader:
    """MSB-first 位读取器。"""
    __slots__ = ("d", "p", "cur", "n")

    def __init__(self, d):
        self.d = d
        self.p = 0
        self.cur = 0
        self.n = 0

    def bit(self):
        if self.n == 0:
            self.cur = self.d[self.p]
            self.p += 1
            self.n = 8
        self.n -= 1
        return (self.cur >> self.n) & 1

    def bits(self, k):
        v = 0
        for _ in range(k):
            v = (v << 1) | self.bit()
        return v


def lzss_decompress(stored: bytes) -> bytes:
    """flag==2：[u32 大端 解压后大小][位流] → 原始字节。"""
    br = _BitReader(stored)
    out_size = br.bits(32)
    W = 0x10000
    win = bytearray(W + 259)
    i = 65277
    out = bytearray()
    remaining = out_size
    while remaining > 0:
        if br.bit():                       # 匹配
            off = br.bits(16)
            length = br.bits(8) + 4
            for k in range(length):
                b = win[off + k]
                out.append(b)
                win[i] = b
                if i < 259:
                    win[i + W] = b
                i += 1
                if i >= W:
                    i = 0
            remaining -= length
        else:                              # 字面量
            b = br.bits(8)
            out.append(b)
            win[i] = b
            if i < 259:
                win[i + W] = b
            i += 1
            if i >= W:
                i = 0
            remaining -= 1
    return bytes(out[:out_size])


def xor_decrypt(stored: bytes) -> bytes:
    """flag==1：按循环密钥 A5 5A A5 5A 整体异或。"""
    out = bytearray(stored)
    for p in range(len(out)):
        out[p] ^= CONTENT_XOR[p & 3]
    return bytes(out)


def lz_escape_decompress(stored: bytes) -> bytes:
    """flag==3：LZ-escape 压缩变体（sub_453F50）。

    存储格式：[u32 LE 解压后大小][1 byte escape标记][压缩流]

    算法：
      - 64KB 环形缓冲，写指针初始 65277（0xFEFD）
      - 前 259 字节镜像到 +0x10000 处理越界
      - 逐字节读取压缩流：
          if byte == escape标记:
              读下一字节 n
              if n < 0xFF:  回引: 长度=n+5, 读2字节LE偏移,
                             从 buf[offset+k] 复制 length 字节
              else (n==0xFF): 输出 escape 标记本身
          else:  字面量字节
    """
    if len(stored) < 5:
        raise ValueError("flag=3 数据不足（至少需要5字节头）")

    decomp_size = struct.unpack_from("<I", stored, 0)[0]
    escape_byte = stored[4]
    src = stored[5:]

    W = 0x10000
    buf = bytearray(W + 259)
    write_pos = 65277  # 0xFEFD
    out = bytearray()

    src_pos = 0
    remaining = decomp_size

    while remaining > 0:
        if src_pos >= len(src):
            break
        b = src[src_pos]
        src_pos += 1

        if b == escape_byte:
            # --- 转义序列 ---
            if src_pos >= len(src):
                break
            n = src[src_pos]
            src_pos += 1

            if n < 0xFF:
                # 回引：长度 = n + 5，偏移 = 下 2 字节 LE
                length = n + 5
                if src_pos + 2 > len(src):
                    break
                offset = struct.unpack_from("<H", src, src_pos)[0]
                src_pos += 2

                for k in range(length):
                    v = buf[offset + k]
                    out.append(v)
                    buf[write_pos] = v
                    if write_pos < 259:
                        buf[write_pos + W] = v
                    write_pos += 1
                    if write_pos >= W:
                        write_pos = 0

                remaining -= length
            else:
                # n == 0xFF：转义标记本身作为字面量输出
                out.append(escape_byte)
                buf[write_pos] = escape_byte
                if write_pos < 259:
                    buf[write_pos + W] = escape_byte
                write_pos += 1
                if write_pos >= W:
                    write_pos = 0
                remaining -= 1
        else:
            # --- 普通字面量 ---
            out.append(b)
            buf[write_pos] = b
            if write_pos < 259:
                buf[write_pos + W] = b
            write_pos += 1
            if write_pos >= W:
                write_pos = 0
            remaining -= 1

    return bytes(out[:decomp_size])


def decode_content(stored: bytes, flag: int):
    """返回 (decoded_bytes, ok, note)。ok=False 表示该 flag 暂未实现，原样返回。"""
    if flag == 0:
        return stored, True, "stored"
    if flag == 1:
        return xor_decrypt(stored), True, "xor"
    if flag == 2:
        return lzss_decompress(stored), True, "lzss"
    if flag == 3:
        return lz_escape_decompress(stored), True, "lz_escape"
    return stored, False, f"flag={flag} 暂未实现，原样输出"


# ----------------------------------------------------------------------------
# list
# ----------------------------------------------------------------------------
def cmd_list(path: str):
    data = open(path, "rb").read()
    count, entries = parse(data)
    print(f"封包文件 : {path}")
    print(f"文件数量 : {count}")
    print(f"总大小   : {len(data)} 字节\n")
    print(f"{'#':>4}  {'数据偏移':>10}  {'存储大小':>10}  {'解压后':>10}  {'flag':<13}  文件名")
    print("-" * 92)
    tot_stored = tot_orig = 0
    for e in entries:
        tot_stored += e["stored_size"]
        tot_orig += e["orig_size"]
        flag_s = f"{e['flag']}:{FLAG_DESC.get(e['flag'], '?')}"
        print(
            f"{e['index']:>4}  {e['data_off']:>10}  {e['stored_size']:>10}  "
            f"{e['orig_size']:>10}  {flag_s:<13}  {e['name']}"
        )
    print("-" * 92)
    print(f"存储合计 : {tot_stored} 字节   解压后合计 : {tot_orig} 字节")


# ----------------------------------------------------------------------------
# unpack（原始存储字节，保证回封）
# ----------------------------------------------------------------------------
def _disk_relpath(name, index, seen):
    rel = name.replace("\\", "/").lstrip("/")
    if not rel:
        rel = f"_noname_{index:04d}"
    key = rel.lower()
    if key in seen:
        base, ext = os.path.splitext(rel)
        rel = f"{base}__dup{index:04d}{ext}"
    seen[key] = True
    return rel


def unpack(path: str, out_dir: str = None, quiet: bool = False):
    data = open(path, "rb").read()
    count, entries = parse(data)
    if out_dir is None:
        out_dir = os.path.splitext(path)[0] + "_unpacked"
    os.makedirs(out_dir, exist_ok=True)

    manifest = {"source": os.path.basename(path), "count": count, "entries": []}
    seen = {}
    for e in entries:
        rel = _disk_relpath(e["name"], e["index"], seen)
        disk = os.path.join(out_dir, *rel.split("/"))
        d = os.path.dirname(disk)
        if d:
            os.makedirs(d, exist_ok=True)
        with open(disk, "wb") as f:
            f.write(data[e["data_off"] : e["data_off"] + e["stored_size"]])
        manifest["entries"].append(
            {
                "path": rel,
                "name_enc_hex": e["name_enc"].hex(),
                "mtime": e["mtime"],
                "flag": e["flag"],
                "orig_size": e["orig_size"],   # 回封时写回 size 字段（解压后大小）
            }
        )
    with open(os.path.join(out_dir, MANIFEST), "w", encoding="utf-8") as f:
        json.dump(manifest, f, ensure_ascii=False, indent=2)

    if not quiet:
        print(f"[完成] 已解包 {count} 个文件（原始存储字节）→ {out_dir}")
        print(f"[提示] 已生成 {MANIFEST}（回封必需，请勿删除/改名）")
        comp = sum(1 for e in entries if e["flag"] == 2)
        if comp:
            print(f"[注意] 其中 {comp} 个文件为压缩存储(flag=2)；要看真实内容请用 decode 子命令")
    return out_dir


# ----------------------------------------------------------------------------
# decode（按 flag 解出真实内容，供查看/翻译，不可回封）
# ----------------------------------------------------------------------------
def cmd_decode(path: str, out_dir: str = None):
    data = open(path, "rb").read()
    count, entries = parse(data)
    if out_dir is None:
        out_dir = os.path.splitext(path)[0] + "_decoded"
    os.makedirs(out_dir, exist_ok=True)
    seen = {}
    n_ok = n_skip = 0
    for e in entries:
        rel = _disk_relpath(e["name"], e["index"], seen)
        stored = data[e["data_off"] : e["data_off"] + e["stored_size"]]
        decoded, ok, note = decode_content(stored, e["flag"])
        if not ok:
            n_skip += 1
            print(f"[跳过] {e['name']}: {note}")
        else:
            n_ok += 1
            if e["flag"] == 2 and len(decoded) != e["orig_size"]:
                print(f"[警告] {e['name']}: 解压后 {len(decoded)} != 记录声明 {e['orig_size']}")
        disk = os.path.join(out_dir, *rel.split("/"))
        d = os.path.dirname(disk)
        if d:
            os.makedirs(d, exist_ok=True)
        with open(disk, "wb") as f:
            f.write(decoded)
    print(f"[完成] 已解码 {n_ok} 个文件 → {out_dir}" + (f"（{n_skip} 个未实现，原样输出）" if n_skip else ""))
    print(f"[提示] decode 输出仅供查看/翻译，不能直接回封；回封请使用 unpack 得到的原始目录")


# ----------------------------------------------------------------------------
# pack（回封，逐字节还原）
# ----------------------------------------------------------------------------
def build(in_dir: str) -> bytes:
    mpath = os.path.join(in_dir, MANIFEST)
    if not os.path.isfile(mpath):
        raise FileNotFoundError(f"找不到 {MANIFEST}，无法回封：{mpath}")
    manifest = json.load(open(mpath, encoding="utf-8"))
    ents = manifest["entries"]
    n = len(ents)

    bodies, names = [], []
    for e in ents:
        disk = os.path.join(in_dir, *e["path"].split("/"))
        with open(disk, "rb") as f:
            bodies.append(f.read())
        names.append(bytes.fromhex(e["name_enc_hex"]))

    rec_start = 2 + 4 * n
    rec_sizes = [16 + len(nm) for nm in names]
    rec_offsets, cur = [], rec_start
    for rs in rec_sizes:
        rec_offsets.append(cur)
        cur += rs
    data_offsets, cur = [], cur
    for body in bodies:
        data_offsets.append(cur)
        cur += len(body)
    total = cur

    buf = bytearray(total)
    struct.pack_into("<H", buf, 0, n)
    for i, ro in enumerate(rec_offsets):
        struct.pack_into("<I", buf, 2 + 4 * i, ro)
    for i in range(n):
        ro = rec_offsets[i]
        # size 字段写“解压后大小”（manifest.orig_size）；缺失时退回存储大小
        orig_size = ents[i].get("orig_size", len(bodies[i]))
        struct.pack_into(
            "<IIIHH", buf, ro,
            data_offsets[i], orig_size, ents[i]["mtime"], ents[i]["flag"], len(names[i]),
        )
        buf[ro + 16 : ro + 16 + len(names[i])] = names[i]
    for i, body in enumerate(bodies):
        buf[data_offsets[i] : data_offsets[i] + len(body)] = body
    return bytes(buf)


def cmd_pack(in_dir: str, out_file: str = None):
    blob = build(in_dir)
    if out_file is None:
        src = json.load(open(os.path.join(in_dir, MANIFEST), encoding="utf-8")).get("source", "rebuilt.cdt")
        parent = os.path.dirname(os.path.abspath(in_dir.rstrip("/\\")))
        out_file = os.path.join(parent, src)
    with open(out_file, "wb") as f:
        f.write(blob)
    print(f"[完成] 已回封 → {out_file}  ({len(blob)} 字节)")


# ----------------------------------------------------------------------------
# replace（把某个内部文件换成新内容，以 flag=0 未压缩存回，自动重算偏移）
# ----------------------------------------------------------------------------
def assemble(records):
    """records: 按 index 顺序的列表，每项 dict(name_enc:bytes, mtime, flag, orig_size, body:bytes)。"""
    n = len(records)
    rec_start = 2 + 4 * n
    rec_sizes = [16 + len(r["name_enc"]) for r in records]
    rec_offsets, cur = [], rec_start
    for rs in rec_sizes:
        rec_offsets.append(cur)
        cur += rs
    data_offsets = []
    for r in records:
        data_offsets.append(cur)
        cur += len(r["body"])
    total = cur

    buf = bytearray(total)
    struct.pack_into("<H", buf, 0, n)
    for i, ro in enumerate(rec_offsets):
        struct.pack_into("<I", buf, 2 + 4 * i, ro)
    for i, r in enumerate(records):
        ro = rec_offsets[i]
        struct.pack_into("<IIIHH", buf, ro,
                         data_offsets[i], r["orig_size"], r["mtime"], r["flag"], len(r["name_enc"]))
        buf[ro + 16: ro + 16 + len(r["name_enc"])] = r["name_enc"]
    for i, r in enumerate(records):
        buf[data_offsets[i]: data_offsets[i] + len(r["body"])] = r["body"]
    return bytes(buf)


def cmd_replace(orig_cdt, inner_name, new_file, out_cdt=None):
    data = open(orig_cdt, "rb").read()
    count, entries = parse(data)
    new_bytes = open(new_file, "rb").read()

    hit = [e for e in entries if e["name"] == inner_name]
    if not hit:
        hit = [e for e in entries if os.path.basename(e["name"]) == os.path.basename(inner_name)]
    if not hit:
        names = ", ".join(e["name"] for e in entries)
        raise ValueError(f"封包内找不到文件 {inner_name!r}；现有：{names}")
    tgt_idx = hit[0]["index"]

    records = []
    for e in entries:
        if e["index"] == tgt_idx:
            records.append({"name_enc": e["name_enc"], "mtime": e["mtime"],
                            "flag": 0, "orig_size": len(new_bytes), "body": new_bytes})
        else:
            records.append({"name_enc": e["name_enc"], "mtime": e["mtime"],
                            "flag": e["flag"], "orig_size": e["orig_size"],
                            "body": data[e["data_off"]: e["data_off"] + e["stored_size"]]})
    blob = assemble(records)
    if out_cdt is None:
        out_cdt = os.path.splitext(orig_cdt)[0] + "_new.cdt"
    with open(out_cdt, "wb") as f:
        f.write(blob)
    print(f"[完成] 已替换 {hit[0]['name']}（以 flag=0 未压缩存入，{len(new_bytes)} 字节）")
    print(f"[完成] 新封包 → {out_cdt}  ({len(blob)} 字节)")
    print(f"[提示] flag=0 未压缩读取需实机确认；若引擎不接受，再补 LZSS 压缩器即可")


# ----------------------------------------------------------------------------
# verify
# ----------------------------------------------------------------------------
def cmd_verify(path: str) -> bool:
    data = open(path, "rb").read()
    with tempfile.TemporaryDirectory() as td:
        unpack(path, td, quiet=True)
        rebuilt = build(td)
    same = rebuilt == data
    print(f"封包文件 : {path}")
    print(f"原始 MD5 : {hashlib.md5(data).hexdigest()}")
    print(f"重建 MD5 : {hashlib.md5(rebuilt).hexdigest()}")
    if same:
        print("[通过] 逐字节一致（bit-perfect round-trip）")
    else:
        m = min(len(data), len(rebuilt))
        for i in range(m):
            if data[i] != rebuilt[i]:
                print(f"[失败] 首个差异位于偏移 {i}: 原 {data[i]:02x} != 重建 {rebuilt[i]:02x}")
                break
        else:
            print(f"[失败] 长度不同: 原 {len(data)} vs 重建 {len(rebuilt)}")
    return same


# ----------------------------------------------------------------------------
# 入口
# ----------------------------------------------------------------------------
USAGE = """\
cdt_tool.py — 18radio (.cdt) 封包工具

命令行：
  python cdt_tool.py list   <xxx.cdt>
  python cdt_tool.py unpack <xxx.cdt> [输出目录]     # 原始存储字节，保证回封
  python cdt_tool.py decode <xxx.cdt> [输出目录]     # 解密/解压后的真实内容（查看用）
  python cdt_tool.py pack   <解包目录> [输出.cdt]
  python cdt_tool.py replace <xxx.cdt> <内部文件名> <新文件> [输出.cdt]   # 以flag=0未压缩换入并重算
  python cdt_tool.py verify <xxx.cdt>

拖放：
  · 把 .cdt 文件拖到本脚本 → 解包（原始字节）到同名 _unpacked 目录
  · 把 解包目录 拖到本脚本 → 按 manifest 回封
"""


def main():
    args = sys.argv[1:]
    if not args:
        print(USAGE)
        return
    cmds = {"list", "unpack", "decode", "pack", "verify", "replace"}
    first = args[0]
    if first in cmds:
        try:
            if first == "list":
                cmd_list(args[1])
            elif first == "unpack":
                unpack(args[1], args[2] if len(args) > 2 else None)
            elif first == "decode":
                cmd_decode(args[1], args[2] if len(args) > 2 else None)
            elif first == "pack":
                cmd_pack(args[1], args[2] if len(args) > 2 else None)
            elif first == "replace":
                cmd_replace(args[1], args[2], args[3], args[4] if len(args) > 4 else None)
            elif first == "verify":
                sys.exit(0 if cmd_verify(args[1]) else 1)
        except IndexError:
            print(USAGE)
        return
    p = first
    if os.path.isdir(p):
        cmd_pack(p)
    elif os.path.isfile(p):
        unpack(p)
    else:
        print(f"无法识别的参数：{p}\n")
        print(USAGE)


if __name__ == "__main__":
    main()
