# ACV1 script.dat archive tool

This Rust tool unpacks, verifies, and rebuilds ACV1 and legacy `script.dat`
archives. It detects the layout automatically.

The game title is required because the archive key is derived from its CP932
bytes. Enter the same title string used by the game executable.

## Build

```powershell
cargo build --release --offline
```

## Usage

```powershell
acv1_dat_tool.exe unpack "<GAME_DIR>\script.dat" --game-title "<GAME_TITLE>"
acv1_dat_tool.exe verify "<GAME_DIR>\script.dat" --game-title "<GAME_TITLE>"
acv1_dat_tool.exe roundtrip "<GAME_DIR>\script.dat" --game-title "<GAME_TITLE>"
acv1_dat_tool.exe pack "<OUTPUT_DIR>\script_unpacked"
```

If `--game-title` is omitted, the program asks for it. Use `--output` to choose
the output path. Dropping an archive starts unpacking; dropping an unpacked
directory starts packing.

Keep `manifest.json` and the `files` directory together. They contain the entry
order and archive data needed for packing. Payloads may grow or shrink. The
compressed archive bytes can differ after packing, but unpacking it should
produce the same member files.

## Limits

This is an archive tool. It does not extract text or create translation JSON.
It supports only the profiled 21-byte ACV1 and legacy indexes with zlib
payloads. Other `script.dat` layouts are rejected.
