# AI6WIN story tools

These Python tools unpack and rebuild AI6WIN ARC archives and extract or inject
MES text.

## Workflow

```powershell
python ai6win_arc_extract.py <MES.ARC> <MES_DIR>
python v1\extract.py <MES_DIR> <JSON_DIR>
python v1\inject.py <MES_DIR> <JSON_DIR> <PATCHED_DIR>
python ai6win_arc_pack.py <PATCHED_DIR> <OUTPUT.ARC> `
  --manifest <MES_DIR>\ai6win_manifest.json --source-arc <MES.ARC>
```

Use `v0` instead of `v1` for an older script variant. Keep
`ai6win_manifest.json` for packing.

Edit `message`. Keep `scr_msg` unchanged. `name` is extracted from the source
speaker prefix and is treated as context by this workflow. Text remains CP932;
use an external carrier-map and font process before injection when needed.

## Limits

- AI6WIN games use several incompatible ARC and MES variants; choose `v0` or
  `v1` from actual script structure.
- `--skip-encode-error` skips invalid entries and can produce a partially
  translated result.
- This toolchain does not build Chinese mappings or fonts.
