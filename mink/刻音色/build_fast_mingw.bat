@echo off
setlocal
cd /d "%~dp0"
gcc -O3 -Wall -Wextra -shared -o cbm_fast.dll cbm_fast.c
if errorlevel 1 (
  echo [build] failed. Make sure MinGW gcc is installed and in PATH.
  exit /b 1
)
echo [build] ok: cbm_fast.dll
