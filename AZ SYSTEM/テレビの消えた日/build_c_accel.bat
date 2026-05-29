@echo off
setlocal
where gcc >nul 2>nul
if errorlevel 1 (
  echo [build] gcc not found. Install MinGW-w64 and add gcc to PATH.
  exit /b 1
)
gcc -O3 -shared -o typ1_interleave.dll typ1_interleave.c
if errorlevel 1 exit /b 1
echo [build] typ1_interleave.dll generated.
