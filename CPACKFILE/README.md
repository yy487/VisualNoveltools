# Ciel nosurge DX PAK unpacker

`nosurge-pak` lists and extracts the five `PACK*.PAK` archives used by the
Windows release of **Ciel nosurge DX**. It validates the complete archive table,
decrypts filenames and payloads, rejects unsafe output paths, and never modifies
the source archive.

## Use

Double-click `nosurge-pak.exe`, or drag a `.PAK` onto it, to review and confirm
the input and output paths interactively. Listing, successful unpacking,
cancellation, and recoverable errors all return to the main menu.

For a fully specified command:

```text
nosurge-pak list <GAME_DIR>\Res_x64\PACK01.PAK --limit 20
nosurge-pak unpack <GAME_DIR>\Res_x64\PACK01.PAK <OUTPUT_DIR>
```

An existing output is refused unless `--overwrite` is supplied or replacement is
approved interactively. Replacement begins only after extraction to a temporary
sibling directory completes successfully. Do not choose the game directory as
the output.

## Build

Install a current stable Rust toolchain, then run:

```text
cargo build --release
```

The executable is written to `target\release\nosurge-pak.exe`.

## Supported format and limits

- Observed header values `0x00020000`, file count, `0x10`, and flags `0x0D`;
  16-byte header and 168-byte table entries. The executable only gives confirmed
  semantics to the count and flags fields.
- Per-entry 20-byte repeating-XOR keys for the 128-byte filename field and file
  payload.
- Stored paths, byte order, offsets, sizes, and all payload bytes are checked.
- The observed game archives store payloads directly after XOR encryption; this
  tool does not implement repacking or any unobserved compression variant.
- Archive filenames must be UTF-8/ASCII and safely representable on Windows.

