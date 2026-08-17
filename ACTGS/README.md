# ACTGS script tools

These Python tools extract and replace text in ACTGS `arc.scr` archives.

- `scr_crypto.py` finds the archive key from the game executable.
- `scr_extract.py` extracts script text to UTF-8 JSON.
- `scr_inject.py` writes translated JSON back to a new archive.

## Usage

```powershell
python scr_crypto.py <GAME_EXE>
python scr_extract.py <GAME_EXE> <ARC_SCR> <JSON_DIR>
python scr_inject.py <GAME_EXE> <ARC_SCR> <JSON_DIR> <OUTPUT_ARC> cp932
```

Use `gbk` as the last argument only when the target game has been patched to
read GBK text.

## Translation JSON

Edit `message`. `name` may also be edited when the entry has a speaker. Keep
`id` unchanged because the injector uses it to find the script command.

## Limits

- The matching game executable is required to find the archive key.
- The injector rebuilds the archive in a contiguous layout. It does not
  preserve unused gaps or trailing data byte for byte.
- The JSON has no separate immutable source-text field.
- Characters unsupported by the selected encoding may be replaced during
  injection, so check the rebuilt script before using it.
