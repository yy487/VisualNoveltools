# -*- coding: utf-8 -*-
from __future__ import annotations

import json
import os
import re
import struct
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Sequence

MAGIC = b"SZS100__"
ENTRY_SIZE = 0x110
NAME_SIZE = 0x100
DEFAULT_ARCHIVE_XOR = 0x90
LCG_A = 0x343FD
LCG_C = 0x269EC3

# 已确认样本：
# - 三国姫1 script.szs:       full_lcg_sub, seed 0x7f501e37
# - 三国姫2 script2.szs:      full_lcg_sub, seed 0x3e9f9d19
# - 三国姫3 script.szs:       reseed_lcg_xor, 等价 seed 0x2bddf641（由 archive 反推首个 rand=0x54b9）
# - 三国姫4 script4.szs:      reseed_lcg_xor, seed 0xbffff1d2
# - 三国姫5 script.szs:       full_lcg_sub, seed 0x7f501e37
# - 天極姫1 script*.szs:      reseed_lcg_xor, seed 0x15ec7646
# - 天極姫2 sys.szs:          reseed_lcg_xor, seed 0x13194ff5
KNOWN_SEEDS = (0x3E9F9D19, 0x7F501E37, 0x15EC7646, 0x13194FF5, 0xBFFFF1D2, 0x2BDDF641)
CRYPTO_MODES = ("full_lcg_sub", "reseed_lcg_xor")


@dataclass
class Entry:
    name: str
    offset: int
    size: int

    @property
    def safe_path(self) -> str:
        # 表内常见 main;main.sd；引擎会把 ; 和 / 规格化成反斜杠。
        parts = self.name.replace(";", "/").replace("\\", "/").split("/")
        clean: list[str] = []
        for p in parts:
            if not p or p in (".", ".."):
                continue
            clean.append(p)
        return os.path.join(*clean) if clean else "unnamed.bin"


@dataclass
class CryptoInfo:
    seed: int
    archive_xor: int = DEFAULT_ARCHIVE_XOR
    mode: str = "full_lcg_sub"
    score: float = 0.0
    source: str = "auto"

    def to_json_obj(self) -> dict:
        return {
            "seed_hex": f"0x{self.seed & 0xFFFFFFFF:08x}",
            "seed": self.seed & 0xFFFFFFFF,
            "archive_xor_hex": f"0x{self.archive_xor & 0xFF:02x}",
            "archive_xor": self.archive_xor & 0xFF,
            "mode": self.mode,
            "score": self.score,
            "source": self.source,
        }


def read_table(data: bytes) -> list[Entry]:
    if len(data) < 0x10 or data[:8] != MAGIC:
        raise ValueError("not an SZS100__ archive")
    count = struct.unpack_from("<I", data, 0x0C)[0]
    table_end = 0x10 + count * ENTRY_SIZE
    if table_end > len(data):
        raise ValueError(f"broken table: count={count}, table_end=0x{table_end:x}, file_size=0x{len(data):x}")

    entries: list[Entry] = []
    for i in range(count):
        base = 0x10 + i * ENTRY_SIZE
        raw_name = data[base:base + NAME_SIZE].split(b"\0", 1)[0]
        name = raw_name.decode("cp932", errors="replace")
        offset, size = struct.unpack_from("<QQ", data, base + 0x100)
        if offset < table_end or offset + size > len(data):
            raise ValueError(f"entry out of range #{i}: {name} off=0x{offset:x} size=0x{size:x}")
        entries.append(Entry(name, offset, size))
    return entries


def write_table(entries: list[Entry]) -> bytearray:
    table = bytearray(0x10 + len(entries) * ENTRY_SIZE)
    table[:8] = MAGIC
    struct.pack_into("<I", table, 0x0C, len(entries))
    for i, ent in enumerate(entries):
        base = 0x10 + i * ENTRY_SIZE
        name_b = ent.name.encode("cp932")
        if len(name_b) >= NAME_SIZE:
            raise ValueError(f"entry name too long: {ent.name!r}")
        table[base:base + len(name_b)] = name_b
        struct.pack_into("<QQ", table, base + 0x100, ent.offset, ent.size)
    return table


