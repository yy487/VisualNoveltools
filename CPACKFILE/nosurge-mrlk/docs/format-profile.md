# Ciel nosurge DX MRLK format profile

## Scope

- Game: Ciel nosurge DX, Windows release.
- Role: inner model, motion, effect, and texture resource containers, often
  named with a `.psarc` extension.
- Implementation: standalone Rust tool `nosurge-mrlk`.
- Source and output paths: user-selected; source archives are read-only and new
  archives are staged before installation.
- Payload formats: opaque-preserved. G1T conversion is not part of this tool.

## Confirmed structure

All integers are little-endian.

```text
0x00  4   "MRLK"
0x04  4   reserved; the executable requires zero
0x08  4   end of the fixed header plus entry table
0x0C  4   file count
0x10  4   filename-table offset
0x14  4   filename-table byte size
0x18  8*N entries: absolute file offset, file size
...       filename bytes, one name per entry, CRLF-delimited and terminated
...       contiguous payload bytes in table order
```

- Filename-table offset equals `0x18 + file_count * 8` in every checked MRLK
  sample.
- The first payload starts immediately after the filename table. Each later
  payload starts at the preceding payload's end, and the last ends at EOF.
- No archive-level encryption, compression, or alignment was observed.
- The executable reads 24 header bytes, rejects a nonzero reserved field, reads
  the name table from the stored offset and size, splits names on CRLF, then
  reads eight bytes per entry from offset `0x18`.

## Filename and payload policy

- All observed names are ASCII, which is also valid UTF-8. No rule is claimed
  for unobserved non-ASCII filename encodings.
- Names and order are copied exactly from the template archive during packing.
- Entry payloads are opaque. A payload may grow or shrink; the packer rebuilds
  every affected offset and size.
- Extra files in the input directory are ignored. Missing template entries are
  errors.
- Path traversal, absolute paths, unsafe Windows names, and case-insensitive
  output collisions are rejected.

## Verification evidence

- 209 MRLK archives were structurally scanned with no header, name-count,
  continuity, range, or payload-coverage mismatch.
- Those archives contain 893 entries across G1T, G1A, OID, G1M, GAN, G1E, GMD,
  GSK, KCM, KRT, and G1EM payloads.
- One `.psarc` sample begins with standard `PSAR` and is outside this tool's
  scope.
- Synthetic tests cover malformed headers, unsafe paths, missing input files,
  exact unchanged round trips, changed-size rebuilding, and preservation of
  unrelated payloads.
- The release parser accepted all 209 observed MRLK archives.
- A real 38-entry `ui_tex.psarc` unpack/pack/unpack cycle produced a byte-exact
  archive and identical file tree. Replacing one entry with a differently sized
  real G1T rebuilt the archive successfully; the changed entry matched and all
  other 37 entries remained byte-identical.
- Fully specified unpack and pack commands passed with spaces, `&`, and Chinese
  characters in output paths.

