# YU-NO PC-98 project profile

- Project: `この世の果てで恋を唄う少女 YU-NO` PC-98 edition
- Runtime: Anex86-compatible PC-98 environment; DOS executable `AI5X.EXE` (AI5 family)
- Source sample: `E:\迅雷下载\この世の果てで恋を唄う少女 YU-NO\anex86\work\yuno.hdi`
- Formal tool directory: `H:\vn-tool\AI5WIN\pc98 yuno`
- Analysis and verification directory: `H:\IDA-PRO-MCP\yuno_pc98`
- Game work directory: the source sample's `work` directory
- Write policy: never overwrite the source HDI, files inside `yuno_unpacked`, or the extracted `work\res`; translation JSON and injected resource trees use new sibling directories
- Implementation: Rust command-line tools `unpack_hdi`, `pack_hdi`, `unpack_yuno`, `pack_yuno`, `unpack_mes`, `pack_mes`, `extract_mes`, `inject_mes`, and `verify_mes`

## Runtime executable

- Evidence: `confirmed-structure` from the MZ/EXEPACK headers and `confirmed-runtime` from the unpacked entry's initialization sequence.
- `AI5X.EXE` is a Microsoft EXEPACK executable, not a natively laid-out AI5 binary.
- The packed MZ entry `0B21:0010` enters the EXEPACK loader stub. It decompresses the program backwards in memory, restores relocation data, and transfers with `RETF`.
- The actual program entry is the EXEPACK header's restored MZ entry `0000:016F`, which appears at IDA address `0x1016F` when the module is loaded at `0x10000`.
- Analysis copy: `analysis\AI5X_unpacked.EXE`, SHA-256 `12d5b99db390228ad84afef9e0bcd8fe534ccdf53a841650f317cc644dfa484c`.
- The unpacked executable is 157,440 bytes (512-byte MZ header plus a 156,928-byte load module) and has 74 relocations.
- `analysis\unexepack.py` is a strict read-only analysis unpacker; it validates the EXEPACK signature, stream termination, in-place cursor meeting, relocation bounds, and restored MZ header before producing the analysis copy.

## HDI and filesystem

- Evidence: `confirmed-structure`
- Container: Anex86 HDI with a 0x1000-byte header
- Disk geometry: 512-byte physical sectors, 17 sectors/track, 8 heads, 615 cylinders
- Disk bytes from header: 42,823,680
- PC-98 partition table: offset 0x1200
- Selected partition entry: offset 0x1200, system bytes `A1 91`
- FAT partition: starts at CHS 1/0/0, image offset 0x12000
- Filesystem: FAT16, 1024 bytes/sector, 2 sectors/cluster, 2 FAT copies, 39 sectors/FAT
- Root entries: 3072
- Logical sectors: 39600
- Encoding: CP932/Shift-JIS is used for short-name decoding unless byte evidence fails to decode
- Unknown HDI, boot, partition, slack, and unallocated data: `opaque-preserved`
- The primary FAT traverses the complete filesystem. The backup FAT has 14,063 entries that differ from the primary and is largely zero after its early entries.
- Real filesystem inventory: 740 files, 2 directories, 30,447,766 payload bytes, 0 cross-linked clusters, and 0 orphan clusters.

## Packing policy

- The source HDI is an immutable template and must match the unpack manifest SHA-256.
- Existing regular files may change size. FAT16 chains and all FAT copies are rebuilt for changed files.
- Existing directories, short names, directory order, attributes, and timestamps are preserved.
- Adding, deleting, or renaming files/directories is currently rejected.
- A no-change unpack/pack must be byte-exact.
- Existing FAT-copy differences are preserved. Only entries touched by changed file chains are synchronized across FAT copies.

## AI5 resource archives

- Evidence: `confirmed-structure` from all 17 containers and `confirmed-runtime` from `AI5X.EXE` file lookup/read code.
- Source containers: `YU-NO\YUNO_A` through `YU-NO\YUNO_Q`; `EVE` and `YUNO_ED.*` are explicitly outside this format/task.
- Archive header: little-endian `u16` entry count followed by little-endian `u16` cipher key.
- The verified key is `0x5501` in all 17 containers.
- Directory records start at archive offset 4 and are 20 bytes each: 14-byte NUL-padded CP932/ASCII filename, little-endian `u32` payload-relative offset, and little-endian `u16` size.
- Directory encryption is stateful per byte. Decryption is `ROR8(encoded, key_low) XOR key_high`, incrementing the high-byte XOR value after every byte. Encryption applies the exact inverse.
- Payload starts at `4 + entry_count * 20`. Entry offsets are relative to that payload start, are contiguous and ordered, and the last entry ends exactly at EOF.
- `AI5X.EXE` reads four bytes past the directory into the first payload before lookup. The tool validates that the payload contains those four bytes but does not treat them as directory data.
- Inventory: 2,719 files total; 904 `.MES`, 1,114 `.GP4`, 270 `.A6`, 201 `.S4`, 107 `.M26`, 107 `.M86`, 8 `.DXX`, 6 `.BIN`, 1 `.DAT`, and 1 `.FNT`.
- All 2,719 names are safe flat filenames, all 17 archives have no duplicate Windows names, and the largest source entry is 64,969 bytes.

