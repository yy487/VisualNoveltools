@echo off
REM build_win.bat — Build igf_lzss.dll on Windows
REM
REM Requires one of:
REM   MinGW-w64:  gcc -shared -O3 -o igf_lzss.dll igf_lzss.c
REM   MSVC:       cl /LD /O2 /Fe:igf_lzss.dll igf_lzss.c
REM
REM If neither is available, install MinGW-w64 from https://winlibs.com/
REM or use the Python fallback (slower but correct).

cd /d "%~dp0"

where gcc >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo Building with MinGW gcc...
    gcc -shared -O3 -o igf_lzss.dll igf_lzss.c
    if %ERRORLEVEL% EQU 0 (
        echo   igf_lzss.dll built successfully.
    ) else (
        echo   Build FAILED.
    )
    goto :done
)

where cl >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo Building with MSVC cl...
    cl /LD /O2 /Fe:igf_lzss.dll igf_lzss.c
    if %ERRORLEVEL% EQU 0 (
        echo   igf_lzss.dll built successfully.
    ) else (
        echo   Build FAILED.
    )
    goto :done
)

echo ERROR: No C compiler found (gcc or cl).
echo Install MinGW-w64 or Visual Studio Build Tools.
echo The tool will fall back to Python LZSS (slower).

:done
pause