def _msvc_lcg_rand8_full_state(state: int) -> tuple[int, int]:
    state = (state * LCG_A + LCG_C) & 0xFFFFFFFF
    k = ((state >> 16) & 0x7FFF) & 0xFF
    return state, k


def _sar32(value: int, bits: int) -> int:
    value &= 0xFFFFFFFF
    if value & 0x80000000:
        value -= 0x100000000
    return value >> bits


def _msvc_lcg_rand8_reseed(seed_or_prev_rand: int) -> tuple[int, int]:
    # 天極姫系函数只返回 rand，并把上一次 rand 当下一轮参数：
    #   eax = ((arg * 0x343fd + 0x269ec3) sar 16) & 0x7fff
    #   byte ^= al
    value = (seed_or_prev_rand * LCG_A + LCG_C) & 0xFFFFFFFF
    rnd = _sar32(value, 16) & 0x7FFF
    return rnd, rnd & 0xFF


def crypt_member(
    data: bytes,
    *,
    seed: int,
    archive_xor: int = DEFAULT_ARCHIVE_XOR,
    mode: str = "full_lcg_sub",
    decrypt: bool = True,
) -> bytes:
    """成员加/解密。

    full_lcg_sub：三国姫1/2样本。
      decrypt=True : plain[i]  = ((stored[i] ^ xor) - rand8[i]) & 0xff
      decrypt=False: stored[i] = ((plain[i]  + rand8[i]) & 0xff) ^ xor

    reseed_lcg_xor：天極姫1/2样本。
      decrypt/encrypt 都是: byte = (byte ^ xor) ^ rand8[i]
      其中下一轮 PRNG 参数不是完整 state，而是上一次 rand 返回值。
    """
    seed &= 0xFFFFFFFF
    archive_xor &= 0xFF
    out = bytearray(len(data))

    if mode == "full_lcg_sub":
        x = seed
        if decrypt:
            for i, b in enumerate(data):
                x, k = _msvc_lcg_rand8_full_state(x)
                out[i] = ((b ^ archive_xor) - k) & 0xFF
        else:
            for i, b in enumerate(data):
                x, k = _msvc_lcg_rand8_full_state(x)
                out[i] = ((b + k) & 0xFF) ^ archive_xor
        return bytes(out)

    if mode == "reseed_lcg_xor":
        x = seed
        for i, b in enumerate(data):
            x, k = _msvc_lcg_rand8_reseed(x)
            out[i] = (b ^ archive_xor) ^ k
        return bytes(out)

    raise ValueError(f"unsupported crypto mode: {mode}")


def decrypt_member(data: bytes, *, seed: int, archive_xor: int = DEFAULT_ARCHIVE_XOR, mode: str = "full_lcg_sub") -> bytes:
    return crypt_member(data, seed=seed, archive_xor=archive_xor, mode=mode, decrypt=True)


def encrypt_member(data: bytes, *, seed: int, archive_xor: int = DEFAULT_ARCHIVE_XOR, mode: str = "full_lcg_sub") -> bytes:
    return crypt_member(data, seed=seed, archive_xor=archive_xor, mode=mode, decrypt=False)


def _unique_keep_order(values: Iterable[int]) -> list[int]:
    seen: set[int] = set()
    out: list[int] = []
    for v in values:
        v &= 0xFFFFFFFF
        if v not in seen:
            seen.add(v)
            out.append(v)
    return out


def extract_seed_candidates_from_c(path: Path) -> tuple[list[int], list[int]]:
    """从 Ghidra/反编译 C 中提取 seed 候选。"""
    text = path.read_text(encoding="latin-1", errors="ignore")
    priority: list[int] = []

    # 三国姫系：脚本读取函数常见为 FUN_xxx(file, dst, size, seed)。
    for fn in ("FUN_00481d40", "FUN_00481c60", "FUN_00481ca0", "FUN_0048c6b0", "FUN_0048e3b0"):
        pat = re.compile(re.escape(fn) + r"\([^;\n]*?,[^;\n]*?,[^;\n]*?,\s*(0x[0-9a-fA-F]{1,8}|\d+)\)")
        for m in pat.finditer(text):
            priority.append(int(m.group(1), 0) & 0xFFFFFFFF)

    # 更宽松：所有 6~8 位 hex 常量靠 SZS 试解密评分筛选。
    general = [int(m.group(1), 16) & 0xFFFFFFFF for m in re.finditer(r"\b0x([0-9a-fA-F]{6,8})\b", text)]
    return _unique_keep_order(priority), _unique_keep_order(general)


