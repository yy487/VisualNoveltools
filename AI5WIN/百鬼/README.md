# Baigui MES script tools

This package contains a finished first-version toolchain for `百鬼 -淫黙された廃墟-` `.MES` scripts.

## Files

```text
opcode.py        Common VM definitions, LZSS, encoding helpers

disassembler.py  .MES -> semantic asm, optional JSON export
assembler.py     asm -> rebuilt .MES, optional JSON import
vm_analysis.md   VM and text-stream analysis notes
asm.txt          sample semantic assembly output
```

## Basic usage

### Disassemble one MES

```bash
python disassembler.py P_01R2.MES -o P_01R2.MES.asm.txt --json P_01R2.MES.json --encoding cp932
```

If `-o` is omitted, the output defaults to:

```text
<input>.asm.txt
```

### Rebuild one MES

```bash
python assembler.py P_01R2.MES.asm.txt -o P_01R2.rebuild.MES
```

### Plain roundtrip test

```bash
python disassembler.py P_01R2.MES -o P_01R2.MES.asm.txt --encoding cp932
python assembler.py P_01R2.MES.asm.txt --plain -o P_01R2.rebuild.plain
```

Compare `P_01R2.rebuild.plain` against the LZSS-decompressed original plain stream.  They should match byte-for-byte.

### JSON injection

1. Export JSON:

```bash
python disassembler.py P_01R2.MES -o P_01R2.MES.asm.txt --json P_01R2.MES.json --encoding cp932
```

2. Edit only `message`; optionally edit `name` when present.

3. Rebuild through the asm IR:

```bash
python assembler.py P_01R2.MES.asm.txt --json P_01R2.MES.json -o P_01R2.chs.MES
```

This performs full plain-stream rebuild, not fixed-offset overwrite.

## Batch mode

Disassemble all `.MES` files in a directory:

```bash
python disassembler.py mes_dir -o asm_dir --json json_dir --encoding cp932
```

Rebuild all `.asm.txt` files in a directory:

```bash
python assembler.py asm_dir --json json_dir -o rebuilt_mes_dir
```

## Drag-and-drop behavior

On Windows, dragging a `.MES` file onto `disassembler.py` runs the same as:

```text
python disassembler.py <dropped file>
```

Dragging an `.asm.txt` file onto `assembler.py` runs the same as:

```text
python assembler.py <dropped asm>
```

## Assembly format

`.cstring1` is the semantic representation of the engine text control:

```text
.cstring1 "竜一"
```

It emits:

```text
01 <cp932 bytes> 00
```

Opaque VM/control bytes are preserved as `.byte` definitions.

Special bytes inside strings use `{{XX}}` placeholders.  The tools never use `\xNN` escapes in asm strings.

## Extraction rules

Dialogue and monologue:

```text
0x0B text block boundary
inside block: one or more 0x01 cstrings
2 strings: name + message
1 string : monologue/message
```

Choice/menu:

```text
選択肢：... label
following short 0x01 cstrings -> _type=choice
```

Resource names such as `.wav`, `.mam`, `.gpr`, `.mes` are preserved in asm but skipped from JSON.

## Important notes

- Default text encoding is `cp932`.
- The tool rebuilds the plain MES stream structurally.
- LZSS recompression is compatible but not byte-identical to the original compressor.
- Unknown regions are preserved as `.byte` and are not modified.
- If a translated string contains characters not encodable in `cp932`, assembly fails; use your CnJpMap/font strategy before rebuilding.
- Choice jump target labeling is not exported yet; choice text itself is exported and injectable.


## JSON 提取/注入入口

`extract.py` 和 `inject.py` 是正式翻译工作流入口；二者都不单独解析二进制，而是调用 `disassembler.py`/`assembler.py` 的 IR 层完成工作。

提取单文件：

```bash
python extract.py P_01R2.MES P_01R2.MES.json --encoding cp932
```

提取目录：

```bash
python extract.py mes json_out --asm-out asm_out --encoding cp932
```

注入单文件：

```bash
python inject.py P_01R2.MES P_01R2.MES.json P_01R2.rebuild.MES
```

注入目录：

```bash
python inject.py mes json_new rebuilt_mes --asm-out asm_rebuild
```

流程固定为：

```text
原始 .MES -> LZSS 解压 -> disassembler IR -> JSON patch -> assembler 全量重建 plain -> LZSS 重压缩
```

因此注入不是 offset 原地覆盖，而是基于反汇编 IR 的全量重建。`--asm-out` 可保留中间 IR 便于排查；不指定时注入器使用临时目录。
