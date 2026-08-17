# Love Letter .o text tool

This Rust tool extracts and injects scenario messages, effects, and choices
from `Love Letter` VM `.o` files.

## Build

```powershell
cargo build --release --offline
```

## Usage

```powershell
cargo run --release --offline -- extract "<OBJ_DIR>" `
  --output "<OUTPUT_DIR>\obj_json"
cargo run --release --offline -- inject "<OBJ_DIR>" `
  "<OUTPUT_DIR>\obj_json" --output "<OUTPUT_DIR>\obj_injected"
```

The commands also accept one `.o` file and one JSON file. Directory injection
copies all source files and rebuilds only objects with matching JSON. Existing
output is rejected.

## Translation JSON

Edit `message`. Keep `scr_msg` and fields beginning with `_` unchanged. Names
are read-only by default; use `--write-names` only for a reviewed literal name
change, and keep `_scr_name` unchanged.

Runtime variables and controls are written as tokens such as `{{VAR:0083}}`,
`{{VAR_FMT:...}}`, and `{{CTRL:hh}}`. Their order and token type must remain
unchanged. Messages may grow or shrink because the string table is rebuilt.

## Limits

The supported message sinks and bytecode layout are specific to this game.
Structurally different `.o` files are rejected. The tool does not rebuild an
outer archive or patch the executable. Injected text must remain representable
by the game's CP932 text path.
