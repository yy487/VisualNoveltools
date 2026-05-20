# YU-RIS 文本提取/注入工作流


## 目录结构

```text
yuris/
├─ common.py      # JSON、编码、读写、统一 TextEntry
├─ crypto.py      # CRC32 key、segment xor、flat xor
├─ yscm.py        # ysc.ybn 命令表解析
├─ ystl.py        # yst_list.ybn 脚本列表解析
├─ yslb.py        # ysl.ybn 标签表解析
├─ ystb.py        # YSTB v5 解析、文本识别、append 注入、重建
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
  "_text_format": "raw",
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

默认只提取 `WORD` 命令。需要额外提取其他命令时：

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
2. 用 `_cmd_index + _expr_index` 定位 `CommandExpression`。
3. 读取旧文本并与 `scr_msg` 校验。
4. 根据 `_text_format` 重新生成表达式数据：
   - `raw`：直接写文本字节。
   - `push_string`：写 `0x4D + u16长度 + 文本字节`。
5. 把新表达式数据追加到 `CommandData` 末尾。
6. 修改当前 expression 的 `instruction_offset / instruction_size`。
7. 重建 header、Command 区、CommandExpression 区、CommandData 区、LineIndex 区。
8. 重新按 segment xor 加密。

这样不会移动旧表达式片段，也不会破坏其他 expression 共享旧 CommandData 的情况。

## xor 模式

默认使用 `segment`，也就是分别对以下分区 xor：

```text
Command
CommandExpression
CommandData
LineIndex
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
- segment xor / flat xor。
- `WORD` 正文提取。
- `raw` 和 `push_string` 两种文本表达式。
- append 型非等长注入。
- `scr_msg` 校验。
- `_cmd_index + _expr_index` 精确定位。
- 同文件 `scr_msg` 唯一匹配 fallback。

暂未完整支持：

- YSTB v2。
- 完整反编译表达式 AST。
- 自动判断所有选项命令。
- 封包/解包 YPF。
- name_define 批量改编码。

## 注意事项

1. 第一版默认只提取 `WORD`，这是为了避免把资源名、变量名、系统参数误提出来。
2. 选项文本需要结合具体游戏的 `ysc.ybn` 命令名确认，建议先用 `--include-raw-candidates` 排查。
3. 如果游戏使用 GBK 注入补丁，可以使用 `--target-encoding gbk`，但这不是默认策略。
4. 如果路径中有空格，Windows 命令行请加引号。
5. 注入前建议保留原始 `ysbin_new` 备份。
