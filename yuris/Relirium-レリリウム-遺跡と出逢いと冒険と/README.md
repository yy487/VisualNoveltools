# YDG to PNG

`ydg2png_batch.py` converts YU-RIS `.ydg` images containing WebP strips to PNG.

## Requirements

```powershell
python -m pip install pillow
```

## Usage

```powershell
python ydg2png_batch.py info <IMAGE.YDG>
python ydg2png_batch.py convert <IMAGE.YDG> <IMAGE.PNG> --overwrite
python ydg2png_batch.py convert <YDG_DIR> <PNG_DIR> -j 8 --overwrite
```

Directory conversion is recursive by default and keeps relative paths. Use
`--no-recursive`, `--suffix`, or `--raw-webp` for the corresponding alternate
outputs.

## Limits

- Supports the `YDG\0` / `YU-RIS` container with embedded `RIFF WEBP` strips.
- Older non-WebP YDG variants are not supported.
- PNG-to-YDG conversion is not implemented.
