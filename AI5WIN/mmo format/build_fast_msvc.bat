@echo off
setlocal
cd /d "%~dp0"
where cl >nul 2>nul
if errorlevel 1 (
    echo [ERR] cl.exe not found. Please run this from "x64 Native Tools Command Prompt for VS".
    exit /b 1
)
cl /nologo /O2 /LD mmo_fast.c /Fe:mmo_fast.dll
if errorlevel 1 exit /b 1
if exist mmo_fast.obj del mmo_fast.obj
if exist mmo_fast.exp del mmo_fast.exp
if exist mmo_fast.lib del mmo_fast.lib
echo [OK] built mmo_fast.dll
