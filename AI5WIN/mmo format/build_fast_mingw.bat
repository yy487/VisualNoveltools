@echo off
setlocal
cd /d "%~dp0"
where gcc >nul 2>nul
if errorlevel 1 (
    echo [ERR] gcc not found. Install MinGW-w64 and add it to PATH.
    exit /b 1
)
gcc -O3 -shared -o mmo_fast.dll mmo_fast.c
if errorlevel 1 exit /b 1
echo [OK] built mmo_fast.dll