def extract_archive_xor_candidates_from_c(path: Path) -> list[int]:
    text = path.read_text(encoding="latin-1", errors="ignore")
    out: list[int] = []
    for fn in ("FUN_00406480", "FUN_00405df0"):
        pat = re.compile(re.escape(fn) + r"\([^;\n]*?,\s*(0x[0-9a-fA-F]{1,2}|\d{1,3})\)")
        for m in pat.finditer(text):
            v = int(m.group(1), 0)
            if 0 <= v <= 0xFF:
                out.append(v)
    out.append(DEFAULT_ARCHIVE_XOR)
    return _unique_keep_order(out)


def extract_seed_candidates_from_exe(path: Path) -> list[int]:
    """从 PE 二进制中提取 imm32 候选。

    同系 seed 一般在读取多个成员时重复出现。这里保留高频 push/mov imm32，
    再靠两种 crypto mode 的试解密评分决定最终 seed 和模式。
    """
    data = path.read_bytes()
    values: list[int] = []
    n = len(data)
    i = 0
    while i + 5 <= n:
        b = data[i]
        if b == 0x68 or 0xB8 <= b <= 0xBF:  # push imm32 / mov reg, imm32
            values.append(struct.unpack_from("<I", data, i + 1)[0])
            i += 5
            continue
        if i + 8 <= n and b == 0xC7 and data[i + 1] == 0x44 and data[i + 2] == 0x24:
            values.append(struct.unpack_from("<I", data, i + 4)[0])
            i += 8
            continue
        if i + 7 <= n and b == 0xC7 and data[i + 1] == 0x45:
            values.append(struct.unpack_from("<I", data, i + 3)[0])
            i += 7
            continue
        i += 1

    values = [v for v in values if v not in (0, 1, 2, 3, 4, 0xFFFFFFFF)]
    cnt = Counter(values)
    frequent = [(v, c) for v, c in cnt.items() if c >= 2]
    frequent.sort(key=lambda x: (-x[1], x[0]))
    return [v for v, _ in frequent[:4000]]


def find_adjacent_decompiler_file(exe_path: Path | None) -> Path | None:
    if exe_path is None:
        return None
    candidates = [
        exe_path.with_suffix(exe_path.suffix + ".c"),
        exe_path.with_suffix(".exe.c"),
        exe_path.with_name(exe_path.name + ".c"),
    ]
    for c in candidates:
        if c.is_file():
            return c
    return None


def _probe_entries(entries: Sequence[Entry]) -> list[Entry]:
    preferred_ext = (".sfn", ".swn", ".sbn", ".ev", ".lbn", ".lb", ".sw", ".sb")
    preferred = [e for e in entries if e.name.lower().endswith(preferred_ext) and e.size > 0]
    # sys.szs 这类都是 .bin/.dat，直接取前若干个成员即可。
    return (preferred or [e for e in entries if e.size > 0])[:8]


