# Outer SB2 Analysis

This report records evidence needed to accept the outer archive stage. Paths are expressed
by role or base name so the document can be moved or published without exposing a machine
environment.

## Sample identity

- File: `sinfonia.sb`
- Size: `3,919,703` bytes (`0x003BCF57`)
- SHA-256: `7a66ac6c15b00399ea31ecb2d88f849f522e3f2cf900e6c3917a6014ea733a45`
- Header values: `[540164691, 0, 0, 1550452, 0, 4, 116, 85, 0, 31910, 1]`

## Parsed coverage

| Index | Tag | Tag offset | Data offset | Payload bytes | End offset |
| ---: | --- | ---: | ---: | ---: | ---: |
| 0 | CODE | `0x0000002C` | `0x00000030` | 1,550,452 | `0x0017A8A4` |
| 1 | TTBL | `0x0017A8A4` | `0x0017A8A8` | 32 | `0x0017A8C8` |
| 2 | FTBL | `0x0017A8C8` | `0x0017A8CC` | 3,770 | `0x0017B786` |
| 3 | FTBL | `0x0017B786` | `0x0017B78A` | 2,905 | `0x0017C2E3` |
| 4 | VTBL | `0x0017C2E3` | `0x0017C2E7` | 0 | `0x0017C2E7` |
| 5 | CSTR | `0x0017C2E7` | `0x0017C2EB` | 1,602,676 | `0x0030375F` |
| 6 | CDBL | `0x0030375F` | `0x00303763` | 8 | `0x0030376B` |
| 7 | DBG_ | `0x0030376B` | `0x0030376F` | 8,744 | `0x00305997` |
| 8 | DBG_ | `0x00305997` | `0x0030599B` | 751,036 | `0x003BCF57` |

The final end offset equals the exact file size. No byte is unassigned and no section
requires padding or alignment.

## Empty VTBL finding

The zero-byte `VTBL.bin` is valid, not an unpacking failure:

- Header field `h[8]` at `0x20` is zero.
- The source bytes at `0x0017C2E3` are the adjacent tags `VTBLCSTR`.
- The engine's VTBL reader stores count zero, skips allocation and record reads, and returns
  success. Non-zero entries would consume three little-endian `u32` fields each.

## Internal consistency

- TTBL has four records with zero members, exactly `4 x 8 = 32` bytes.
- FTBL counts are 116 and 85. Every record is a length-prefixed NUL-terminated name plus
  three `u32` fields, and both tables end at the next tag.
- CSTR has 31,910 `(offset, span)` records. The table is 255,280 bytes; all sample offsets
  equal the cumulative preceding spans; the 1,347,396-byte pool ends exactly at CDBL.
- CDBL has one eight-byte record.
- The first DBG table has 150 length-prefixed records and ends exactly at the second DBG tag.
- The second DBG table has 62,586 twelve-byte records plus its four-byte count, ending at EOF.

## Loader evidence

Read-only IDA analysis of the game's AGSI runtime established these boundaries:

- `sub_100023B0` reads exactly `0x2C` header bytes.
- `sub_10002400` reads and compares each four-byte tag without transformation.
- `sub_10005C10` consumes the nine sections in the order shown above.
- `sub_10002080` implements VTBL as three `u32` fields per record and accepts count zero.
- `sub_100019E0` reads CSTR metadata then exactly `sum(span)` pool bytes.
- `sub_10001B30` nibble-swaps every CSTR pool byte; this is reversible local obfuscation,
  not outer compression.
- The BinFile layer uses sequential `fread`; no outer decompression, decryption, seek-based
  table, or section alignment was found.

These are static loader-code observations. No runtime breakpoint or game modification was
required for the outer archive conclusion.

## Round-trip result

The pre-existing dump was rebuilt without modifications and compared against the source:

- Rebuilt size: `3,919,703` bytes
- Rebuilt SHA-256: `7a66ac6c15b00399ea31ecb2d88f849f522e3f2cf900e6c3917a6014ea733a45`
- Byte equality: `true`

This proves the original outer unpack happened to be correct for this sample. The previous
tool's reported incompatibility is therefore expected to be in later CSTR/CODE text
recognition rather than in the SB2 container split.

## Remaining limits

- The exact names of the two version-like header fields remain unconfirmed.
- TTBL member and VTBL business semantics remain opaque even though their serialized widths
  are confirmed by the loader.
- The loader does not enforce EOF after the second DBG table. The tool preserves any such
  bytes as an opaque tail, but the analyzed sample has none.
- Script VM, CP932 validation, text controls, names, messages, and injection are deferred to
  the next numbered stage.

