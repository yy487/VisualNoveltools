# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def run_module(script_name: str, rest: list[str]) -> int:
    script = Path(__file__).with_name(script_name)
    cmd = [sys.executable, str(script)] + rest
    return subprocess.call(cmd)


def main() -> None:
    parser = argparse.ArgumentParser(description="BGI V1 工作流调度入口")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_extract = sub.add_parser("extract", help="提取为熟悉的 JSON 格式")
    p_extract.add_argument("args", nargs=argparse.REMAINDER)

    p_inject = sub.add_parser("inject", help="从熟悉的 JSON 格式注入")
    p_inject.add_argument("args", nargs=argparse.REMAINDER)

    args = parser.parse_args()
    if args.cmd == "extract":
        raise SystemExit(run_module("extract.py", args.args))
    if args.cmd == "inject":
        raise SystemExit(run_module("inject.py", args.args))


if __name__ == "__main__":
    main()
