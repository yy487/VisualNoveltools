# -*- coding: utf-8 -*-
"""最小自测：构造 YSCM/YSTB，验证 raw 与 push_string 的提取/注入。

运行：
  cd yuris_workflow
  python tests/smoke_test.py
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / "_smoke_tmp"


def make_ysc(path: Path, *, raw: bool) -> None:
    result_type = 3 if raw else 1
    data = bytearray(b"YSCM")
    data += (500).to_bytes(4, "little")
    data += (1).to_bytes(4, "little")
    data += (0).to_bytes(4, "little")
    data += b"WORD\x00"
    data += bytes([1])
    data += b"\x00" + bytes([result_type, 0])
    path.write_bytes(data)


def make_ystb(path: Path, expr_data: bytes) -> None:
    cmd = bytes([0, 1, 0, 0])
    expr = bytes([0, 0, 0, 0]) + len(expr_data).to_bytes(4, "little", signed=True) + (0).to_bytes(4, "little", signed=True)
    line = (1).to_bytes(4, "little", signed=True)
    header = bytearray(b"YSTB")
    header += (500).to_bytes(4, "little")
    header += (1).to_bytes(4, "little")
    header += len(cmd).to_bytes(4, "little")
    header += len(expr).to_bytes(4, "little")
    header += len(expr_data).to_bytes(4, "little")
    header += len(line).to_bytes(4, "little")
    header += b"\0" * 4
    path.write_bytes(header + cmd + expr + expr_data + line)


def run_case(name: str, raw: bool, original: str, translated: str) -> None:
    case = WORK / name
    (case / "ysbin").mkdir(parents=True)
    make_ysc(case / "ysbin" / "ysc.ybn", raw=raw)
    payload = original.encode("cp932")
    expr_data = payload if raw else bytes([0x4D]) + len(payload).to_bytes(2, "little") + payload
    make_ystb(case / "ysbin" / "yst00001.ybn", expr_data)

    subprocess.check_call([
        sys.executable, "-m", "yuris.pipeline", "extract", str(case / "ysbin"), str(case / "json"), "--ysc", str(case / "ysbin" / "ysc.ybn")
    ], cwd=ROOT)
    entries = json.loads((case / "json" / "yuris_text.json").read_text("utf-8"))
    assert entries[0]["scr_msg"] == original
    entries[0]["message"] = translated
    (case / "trans").mkdir()
    (case / "trans" / "yuris_text.json").write_text(json.dumps(entries, ensure_ascii=False), "utf-8")
    subprocess.check_call([
        sys.executable, "-m", "yuris.pipeline", "inject", str(case / "ysbin"), str(case / "trans"), str(case / "out"), "--ysc", str(case / "ysbin" / "ysc.ybn")
    ], cwd=ROOT)
    subprocess.check_call([
        sys.executable, "-m", "yuris.pipeline", "extract", str(case / "out"), str(case / "json2"), "--ysc", str(case / "ysbin" / "ysc.ybn")
    ], cwd=ROOT)
    entries2 = json.loads((case / "json2" / "yuris_text.json").read_text("utf-8"))
    assert entries2[0]["scr_msg"] == translated


def main() -> None:
    if WORK.exists():
        shutil.rmtree(WORK)
    WORK.mkdir()
    run_case("raw", True, "こんにちは", "こんばんは")
    run_case("push", False, "選択肢", "分岐")
    run_args_option_case()
    run_v2_case()
    print("[smoke] OK")


# 追加兼容测试：YU-RIS-Script-Editor 风格 args 扫描选项和 v2。
def make_ystb_option(path: Path, original: str) -> None:
    marker = b'\x4D\x0C\x00\x22ES.SEL.SET\x22'
    payload = original.encode('cp932')
    opt = b'\x4D' + (len(payload) + 2).to_bytes(2, 'little') + b'"' + payload + b'"'
    cmd = bytes([2, 1, 0, 0]) + bytes([3, 1, 0, 0])
    expr1 = b'\x00\x00\x03\x00' + len(marker).to_bytes(4, 'little', signed=True) + (0).to_bytes(4, 'little', signed=True)
    expr2 = b'\x00\x00\x00\x00' + len(opt).to_bytes(4, 'little', signed=True) + len(marker).to_bytes(4, 'little', signed=True)
    data = marker + opt
    line = (1).to_bytes(4, 'little', signed=True) + (2).to_bytes(4, 'little', signed=True)
    header = bytearray(b'YSTB')
    header += (500).to_bytes(4, 'little')
    header += (2).to_bytes(4, 'little')
    header += len(cmd).to_bytes(4, 'little')
    header += (len(expr1) + len(expr2)).to_bytes(4, 'little')
    header += len(data).to_bytes(4, 'little')
    header += len(line).to_bytes(4, 'little')
    header += b'\0' * 4
    path.write_bytes(header + cmd + expr1 + expr2 + data + line)


def make_ystb_v2(path: Path, original: str) -> None:
    payload = original.encode('cp932')
    entry = b'\x00\x00\x00\x00' + len(payload).to_bytes(4, 'little') + (0).to_bytes(4, 'little')
    code = bytes([0x54, 1]) + b'\x00' * 4 + entry
    header = bytearray(b'YSTB')
    header += (250).to_bytes(4, 'little')
    header += len(code).to_bytes(4, 'little')
    header += len(payload).to_bytes(4, 'little')
    header += (0).to_bytes(4, 'little')
    header += b'\0' * (0x20 - len(header))
    path.write_bytes(header + code + payload)


def run_args_option_case() -> None:
    case = WORK / 'option_args'
    (case / 'ysbin').mkdir(parents=True)
    make_ystb_option(case / 'ysbin' / 'yst00002.ybn', '選択A')
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'extract', str(case / 'ysbin'), str(case / 'json'), '--extract-mode', 'args'
    ], cwd=ROOT)
    entries = json.loads((case / 'json' / 'yuris_text.json').read_text('utf-8'))
    assert len(entries) == 1 and entries[0]['_type'] == 'choice' and entries[0]['scr_msg'] == '選択A'
    entries[0]['message'] = '選択B'
    (case / 'trans').mkdir()
    (case / 'trans' / 'yuris_text.json').write_text(json.dumps(entries, ensure_ascii=False), 'utf-8')
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'inject', str(case / 'ysbin'), str(case / 'trans'), str(case / 'out')
    ], cwd=ROOT)
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'extract', str(case / 'out'), str(case / 'json2'), '--extract-mode', 'args'
    ], cwd=ROOT)
    entries2 = json.loads((case / 'json2' / 'yuris_text.json').read_text('utf-8'))
    assert entries2[0]['scr_msg'] == '選択B'


def run_v2_case() -> None:
    case = WORK / 'v2'
    (case / 'ysbin').mkdir(parents=True)
    make_ystb_v2(case / 'ysbin' / 'yst00003.ybn', 'こんにちは')
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'extract', str(case / 'ysbin'), str(case / 'json'), '--extract-mode', 'args'
    ], cwd=ROOT)
    entries = json.loads((case / 'json' / 'yuris_text.json').read_text('utf-8'))
    assert len(entries) == 1 and entries[0]['scr_msg'] == 'こんにちは'
    entries[0]['message'] = 'こんばんは'
    (case / 'trans').mkdir()
    (case / 'trans' / 'yuris_text.json').write_text(json.dumps(entries, ensure_ascii=False), 'utf-8')
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'inject', str(case / 'ysbin'), str(case / 'trans'), str(case / 'out')
    ], cwd=ROOT)
    subprocess.check_call([
        sys.executable, '-m', 'yuris.pipeline', 'extract', str(case / 'out'), str(case / 'json2'), '--extract-mode', 'args'
    ], cwd=ROOT)
    entries2 = json.loads((case / 'json2' / 'yuris_text.json').read_text('utf-8'))
    assert entries2[0]['scr_msg'] == 'こんばんは'


if __name__ == "__main__":
    main()
