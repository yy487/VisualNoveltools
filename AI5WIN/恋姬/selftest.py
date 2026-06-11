# -*- coding: utf-8 -*-
from __future__ import annotations
from pathlib import Path
import tempfile
import subprocess
import sys

ROOT = Path(__file__).resolve().parent

def main() -> None:
    mes = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('/mnt/data/mes')
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        js = td / 'json'
        out = td / 'out'
        subprocess.check_call([sys.executable, str(ROOT / 'extract.py'), str(mes), str(js)])
        subprocess.check_call([sys.executable, str(ROOT / 'inject.py'), str(mes), str(js), str(out), '--copy-unmatched'])
    print('[selftest] parse/extract/inject finished')

if __name__ == '__main__':
    main()
