# ONI runtime image dumper

`oni_dump.c` builds a DLL that hooks the supported `鬼ノ棲ム桜` executable and
writes decoded runtime images to `_dump\images` as BMP files. It can capture
images embedded in `oni.dat` and resources assembled by the engine at runtime.

## Build

MSVC:

```powershell
cl /LD /O2 oni_dump.c /link user32.lib gdi32.lib kernel32.lib /out:oni_dump.dll
```

MinGW:

```powershell
gcc -shared -o oni_dump.dll oni_dump.c -luser32 -lgdi32 -lkernel32 -O2
```

Load the DLL with a separate launcher, an import-table tool, or a correctly
forwarding proxy DLL. Running the game then creates `_dump\dump_log.txt` and
the captured BMP files.

## Limits

- Hook addresses and global buffers are hard-coded for one executable build.
- The inline hook temporarily restores original bytes while calling the target
  and is not thread-safe.
- Proxy DLL use requires forwarding every export expected from the real system
  DLL; simply renaming this DLL is not sufficient.
- The dumper only exports runtime images. It does not rebuild GR2 or `oni.dat`.
