# ORETUBAR TOB/SOB tools

Two Rust command-line tools for the ORETUBAR script formats:

- `oretubar-tob` parses `TOB0` files (`.tob`) and writes translated copies.
- `oretubar-sob` parses `SOB0` files (`.sob`) from the original.

Build with `cargo build --release --bins`. Both tools accept a file or a directory:

```text
oretubar-sob extract --input <SCRIPT_DIR_OR_FILE> --output <JSON_DIR_OR_FILE> --encoding gbk --overwrite
oretubar-sob inject --input <SCRIPT_DIR_OR_FILE> --translation <JSON_DIR_OR_FILE> --output <NEW_DIR_OR_FILE> --encoding gbk --overwrite
oretubar-sob repair --input <CHS_DIR_OR_FILE> --baseline <ROW_DIR_OR_FILE> --output <FIXED_DIR_OR_FILE> --report <REPORT.json> --encoding gbk --overwrite
oretubar-tob extract --input <TOB_DIR_OR_FILE> --output <JSON_DIR_OR_FILE> --encoding sjis --overwrite
oretubar-tob inject --input <TOB_DIR_OR_FILE> --translation <JSON_DIR_OR_FILE> --output <NEW_DIR_OR_FILE> --encoding sjis --overwrite
```

Run either executable without arguments to open its interactive menu. A single
file/directory argument pre-fills the Input prompt; it does not start a write.
The menu returns after extraction, injection, cancellation, or a recoverable
error. Output replacement is confirmed separately before any files are written.

Every operation requires an explicit source encoding: use `sjis` for CP932/SJIS scripts or `gbk` for CHS scripts. There is no automatic encoding guess. Extracted JSON is UTF-8. `scr_msg` is the immutable source body; only `message` is written back. `_file`, `_index`, `_offset`, `_type`, `_encoding`, `_scr_name`, and `_scr_tag` are validation metadata. SOB files use five relocation groups beginning at file offset `0x08`; `_offset` is a payload-relative target from the final internal-pointer group, while `_index` identifies its target field in that group. The metadata payload length is rebuilt whenever records are appended. SOB sentence records normally use `#name\\tag$&body$`; `name` is the text after `#` and before the first backslash, `_scr_name` keeps the original name, and `_scr_tag` keeps internal tags such as `TAK...`, `SIN...`, or `Y...`. CHS files may use `#name&body$` or `#name\\tag＄＆body$`; these separators are preserved. Structural separators and trailing `$` are not included in `message`. Speakerless and empty-name records are supported. No additional text pre-processing is applied yet.

The parsers validate magic values, relocation groups, metadata payload size, record bounds, sentence delimiters, source text, and target encoding. CHS files produced by the existing translation pipeline may use `#name&body$` or `#name\\tag＄＆body$` in addition to the original `#name\\tag$&body$`; both layouts are preserved on unchanged injection. Injection refuses missing/duplicate offsets, changed source text, changed `_scr_name`/`_scr_tag`, NUL bytes, malformed selections, and unrepresentable characters. Resource/control strings such as `PIC\\...` and `ScSkip` are not extracted. A changed sentence inside a `30 00` command rebuilds that whole contiguous command block, including all of its sentence children and opaque suffix bytes.

`repair` takes a CHS tree and the matching ROW tree. It rewrites only the CHS offset table in a new output tree: when a ROW/CHS index pair proves that a source record was relocated, other table entries referring to the same ROW record follow that translated offset. It also corrects stale entries that point two bytes into a confirmed translated record, while ignoring numeric collisions with already-proven translated offsets. It never guesses any other mapping for a ROW offset outside the ROW payload. Unresolved offsets are retained in the output and listed once per file/offset in the UTF-8 report; the report includes all referencing indexes, the current CHS message, and neighboring ROW/CHS context when available.

TOB injection rebuilds translated text at the end of the script payload and updates the chunk offset table. It also relocates the TOB header labels, confirmed `00 02 FF 00` jump operands, and the legacy trailing-reference pattern when their targets move. Unknown bytecode and unrecognized references remain unchanged. SOB injection updates group5 internal targets and the payload length; `repair` additionally reconnects translated ROW/CHS `30 00` composite blocks. Each unresolved repair issue includes up to two neighboring ROW/CHS entries with their offsets and decoded Japanese/Chinese text when available, so the original sentence can be located manually.
