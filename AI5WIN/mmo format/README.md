# MMO to PNG

`mmo2png.py` converts AI5WIN `.MMO` images to PNG. It accepts files or
directories and scans directories recursively by default.

## Requirements

```powershell
pip install pillow
```

The converter works in pure Python. For faster decoding, build
`mmo_fast.dll` with either `build_fast_msvc.bat` or
`build_fast_mingw.bat`.

## Usage

```powershell
python mmo2png.py <IMAGE.MMO> -o <IMAGE.PNG>
python mmo2png.py <MMO_DIR> -o <PNG_DIR>
python mmo2png.py <MMO_DIR> -o <PNG_DIR> --no-recursive
python mmo2png.py <IMAGE.MMO> --list --fast-info
```

Use `--no-fast` to force the Python decoder or `--no-flip` to keep the
game's bottom-up image orientation.

## Limits

- This tool only decodes MMO files; it cannot rebuild them.
- The C helper accelerates the main RGB stream only. Alpha data is handled by
  the Python path.
- The common 24-bit, no-alpha layout is the tested path.