def _score_plain_sample(plain: bytes, name: str) -> float:
    if not plain:
        return -999.0
    n = len(plain)
    zero_ratio = plain.count(0) / n
    ascii_zero_ratio = sum((32 <= b < 127) or b in (0, 9, 10, 13) for b in plain) / n
    bad_ctrl_ratio = sum((b < 32 and b not in (0, 9, 10, 13)) for b in plain) / n
    high_entropy_penalty = sum(b >= 0xF0 for b in plain) / n

    # cp932 假名/汉字不是 ASCII，因此不能简单惩罚高位字节；只惩罚明显随机的控制字节。
    score = ascii_zero_ratio + 2.8 * zero_ratio - 1.8 * bad_ctrl_ratio - 0.25 * high_entropy_penalty

    lower_name = name.lower()
    if lower_name.endswith((".sfn", ".swn", ".sbn", ".ev", ".lbn")):
        score *= 2.0
    elif lower_name.endswith((".lb", ".sw", ".sb", ".tko")):
        score *= 1.2

    # 常见明文特征：NUL 填充 ASCII 表、标签、文件路径、脚本名。
    tokens = (
        b"main.txt", b"start", b"mode", b".txt", b"main",
        b"Data\\", b"data\\", b".bin", b".dat", b".wma",
        b"prologue", b"shake", b"bg\\", b"menu\\",
    )
    for token in tokens:
        if token in plain:
            score += 2.0
    if re.match(rb"^[A-Za-z0-9_.\\/ -]{3,}\x00", plain):
        score += 2.0
    if plain[:64].count(0) >= 24:
        score += 0.6

    # 很多有效表开头是小整数/偏移表，前 4 字节小端值通常不会离谱。
    if n >= 4:
        first_u32 = struct.unpack_from("<I", plain, 0)[0]
        if first_u32 < 0x02000000:
            score += 0.2
    return score


def score_crypto(
    data: bytes,
    entries: Sequence[Entry],
    *,
    seed: int,
    archive_xor: int,
    mode: str,
    sample_size: int = 512,
) -> float:
    total = 0.0
    probes = _probe_entries(entries)
    for ent in probes:
        stored = data[ent.offset:ent.offset + min(ent.size, sample_size)]
        plain = decrypt_member(stored, seed=seed, archive_xor=archive_xor, mode=mode)
        total += _score_plain_sample(plain, ent.name)
    return total / max(len(probes), 1)


def detect_crypto(
    archive_path: Path,
    *,
    exe_path: Path | None = None,
    decompiler_c_path: Path | None = None,
    archive_xor: int | None = None,
    show_top: int = 10,
) -> tuple[CryptoInfo, list[CryptoInfo]]:
    data = archive_path.read_bytes()
    entries = read_table(data)

    priority: list[int] = []
    general: list[int] = []
    sources: list[str] = []

    c_path = decompiler_c_path or find_adjacent_decompiler_file(exe_path)
    xor_candidates: list[int] = []
    if c_path and c_path.is_file():
        p, g = extract_seed_candidates_from_c(c_path)
        priority.extend(p)
        general.extend(g)
        xor_candidates.extend(extract_archive_xor_candidates_from_c(c_path))
        sources.append(str(c_path))

    if exe_path and exe_path.is_file():
        general.extend(extract_seed_candidates_from_exe(exe_path))
        sources.append(str(exe_path))

    general.extend(KNOWN_SEEDS)
    candidates = _unique_keep_order(priority + general)
    if not candidates:
        raise ValueError("no seed candidates found from exe / decompiler C")

    if archive_xor is not None:
        xor_candidates = [archive_xor & 0xFF]
    else:
        xor_candidates = _unique_keep_order(xor_candidates + [DEFAULT_ARCHIVE_XOR])

    def rank(seed_list: Sequence[int], sample_size: int) -> list[CryptoInfo]:
        out: list[CryptoInfo] = []
        for seed in seed_list:
            for ax in xor_candidates:
                for mode in CRYPTO_MODES:
                    sc = score_crypto(data, entries, seed=seed, archive_xor=ax, mode=mode, sample_size=sample_size)
                    out.append(CryptoInfo(seed=seed, archive_xor=ax, mode=mode, score=sc, source=", ".join(sources) or "built-in candidates"))
        out.sort(key=lambda x: x.score, reverse=True)
        return out

    ranked = rank(_unique_keep_order(priority), 512) if priority else []
    if not ranked or ranked[0].score < 2.0:
        ranked = rank(candidates, 384)

    # 对前若干用更大样本复评，避免短样本偶然高分。
    seen: set[tuple[int, int, str]] = set()
    top: list[tuple[int, int, str]] = []
    for r in ranked[:80]:
        pair = (r.seed & 0xFFFFFFFF, r.archive_xor & 0xFF, r.mode)
        if pair not in seen:
            seen.add(pair)
            top.append(pair)

    refined: list[CryptoInfo] = []
    for seed, ax, mode in top:
        sc = score_crypto(data, entries, seed=seed, archive_xor=ax, mode=mode, sample_size=4096)
        refined.append(CryptoInfo(seed=seed, archive_xor=ax, mode=mode, score=sc, source=", ".join(sources) or "built-in candidates"))
    refined.sort(key=lambda x: x.score, reverse=True)

    # 如果 EXE/known seed 候选得分过低，通常表示 EXE 被壳保护或 seed 不以明文 imm32 出现。
    # 对 reseed_lcg_xor 可以直接从 archive 特征反推首个 15-bit rand，再构造等价 seed。
    if (not refined) or refined[0].score < 2.0:
        brute: list[CryptoInfo] = []
        for ax in xor_candidates:
            brute.extend(detect_reseed_by_archive_bruteforce(data, entries, archive_xor=ax, show_top=show_top))
        brute.sort(key=lambda x: x.score, reverse=True)
        if brute and ((not refined) or brute[0].score > refined[0].score):
            refined = brute

    if not refined:
        raise ValueError("failed to score seed candidates")
    return refined[0], refined[:show_top]



