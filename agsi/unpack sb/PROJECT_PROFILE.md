# Project Profile: AGSI SB2 Outer Archive

Evidence status in this profile uses `confirmed-structure`, `confirmed-runtime`,
`hypothesis`, and `opaque-preserved`.

## Scope and ownership

- Project: `天空のシンフォニア` outer script archive.
- Runtime: AGSI (`agsi.dll`).
- Analysis sample role: original `sinfonia.sb` archive.
- Development role: numbered `01_sb_unpack` work stage.
- Existing archive and existing unpack directories are read-only inputs.
- New outputs are written below the stage's `output` or `verification` directories.

## Archive profile

- Extension: `.sb`.
- Magic: ASCII `SB2 ` at offset `0x00` (`confirmed-structure`, `confirmed-runtime`).
- Header: 44 bytes, 11 little-endian `u32` values (`confirmed-structure`).
- Section order: `CODE/TTBL/FTBL/FTBL/VTBL/CSTR/CDBL/DBG_/DBG_`
  (`confirmed-structure`, `confirmed-runtime`).
- No outer compression or encryption is present in the analyzed sample
  (`confirmed-structure`, `confirmed-runtime`).
- CSTR content obfuscation is a reversible nibble swap inside the CSTR section
  (`confirmed-runtime`) and is outside this stage's write scope.
- Alignment/padding between sections: none in the analyzed sample (`confirmed-structure`).
- Unknown trailing data: preserved as `TAIL.bin` when present (`opaque-preserved`).

## Header field roles

| Index | Offset | Confirmed role |
| ---: | ---: | --- |
| 0 | `0x00` | `SB2 ` magic |
| 1 | `0x04` | Version-like field; loader requires value `<= 0`, exact name unconfirmed |
| 2 | `0x08` | Version-like field; loader requires value `== 0`, exact name unconfirmed |
| 3 | `0x0C` | CODE byte length |
| 4 | `0x10` | Global Frame slot count |
| 5 | `0x14` | TTBL record count |
| 6 | `0x18` | First FTBL record count |
| 7 | `0x1C` | Second FTBL record count |
| 8 | `0x20` | VTBL record count |
| 9 | `0x24` | CSTR record count |
| 10 | `0x28` | CDBL record count |

## Runtime-backed record boundaries

- TTBL contains the header-declared number of records. Each record starts with two
  `u32` fields. The second controls a member loop; each serialized member consumes
  `2 x u32 + 16 bytes + u32` (`confirmed-runtime`). The analyzed sample has four
  TTBL records and zero members, so non-empty member bytes are not sample-confirmed.
- VTBL consumes three `u32` values per record (`confirmed-runtime`). The analyzed
  sample count is zero, so no non-empty VTBL payload occurs in the sample.
- CSTR contains `count` pairs of `(offset, span)` followed by exactly `sum(span)`
  bytes. Runtime string lookup uses the first field as a pool offset. The tool also
  rejects offsets that point beyond the runtime-sized pool (`confirmed-runtime`).
- CDBL consumes eight bytes per record (`confirmed-runtime`).
- Both DBG sections are self-describing record tables (`confirmed-structure`).

## Script and localization state

- Script VM and text extraction are not finalized in this stage.
- The CSTR section is a candidate string table, but export filtering, names, messages,
  choices, controls, and newline rules require representative samples and user confirmation.
- Translation interchange will use per-source UTF-8 JSON with immutable `scr_msg` and
  writable `message`; no TXT or DSAT workflow will be introduced.
- Name/message organization: not yet confirmed.
- Variable-length injection: not yet enabled.
