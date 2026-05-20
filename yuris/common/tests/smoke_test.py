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
    print("[smoke] OK")


if __name__ == "__main__":
    main()
