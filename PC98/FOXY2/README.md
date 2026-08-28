# foxy2_d88_splitter

Rust tool for Foxy 2 PC-98 D88 resources, AI5 `.MES` translation JSON, coupled character substitution/font redraw, and D88 rebuilding.

Build and verify offline:

```text
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

Run without arguments for the persistent interactive menu. A single path prefills the editable interactive flow; no write occurs before confirmation. Fully specified commands are non-interactive:

```text
foxy2_d88_splitter.exe extract --input <D88_FILE_OR_DIRECTORY> --output <RESOURCE_DIRECTORY> [--overwrite]
foxy2_d88_splitter.exe mes-extract --input <RESOURCE_DIRECTORY> --output <TRANSLATION_DIRECTORY> [--overwrite]
foxy2_d88_splitter.exe mes-inject --input <RESOURCE_DIRECTORY> --translation <TRANSLATION_DIRECTORY> --output <NEW_RESOURCE_DIRECTORY> --font <FONT_TMP> --subs <SUBS_JSON> --font-output <NEW_FONT_TMP> [--font-face <FACE>] [--overwrite]
foxy2_d88_splitter.exe pack --input <RESOURCE_DIRECTORY> --output <D88_DIRECTORY> [--overwrite]
```

## Operations

`extract` parses the ELF-DOS directory at logical offset `0xA0000`. Logical track 80 contains that directory and is skipped when a resource crosses it. Each resource workspace preserves `original.d88`, resource order, offsets, sizes, spans, hashes, and unknown bytes.

`mes-extract` follows each resource manifest and parses `.MES` files as AI5 bytecode with a CP932 pair dictionary. It writes one UTF-8 JSON per MES. Foxy 2 uses dictionary base `0x80`. Files with no AI5 text produce an empty `entries` array.

Editable JSON fields are `name` when present and `message`. Validation fields such as `scr_msg`, `_scr_name`, `_file`, `_index`, `_offset`, `_byte_length`, `_dict_base`, and `_encoding` must remain unchanged. A source prefix such as `［リサ］` becomes writable `name: "リサ"`; unnamed text has no `name` field.

`mes-inject` validates every entry before writing. It accepts LF as the display newline and writes CP932 `81 93`; CR and other control characters are rejected. ASCII digits and punctuation are converted to fullwidth forms. ASCII letters are not converted and are rejected because their bytes are AI5 operators; use explicit fullwidth letters when required.

Characters that CP932 cannot encode, or that CP932 places outside the JIS pages loaded by NP2's `font.tmp`, are looked up in `subs_cn_jp.json` as `Chinese character -> CP932 carrier`. The carrier is written to MES, using or extending that file's AI5 dictionary when necessary, and the carrier's loaded 16x16 slot in `font.tmp` is redrawn as the Chinese character. All translated names and messages share one mapping set; conflicting uses of one carrier are rejected. A carrier is also rejected when it appears literally anywhere in translated text, because redrawing a font slot changes every use of that character.

When a MES dictionary already has all 128 slots, injection may reuse a slot whose original character is no longer needed anywhere in that file's translated text. A slot still needed by the translated text is never overwritten.

Font redraw uses Windows GDI with `新宋体` by default, height 16, weight 400, `DEFAULT_CHARSET`, `NONANTIALIASED_QUALITY`, and `FIXED_PITCH`. The input must be the confirmed 2048x2048, 1bpp NP2 BMP. Its header and all non-target bytes are preserved, and every changed slot is read back pixel-for-pixel.

`pack` writes modified resource payloads into preserved D88 sector data. Replacements within their spans stay in place. If a resource grows, later resources are reflowed through the disk data address space while skipping directory track 80; start track, start offset, primary size, and span fields are rebuilt. D88 and sector headers remain unchanged. Packing fails before output if disk capacity is exceeded.

## Limits

Only the confirmed ELF-DOS directory and Foxy 2 AI5 text are handled. Other resource formats and the FAT12 filesystem on disk D remain opaque. The structural modified round trip is verified; final visual/runtime testing still requires launching the rebuilt D88 with the matching redrawn `font.tmp` in the target emulator.
