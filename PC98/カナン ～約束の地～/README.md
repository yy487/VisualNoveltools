# Canaan localization tools

Rust localization tools for `Canaan - Yakusoku no Chi` / System-98. The implemented
surface covers main-story JSON extraction/injection, `DISK_X.CAT/LIB` unpack/repack,
and structure-aware replacement of files in the original PC-98 HDI. Emulator font
generation and Unicode-to-font mapping are intentionally outside this tool: the
`message` text supplied for injection must already be mapped and encodable as CP932.

## Script text commands

```powershell
extract.exe CANAAN_unpacked
extract.exe CANAAN_unpacked --output story_json
extract.exe DISK_J\cs01_01.s --output cs01_01.s.json

inject.exe CANAAN_unpacked story_json
inject.exe CANAAN_unpacked story_json --output CANAAN_injected
inject.exe DISK_J\cs01_01.s cs01_01.s.json --output cs01_01_injected.s
```

`extract.exe` scans the confirmed `csNN_NN.s` main-story scripts and writes one
UTF-8 JSON file per source script. It includes narration, dialogue, and story choices.
It excludes title/configuration/name-entry scripts, galleries, bonus/credits scripts,
and structurally validated save/load UI embedded in story scripts. No speaker `name`
field is generated: the game has no independent name box, and speaker identity is
conveyed by scene context and graphics rather than a writable text field.

Each JSON entry uses this contract:

```json
{
  "_file": "DISK_J/cs01_06.s",
  "_index": 12,
  "_type": "text",
  "_channel": 9,
  "_encoding": "CP932",
  "_segments": [
    {
      "_inst_offset": 2707,
      "_offset": 6838,
      "_size": 86,
      "_opcode": "0x15",
      "_part_start": 0,
      "_part_end": 5,
      "_page_index": 0,
      "_page_count": 1
    }
  ],
  "scr_msg": "immutable source text",
  "message": "translated text written back"
}
```

`scr_msg` and all underscore-prefixed fields are immutable validation data. Edit only
`message`. Extraction initializes `message == scr_msg`. JSON is UTF-8 without BOM;
script text is strictly encoded as CP932 during injection. Unmappable characters,
NUL/CR characters, malformed markers, changed metadata, missing entries, duplicate
files, old v1 JSON, and mismatched source text are rejected before any output is
written. See `TRANSLATION_GUIDE.zh-CN.md` for the Chinese translation and post-editing
rules.

Version 2 treats every source `0x01` PAGE as a hard JSON boundary: one entry is one
display page, and `[[PAGE]]` is therefore forbidden inside `message`. Consecutive
`0x15` streams proven by the VM CFG to render on the same page are combined into one
translation string. `_segments` records their original instructions and page slices;
injection validates this metadata and splits changed text back across the original
streams without cutting a CP932 character or reserved marker. A `[[WAIT]]` boundary
is an exact split anchor. A source segment boundary immediately before a newline is
also preserved when the translation keeps the source newline count. Other boundaries
are distributed by original display-width proportion, while unchanged JSON always
rebuilds byte exactly.

The reserved structures are:

- JSON `\n`: display newline (`0x0D`). Channel 9 text may add or move newlines;
  channel 8 choice newlines are immutable because they separate menu items.
- `[[PAGE]]`: reserved source page boundary (`0x01`); not valid inside a v2 page
  `message`. Source page controls are represented by separate entries and restored
  automatically.
- `[[WAIT]]`: wait without clearing (`0x02`); must be preserved.
- `[[VAR:NN]]`: runtime string insertion (`0x04 NN`); must be preserved.
- `[[G:XXXX]]`: original two-byte game glyph by JIS code. It may be preserved or
  replaced by already-mapped translation text.
- `[[GRAW:XX]]`: original single-byte game glyph.
- `[[CTRL:HEX]]`: another confirmed control and its arguments; must be preserved.

`[[` is reserved for these markers. Unknown markers are rejected.

