# Shangri-La 1 MES tools

Structure-aware archive, compression, script, and UTF-8 JSON tools for the
supplied 2005 `Shangri-La 1` AI5WIN build. These tools are specific to this
game's verified `MES.ARC` and decompressed MES bytecode; they are not claimed
to support other AI5WIN games.

## Included programs

- `unpack.exe`: extract the compressed payloads from `mes.arc`.
- `decompress.exe`: decompress one payload or a flat unpacked directory.
- `inspect.exe`: strictly parse decompressed scripts and print diagnostics.
- `extract.exe`: create per-script UTF-8 translation JSON.
- `inject.exe`: validate JSON and rebuild decompressed scripts.
- `compress.exe`: create the game's LZSS payload streams.
- `pack.exe`: rebuild `mes.arc` using the original archive as a template.

All writing commands refuse to replace an existing output. Positional paths
support Windows drag and drop. `--output` is available when one explicit output
is needed.

## End-to-end workflow

```powershell
unpack.exe --output 'work\mes_unpacked' 'game\mes.arc'
decompress.exe --output 'work\mes_decompressed' 'work\mes_unpacked'
extract.exe --output 'work\mes_json' 'work\mes_decompressed'

# Edit only each entry's message field in the UTF-8 JSON files.

inject.exe --output 'work\mes_injected' 'work\mes_decompressed' 'work\mes_json'
compress.exe --output 'work\mes_compressed' 'work\mes_injected'
pack.exe --output 'work\mes_patched.arc' 'game\mes.arc' 'work\mes_compressed'
```

Keep the original `mes.arc`. `pack.exe` requires it as a template so the
verified filename set and entry order cannot be guessed or reordered. Install
the result under the filename expected by the game only after keeping a backup
and testing the new archive separately.

Every program provides `--help`. Without `--output`, the defaults are:

```text
mes.arc                 -> mes_unpacked/
A.MES                   -> A_decompressed.MES
scripts/                -> scripts_decompressed/
A.MES                   -> A.MES.json
scripts/                -> scripts_json/
A.MES + A.MES.json      -> A_injected.MES
scripts/ + scripts_json -> scripts_injected/
A.MES                   -> A_compressed.MES
scripts/                -> scripts_compressed/
mes.arc + payloads/     -> mes_packed.arc
```

Directory inputs must be flat and contain only the files for that operation.
Directory injection copies every source script to a new output tree, including
files with no JSON, then replaces only scripts that have translations.

## JSON contract

JSON is UTF-8 without BOM, pretty-printed with LF line endings. One source
script produces one JSON file when it has extractable text. Example:

```json
{
  "_format": "shangri-la1-mes-json-v1",
  "_file": "A.MES",
  "entries": [
    {
      "_file": "A.MES",
      "_index": 0,
      "_offset": 960,
      "_inst_offset": 951,
      "_size": 36,
      "_type": "dialogue",
      "_opcode": 1,
      "_encoding": "CP932",
      "_policy": "relocate",
      "name": "五月",
      "scr_msg": "はじめまして。あなたがトオルさんね。",
      "message": "はじめまして。あなたがトオルさんね。"
    }
  ]
}
```

Only `message` is writable. `scr_msg` is immutable source text and is always
used for validation. All underscore-prefixed fields and `name` are also
immutable. Injection rejects missing, reordered, duplicated, stale, or edited
source metadata instead of performing global text replacement.

The confirmed leading fullwidth `［name］` prefix is structural. Extraction
places its contents in read-only `name`, removes the brackets from `scr_msg`
and `message`, then automatically restores the original name and brackets on
injection. Unnamed text has no `name` field and uses `_type: "unnamed"`.

The current profile extracts all 1,273 named dialogue strings and 64 unnamed
strings from the supplied scripts. The two long battle-message strings in
`BUTAI.MES` and `BUTAI1.MES` are each preserved as one indivisible entry; the
tool does not split them on punctuation. Four standalone `▲`/`▼` navigation
glyph strings in `LIBLARY.LIB` are skipped.

## Encoding and text limits

- Source and rebuilt display strings are CP932 and NUL-terminated.
- Injection rejects NUL, CR, LF, empty bodies, and characters that CP932 cannot
  encode. Unsupported characters are reported instead of replaced.
- Simplified Chinese normally cannot be represented in CP932. A separate,
  runtime-compatible character mapping/font patch would be required before
  this injector could safely emit such text.
- No inline control codes or real line breaks occur in the verified display
  strings. Ideographic spaces are ordinary text and are preserved.
- A rebuilt decompressed script may not exceed the runtime's 64,000-byte
  script buffer.

## Confirmed formats

`MES.ARC` has no magic. It starts with a little-endian `u32` entry count,
followed by `0x28`-byte records:

- 32-byte NUL-terminated ASCII filename field, every byte XOR `0x5F`.
- little-endian stored size XOR `0x46831582`.
- little-endian stored offset XOR `0x17528913`.
- payload bytes stored directly at the decoded range.

The supplied archive has 36 contiguous entries and no alignment padding.
`unpack.exe` and `pack.exe` validate names, Windows case-insensitive
duplicates, path safety, counts, bounds, overlap, and exact template membership.

Each archive payload uses a separate LZSS layer matching `sub_438670`:

- 4,096-byte zeroed ring buffer.
- initial write position `0xFEE`.
- LSB-first flag bits.
- 12-bit absolute ring offset.
- match lengths from 3 through 18 bytes.

A decompressed script begins with a little-endian `u32` offset count followed
by that many little-endian relative code offsets. Code starts at
`4 + offset_count * 4`. Display text is opcode `0x01` plus a NUL-terminated
CP932 string. The parser covers all 25 opcode values used by the supplied
scripts, postfix expressions, parameter lists, inline strings, and opaque
instruction data.

Variable-length injection rebuilds the offset table and every proven code
reference used by opcodes `0x0B`, `0x0C`, `0x10`, `0x14`, and `0x1C`. Across
the real scripts, all 7,162 such table/jump references point to exact
instruction boundaries. The four-byte operands of opcodes `0x17` and `0x1F`
are preserved data values, not treated as code targets.

## Validation performed

- Archive parse/unpack: 36 files, 146,967 stored payload bytes, no gaps or
  trailing bytes; every extracted range matched by SHA-256.
- Decompression: 36 files and 376,036 decompressed bytes.
- Script parse/rebuild: 34,774 instructions, 25 opcode values, 36/36 byte-exact
  results, and 1,341/1,341 CP932 strings with exact decode/encode round trips.
- JSON extract: 27 JSON files, 1,337 entries, four skipped UI glyphs, zero
  warnings.
- Unchanged JSON injection: 1,337 unchanged entries and 36/36 source/output
  SHA-256 matches.
- Archive unchanged pack: rebuilt archive is byte-identical to the supplied
  `mes.arc`.
- LZSS round trip: all 36 recompressed payloads decompress to the exact source
  scripts.
- Modified round trip: real short and long edits, a duplicate-string targeted
  by `_index`, re-extraction, compression, packing, unpacking, and decompression
  all retained the intended edit while all unrelated files stayed unchanged.

The build checks are:

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

## Known limits

- Only the supplied Shangri-La 1 build and its verified flat archive are
  supported.
- No separate Japanese choice-string structure was found; unnamed strings are
  not guessed to be choices.
- The two battle-message aggregates remain indivisible because no structural
  inner delimiter or independent bytecode reference was found.
- The tools rebuild data but do not patch the game executable, font renderer,
  locale behavior, or glyph set.
