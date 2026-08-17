# YU-RIS text tools

The `yuris` Python package extracts and injects text from YSTB v5 and v2
scripts. It can use YSCM command definitions, YSTL script names, and YSLB
labels when those files are available.

## Extract

```powershell
python -m yuris.pipeline extract <SCRIPT_DIR> <JSON_DIR> `
  --ysc <YSC.YBN> --yst-list <YST_LIST.YBN> --ysl <YSL.YBN> `
  --key-text <GAME_KEY>
```

Use `--key-hex` for a known four-byte key or omit the key for already decrypted
files. `--extract-mode both` combines command-based extraction with argument
scanning; `word` and `args` select one method.

## Inject

```powershell
python -m yuris.pipeline inject <SCRIPT_DIR> <JSON_DIR> <OUTPUT_DIR> `
  --ysc <YSC.YBN> --key-text <GAME_KEY>
python -m yuris.pipeline check-json <JSON_DIR> --encoding cp932
```

Edit `message`. Keep `scr_msg` and fields beginning with `_` unchanged. `name`
is optional speaker context and is not a universal writable slot.

CP932 is the default output. Use `--target-encoding gbk` only with a matching
runtime and font patch. `--xor-mode flat` is available for scripts made for the
older YURIS_TOOLS workflow; normal files use segmented XOR.

## Limits

- Only the implemented YSTB v5/v2 command and argument layouts are supported.
- Raw candidate scanning is diagnostic and can include non-dialogue strings.
- The tools do not unpack outer game archives or patch fonts and executables.
