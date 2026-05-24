# 刻音色 `.s` 脚本文本提取 / 注入工具

## 依据

样本 `scr.zip` 中 `.s` 文件是明文 CP932 的脚本字节码流。结合 `keyinse.exe.c` 的脚本解释器分发逻辑，可确认指令基本形式为：

```text
byte0  opcode
byte1  固定指令长度，或变长指令中 inline 字符串起始偏移
byte2  子参数 / 字符串长度 / flag
byte3  子参数 / 字符串长度
...
```

正文显示指令：

```text
04 04 flag payload_len  <cp932 text> 81 94 00 [CD padding]
```

其中 `81 94` 是 CP932 的 `＃`，引擎绘制函数将 `0x8194` 当作行/句中断控制；尾部 `00` 和 `CD` padding 不属于正文。`payload_len` 是从正文 payload 起点开始计算的字节数，包含 `81 94 00` 和 padding。

选项指令：

```text
1B 08 text_len 00  target_u32_le  <cp932 choice text>
```

`target_u32_le` 是脚本内绝对偏移。目标位置通常会继续执行 `01 ... <file.s>` 这类脚本切换指令；如果工具能解析到目标脚本文件，会在 JSON 中额外输出 `_target_file`。

## JSON 格式

输出为 UTF-8 JSON 数组，每条文本使用统一格式：

