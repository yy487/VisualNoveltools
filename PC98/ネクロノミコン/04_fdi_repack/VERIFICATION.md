# FDI repacker v0.2.0 verification — 2026-07-18

- `cargo fmt -- --check`: pass
- `cargo test --offline`: 3 passed
- `cargo clippy --offline --all-targets -- -D warnings`: pass
- `cargo build --release --offline --bins`: pass
- Release `fdi_repack.exe --help`: pass
- `NECRONOMICON_A.FDI` empty-replacement round trip: SHA-256 byte-exact
- `NECRONOMICON_C.FDI` real replacement test: 59 MES replaced, FAT copies and every replaced/unreplaced file verified, 9 free clusters before and 0 after

v0.2.0 preflights final cluster capacity and deterministically processes
shrinking replacements before growing replacements. This permits safe exact-fit
images without transient allocation failures.

## Final A–K rebuild

- 11/11 original FDI images rebuilt with their real per-disk MES sets
- Replacements per disk: A 67, B 5, C 59, D 24, E 34, F 9, G 2, H 3, I 3, J 1, K 1
- Every repack verified both FAT copies, every replacement, and every unreplaced file
- Every final image then passed an independent empty-replacement repack with
  SHA-256 byte equality
- C disk is an exact fit with 0 free clusters; all other disks retain free space