System-98 does not automatically wrap text. Width validation uses the renderer's
actual cursor units: ordinary fullwidth characters and game glyphs consume 2 units;
ASCII and halfwidth characters consume 1. Channel 9 permits 70 units per line
(35 ordinary fullwidth characters), while channel 8 permits 60 units (30 fullwidth).
For the few channel 8 source lines that already exceed 60 units, the corresponding
translated line may use the original measured width but may not exceed it; this is a
source-relative exception, not a global increase of the channel 8 limit.
Runtime names are included in the calculation. Over-width text is rejected with the
file, entry, line, measured width, and limit; the tool never truncates or silently
rewrites the translation. Vertical page capacity varies with current scene/window
state, so the tool does not impose an unproven global lines-per-page limit.

Text byte length is not limited to the original slot, and the tool never truncates.
A unique replacement no longer than
its source stream may be written in place. Longer text, or text referenced by multiple
instructions, is appended to the script and only the target `0x15` pointer is changed.
No existing instruction, jump, table, or unknown block is moved. The game allocates a
fixed script buffer, so the rebuilt script must remain at or below the conservative
`0xBC00` byte limit; exceeding it is reported as an error, never truncated.

Directory injection first validates every JSON and prepares every patch in memory,
then copies the complete source tree to a new output directory and replaces only the
translated scripts. Files without translation JSON, manifests, original archive
copies, and unknown resources are retained byte for byte. Source and existing output
paths are never overwritten.

## Script verification

The page-level v2 workflow has been verified against the complete real corpus:

- 476 scripts scanned; 464 main-story JSON files generated.
- 23,615 source main-story streams were projected into 23,885 display-page entries
  and 24,473 hidden page segments. There are 858 internal PAGE boundaries and 588
  CFG-proven same-page stream boundaries; three progressive character-by-character
  effects are kept as complete quoted lines.
- 6,450 reachable save/load UI entries and 150 whole-file system/gallery entries
  were excluded; extraction reported zero warnings.
- All 464 JSON files were UTF-8 without BOM and LF-only, contained no `name` fields,
  and initialized every `message` equal to its immutable `scr_msg`.
- Unchanged injection validated all 23,885 page entries and reported all 23,615 source
  streams unchanged. The copied source and output trees both contained 2,662 files,
  with zero relative-path/SHA-256 differences.
- Real v2 modification covered two pages: one removed the source layout newline inside
  `ヴィアンカ＝Ａ＝イジュ\nウイン`; the other lengthened a `[[WAIT]]` dialogue
  spanning two original `0x15` instructions and added a display newline. Injection
  patched three streams: one in place and two by append/relocation, adding 79 bytes.
  Re-extraction matched all 23,885 requested page messages exactly, and only the two
  intended script files changed in the copied resource tree.
- A 72-unit channel 9 line was rejected against the 70-unit limit before an output
  path was created. No content was truncated.
- Release `extract.exe`/`inject.exe` passed file-mode round trip and SHA-256 equality
  using paths containing spaces, `&`, and Chinese characters. Existing outputs were
  rejected with nonzero exit codes.

## Archive commands

```powershell
unpack.exe DISK_A.CAT
unpack.exe CANAAN --output CANAAN_unpacked
pack.exe DISK_A_unpacked --output DISK_A_packed
pack.exe CANAAN_unpacked --output CANAAN_packed
```

`unpack.exe` accepts CAT files or a directory containing paired CAT/LIB files.
`pack.exe` accepts one unpacked archive directory or a root containing archive
subdirectories. Its output is always a new directory containing the original CAT/LIB
filenames. Outputs must not already exist; source files are never overwritten.

Each unpack directory contains decoded files, `_manifest.json`, `_original.cat`, and
`_original.lib`. The original pair is hash-validated during packing. If no decoded
file changed, packing copies the original CAT/LIB bytes exactly. If files changed,
unchanged stored blobs are reused and only modified compressed entries are encoded
again. Entry order, raw CP932 name bytes, storage type, and trailing bytes are
preserved.

Supported structures:

- `Cat0`: raw 22-byte records.
- `Cat1`: LZSS-compressed 22-byte records.
- `Lib0`: count followed by contiguous stored blobs.
- Entry type `0`: raw payload.
- Entry type `1`: four-byte decoded-size marker plus LZSS stream.

Malformed counts, offsets, sizes, compressed streams, duplicate names, unsafe paths,
edited manifests, missing files, and unexpected files are rejected.

## Archive verification

The archive implementation has been verified against all 12 original pairs:

