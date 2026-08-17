# RxPJADV Python tools

`rxpjadv.py` handles GAMEDAT PAC2 archives and PJADV `textdata`/`scenario`
files.

## Archive usage

```powershell
python rxpjadv.py pack-list <ARCHIVE.DAT>
python rxpjadv.py unpack <ARCHIVE.DAT> <UNPACKED_DIR>
python rxpjadv.py pack <UNPACKED_DIR> <OUTPUT.DAT>
```

Keep the generated manifest when archive order must be preserved.

## Text usage

```powershell
python rxpjadv.py text-export <TEXTDATA.BIN> <SCENARIO.DAT> <TEXT.JSON>
python rxpjadv.py text-import <TEXTDATA.BIN> <SCENARIO.DAT> <TEXT.JSON> `
  --out-textdata <NEW_TEXTDATA.BIN> --out-scenario <NEW_SCENARIO.DAT>
```

Edit `msg`. Keep `scr_msg` and fields beginning with `_` unchanged. `name` is
read-only unless `text-import` is run with `--update-name`.

The injector appends changed strings to `textdata` and updates the matching
scenario offsets. Use `--no-strict` only when skipped validation failures are
acceptable.

Other useful commands:

```powershell
python rxpjadv.py xor <INPUT> <OUTPUT> <KEY>
python rxpjadv.py filename-list <FILENAME.DAT>
python rxpjadv.py textdata-json <TEXTDATA.BIN> <OUTPUT.JSON>
```

## Limits

- Text is CP932 by default.
- Archive names are limited to 31 encoded bytes and are written as ASCII by
  the current packer.
- Only the documented message, choice, chapter, and comment opcodes are
  translated.