def _canonical_seed_for_reseed_first_rand(first_rand: int) -> int:
    """为 reseed_lcg_xor 构造一个等价 seed。

    reseed_lcg_xor 的流只依赖第一轮 rand 结果；若 EXE 被保护壳处理、
    原 seed 静态不可见，可以从 archive 已知明文特征反推 first_rand，
    再构造任意一个满足 f(seed)==first_rand 的 seed。这样 pack/unpack 的
    字节流与游戏实际解密流完全一致。
    """
    first_rand &= 0x7FFF
    # 令 (seed * A + C) 的高 16 位等于 first_rand，低 16 位取 0。
    inv_a = pow(LCG_A, -1, 1 << 32)
    value = (first_rand << 16) & 0xFFFFFFFF
    return ((value - LCG_C) * inv_a) & 0xFFFFFFFF


def _quick_reseed_stream(first_rand: int, n: int) -> bytes:
    out = bytearray(n)
    x = first_rand & 0x7FFF
    for i in range(n):
        out[i] = x & 0xFF
        value = (x * LCG_A + LCG_C) & 0xFFFFFFFF
        x = _sar32(value, 16) & 0x7FFF
    return bytes(out)


def _quick_plain_score(plain: bytes, name: str) -> float:
    if not plain:
        return -999.0
    n = len(plain)
    zero = plain.count(0)
    printable = sum((32 <= b < 127) or b in (0, 9, 10, 13) for b in plain)
    bad_ctrl = sum((b < 32 and b not in (0, 9, 10, 13)) for b in plain)
    high_f0 = sum(b >= 0xF0 for b in plain)
    score = printable / n + 2.8 * zero / n - 1.8 * bad_ctrl / n - 0.25 * high_f0 / n
    lower = name.lower()
    if lower.endswith((".sfn", ".swn", ".sbn", ".ev", ".lbn")):
        score *= 2.0
    elif lower.endswith((".lb", ".sw", ".sb", ".tko")):
        score *= 1.2
    for token in (b"main.txt", b"start", b"mode", b".txt", b"main", b"Data\\", b"data\\", b"shake", b"bg\\"):
        if token in plain:
            score += 2.0
    if re.match(rb"^[A-Za-z0-9_.\\/ -]{3,}\x00", plain):
        score += 2.0
    if n >= 4 and struct.unpack_from("<I", plain, 0)[0] < 0x02000000:
        score += 0.2
    return score