- 2,650 entries: 2,093 raw and 557 compressed.
- Unpack, unchanged pack: all 24 CAT/LIB SHA-256 hashes matched the originals.
- Re-unpack: 2,686 resource/manifest/original-copy files matched by relative path and SHA-256.
- Modified round trip: one raw entry and one compressed entry were lengthened; all 1,013
  resources in the two test archives matched after pack and re-unpack, while 1,011
  unchanged stored blobs were reused.
- CLI paths containing spaces, `&`, and Chinese characters passed; existing outputs
  were rejected with a nonzero exit code.
- Project formatting, 32 ordinary tests, Clippy with `-D warnings`, all release
  binaries, and every release `--help` entry passed. Real v2 pack/unpack verification
  compared all 2,650 decoded resources with zero differences; only `DISK_K` and
  `DISK_L` changed, while the other ten archive pairs remained byte exact.

The verification artifacts are kept outside this tool directory under the project
verification workspace and are not part of the release payload.

## HDI command

```powershell
build_hdi.exe "Canaan - Yakusoku no Chi.hdi" CANAAN_packed
build_hdi.exe source.hdi replacement_files --output translated.hdi
```

`build_hdi.exe` accepts an original Anex86 HDI and one directory containing direct
replacement files. The default destination inside the image is `CANAAN`; use
`--destination` for another existing 8.3 directory. The default output is
`SOURCE_patched.hdi`. The source image is never overwritten, and an existing output
is rejected.

The updater parses and validates the HDI geometry, PC-98 partition entry, FAT12 BPB,
both FAT copies, every directory and file chain, duplicate entries, cross-links, and
lost clusters before changing anything. It then plans all allocations in memory,
preserves file attributes and timestamps, updates both FAT copies, and writes a new
image only after every replacement has passed validation. Replacement names must be
ASCII 8.3 names and must already exist in the destination directory.

The original image uses a 0x1000-byte HDI header, 256-byte physical sectors,
33 sectors/track, 8 heads, 310 cylinders, and one FAT12 partition. The FAT12 volume
uses 1024-byte logical sectors, 8 sectors/cluster, 2 FAT copies, 1,280 root entries,
2,468 data clusters, and has 452 free clusters (3,702,784 bytes) in the original image.

HDI verification completed so far:

- All 24 original `DISK_A..L.CAT/LIB` files read from the HDI matched the extracted
  `dump\CANAAN` files byte for byte.
- Replacing those 24 files with unchanged repacks produced a 20,955,136-byte HDI with
  the same SHA-256 as the source:
  `215EC12F96E06E1BE8797CB6DE2E3C87603F28188401248D7900538FB3567D4E`.
- A real image file was grown from 19 to 9,000 bytes across clusters in memory; the
  replacement read back exactly and an unrelated FNT file was unchanged.
- Four genuinely modified repacks (`DISK_A/J.CAT/LIB`) were written by the CLI to a
  new HDI and read back exactly through the Rust FAT12 parser.
- The final v2 modification test replaced all 24 `DISK_A..L.CAT/LIB` files in a new
  20,955,136-byte HDI. Four outer files (`DISK_K/L.CAT/LIB`) changed, 20 were
  unchanged, no new cluster was required, and all 24 files read back byte exactly
  through the Rust FAT12 parser.
- Paths containing spaces, `&`, and Chinese characters passed. Existing output was
  rejected with a nonzero exit code.

The completed localized build was also verified end to end on the translated corpus:

- Injection accepted 464 JSON files and 23,885 page entries with zero warnings. It
  changed 464 story scripts; every other file in the 2,686-file packable tree retained
  its SHA-256. Re-extraction matched all 23,885 requested `message` values exactly.
- All 12 rebuilt archives were unpacked again. Their 2,650 decoded resources retained
  the same order and SHA-256 as the injected resource tree.
- The source HDI SHA-256 was
  `490A3968233519EDA363A5D37D71E93EA992217D73890EED653C16E52D8D1D82`.
  The localized build replaced all 24 archive files; six `DISK_J/K/L.CAT/LIB` files
  changed, 18 were byte-identical, and all 24 read back exactly through the FAT12
  parser. The 20,955,136-byte output SHA-256 was
  `FC85E2AE1DB0E95E3421343F995095ACA121E6BA26856343A15FC91EC98AC086`.

Booting the final localized HDI in the target emulator remains an end-to-end release
check; no desktop emulator was controlled during this implementation stage.
