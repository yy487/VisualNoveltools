# b_mes AI5WIN MES tool

本工具处理当前标题中已经解压过 LZSS 的 AI5WIN `.MES` 字节码。

## 文件

- `b_mes_opcode.py`：VM opcode/表达式格式定义。注意不能命名为 `opcode.py`，否则会覆盖 Python 标准库 `opcode`。
- `disassembler.py`：语义反汇编，支持 asm 逐字节重建。
- `assembler.py`：asm 重汇编。
- `common.py`：提取/注入共用逻辑。
- `extract.py`：剧情文本/选择支导出 JSON。
- `inject.py`：从 JSON 变长注入并重定位跳转。
- `vm_analysis.md`：当前 VM 分析记录。

## 反汇编/汇编

```bat
python disassembler.py mes\0105.MES -o 0105.MES.asm.txt --encoding cp932
python assembler.py 0105.MES.asm.txt -o 0105.MES.rebuild --encoding cp932
```

## 文本提取

默认只提取剧情/菜单文本，跳过 `0.MES`、`NAME.MES`、`STAND*.MES`、`FLAGINI.MES`、`START.MES`、`TEST.MES` 等资源/表文件，避免把表数据误当成文本。`LBATTLE/LCLIMAX/LEND/LOMAKE` 等 L 系列脚本现在会正常纳入提取；误扫的二进制 cstring 会由占位符过滤规则剔除。

```bat
python extract.py mes json --encoding cp932
```

如确实要调试所有 MES 的原始 TEXT 槽：

```bat
python extract.py mes json_all --encoding cp932 --all-files
```

## JSON 格式

对话按 `name + scr_msg + message` 输出，其中 `name` 为 `【】` 内部名字，不含括号。选项不继承角色名。

```json
{
  "name": "小十郎",
  "scr_msg": "（身体‥‥‥が‥‥‥）",
  "message": "（身体‥‥‥が‥‥‥）",
  "_file": "0105.MES",
  "_index": 0,
  "_type": "dialogue",
  "_inst_offset": 275,
  "_name_inst_offset": 205,
  "_name_scr": "【小十郎】",
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

`{{EB:D4}}` 这类占位符表示原脚本中 cp932 无法直接解码的原始字节。注入时会按原始字节写回，不要随手删除，除非你确认它是可替换的字形码。

## 注入

```bat
python inject.py mes json\new trans --encoding cp932 --report inject_report.json
```

`message` 改动会写回正文 TEXT；`name` 改动会自动写回对应的 `【name】` TEXT 槽。工具会重建整段 MES，并重定位 `0x09 / 0x0A / 0x0E / 0x12` 中的绝对跳转目标。

## 当前验证

- 默认提取：216 个 JSON 文件，22667 条，选择支 254 条，带 name 的正文 13323 条。
- 零修改注入：255 个 MES 逐字节一致，bad=0。

## Gaiji / control placeholder policy

This build applies the project-local gaiji policy requested for the current title:

- `scr_msg` always keeps original raw placeholders such as `{{EB:A9}}` for locate/verify.
- `message` maps confirmed glyphs to readable characters.
- Every unconfirmed `{{EB:xx}}` placeholder is removed from `message` during extraction.

Confirmed mappings are stored in `gaiji_policy.json` and currently include:

```json
{
  "EB:A1": "♪",
  "EB:A5": "！",
  "EB:A6": "！",
  "EB:A8": "？",
  "EB:A9": "！？",
  "EB:AA": "！",
  "EB:B9": "♪",
  "EB:BA": "ォ",
  "EB:BB": "ァ"
}
```

Unknown EB placeholders such as `EB:A0`, `EB:A2`, `EB:A7`, `EB:AB..EB:CE`, `EB:D4` are dropped from `message`.  The original bytes are still visible in `scr_msg`.

Example:

```json
{
  "scr_msg": "「何事{{EB:A9}}」",
  "message": "「何事！？」"
}
```

```json
{
  "scr_msg": "「ど、どうしてそれを‥‥」{{EB:A2}}",
  "message": "「ど、どうしてそれを‥‥」"
}
```

A summary of mapped/dropped counts for the supplied MES set is in `gaiji_policy_report.json`.