```json
{
  "scr_msg": "原文",
  "message": "原文",
  "_file": "s23.s",
  "_index": 0,
  "_offset": 62,
  "_inst_offset": 58,
  "_size": 44,
  "_type": "monologue",
  "_opcode": "0x04",
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

规则：

- `scr_msg` 是原始脚本文本，只用于定位和校验，不要改。
- `message` 是实际写回文本，翻译只改这里。
- 正文内部的分页 / 等待控制符 `＃` 只保留在 `scr_msg` 中，提取时会从 `message` 中隐藏。注入时会根据 `scr_msg` 中 `＃` 的原始字节位置自动补回。
- 没有明确可写角色名槽位，因此默认不输出 `name`。
- 如果正文前存在类似 `ka23008` 的语音资源，会附带 `_voice` 和 `_speaker_code` 作为辅助上下文。
- 选项输出 `_type: "choice"`，并保留 `_target`。如果能从目标 offset 解析到分支脚本，会额外输出 `_target_file`；连续 `1A + 多个 1B + 19` 的菜单还会输出 `_choice_group`、`_choice_order`、`_choice_count`，方便校对游戏内选择菜单。


## 正文内部 `＃` 的处理

正文末尾的 `81 94 00` 是文本块结束控制，工具不会导出到 JSON。正文中间出现的 `＃` 则是同一条逻辑文本内部的分页 / 等待控制。新版工具采用以下策略：

```json
{
  "scr_msg": "な……なぜって……＃ああぁ……こんな……こんな事……＃ひ、ひどい……酷すぎますぅ……うぅ。。",
  "message": "な……なぜって……ああぁ……こんな……こんな事……ひ、ひどい……酷すぎますぅ……うぅ。。",
  "_page_mark": "＃",
  "_page_mark_byte_offsets": [18, 52],
  "_message_page_mark_mode": "auto_from_scr_msg"
}
```

翻译时只改 `message`，不要改 `scr_msg`。注入时流程为：

1. 读取 `scr_msg` 中 `＃` 出现的位置；
2. 先删除 `message` 中误输入的 `＃`，避免重复插入；
3. 按 `scr_msg` 里记录的 CP932 字节位置把 `＃` 自动插回 `message`；
4. 再重建 `04` 正文指令。

这样 JSON 里 `message` 更干净，翻译时不用手动维护分页符；同时回写后仍能保留原脚本的分页 / 等待节奏。零修改回环仍然可以保持 byte-exact。

## 命令

单文件提取：

```bash
python extract.py s23.s json/s23.s.json
```

目录批量提取：

```bash
python extract.py scr json
```

单文件注入：

```bash
python inject.py s23.s json/s23.s.json out/s23.s
```

目录批量注入：

```bash
python inject.py scr json out_scr --stats-json inject_stats.json
```

## 注入模式

默认模式：

```bash
python inject.py scr json out_scr --mode relocate
```

`relocate` 会重建被修改的正文 / 选项指令，并修正当前已确认的脚本内绝对偏移：

- `05`: `p+4` 的目标偏移
- `1B`: 选项目标 `p+4`
- `1C` - `21`: 条件跳转目标 `p+12`
- `33`: 条件跳转目标 `p+4`

保守原地模式：

```bash
python inject.py scr json out_scr --mode in-place
```

`in-place` 不改变文件大小，不移动后续指令。正文会在原 payload 内重写并补 `CD`，选项会在原长度内重写并补半角空格。超长会报错并跳过。

## 长度限制

- 正文 payload 长度字段是 1 字节，重建后 payload 不能超过 `255` 字节。
- 选项文本长度字段是 1 字节，文本不能超过 `255` 字节。
- CP932 不可编码字符会在注入时报错；需要先做 Cn/Jp 字符映射或字体方案。

## 选择跳转结构补充

截图中这类人物选择、地点选择都属于 `0x1B` 选项指令。典型结构为：

```text
35 ...              # 选择界面/状态准备
1A 04 00 00        # 选择组开始
1B 08 len 00 target_u32 text
1B 08 len 00 target_u32 text
...
19 04 00 count     # 选择组结束，count 为选项数
```

已经在样本中确认：

- `sce02b.s`：人物选择，4 个选项，分别跳到 `sce02b1.s` 到 `sce02b4.s`。
- `sce02c.s`：地点选择，5 个选项，分别跳到 `sce02c1.s` 到 `sce02c5.s`。

工具会把这些选项作为普通 JSON 条目导出，但 `_type` 为 `choice`，并附带 `_target` / `_target_file` / `_choice_order` 等内部字段。

## 已验证

在提供的 `scr.zip` 上：

- 可完整解析 786 个 `.s` 文件；
- 提取正文 / 选项共 23828 条；
- `S23.S` 对应 `s23.s`，提取 821 条，其中包括 10 条选项；
- 零修改注入可 byte-exact 回环。

## 已知限制

1. 角色名不是独立文本槽，默认不输出 `name`。如需显示上下文名，可以自行提供 voice prefix 到角色名的映射，在提取时加 `--name-map name_map.json`。
2. `relocate` 只修正目前从解释器中确认的脚本内偏移字段。若后续发现其他 opcode 也保存绝对偏移，需要补充 `TARGET_FIELD_OFFSETS`。
3. 工具只处理散 `.s` 文件；如果实际游戏还有外层封包，需要先解包再运行本工具，注入后再重封包。


## 2026-05-24 page-mark-auto-v3

本版明确采用“scr_msg 保留 ＃、message 隐藏 ＃、注入时自动补回”的分页标记策略。

- `scr_msg`：保留原始 `＃`，只用于定位、校验和继承分页位置。
- `message`：提取时自动移除 `＃`，翻译时通常不需要手动填写。
- 注入正文 `0x04` 时：先移除 `message` 中误写的 `＃`，再按 `scr_msg` 的分页标记位置自动插回。
- 选项 `0x1B` 不走分页标记自动补回，避免误处理选择文本。

可用以下命令确认版本：

```bash
python extract.py --version
python inject.py --version
```

## v4：文本渲染闪退保护

`crash.dmp` 显示异常点在游戏本体 `004100C5`，对应 `FUN_0040fda0` 字形渲染函数，异常码为 `0xC0000005`，是在绘制文本时读写了异常的 DIB 像素地址。这类问题通常不是脚本 opcode 边界错位，而是某条翻译文本让游戏的旧文本渲染器越界。

源码里 `FUN_00438880` / `FUN_0040fda0` 的关键限制：

- 正文按 2 字节 CP932 单元读取，半角 ASCII、半角空格、半角数字/字母会造成读取错位风险。
- 正文里的 `＃` / `0x8194` 是手动换行控制；末尾的 `81 94 00` 是文本结束。
- 单行约 23 个双字节字符后会自动换行。
- 临时文本位图高度很小，安全行数约 4 行；翻译过长或换行位置不合适会导致渲染函数越界。

因此 v4 的注入默认增加两层保护：

```bat
python inject.py scr json out_scr --mode relocate --stats-json inject_stats.json
```

默认行为：

- `--page-mark-mode proportional`：`message` 里仍然不显示 `＃`，注入时按原文换行比例自动回填，而不是死用原字节偏移。
- `--layout-policy skip`：如果某条译文含半角字符、编码后奇数字节长度、或估算渲染行数超过安全上限，就跳过该条并写入 warning，避免生成会闪退的脚本。

需要复现 v3 固定字节位置行为：

```bat
python inject.py scr json out_scr --mode relocate --page-mark-mode byte-offset
```

需要强制注入并只输出警告：

```bat
python inject.py scr json out_scr --mode relocate --layout-policy warn --stats-json inject_stats.json
```

如果游戏在某一句之后闪退，优先看 `inject_stats.json` 中该文件后续几条的 `unsafe text layout` warning，通常不是屏幕上已经显示出的那一句，而是下一条文本开始预渲染时崩。


## v6：超长文本自动分段防崩

从 v6 开始，`inject.py` 默认使用：

```bat
python inject.py liudang chs scr --mode relocate --stats-json inject_stats.json
```

等价于：

```bat
--page-mark-mode auto-fit --layout-policy skip
```

规则：

1. `scr_msg` 继续保留原始 `＃`。
2. `message` 默认不显示 `＃`，正常翻译不用手动写。
3. 注入时如果 `message` 没有 `＃`，会按 `scr_msg` 的分段比例自动补回。
4. 如果你在 `message` 里手动写了 `＃`，v6 会尊重这个手动分段，不再先删除它。
5. 如果某一段仍然过长，v6 会自动追加 `＃`，避免游戏文本渲染函数越界崩溃。

针对类似：

```json
"scr_msg": "目立ちこそしないが＃一目で上質と解る家具の数々＃資産家の家だとは思っていたけど……。",
"message": "雖然毫不起眼，但一眼望去便知皆是上乘之作的累累家具，我此前雖然就想過這里定是資產階級的宅邸……。"
```

旧逻辑可能生成一个过长连续段，游戏会在下一句渲染时闪退。v6 会自动插入 `＃` 分段。

如果你想完全手动控制分段：

```bat
python inject.py liudang chs scr --mode relocate --page-mark-mode manual --stats-json inject_stats.json
```

如果要复现旧版比例补 `＃`：

```bat
python inject.py liudang chs scr --mode relocate --page-mark-mode proportional --stats-json inject_stats.json
```


## Beta: drop-page-mark 模式

本 beta 版默认采用：

```bash
--page-mark-mode drop --layout-policy warn
```

含义：

1. `scr_msg` 仍然保留原始 `＃`，用于校验。
2. `message` 里默认不显示 `＃`。
3. 注入时不会根据 `scr_msg` 自动补回 `＃`。
4. 即使 `message` 里手动写了 `＃`，`drop` 模式也会在写回前移除。
5. 文本块末尾必须存在的 `81 94 00` 终止结构仍由工具自动生成；本模式只移除正文内部的分页/中断 `＃`。
6. 默认 `layout-policy=warn`，长文本风险只报警不跳过，方便实机测试。若想恢复保护可加 `--layout-policy skip`。

常用命令：

```bash
python inject.py liudang chs scr --mode relocate --stats-json inject_stats.json
```

显式指定：

```bash
python inject.py liudang chs scr --mode relocate --page-mark-mode drop --layout-policy warn --stats-json inject_stats.json
```

风险：游戏文本渲染器可能因为单段过长而闪退；这个 beta 版就是用于验证“完全不插正文 `＃`”的实机行为。
