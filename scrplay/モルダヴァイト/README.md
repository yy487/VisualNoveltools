# SCR:0034 ScrPlayer 文本提取/注入工具

## 格式结论

文件结构：

```text
0x00  header 0x14 bytes, magic starts with SCR:0034
0x14  u32 cmd_len
0x18  command section
      u32 str_len
      string section, XOR 0x7F encrypted, NUL separated
```

主要 OP：

```text
0x62 len=0x10  正文/对话，param1 多为语音/资源，param2 为文本槽
0x6A len=0x0C  选项，param1 为选项文本
0xA5 len=0x08  系统确认/提示，param0 为文本
```


## 提取

单文件：

```bash
python extract.py adv_01_intro.scr json/adv_01_intro.scr.json
```

目录：

```bash
python extract.py mew json
```

输出 JSON 字段：

```json
{
  "name": "ステラ",
  "scr_msg": "｢ﾁｮｯﾄﾞﾞ誰が子供ﾖｯ!｣",
  "message": "｢ﾁｮｯﾄﾞﾞ誰が子供ﾖｯ!｣",
  "_file": "adv_01_intro.scr",
  "_index": 12,
  "_type": "dialogue",
  "_opcode": "0x62",
  "_cmd_offset": 468,
  "_param_index": 2,
  "_slot_index": 29,
  "_slot_offset": 618,
  "_slot_size": 27,
  "_prefix": "ステラ\n",
  "_suffix": "\n",
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

只改 `message`。`scr_msg` 用于校验，不要改。`name` 可以改，改了会同步写回对话槽的名字前缀。

如果想用 pasted code 里的 `fixOrig` 风格，把半角假名/标点转成更正常的显示字符：

```bash
python extract.py mew json --fix-orig
python inject.py mew json out_mew --fix-orig --copy-missing
```

注意：`--fix-orig` 注入时会做反向转换，适合日文原文可读化；中文译文仍然需要 cp932 可编码或映射。

## 注入

单文件：

```bash
python inject.py adv_01_intro.scr json/adv_01_intro.scr.json out/adv_01_intro.scr
```

目录：

```bash
python inject.py mew json out_mew --copy-missing --stats-json inject_stats.json
```

## 已知限制

1. 当前工具默认编码是 `cp932`，不直接写 GBK/UTF-8 到脚本。
2. 资源名 OP 只做 offset 修正，不导出翻译，避免误改图片/语音/BGM 名导致运行时找不到资源。
3. 如果后续发现未配置的新字符串引用 OP，需要在 `scr0034_common.py` 的 `STR_CODE_CONFIG` 里补 `-1` 或 `1`。
4. 当前 JSON 是一文件一 JSON，不做封包；如果外层还有 `.mew`/包文件，需要先解包后处理，再用原封包器打回。

## 2026-05-29 v2 修正

`0xD2`、`0xD1` 等资源/控制类 opcode 的参数不是纯字符串 offset，可能混有普通数值 ID/flag。
因此 v2 把所有 `-1` 参数改为“探测式字符串引用”：

- 参数值命中字符串区 offset：参与字符串区重建和 offset 修正；
- 参数值未命中字符串区 offset：按普通数值原样保留；
- 只有 `0x62:param2`、`0x6A:param1`、`0xA5:param0` 这些导出文本参数会严格报错。

这可以修复：

```text
ValueError: string offset not found: cmd+0xF8 op=0xD2 param0=0x5
```
