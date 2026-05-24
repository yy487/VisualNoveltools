@echo off
setlocal
cd /d "%~dp0"
cl /O2 /LD cbm_fast.c /Fe:cbm_fast.dll
if errorlevel 1 (
  echo [build] failed. Run this from a Developer Command Prompt for Visual Studio.
  exit /b 1
)
echo [build] ok: cbm_fast.dll
