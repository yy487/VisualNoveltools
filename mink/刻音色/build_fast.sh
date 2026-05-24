#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
uname_s="$(uname -s)"
if [[ "$uname_s" == "Darwin" ]]; then
  cc -O3 -Wall -Wextra -fPIC -shared -o libcbm_fast.dylib cbm_fast.c
  echo '[build] ok: libcbm_fast.dylib'
else
  cc -O3 -Wall -Wextra -fPIC -shared -o libcbm_fast.so cbm_fast.c
  echo '[build] ok: libcbm_fast.so'
fi
