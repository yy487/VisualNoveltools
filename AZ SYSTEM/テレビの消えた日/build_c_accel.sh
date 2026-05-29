#!/usr/bin/env bash
set -euo pipefail
gcc -O3 -shared -fPIC -o typ1_interleave.so typ1_interleave.c
echo '[build] typ1_interleave.so generated.'
