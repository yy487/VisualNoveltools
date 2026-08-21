# Rendezvous SCR extractor

`rendezvous-scr` is a Rust extractor and injector for the `SCR:2005` version 5
scripts used by *Rendezvous Melty Selection*. It parses the command section and
XOR-protected string table, exports each source script to its own readable UTF-8
JSON file, and rebuilds scripts after translation.

The tool does not scan arbitrary byte runs. It follows the version 5 instruction
parameters that reference the string table.

## Build

```powershell
cargo build --release
```

The executable is written to `target/release/rendezvous-scr.exe` on Windows.

## Use

Fully specified, non-interactive extraction:

```powershell
rendezvous-scr extract --input "<GAME_DIR>/mes" --output "<OUTPUT_DIR>"
```

Input and output have a one-to-one mapping while preserving relative folders:
`route/00_yuk.scr` becomes `route/00_yuk.json`. Scripts without translatable
entries still produce an empty JSON array. There is no aggregate-output mode.

Add `--overwrite` to replace the complete existing output directory. Existing
output is rejected by default. Add `--raw-text` to disable the default
`fixOrig` normalization.

Injection uses three separate directories:

```powershell
rendezvous-scr inject --source "<GAME_DIR>/mes" --translation "<TRANSLATION_DIR>" --output "<OUTPUT_DIR>"
```

`--source` is the untouched original script tree. `--translation` contains the
matching JSON files. `--output` receives a complete copy of the source tree with
translated scripts rebuilt in place. The source tree is never modified. Add
`--overwrite` only when the complete existing output directory may be replaced.


## Text handling

The default `fixOrig` mode reproduces the source extractor's character table:
legacy half-width kana and punctuation are mapped to the game's full-width or
hiragana display forms. `--raw-text` exports the decoded CP932 text unchanged.

The structural LF at the end of every message slot is removed. Interior LF,
ruby bytes `04`/`05`/`06`, backslash display controls, and unknown text content
are preserved in `scr_msg` and `message`. These in-message controls are editable:
the injector allows translators to delete, replace, or reorder them.
The injector implements that rule: controls inside `message` may be removed,
replaced, or reordered. A trailing backslash without its ASCII target is rejected
as malformed syntax.

