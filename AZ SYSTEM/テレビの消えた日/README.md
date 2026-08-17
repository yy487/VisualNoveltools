# TVLost ARC and CPB tools

These tools handle `graphic.arc` from `テレビの消えた日`.

- `cpb2png.py` reads encrypted CPB entries directly from `graphic.arc` and
  converts supported images to PNG.
- `tvlost_arc_unpack.py` extracts files from the outer encrypted ARC format.

## Requirements

```powershell
pip install pillow
```

`build_c_accel.bat` builds the optional Windows image accelerator.

## Usage

```powershell
python cpb2png.py <GRAPHIC.ARC> <PNG_DIR> --names bg002b.cpb black.cpb
python cpb2png.py <GRAPHIC.ARC> <PNG_DIR> --all
python tvlost_arc_unpack.py <ARCHIVE.ARC> <OUTPUT_DIR>
```

## Limits

- CPB conversion currently supports `TYP1` 24-bit images. The `--all` command
  skips 8-bit and 32-bit entries.
- `cpb2png.py` must read the original `graphic.arc`; CPB files extracted with
  only the outer ARC key are not valid converter inputs.
- The default keys are specific to this game.
- PNG-to-CPB and ARC rebuilding are not supported.
