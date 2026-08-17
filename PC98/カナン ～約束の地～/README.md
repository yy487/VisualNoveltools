# Canaan localization tools

These Rust tools handle the PC-98 version of `Canaan - Yakusoku no Chi`.

- `unpack` extracts `DISK_X.CAT/LIB` archives.
- `pack` rebuilds those archives.
- `extract` extracts main-story text to UTF-8 JSON.
- `inject` writes translated JSON back to the scripts.
- `build_hdi` replaces rebuilt archives in the original HDI.

## Build

```powershell
cargo build --release --offline --bins
```

## Main workflow

Extract the CAT/LIB archives:

```powershell
unpack.exe "<ARCHIVE_DIR>" --output "<UNPACKED_DIR>"
```

Extract and inject story text:

```powershell
extract.exe "<UNPACKED_DIR>" --output "<JSON_DIR>"
inject.exe "<UNPACKED_DIR>" "<JSON_DIR>" --output "<INJECTED_DIR>"
```

Rebuild the CAT/LIB archives and write them into a new HDI:

```powershell
pack.exe "<INJECTED_DIR>" --output "<PACKED_DIR>"
build_hdi.exe "<ORIGINAL.HDI>" "<PACKED_DIR>" --output "<OUTPUT.HDI>"
```

Each command also accepts a single matching file where applicable. Output paths
must not already exist.

## Translation JSON

```json
{
  "scr_msg": "Original text",
  "message": "Translated text"
}
```

Edit only `message`. `scr_msg` and fields beginning with `_` are source and
layout metadata. This project does not use a `name` field.

The following text markers have structural meaning:

- `\n` is a display newline.
- `[[WAIT]]`, `[[VAR:NN]]`, and `[[CTRL:HEX]]` must be preserved.
- `[[G:XXXX]]` and `[[GRAW:XX]]` represent original game glyphs.
- `[[PAGE]]` is restored between JSON entries and cannot be added to `message`.

`[[` is reserved for supported markers. Unknown or malformed markers are
rejected.

## Limits

- Only confirmed main-story scripts are extracted. Configuration, name-entry,
  gallery, bonus, credits, and save/load UI text are excluded.
- Injected text must already be mapped to valid CP932. Font generation and
  Unicode mapping are not included.
- System-98 does not wrap text automatically. Dialogue lines are limited to 70
  cursor units and choice lines to 60 units, subject to recorded source
  exceptions.
- Rebuilt scripts must not exceed `0xBC00` bytes.
- Archive entries may change size, but entries cannot be added, removed, or
  renamed.
- `build_hdi` replaces existing ASCII 8.3 files in an existing FAT12 directory;
  it does not add new paths.
