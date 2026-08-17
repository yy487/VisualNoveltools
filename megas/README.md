# MyMerryMay MPK, script, and font tool

`mpk_tool` handles the `DataMA`, `DataMB`, and `DataMS` resources used by
`MyMerryMayWithbe`.

- It unpacks and rebuilds MPK archives.
- It extracts and injects `.msb` and `.scx` scripts.
- It redraws the `system_win` bitmap fonts from TTF files.

## Build

```powershell
cargo build --release --offline --bins
```

Run `mpk_tool.exe` without arguments for the interactive workflow. It can
prepare a translation workspace, build translated scripts and fonts, or run
archive and font operations separately.

## Translation workflow

Select a game resource directory containing:

```text
mes00.mpk
script.mpk
system_win.mpk
```

The prepare flow creates a `<name>_translation` workspace. Edit only the JSON
files under `translation_json`; do not edit `.mpk_tool`.

Each translation entry contains only the writable fields:

```json
{
  "name": "校長先生",
  "message": "……君も災難だったね"
}
```

Some entries have no `name`. Do not add, remove, or reorder files or entries.
The build flow merges the translations with the protected source metadata,
asks for fonts, and writes translated resources under `chs`. Pack the three
manifest-bearing output directories to create replacement MPK archives.

## Command line

```text
mpk_tool.exe prepare [--output DIR] PACKAGE_DIR
mpk_tool.exe unpack [--output DIR] ARCHIVE.mpk [ARCHIVE.mpk ...]
mpk_tool.exe pack [--output ARCHIVE.mpk] UNPACKED_DIR
mpk_tool.exe extract [--output DIR] SCRIPT_FILE_OR_DIR
mpk_tool.exe inject [--output PATH] SCRIPT_FILE_OR_DIR JSON_FILE_OR_DIR
mpk_tool.exe font-build [options] FONT_DIR_OR_BIN
mpk_tool.exe verify FILE_OR_DIR
mpk_tool.exe map
```

The older `extract` and `inject` commands use full JSON. Edit `message` and a
present `name`; keep `scr_msg`, `_scr_name`, and fields beginning with `_`
unchanged. `<g:XXXX>` and `<b:XX>` are byte-preserving placeholders and should
not be changed unless their exact meaning is known.

`font-build` accepts a main TTF and optional donor fonts. It redraws the glyph
slots used by translated text while keeping the BIN metrics and indexes.

## Limits

Compressed MPK members are not supported. The font reader expects a supported
single-font TTF; export a face from a complex collection first. Translation
characters must have an available carrier slot and glyph, and carrier-slot
conflicts stop the build. Command-line output paths are not overwritten.
