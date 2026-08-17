# Studio e.go! V2 tools

These Python tools unpack, translate, and rebuild resources from `月神楽`.

- `tev2_unpack.py` extracts `game*.dat` archives.
- `tev2_decompile.py` exports supported text files to UTF-8 JSON.
- `tev2_compile.py` rebuilds translated text files.
- `Studio_e-go_V2_pack.py` rebuilds game archives.
- `tev2_batch.py` provides the same workflow through one batch command.

Supported text carriers are fixed DAT text tables, `BtText.dat`, and
structured text entries found in `*.scr`.

## Basic workflow

```powershell
python tev2_unpack.py <GAME_DAT> <UNPACK_DIR>
python tev2_decompile.py <UNPACK_DIR>\files <JSON_DIR> --text-encoding cp932 --jobs 0 --skip-errors
python tev2_compile.py <JSON_DIR> <PATCHED_FILES> --source-root <UNPACK_DIR>\files --text-encoding gbk --jobs 0 --skip-errors
python Studio_e-go_V2_pack.py <PATCHED_UNPACK_DIR> <OUTPUT_DAT> --quiet
```

Directory input enables batch decompile and compile automatically. For several
`game*.dat` archives, use:

```powershell
python tev2_batch.py unpack <GAME_DIR> <UNPACK_ROOT>
python tev2_batch.py export-text <FILES_ROOT> <JSON_DIR> --text-encoding cp932 --jobs 0 --skip-errors
python tev2_batch.py import-text <JSON_DIR> <PATCHED_FILES> --source-root <FILES_ROOT> --text-encoding gbk --jobs 0 --skip-errors
python tev2_batch.py pack <PATCHED_UNPACK_ROOT> <OUTPUT_DIR> --quiet --skip-errors
```

Place the rebuilt files back into a copy of the unpacked `files/` tree before
packing.

## Translation JSON

Edit `entries[].text`. Keep `original_text`, offsets, raw data, headers, and
other structural fields unchanged.

Fixed tables reject text that does not fit their record. `BtText.dat` and the
supported SCR text records can be rebuilt with different text lengths.

## Limits

- `Script.dat` is not supported as a complete script archive.
- SCR opcode and operand semantics are not fully disassembled. Only text nodes
  recognized by the structured parser are exported.
- Output encoding must be supported by the target executable and font.
