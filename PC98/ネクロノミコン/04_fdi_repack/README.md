# NECRONOMICON FDI repacker

Version 0.2.0 preflights the final FAT12 cluster requirement and applies
replacements in deterministic cluster-delta order. Files that shrink are
processed before files that grow, so an image that exactly fits cannot fail
because temporary allocation happened before reusable clusters were released.

`fdi_repack` replaces existing files in a PC-98 FAT12 disk image and writes a
new FDI. It preserves the 0x1000-byte FDI header and refuses to overwrite the
input or an existing output.

```powershell
fdi_repack.exe "<FDI_DIR>\NECRONOMICON_A.FDI" `
  --replacements "<TRANSLATED_DIR>\DISK_A" `
  --output "<TRANSLATED_DIR>\NECRONOMICON_A.translated.FDI"
```

The replacement tree is relative to the disk root. Its files must already
exist in the image and must be valid CP932 8.3 names. New files, long names,
and ambiguous normalized names are rejected. Both FAT12 copies are checked,
and files may grow, shrink, or become zero length as free clusters are
allocated or released.

Before writing, the tool snapshots every file. After writing it verifies the
FAT copies, directory/file count, every replacement byte sequence, and every
unreplaced file byte sequence. A replacement MES should be produced first by
`mes_inject`; this tool does not parse or translate MES itself.

The FDI layout currently validated from the game is 1024-byte sectors, one
sector per cluster, two FATs, 192 root entries, two sectors per FAT, and 1232
total sectors. The parser reads these fields from the boot sector and rejects
inconsistent image sizes. It does not modify fonts or the game's text
renderer; CP932 representability and display width remain separate limits.

## MES JSON to FDI workflow

`mes_inject` may emit MES files of any length. Keep its output tree rooted at
the disk directory and pass the matching disk subtree as replacements:

```powershell
mes_inject.exe json --source-root "<MES_ROOT>" --output rebuilt

fdi_repack.exe "<FDI_DIR>\NECRONOMICON_A.FDI" `
  --replacements rebuilt\DISK_A `
  --output NECRONOMICON_A.translated.FDI
```

The repacker updates the directory-entry file size, releases or allocates
clusters, writes both FAT12 copies, then reads every file back through the new
chains. The translated FDI is a new file; the original image is not modified.

## Verification performed

An empty replacement tree round-tripped `NECRONOMICON_A.FDI` byte-for-byte.
Growth and shrink tests both passed. A real variable-length `NA_01A.MES` was
grown from 1859 to 2293 bytes and repacked into a new A-disk FDI. Its FAT12
chain grew from two clusters to three (`12,13,1146`); independently following
the directory entry and FAT12 chain recovered all 2293 bytes with SHA-256
`C334FE029B5C0734F5228FDE41141865A2D3D5D1A696432605A356F2826696A5`,
exactly matching the MES supplied to the repacker.
