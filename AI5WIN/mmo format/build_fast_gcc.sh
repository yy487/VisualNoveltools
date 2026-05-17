#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
gcc -O3 -fPIC -shared -o mmo_fast.so mmo_fast.c
echo "[OK] built mmo_fast.so"