def detect_reseed_by_archive_bruteforce(
    data: bytes,
    entries: Sequence[Entry],
    *,
    archive_xor: int,
    show_top: int = 10,
) -> list[CryptoInfo]:
    """不依赖 EXE 常量，从 archive 特征反推 reseed_lcg_xor 的首个 rand。

    适用于 EXE 被 protect.dll/壳处理、imm32 候选不可用的样本，例如三国姫3。
    搜索空间只有 0x8000，因为 reseed 模式每轮 state 被截断到 15 bit。
    """
    probes = _probe_entries(entries)
    sample_size = 256
    samples = [(e, data[e.offset:e.offset + min(e.size, sample_size)]) for e in probes]
    best: list[tuple[float, int]] = []

    for first_rand in range(0x8000):
        stream = _quick_reseed_stream(first_rand, sample_size)
        total = 0.0
        for ent, stored in samples:
            plain = bytes(((b ^ archive_xor) ^ stream[i]) for i, b in enumerate(stored))
            total += _quick_plain_score(plain, ent.name)
        score = total / max(len(samples), 1)
        if len(best) < max(show_top * 4, 20) or score > best[-1][0]:
            best.append((score, first_rand))
            best.sort(reverse=True)
            del best[max(show_top * 4, 20):]

    refined: list[CryptoInfo] = []
    for _, first_rand in best:
        seed = _canonical_seed_for_reseed_first_rand(first_rand)
        score = score_crypto(data, entries, seed=seed, archive_xor=archive_xor, mode="reseed_lcg_xor", sample_size=4096)
        refined.append(CryptoInfo(seed=seed, archive_xor=archive_xor, mode="reseed_lcg_xor", score=score, source=f"archive-bruteforce first_rand=0x{first_rand:04x}"))
    refined.sort(key=lambda x: x.score, reverse=True)
    return refined[:show_top]


def manifest_obj(entries: list[Entry], crypto: CryptoInfo) -> dict:
    return {
        "_format": "SZS100__",
        "_szs_crypto": crypto.to_json_obj(),
        "entries": [{**asdict(e), "safe_path": e.safe_path} for e in entries],
    }


def load_manifest(path: Path) -> tuple[list[dict], dict | None]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, dict):
        return data.get("entries", []), data.get("_szs_crypto")
    if isinstance(data, list):
        return data, None
    raise ValueError(f"unsupported manifest format: {path}")


def unpack_archive(archive_path: Path, out_dir: Path, *, crypto: CryptoInfo) -> list[Entry]:
    data = archive_path.read_bytes()
    entries = read_table(data)
    out_dir.mkdir(parents=True, exist_ok=True)
    for ent in entries:
        stored = data[ent.offset:ent.offset + ent.size]
        plain = decrypt_member(stored, seed=crypto.seed, archive_xor=crypto.archive_xor, mode=crypto.mode)
        out_path = out_dir / ent.safe_path
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(plain)
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest_obj(entries, crypto), ensure_ascii=False, indent=2),
        encoding="utf-8",
        newline="\n",
    )
    return entries


def pack_archive(in_dir: Path, out_archive: Path, *, crypto: CryptoInfo | None = None) -> list[Entry]:
    manifest_path = in_dir / "manifest.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(f"missing manifest: {manifest_path}")
    items, manifest_crypto = load_manifest(manifest_path)

    if crypto is None:
        if not manifest_crypto:
            raise ValueError("missing crypto info; pass --seed/--xor/--mode or use manifest generated by this tool")
        seed = int(str(manifest_crypto.get("seed_hex") or manifest_crypto.get("seed")), 0)
        ax = int(str(manifest_crypto.get("archive_xor_hex") or manifest_crypto.get("archive_xor", DEFAULT_ARCHIVE_XOR)), 0)
        mode = str(manifest_crypto.get("mode") or "full_lcg_sub")
        crypto = CryptoInfo(seed=seed, archive_xor=ax, mode=mode, source="manifest")

    entries: list[Entry] = []
    blobs: list[bytes] = []
    offset = 0x10 + len(items) * ENTRY_SIZE
    for item in items:
        name = item["name"]
        safe_path = item.get("safe_path") or Entry(name, 0, 0).safe_path
        plain = (in_dir / safe_path).read_bytes()
        stored = encrypt_member(plain, seed=crypto.seed, archive_xor=crypto.archive_xor, mode=crypto.mode)
        entries.append(Entry(name=name, offset=offset, size=len(stored)))
        blobs.append(stored)
        offset += len(stored)

    table = write_table(entries)
    out_archive.parent.mkdir(parents=True, exist_ok=True)
    with out_archive.open("wb") as f:
        f.write(table)
        for blob in blobs:
            f.write(blob)
    return entries
