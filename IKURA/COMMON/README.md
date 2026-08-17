# IKURA / ISF tools

These Python tools unpack DRS or MPX packages, extract ISF/SNR text to JSON,
inject translations, and rebuild the package.

## Workflow

```powershell
python unpack.py <PACKAGE> <UNPACKED_DIR> --exe <GAME.EXE>
python extract.py <UNPACKED_DIR> <JSON_DIR> --engine MPX
python inject.py <UNPACKED_DIR> <JSON_DIR> <PATCHED_DIR> --engine MPX
python pack.py <PATCHED_DIR> <OUTPUT_PACKAGE> --order <UNPACKED_DIR>\file_order.json
```

Use `--secret` instead of `--exe` when the XOR secret is already available.
Use `--engine DRS` for older DRS scripts. Keep `file_order.json` for packing.

Edit `message`. `name` is speaker context paired from a separate source slot
and is not a general writable field. Keep `scr_msg` and fields beginning with
`_` unchanged.

## Limits

- Text must be CP932; character mapping and font changes are separate work.
- A standalone visible line matching `【name】` can be classified as a speaker
  slot by the current pairing rule.
- Only the profiled ISF/SNR text opcodes and jump table are rebuilt.