## AI5 archive packing policy

- Each extracted archive has its own UTF-8 `.yuno_manifest.json` preserving the header, order, raw names, original offsets/sizes, and per-resource SHA-256.
- Existing resources may grow or shrink; every following offset and size is rebuilt.
- Each resource must remain at most 65,535 bytes because the on-disk size field is 16-bit.
- Adding, deleting, or renaming resources and editing the manifest are rejected.
- The packer builds in memory, reparses the rebuilt archive, verifies every resource hash, and refuses to write if unchanged resources do not produce a byte-exact container.

## Script and localization profile

- Outer AI5 archive format: fully parsed and rebuildable as documented above.
- Evidence: `confirmed-runtime` from the unpacked `AI5X.EXE` interpreter and decompressor, plus `confirmed-structure` from all 904 real `.MES` files.
- `.MES` compression: 4 KiB zero dictionary, initial write position 1, MSB-first bitstream; literal is `1 + 8 bits`, match is `0 + 12-bit index + 4-bit length`, zero index terminates, and match length is 2..17.
- The decompressed stream starts with a little-endian `u16` entry offset. Bytes from 2 to that offset are zero to 48 two-byte CP932 dictionary characters.
- Text tokens use fixed dictionary base `0xD0`. Direct two-byte CP932 lead tokens are `0x60..0x7F` and `0xC0..0xCF`, restored by adding `0x20`.
- Runtime command `0x11` anchors displayed text. Structural traversal skips `06...06` strings and `07/08/09` immediates before recognizing commands.
- Runtime control flow advances live `SI` pointers and uses nested `01/00` blocks; no serialized byte jump offsets requiring relocation were found. The decoded stream can therefore be rebuilt for variable-length text.
- User-confirmed name/message rule: in text beginning with `【`, content through `】` is the speaker name and following visible text is the message.
- Static names are writable through `name`, with immutable `_scr_name` source validation.
- The male protagonist name is editable in-game and dynamically expanded by control bytes `13 30 03`. Such entries use `_name_dynamic: true`, preserve `_name_controls`, and omit writable `name` and `_scr_name`.
- `0x13` and `0x16` may separate visible message parts. They are `opaque-preserved` in `_message_controls`; corresponding text uses immutable `scr_msg_parts` and writable `message_parts`.
- Translation interchange is per-script UTF-8 JSON without BOM, format `yuno-pc98-mes-v1`. `scr_msg` and `scr_msg_parts` are immutable.
- Injection is variable-length and rebuilds the decoded script and LZSS stream. The decoded runtime limit and outer AI5 resource size limit are both 65,535 bytes.
- Writable text is limited to dictionary characters or supported two-byte CP932 tokens. NUL, newlines, structural brackets in names, unencodable characters, and unsupported single-byte characters are rejected.

## Verification

- Source SHA-256: `9DBFAA9827A6EB7FE50AF12F8B25E126A292740062549F2995BD3294158A4B04`
- Real no-change unpack/pack is byte-exact; the second unpack has the same 741 host files (including manifest) and hashes.
- Real changed round-trip for `A.TXT` preserves all 739 other disk files.
- All 17 `YUNO_A` ... `YUNO_Q` archives pass `archive -> unpack -> unchanged pack -> unpack`; every rebuilt archive is byte-exact and both extracted trees contain the same 2,736 files including manifests.
- Real archive modification round-trip: `YUNO_A/D_B01.MES` was lengthened from 1,609 to 1,617 bytes; the rebuilt archive grew by 8 bytes and all 250 re-extracted resources matched the modified input.
- All 904 real `.MES` files pass decompression, recompression, second decompression, and structured text extraction: 3,132,939 stored bytes, 6,596,121 decoded bytes, and 66,140 text entries.
- Extraction inventory: 42,927 dynamic-name entries, 979 multipart entries, and 0 warnings.
- Latest release unchanged injection copied the complete resource tree: 2,736 source files and 2,736 output files, with 0 missing, 0 extra, and 0 SHA-256 mismatches.
- Real `01.MES` modification round-trip changed multipart text, a static name, and a longer message. The stored size changed from 4,808 to 4,824 bytes; re-extraction matched all intended fields, and all 92 dynamic-name control payloads remained unchanged.
- `cargo fmt -- --check`, 20 unit tests, `cargo clippy --offline --all-targets -- -D warnings`, offline release build, and `--help` for all 9 release binaries pass.
