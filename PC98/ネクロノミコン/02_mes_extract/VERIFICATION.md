# MES v0.2.1 verification — 2026-07-18

- `cargo fmt -- --check`: pass
- `cargo test --offline`: 14 passed
- `cargo clippy --offline --all-targets -- -D warnings`: pass
- `cargo build --release --offline --bins`: pass
- Both release binaries `--help`: pass
- User mapping corpus: 278 JSON, 24,161 entries
- Injection: 23,655 patched, 506 unchanged, 278 changed files, 0 failed
- Full re-extraction before the C-disk capacity-only spacing adjustment: 24,161/24,161 translated names and messages matched
- C-disk re-extraction after the capacity-only spacing adjustment: 4,947/4,947 translated names and messages matched

v0.2.1 fixes a CP932 boundary bug found by the mapped Chinese corpus. The
custom hiragana encoder now advances by complete valid Shift-JIS pairs before
considering `82 9F..F1` compression, preventing bytes from adjacent kanji from
being combined across a character boundary.

## Final actual-media rebuild

- Original A–K FDI FAT12 directories: 208 MES, 17,502 entries
- Final translated JSON selection: 208 files from `work\new`
- Injection: 17,116 patched, 386 unchanged, 208 changed files, 0 failed
- Re-extraction comparison: 17,502/17,502 names and messages matched
- The earlier 278-file dump contains wrong-disk duplicates in D/G/H/I/J/K and
  is not the final media file list.
