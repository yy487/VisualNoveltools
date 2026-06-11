# AI5WIN MES 文本提取/注入工具

适用对象：本次样本中的 AI5WIN `.MES` 脚本。工具按 VM 指令结构解析，不做裸扫文本。

## JSON 格式

导出为 UTF-8 JSON 数组。面向翻译的字段为：

```json
{
  "name": "あんず",
  "scr_msg": "「原文」",
  "message": "「译文」",
  "_file": "S01.MES",
  "_index": 0,
  "_type": "dialogue"
}
```

- `scr_msg`：原始脚本文本，只用于定位校验，不要修改。
- `message`：实际注入文本。
- `name`：只有明确存在说话人时输出。动态男主名不再导出占位符，统一写死为 `小十郎`。

## 外字规则

提取时：

```text
EB A4 -> ！
EB A8 -> ？
EB AC -> っ
EB AA -> ♪
EB AE -> ♥
EB AB -> ッ
EB AD -> ！！
```

以下不能确认的外字在 `message` 内删除，但 `scr_msg` 仍保留 token：

```text
EB A1 / EB A2 / EB A3 / EB A6 / EB A7 / EB A9
```

注入时会反向把上面的可见符号编码回对应外字字节。`{{EB:A4}}` 这类 token 也可直接写在 `message` 中。

## 运行时男主名规则

原脚本中的运行时名字调用：

```text
0F 02 00 FF 00
```

当前按要求不再保留 `{{CALL:0F}}` 占位符，提取显示为 `小十郎`，注入重建时直接把这条调用指令替换成：

```text
01 "小十郎" 00
```

也就是在 MES 指令流内部原位重建为普通 TEXT 指令，不做尾部追加。其它无法确认含义的内联 `0F` 调用不显示在 `message/scr_msg`，但原始 raw 会留在 `_inline_controls`，注入时原样保留。

## ruby / 注音规则

`0B 18` ruby 结构不作为翻译条目导出。注入重建时，ruby 参数字符串统一替换成同字符数的全角空格，例如：

```text
ﾊﾊ -> 　　
```

这样保留原 ruby 指令结构，但实际不显示注音。

## 命令

提取目录：

```bash
python extract.py mes json
```

注入目录：

```bash
python inject.py mes json trans --copy-unmatched
```

单文件：

```bash
python extract.py S57.MES S57.MES.json
python inject.py S57.MES S57.MES.json S57_new.MES
```

默认会把静态 `name` 字段注回对应 `【name】` 行。若只想注正文：

```bash
python inject.py mes json trans --no-inject-names --copy-unmatched
```

## 注入模式

注入采用整文件重建：

1. 解析所有 opcode、文本槽和跳转目标。
2. 替换正文文本槽、静态 name 槽和 ruby 槽。
3. 重新生成整个 MES bytecode。
4. 按长度变化修正 `09/0A/0E/12` 的绝对 target。

不采用尾部追加，也不把新字符串追加到文件末尾。

## 当前限制

- 当前只导出 `0x01` 主文本：dialogue / monologue / choice。
- `0x02` system text 和函数参数文本默认不作为翻译正文导出，避免误改资源名、文件名和系统参数。
- 变长后会修正已解析的绝对跳转 target；如果存在尚未识别的隐藏 offset 表，需要后续按实际报错补充。

## 2026-06-11 修正：写死小十郎后的 multipart 注入

包含运行时男主名的多段正文会导出为顶层 `message` 中直接显示 `小十郎`，同时保留 `message_parts` 和 `_inline_controls`。
翻译时只需要改顶层 `message`。单槽条目会始终以顶层 `message` 为准；多槽且含 `_inline_controls.placeholder == "小十郎"` 的条目会把顶层 `message` 回拆到原来的 TEXT 槽。

例如原始：

```json
{
  "message": "「さらばじゃ、小十郎」",
  "message_parts": ["「さらばじゃ、", "」"],
  "_inline_controls": [{"after_part": 0, "placeholder": "小十郎"}]
}
```

只修改：

```json
"message": "「さらばじゃ、小十郎どの」"
```

即可注回。无需同步修改 `message_parts`。

## 2026-06-11 修正：强制顶层 message 注入

当前版本所有多槽条目也优先使用顶层 `message` 注入。

规则：

1. 单槽条目：直接把顶层 `message` 写入该槽。
2. 多槽 + 可见 `小十郎` 占位：优先按 `小十郎` 回拆，尽量保留原控制调用位置。
3. 其它无法安全回拆的多槽：强制把完整顶层 `message` 写入第一个 TEXT 槽，后续 TEXT 槽写空字符串。

因此翻译时原则上只改 `message` 即可，不需要同步维护 `message_parts`。`message_parts` 仅作为结构参考和旧版兼容字段保留。

这个模式仍然是整文件重建，不是尾部追加。需要注意：如果某条多槽文本中间夹了未知 `0F` 控制调用，强制压平后该控制调用仍会按原位置执行，只是它前后的后续 TEXT 槽会被清空。
