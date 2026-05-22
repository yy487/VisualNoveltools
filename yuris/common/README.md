# YU-RIS 文本提取/注入工作流

## 目录结构

```text
yuris/
├─ common.py      # JSON、编码、读写、统一 TextEntry
├─ crypto.py      # CRC32 key、segment xor、flat xor
├─ yscm.py        # ysc.ybn 命令表解析
├─ ystl.py        # yst_list.ybn 脚本列表解析
├─ yslb.py        # ysl.ybn 标签表解析
├─ ystb.py        # YSTB v5/v2 解析、args 扫描、append 注入、重建
├─ extract.py     # 提取入口
├─ inject.py      # 注入入口
└─ pipeline.py    # 流程入口
```

## 统一 JSON 格式

提取结果使用我们统一的格式：

```json
{
  "name": "角色名",
  "scr_msg": "原始脚本文本",
  "message": "原始脚本文本",
  "_file": "yst00012.ybn",
  "_source": "userscript/scene/foo.txt",
  "_script_id": 12,
  "_index": 0,
  "_cmd_index": 83,
  "_expr_index": 0,
  "_expr_id": 0,
  "_expr_offset": 3296,
  "_expr_size": 22,
  "_offset": 1184,
  "_text_format": "raw",
  "_extract_source": "command",
  "_opcode": "WORD",
  "_opcode_id": 37,
  "_type": "message",
  "_label": "start"
}
```

说明：

- `scr_msg` 只用于定位和校验，不要修改。
- `message` 是实际写回的新文本。
- 没有角色名时不会输出 `name`。
- `_offset` 是兼容字段，对应表达式/参数项在索引区里的偏移。
- `_extract_source` 表示提取来源：`command`、`args_scan` 或 `v2_args_scan`。
- `_is_option: true` 表示该条来自选项文本。
- 下划线字段用于注入定位，翻译时不要修改。

## 提取命令

推荐使用 pipeline：

```bat
python -m yuris.pipeline extract ysbin_new json_out --ysc ysbin\ysc.ybn --yst-list ysbin\yst_list.ybn --ysl ysbin\ysl.ybn --key-text "游戏key字符串"
```

也可以直接指定 4 字节 key：

```bat
python -m yuris.pipeline extract ysbin_new json_out --ysc ysbin\ysc.ybn --key-hex 12345678
```

如果已有文件未加密，可以不传 key：

```bat
python -m yuris.pipeline extract ysbin_new json_out --ysc ysbin\ysc.ybn
```

默认 `--extract-mode both`，即：

1. 先用 `YSCM + WORD` 做结构化提取；
2. 再用 `YU-RIS-Script-Editor` 风格的 args 扫描补充未覆盖的位置，包括 `ES.SEL.SET` 后面的选项文本。

可选模式：

```bat
--extract-mode word   只提取 YSCM 指定命令，默认 WORD
--extract-mode args   只按 args_index 扫描，可不传 --ysc，也适合 YSTB v2
--extract-mode both   默认，两套逻辑合并并按 cmd/expr 去重
```

只跑 args 扫描示例：

```bat
python -m yuris.pipeline extract ysbin_new json_out --extract-mode args
```

需要额外提取其他命令时：

```bat
python -m yuris.pipeline extract ysbin_new json_out --ysc ysbin\ysc.ybn --command WORD --command ES.SEL.SET
```

如果想临时排查所有可疑字符串：

```bat
python -m yuris.pipeline extract ysbin_new json_out --ysc ysbin\ysc.ybn --include-raw-candidates
```

## 注入命令

```bat
python -m yuris.pipeline inject ysbin_new json_trans ysbin_chs --ysc ysbin\ysc.ybn --key-text "游戏key字符串"
```

如果 JSON 里已有 `_cmd_index + _expr_index`，也可以不传 `--ysc`：

```bat
python -m yuris.pipeline inject ysbin_new json_trans ysbin_chs
```

默认读取和写回编码都是 `cp932`。如果你明确采用旧工具的 GBK 注入方案：

```bat
python -m yuris.pipeline inject ysbin_new json_trans ysbin_chs --ysc ysbin\ysc.ybn --target-encoding gbk --key-text "游戏key字符串"
```

## 编码检查

```bat
python -m yuris.pipeline check-json json_trans --encoding cp932
```

如果输出不可编码字符，建议先做 CnJpMap / 字体重绘 / 替换映射，不建议直接把通用流程默认切到 GBK。

## 注入策略

注入不是原地覆盖，而是：

1. 解密 YSTB。
2. 用 `_cmd_index + _expr_index` 定位表达式/参数项。
3. 读取旧文本并与 `scr_msg` 校验。
4. 根据 `_text_format` 重新生成表达式数据。
5. 把新数据追加到 `CommandData` 或 v2 参数区末尾。
6. 修改当前 expression/参数项的 `offset / size`。
7. 重建 header 和各分区。
8. 重新按 segment xor 加密。

支持的 `_text_format`：

```text
raw                         裸文本
args_raw                    args 扫描得到的裸文本
v2_raw                      YSTB v2 参数区裸文本
push_string                 0x4D + u16长度 + payload
push_string_quoted          0x4D + u16长度 + 0x22 + payload + 0x22
option_push_string          选项 PushString 兼容格式
option_push_string_quoted   ES.SEL.SET 后常见的带引号选项 PushString
```

`YU-RIS-Script-Editor` 中常见的选项格式是：

```text
4D 0C 00 22 45 53 2E 53 45 4C 2E 53 45 54 22    ; "ES.SEL.SET"
4D LL LL 22 <option text> 22                      ; 选项文本
```

当前工具会在 args 扫描中识别这个结构，并把后一条输出为 `_type: choice`。

## xor 模式

默认使用 `segment`，也就是分别对以下分区 xor：

```text
YSTB v5: Command / CommandExpression / CommandData / LineIndex
YSTB v2: CodeSegment / ArgsSegment
```

兼容旧 `YURIS_TOOLS` 的整体模式可以用：

```bat
--xor-mode flat
```

## 当前支持范围

已实现：

- YSCM 命令表读取。
- YSTL 脚本源路径读取。
- YSLB 标签读取。
- YSTB v5 header / command / expression / command data / line index 解析。
- YSTB v5 的 args_index 扫描补充。
- YSTB v2 基础文本扫描与 append 注入。
- segment xor / flat xor。
- `WORD` 正文提取。
- `ES.SEL.SET` 后续选项文本提取。
- `raw`、`push_string`、`push_string_quoted` 等文本表达式。
- append 型非等长注入。
- `scr_msg` 校验。
- `_cmd_index + _expr_index` 精确定位。
- 同文件 `scr_msg` 唯一匹配 fallback。

暂未完整支持：

- 完整反编译表达式 AST。
- 自动判断所有游戏特有选项命令。
- YPF 重封包。
- name_define 批量改编码。
- 复杂文本控制码的语义级重写。

## 注意事项

1. 默认 `both` 模式比旧版多提取 args 扫描补充项。如果你只想要最保守的正文，使用 `--extract-mode word`。
2. `args` 模式可能提取到候选文本，工具会尽量过滤资源名，但仍建议人工抽查。
3. 如果游戏使用 GBK 注入补丁，可以使用 `--target-encoding gbk`，但这不是默认策略。
4. 如果路径中有空格，Windows 命令行请加引号。
5. 注入前建议保留原始 `ysbin_new` 备份。
